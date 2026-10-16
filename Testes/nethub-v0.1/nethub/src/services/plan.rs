//! PlanService: dry-run. Mostra exatamente o que será feito (comandos, risco, requisitos) antes de
//! executar, com capacidades reverificadas ao vivo. O dry-run é registrado como evidência e é o que
//! habilita ações de risco CRÍTICO (RN-05).
use super::audit::{AuditEntry, AuditService};
use super::capability::CapabilityService;
use super::catalog::{shortfall, ActionCatalogService};
use super::discovery::scan_request;
use super::risk::{plan_hash, Assessment, RiskPolicyService};
use super::workflow::WorkflowService;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::IHostRepository;
use crate::util::*;
use std::sync::Arc;

pub struct PlanService {
    hosts: Arc<dyn IHostRepository>,
    caps: Arc<CapabilityService>,
    catalog: Arc<ActionCatalogService>,
    workflows: Arc<WorkflowService>,
    risk: Arc<RiskPolicyService>,
    audit: Arc<AuditService>,
}

fn step(order: u32, op: &str, desc: &str, cmd: Option<String>, risk: RiskLevel) -> DryRunStep {
    DryRunStep { order, operation_id: op.into(), description: desc.into(), command: cmd, risk, requirement: None, requirement_met: None }
}

pub fn validate_params(op_id: &str, p: &Params) -> Vec<String> {
    let g = |k: &str| p.get(k).map(|s| s.trim()).unwrap_or("");
    let mut b = vec![];
    match op_id {
        "cmd.exec" => {
            if g("command").is_empty() {
                b.push("comando vazio".into());
            }
        }
        "service.status" | "service.restart" => {
            if !valid_unit_name(g("service")) {
                b.push(format!("nome de serviço inválido: '{}'", g("service")));
            }
        }
        "file.upload" => {
            if !std::path::Path::new(g("local")).is_file() {
                b.push(format!("arquivo local inexistente ou não regular: '{}'", g("local")));
            }
            if !safe_path_chars(g("local")) || !safe_path_chars(g("remote")) {
                b.push("caminho com caractere não suportado (aspas, barra invertida, quebra de linha) ou vazio".into());
            }
        }
        "file.download" => {
            if !safe_path_chars(g("local")) || !safe_path_chars(g("remote")) {
                b.push("caminho com caractere não suportado (aspas, barra invertida, quebra de linha) ou vazio".into());
            }
            if std::path::Path::new(g("local")).exists() && g("overwrite") != "true" {
                b.push(format!("'{}' já existe (use --overwrite)", g("local")));
            }
        }
        "discovery.scan" => {
            if let Err(e) = parse_target(g("target")) {
                b.push(e);
            }
        }
        _ => {}
    }
    b
}

impl PlanService {
    pub fn new(hosts: Arc<dyn IHostRepository>, caps: Arc<CapabilityService>, catalog: Arc<ActionCatalogService>, workflows: Arc<WorkflowService>, risk: Arc<RiskPolicyService>, audit: Arc<AuditService>) -> Self {
        PlanService { hosts, caps, catalog, workflows, risk, audit }
    }

    pub fn dry_run(&self, req: &ActionRequest) -> Res<DryRunReport> {
        let op = self.catalog.find(&req.operation_id)?.ok_or_else(|| NhError::NotFound(format!("operação '{}'", req.operation_id)))?;
        let mut host = if op.needs_host {
            let id = req.host_id.ok_or_else(|| NhError::Invalid("esta operação exige um host".into()))?;
            Some(self.hosts.get(id)?.ok_or_else(|| NhError::NotFound(format!("host #{id}")))?)
        } else {
            None
        };
        // reverifica capacidades ao vivo: o plano reflete o que é possível AGORA
        if let Some(h) = &host {
            if h.cap_state(cap::REMOTE_SHELL) >= CapabilityState::Reachable && op.id != "capability.inspect" {
                let _ = self.caps.inspect(h);
                host = self.hosts.get(h.id)?;
            }
        }

        let p = &req.params;
        let mut blockers: Vec<String> = vec![];
        let mut warnings: Vec<String> = vec![];
        for (k, q) in &op.params {
            if p.get(k).map(|v| v.trim().is_empty()).unwrap_or(true) {
                blockers.push(format!("parâmetro '{k}' ({q}) ausente"));
            }
        }
        if blockers.is_empty() {
            blockers.extend(validate_params(&op.id, p));
        }
        if let (Some(h), Some((c, min))) = (&host, &op.requires) {
            if let Some(why) = shortfall(h, c, *min) {
                blockers.push(why);
            }
        }

        let g = |k: &str| p.get(k).map(|s| s.trim().to_string()).unwrap_or_default();
        let mut steps: Vec<DryRunStep> = vec![];
        let asmt: Assessment;
        if op.is_workflow {
            let w = self.workflows.find(&op.id)?;
            if let Err(e) = self.workflows.resolve_params(&w, p) {
                blockers.push(e.to_string());
            }
            asmt = self.workflows.assess(&w, p);
            for s in &w.steps {
                let sp: Params = s.params.iter().map(|(k, v)| (k.clone(), v.replace("{service}", &g("service")))).collect();
                let unit = normalize_unit(sp.get("service").map(|s| s.as_str()).unwrap_or(""));
                let cmd = match s.operation_id.as_str() {
                    "service.status" => Some(format!("systemctl show {} -p LoadState -p ActiveState -p SubState -p MainPID -p ActiveEnterTimestampMonotonic", sh_quote(&unit))),
                    "service.restart" => Some(format!("sudo -n systemctl restart {}   (+ estado antes/depois e validação)", sh_quote(&unit))),
                    "health.check" => Some("script somente leitura: /proc/loadavg, /proc/meminfo, df, systemctl --failed".into()),
                    "precheck.update" => Some("apt-get -s upgrade; dpkg --audit; apt-mark showhold; df; systemctl --failed (somente leitura)".into()),
                    _ => None,
                };
                let mut ds = step(s.order, &s.operation_id, &self.workflows.describe_step(s, p), cmd, self.workflows.step_risk(s, p));
                if s.operation_id == "capability.check" {
                    let c = sp.get("capability").cloned().unwrap_or_default();
                    let min = CapabilityState::parse(sp.get("level").map(|s| s.as_str()).unwrap_or(""));
                    ds.requirement = Some(format!("{} ≥ {}", cap::title(&c), min.label()));
                    ds.requirement_met = host.as_ref().map(|h| shortfall(h, &c, min).is_none());
                }
                steps.push(ds);
            }
        } else {
            asmt = self.risk.assess(&op.id, p);
            let r = asmt.risk;
            match op.id.as_str() {
                "cmd.exec" => steps.push(step(1, &op.id, "Executar o comando via SSH (não interativo, com timeout)", Some(g("command")), r)),
                "service.restart" => {
                    let unit = normalize_unit(&g("service"));
                    steps.push(step(1, "service.status", "Registrar o estado anterior do serviço", Some(format!("systemctl show {unit} …")), RiskLevel::Low));
                    steps.push(step(2, &op.id, "Reiniciar o serviço", Some(format!("sudo -n systemctl restart {}", sh_quote(&unit))), r));
                    steps.push(step(3, "service.validate", "Validar: ativo e (PID ou instante de ativação) mudou", Some(format!("systemctl show {unit} …")), RiskLevel::Low));
                }
                "file.upload" => {
                    steps.push(step(1, "file.hash", "Calcular SHA-256 local e consultar o arquivo remoto (sobrescrita?)", Some(format!("sha256 {}; ssh {} sha256sum {}", g("local"), host.as_ref().map(|h| h.label()).unwrap_or_default(), g("remote"))), RiskLevel::Low));
                    steps.push(step(2, &op.id, "Enviar via SFTP", Some(format!("sftp put {} {}", g("local"), g("remote"))), r));
                    steps.push(step(3, "file.verify", "Comparar SHA-256 remoto com o local", None, RiskLevel::Low));
                }
                "file.download" => {
                    steps.push(step(1, "file.hash", "Calcular SHA-256 do arquivo remoto", Some(format!("sha256sum {}", g("remote"))), RiskLevel::Low));
                    steps.push(step(2, &op.id, "Baixar via SFTP", Some(format!("sftp get {} {}", g("remote"), g("local"))), r));
                    steps.push(step(3, "file.verify", "Comparar SHA-256 local com o remoto", None, RiskLevel::Low));
                }
                "discovery.scan" => {
                    let sr = scan_request(p);
                    steps.push(step(1, &op.id, "Varredura TCP (nmap -sT) e gravação no inventário", Some(format!("nmap -sT -p {} {}{}", sr.ports.clone().unwrap_or_else(|| crate::adapters::nmap::DEFAULT_PORTS.into()), sr.target, if sr.service_detection { " -sV" } else { "" })), r));
                    warnings.push("hosts do alvo que não responderem serão marcados como 'down' no inventário".into());
                }
                "health.check" => steps.push(step(1, &op.id, "Coletar métricas (somente leitura)", Some("/proc/loadavg, /proc/meminfo, df, systemctl --failed".into()), r)),
                other => steps.push(step(1, other, &op.name, None, r)),
            }
            for s in steps.iter_mut() {
                if s.operation_id == op.id {
                    if let Some((c, min)) = &op.requires {
                        s.requirement = Some(format!("{} ≥ {}", cap::title(c), min.label()));
                        s.requirement_met = host.as_ref().map(|h| shortfall(h, c, *min).is_none());
                    }
                }
            }
        }
        if let Some(h) = &host {
            if h.status == "down" {
                warnings.push("o host está marcado como 'down' desde a última descoberta".into());
            }
        }
        let highest = steps.iter().map(|s| s.risk).max().unwrap_or(asmt.risk).max(asmt.risk);
        let hash = plan_hash(&op.id, host.as_ref().map(|h| h.id).or(req.host_id), p);
        let expected = host.as_ref().map(|h| h.label()).unwrap_or_else(|| g("target"));
        let report = DryRunReport {
            operation_id: op.id.clone(),
            operation_name: op.name.clone(),
            host: host.as_ref().map(|h| h.label()),
            steps,
            blockers,
            warnings,
            highest_risk: highest,
            reasons: asmt.reasons,
            plan_hash: hash.clone(),
            confirmation: highest.confirmation_level(),
            expected_target: expected,
        };
        self.audit.record(
            AuditEntry::new("dry-run", host.as_ref(), highest, &format!("dry-run {}", op.id)).evidence(serde_json::json!({
                "plan_hash": hash,
                "operation": op.id,
                "params": p,
                "blockers": report.blockers,
                "warnings": report.warnings,
                "highest_risk": highest.as_str(),
                "steps": report.steps.iter().map(|s| serde_json::json!({"order": s.order, "op": s.operation_id, "description": s.description, "command": s.command, "risk": s.risk.as_str()})).collect::<Vec<_>>(),
            })),
        )?;
        Ok(report)
    }
}
