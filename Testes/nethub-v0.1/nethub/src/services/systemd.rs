//! ServiceManagementService: consulta e reinício de serviços systemd com verificação pós-ação (RF-12).
use super::audit::{AuditEntry, AuditService};
use super::guard::Guard;
use super::risk::{plan_hash, RiskPolicyService};
use super::validator::PostActionValidator;
use super::Done;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use crate::util::{normalize_unit, sh_quote, valid_unit_name};
use std::sync::Arc;

pub struct ServiceManagementService {
    remote: Arc<dyn IRemoteExecutor>,
    guard: Arc<Guard>,
    risk: Arc<RiskPolicyService>,
    audit: Arc<AuditService>,
    validator: PostActionValidator,
}

pub fn parse_show(name: &str, out: &str) -> ServiceState {
    let mut s = ServiceState { name: name.to_string(), ..Default::default() };
    for line in out.lines() {
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().to_string();
            match k.trim() {
                "LoadState" => s.load_state = v,
                "ActiveState" => s.active_state = v,
                "SubState" => s.sub_state = v,
                "MainPID" => s.main_pid = v.parse().unwrap_or(0),
                "ActiveEnterTimestampMonotonic" => s.enter_ts = v,
                _ => {}
            }
        }
    }
    s
}

pub fn parse_list(out: &str) -> Vec<SystemdService> {
    out.lines()
        .filter_map(|l| {
            let t: Vec<&str> = l.split_whitespace().collect();
            if t.len() < 4 || !t[0].contains('.') {
                return None;
            }
            Some(SystemdService {
                name: t[0].to_string(),
                load_state: t[1].to_string(),
                active_state: t[2].to_string(),
                sub_state: t[3].to_string(),
                description: t[4..].join(" "),
            })
        })
        .filter(|s| s.load_state != "not-found")
        .collect()
}

fn check_name(name: &str) -> Res<String> {
    if !valid_unit_name(name) {
        return Err(NhError::Invalid(format!("nome de serviço inválido: '{name}'")));
    }
    Ok(normalize_unit(name))
}

impl ServiceManagementService {
    pub fn new(remote: Arc<dyn IRemoteExecutor>, guard: Arc<Guard>, risk: Arc<RiskPolicyService>, audit: Arc<AuditService>) -> Self {
        ServiceManagementService { remote, guard, risk, audit, validator: PostActionValidator }
    }

    pub fn status_raw(&self, host: &Host, name: &str) -> Res<ServiceState> {
        let unit = check_name(name)?;
        let script = format!(
            "systemctl show {} -p LoadState -p ActiveState -p SubState -p MainPID -p ActiveEnterTimestampMonotonic --no-pager 2>&1",
            sh_quote(&unit)
        );
        let r = self.remote.execute(host, &RemoteRequest { script, sudo: false, timeout_secs: 20 })?;
        if !r.ok() {
            return Err(NhError::Failed(format!("systemctl show falhou: {}", if r.message.is_empty() { r.stderr.trim().to_string() } else { r.message })));
        }
        Ok(parse_show(&unit, &r.stdout))
    }

    pub fn list(&self, host: &Host) -> Res<Done<Vec<SystemdService>>> {
        let params = Params::new();
        let asmt = self.risk.assess("service.list", &params);
        self.guard.authorize("service.list", Some(host), Some((cap::SERVICE_MANAGEMENT, CapabilityState::Authenticated)), &params, &asmt, &Confirmation::None, &host.identity_tokens(), "listar serviços")?;
        let script = "systemctl list-units --type=service --all --no-legend --no-pager --plain 2>/dev/null".to_string();
        let r = self.remote.execute(host, &RemoteRequest { script, sudo: false, timeout_secs: 25 })?;
        let list = if r.ok() { parse_list(&r.stdout) } else { vec![] };
        let rec = self.audit.record(
            AuditEntry::new("service.list", Some(host), RiskLevel::Low, "systemctl list-units --type=service --all")
                .from_result(&r)
                .evidence(serde_json::json!({ "services": list.len(), "plan_hash": plan_hash("service.list", Some(host.id), &params) })),
        )?;
        if !r.ok() {
            return Err(NhError::Failed(format!("falha ao listar serviços: {}", r.message)));
        }
        Ok(Done { value: list, audit: Some(rec) })
    }

    pub fn status(&self, host: &Host, name: &str) -> Res<Done<ServiceState>> {
        let unit = check_name(name)?;
        let mut params = Params::new();
        params.insert("service".into(), unit.clone());
        let asmt = self.risk.assess("service.status", &params);
        self.guard.authorize("service.status", Some(host), Some((cap::SERVICE_MANAGEMENT, CapabilityState::Authenticated)), &params, &asmt, &Confirmation::None, &host.identity_tokens(), &format!("systemctl show {unit}"))?;
        let st = self.status_raw(host, &unit);
        let (status, ev) = match &st {
            Ok(s) => (OperationStatus::Success, serde_json::json!({ "state": s })),
            Err(e) => (OperationStatus::Failed, serde_json::json!({ "error": e.to_string() })),
        };
        let rec = self.audit.record(AuditEntry::new("service.status", Some(host), RiskLevel::Low, &format!("systemctl show {unit}")).status(status).evidence(ev))?;
        Ok(Done { value: st?, audit: Some(rec) })
    }

    /// Núcleo do reinício: estado antes → restart → estado depois → validação. Sem guard/auditoria.
    pub fn restart_raw(&self, host: &Host, name: &str) -> Res<RestartOutcome> {
        let unit = check_name(name)?;
        let before = self.status_raw(host, &unit)?;
        if !before.exists() {
            // serviço inexistente: NÃO afirmamos reinício bem-sucedido (UC-05)
            return Ok(RestartOutcome {
                service: unit.clone(),
                before,
                after: None,
                exec: None,
                validation: ValidationResult { passed: false, checks: vec![Check { name: "serviço existe".into(), passed: false, detail: "unidade não encontrada (not-found)".into() }] },
                status: OperationStatus::Failed,
                message: format!("serviço '{unit}' não existe neste host: nada foi reiniciado"),
            });
        }
        let exec = self.remote.execute(host, &RemoteRequest { script: format!("systemctl restart {} 2>&1", sh_quote(&unit)), sudo: true, timeout_secs: 90 })?;
        // aguarda o serviço estabilizar (até ~5 s) antes de validar
        let mut after = self.status_raw(host, &unit).ok();
        for _ in 0..5 {
            if after.as_ref().map(|a| a.is_active()).unwrap_or(false) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            after = self.status_raw(host, &unit).ok();
        }
        let validation = match &after {
            Some(a) => self.validator.validate_restart(&before, a, &exec),
            None => ValidationResult { passed: false, checks: vec![Check { name: "estado posterior consultável".into(), passed: false, detail: "não foi possível consultar o serviço após o reinício".into() }] },
        };
        let (status, message) = if validation.passed {
            (OperationStatus::Success, format!("'{unit}' reiniciado e validado"))
        } else if !exec.ok() {
            (OperationStatus::Failed, format!("restart falhou: {}", if exec.stdout.trim().is_empty() { exec.message.clone() } else { exec.stdout.trim().to_string() }))
        } else {
            (OperationStatus::Failed, "comando executado, mas a VALIDAÇÃO pós-ação falhou".to_string())
        };
        Ok(RestartOutcome { service: unit, before, after, exec: Some(exec), validation, status, message })
    }

    pub fn restart(&self, host: &Host, name: &str, conf: &Confirmation) -> Res<Done<RestartOutcome>> {
        let unit = check_name(name)?;
        let mut params = Params::new();
        params.insert("service".into(), unit.clone());
        let asmt = self.risk.assess("service.restart", &params);
        let cmd = format!("sudo -n systemctl restart {unit}");
        self.guard.authorize("service.restart", Some(host), Some((cap::SERVICE_MANAGEMENT, CapabilityState::Authorized)), &params, &asmt, conf, &host.identity_tokens(), &cmd)?;
        let out = self.restart_raw(host, &unit)?;
        let mut e = AuditEntry::new("service.restart", Some(host), asmt.risk, &cmd).status(out.status);
        if let Some(x) = &out.exec {
            e = e.from_result(x).status(out.status);
        }
        let rec = self.audit.record(e.evidence(serde_json::json!({
            "risk_reasons": asmt.reasons,
            "plan_hash": plan_hash("service.restart", Some(host.id), &params),
            "before": out.before,
            "after": out.after,
            "validation": out.validation.to_json(),
            "message": out.message,
        })))?;
        Ok(Done { value: out, audit: Some(rec) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_show_and_list() {
        let s = parse_show("nginx.service", "LoadState=loaded\nActiveState=active\nSubState=running\nMainPID=812\nActiveEnterTimestampMonotonic=12345\n");
        assert!(s.exists() && s.is_active());
        assert_eq!(s.main_pid, 812);
        let nf = parse_show("x.service", "LoadState=not-found\nActiveState=inactive\nSubState=dead\nMainPID=0\n");
        assert!(!nf.exists());
        let l = parse_list("nginx.service loaded active running A high performance web server\ncups.service loaded inactive dead CUPS\nghost.service not-found inactive dead ghost.service\n");
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].description, "A high performance web server");
    }
}
