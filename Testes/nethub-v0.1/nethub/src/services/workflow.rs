//! WorkflowService: operações compostas com pré-condições, passos validados, política de falha e
//! uma única evidência encadeada com o resultado de cada etapa (RF-07/RF-08, UC-07).
//! Rollback genérico NÃO existe no MVP: a política COMPENSATE é reservada e tratada como STOP.
use super::audit::{AuditEntry, AuditService};
use super::catalog::{base_risk, shortfall};
use super::guard::Guard;
use super::health::HealthCheckService;
use super::risk::{Assessment, RiskPolicyService};
use super::systemd::ServiceManagementService;
use super::Done;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use crate::util::valid_unit_name;
use std::sync::Arc;
use std::time::Instant;

pub fn builtin_workflows() -> Vec<Workflow> {
    let step = |order: u32, op: &str, desc: &str, params: &[(&str, &str)]| WorkflowStep {
        order,
        operation_id: op.into(),
        description: desc.into(),
        params: params.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
    };
    vec![
        Workflow {
            id: "restart-service-validated".into(),
            name: "Reiniciar serviço com validação".into(),
            description: "Confirma privilégio e existência do serviço, reinicia e prova que reiniciou (ativo + PID/instante mudou)".into(),
            failure_policy: FailurePolicy::Stop,
            params: vec![("service".into(), "Nome do serviço systemd".into())],
            requires_capability: cap::SERVICE_MANAGEMENT.into(),
            requires_state: CapabilityState::Authorized,
            steps: vec![
                step(1, "capability.check", "Confirmar privilégio para alterar serviços (sudo/root)", &[("capability", cap::SERVICE_MANAGEMENT), ("level", "AUTHORIZED")]),
                step(2, "service.status", "Registrar estado anterior e confirmar que {service} existe", &[("service", "{service}"), ("assert", "exists")]),
                step(3, "service.restart", "Reiniciar {service} e validar o resultado", &[("service", "{service}")]),
                step(4, "health.check", "Coletar a saúde do host depois da mudança (informativo)", &[]),
            ],
        },
        Workflow {
            id: "pre-update-check".into(),
            name: "Verificação pré-atualização".into(),
            description: "Somente leitura (Debian/Ubuntu): decide GO/NO-GO antes de atualizar pacotes".into(),
            failure_policy: FailurePolicy::Stop,
            params: vec![],
            requires_capability: cap::REMOTE_SHELL.into(),
            requires_state: CapabilityState::Authenticated,
            steps: vec![
                step(1, "capability.check", "Confirmar acesso SSH autenticado", &[("capability", cap::REMOTE_SHELL), ("level", "AUTHENTICATED")]),
                step(2, "health.check", "Disco e memória dentro dos limites", &[("gate", "disk,memory")]),
                step(3, "precheck.update", "Travas do apt/dpkg, espaço, pacotes pendentes, reinício pendente, retidos e unidades com falha", &[]),
            ],
        },
    ]
}

const PRECHECK_SCRIPT: &str = r##"
if ! command -v apt-get >/dev/null 2>&1; then echo "PKG=none"; exit 0; fi
echo "PKG=apt"
if command -v pgrep >/dev/null 2>&1; then
  if pgrep -x 'apt|apt-get|dpkg|aptitude|unattended-upgr' >/dev/null 2>&1; then echo "LOCK=busy"; else echo "LOCK=free"; fi
else echo "LOCK=unknown"; fi
SIM=$(apt-get -s upgrade 2>/dev/null | grep '^Inst ')
echo "UPG=$(printf '%s\n' "$SIM" | grep -c '^Inst ')"
echo "SEC=$(printf '%s\n' "$SIM" | grep -ci security)"
if [ -f /var/run/reboot-required ]; then echo "REBOOT=yes"; else echo "REBOOT=no"; fi
echo "HOLD=$(apt-mark showhold 2>/dev/null | wc -l)"
echo "AUDIT=$(dpkg --audit 2>/dev/null | wc -l)"
echo "FREE_ROOT_KB=$(df -P -k / 2>/dev/null | awk 'NR==2{print $4}')"
echo "FREE_VAR_KB=$(df -P -k /var 2>/dev/null | awk 'NR==2{print $4}')"
echo "LISTS_AGE=$(( ($(date +%s) - $(stat -c %Y /var/lib/apt/lists 2>/dev/null || date +%s)) / 86400 ))"
if [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1; then
  echo "FAILED=$(systemctl list-units --type=service --state=failed --no-legend --no-pager --plain 2>/dev/null | wc -l)"
fi
"##;

#[derive(Clone, Copy, PartialEq)]
enum Lvl {
    Pass,
    Warn,
    Fail,
}

/// Avalia a saída do script de pré-atualização. Pura e testável.
pub fn evaluate_precheck(out: &str) -> (StepStatus, String, Vec<String>) {
    let kv: std::collections::BTreeMap<String, String> =
        out.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.trim().to_string(), v.trim().to_string())).collect();
    let num = |k: &str| kv.get(k).and_then(|v| v.parse::<i64>().ok());
    let mut items: Vec<(Lvl, String)> = vec![];

    if kv.get("PKG").map(|s| s.as_str()) != Some("apt") {
        let d = vec!["✗ gerenciador de pacotes não suportado (o MVP cobre apt/Debian/Ubuntu)".to_string()];
        return (StepStatus::Failed, "NO-GO: host fora do escopo suportado".into(), d);
    }
    match kv.get("LOCK").map(|s| s.as_str()) {
        Some("free") => items.push((Lvl::Pass, "apt/dpkg livres (nenhum processo de pacotes em execução)".into())),
        Some("busy") => items.push((Lvl::Fail, "apt/dpkg em uso por outro processo".into())),
        _ => items.push((Lvl::Warn, "não foi possível verificar travas do apt (pgrep ausente)".into())),
    }
    match num("AUDIT") {
        Some(0) => items.push((Lvl::Pass, "dpkg consistente (dpkg --audit sem pendências)".into())),
        Some(n) => items.push((Lvl::Fail, format!("dpkg reporta pacotes inconsistentes ({n} linhas em dpkg --audit)"))),
        None => items.push((Lvl::Warn, "dpkg --audit indisponível".into())),
    }
    for (key, label, fail_mb, warn_mb) in [("FREE_ROOT_KB", "/", 300, 1024), ("FREE_VAR_KB", "/var", 200, 500)] {
        match num(key) {
            Some(kb) => {
                let mb = kb / 1024;
                let lvl = if mb < fail_mb { Lvl::Fail } else if mb < warn_mb { Lvl::Warn } else { Lvl::Pass };
                items.push((lvl, format!("espaço livre em {label}: {mb} MiB")));
            }
            None => items.push((Lvl::Warn, format!("espaço livre em {label} indisponível"))),
        }
    }
    match (num("UPG"), num("SEC")) {
        (Some(u), s) => items.push((Lvl::Pass, format!("{u} pacote(s) pendente(s){}", s.map(|s| format!(" ({s} de segurança)")).unwrap_or_default()))),
        _ => items.push((Lvl::Warn, "lista de pacotes pendentes indisponível".into())),
    }
    if let Some(age) = num("LISTS_AGE") {
        if age > 7 {
            items.push((Lvl::Warn, format!("índices do apt com {age} dias: a contagem de pendentes pode estar desatualizada (apt update)")));
        }
    }
    if kv.get("REBOOT").map(|s| s == "yes").unwrap_or(false) {
        items.push((Lvl::Warn, "reinício já pendente (/var/run/reboot-required)".into()));
    }
    if let Some(n) = num("HOLD") {
        if n > 0 {
            items.push((Lvl::Warn, format!("{n} pacote(s) retido(s) (apt-mark hold)")));
        }
    }
    match num("FAILED") {
        Some(0) => items.push((Lvl::Pass, "nenhuma unidade systemd em falha".into())),
        Some(n) => items.push((Lvl::Warn, format!("{n} unidade(s) systemd em falha"))),
        None => {}
    }
    let details: Vec<String> = items
        .iter()
        .map(|(l, t)| format!("{} {}", match l { Lvl::Pass => "✓", Lvl::Warn => "!", Lvl::Fail => "✗" }, t))
        .collect();
    if items.iter().any(|(l, _)| *l == Lvl::Fail) {
        (StepStatus::Failed, "NO-GO: há impedimentos para atualizar agora".into(), details)
    } else if items.iter().any(|(l, _)| *l == Lvl::Warn) {
        (StepStatus::Warning, "GO com ressalvas: revise os alertas".into(), details)
    } else {
        (StepStatus::Passed, "GO: nenhum impedimento encontrado".into(), details)
    }
}

pub struct WorkflowService {
    repo: Arc<dyn IWorkflowRepository>,
    hosts: Arc<dyn IHostRepository>,
    remote: Arc<dyn IRemoteExecutor>,
    health: Arc<HealthCheckService>,
    services: Arc<ServiceManagementService>,
    guard: Arc<Guard>,
    risk: Arc<RiskPolicyService>,
    audit: Arc<AuditService>,
}

fn subst(s: &str, p: &Params) -> String {
    let mut out = s.to_string();
    for (k, v) in p {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

fn step_params(step: &WorkflowStep, p: &Params) -> Params {
    step.params.iter().map(|(k, v)| (k.clone(), subst(v, p))).collect()
}

impl WorkflowService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repo: Arc<dyn IWorkflowRepository>,
        hosts: Arc<dyn IHostRepository>,
        remote: Arc<dyn IRemoteExecutor>,
        health: Arc<HealthCheckService>,
        services: Arc<ServiceManagementService>,
        guard: Arc<Guard>,
        risk: Arc<RiskPolicyService>,
        audit: Arc<AuditService>,
    ) -> Self {
        WorkflowService { repo, hosts, remote, health, services, guard, risk, audit }
    }

    pub fn list(&self) -> Res<Vec<Workflow>> {
        self.repo.list()
    }

    pub fn find(&self, id: &str) -> Res<Workflow> {
        let id = id.strip_prefix("workflow.").unwrap_or(id);
        self.repo.find(id)?.ok_or_else(|| NhError::NotFound(format!("workflow '{id}'")))
    }

    pub fn resolve_params(&self, w: &Workflow, given: &Params) -> Res<Params> {
        for (k, q) in &w.params {
            let v = given.get(k).map(|s| s.trim()).unwrap_or("");
            if v.is_empty() {
                return Err(NhError::Invalid(format!("parâmetro '{k}' ({q}) é obrigatório para o workflow '{}'", w.id)));
            }
            if k == "service" && !valid_unit_name(v) {
                return Err(NhError::Invalid(format!("nome de serviço inválido: '{v}'")));
            }
        }
        Ok(given.clone())
    }

    /// Risco do workflow = maior risco entre as etapas (com os parâmetros resolvidos).
    pub fn assess(&self, w: &Workflow, params: &Params) -> Assessment {
        let mut risk = RiskLevel::Low;
        let mut reasons: Vec<String> = vec![];
        for s in &w.steps {
            let sp = step_params(s, params);
            let a = if s.operation_id == "service.restart" { self.risk.assess("service.restart", &sp) } else { Assessment { risk: base_risk(&s.operation_id), reasons: vec![] } };
            risk = risk.max(a.risk);
            for r in a.reasons {
                if !reasons.contains(&r) {
                    reasons.push(r);
                }
            }
        }
        if reasons.is_empty() {
            reasons.push("workflow somente leitura".into());
        }
        Assessment { risk, reasons }
    }

    pub fn step_risk(&self, step: &WorkflowStep, params: &Params) -> RiskLevel {
        let sp = step_params(step, params);
        if step.operation_id == "service.restart" {
            self.risk.assess("service.restart", &sp).risk
        } else {
            base_risk(&step.operation_id)
        }
    }

    pub fn describe_step(&self, step: &WorkflowStep, params: &Params) -> String {
        subst(&step.description, params)
    }

    fn run_step(&self, host: &Host, step: &WorkflowStep, params: &Params) -> StepResult {
        let t0 = Instant::now();
        let sp = step_params(step, params);
        let g = |k: &str| sp.get(k).map(|s| s.as_str()).unwrap_or("");
        let mut sr = StepResult {
            order: step.order,
            operation_id: step.operation_id.clone(),
            description: subst(&step.description, params),
            status: StepStatus::Passed,
            message: String::new(),
            details: vec![],
            evidence: serde_json::json!({}),
            duration_ms: 0,
        };
        let fail = |sr: &mut StepResult, m: String| {
            sr.status = StepStatus::Failed;
            sr.message = m;
        };
        match step.operation_id.as_str() {
            "capability.check" => {
                let fresh = self.hosts.get(host.id).ok().flatten().unwrap_or_else(|| host.clone());
                let min = CapabilityState::parse(g("level"));
                match shortfall(&fresh, g("capability"), min) {
                    Some(why) => fail(&mut sr, why),
                    None => {
                        sr.message = format!("{}: {}", cap::title(g("capability")), fresh.cap_state(g("capability")).label());
                        sr.evidence = serde_json::json!({ "capability": g("capability"), "state": fresh.cap_state(g("capability")).as_str() });
                    }
                }
            }
            "service.status" => match self.services.status_raw(host, g("service")) {
                Ok(s) => {
                    sr.details.push(format!("{} — {}/{} (PID {})", s.name, s.active_state, s.sub_state, s.main_pid));
                    sr.evidence = serde_json::json!({ "state": s });
                    if g("assert") == "exists" && !s.exists() {
                        fail(&mut sr, format!("serviço '{}' não existe neste host", s.name));
                    } else {
                        sr.message = format!("estado anterior registrado: {}", s.active_state);
                    }
                }
                Err(e) => fail(&mut sr, e.to_string()),
            },
            "service.restart" => match self.services.restart_raw(host, g("service")) {
                Ok(o) => {
                    sr.message = o.message.clone();
                    sr.details = o.validation.checks.iter().map(|c| format!("{} {} — {}", if c.passed { "✓" } else { "✗" }, c.name, c.detail)).collect();
                    sr.evidence = serde_json::json!({ "before": o.before, "after": o.after, "validation": o.validation.to_json() });
                    if o.status != OperationStatus::Success {
                        sr.status = StepStatus::Failed;
                    }
                }
                Err(e) => fail(&mut sr, e.to_string()),
            },
            "health.check" => match self.health.collect_raw(host) {
                Ok((rep, _)) => {
                    let gates: Vec<&str> = g("gate").split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    let prefixes: Vec<&str> = gates.iter().map(|x| match *x { "disk" => "disco", "memory" => "memória", "load" => "carga", o => o }).collect();
                    let relevant: Vec<&Metric> = rep.metrics.iter().filter(|m| prefixes.is_empty() || prefixes.iter().any(|p| m.name.starts_with(p))).collect();
                    let worst = relevant.iter().map(|m| m.status).filter(|s| *s >= MetricStatus::Warn).max();
                    sr.details = relevant.iter().map(|m| format!("{} {}: {}", match m.status { MetricStatus::Ok => "✓", MetricStatus::Unavailable => "?", _ => "!" }, m.name, m.detail)).collect();
                    sr.message = rep.summary.clone();
                    sr.evidence = serde_json::json!({ "overall": rep.overall.as_str(), "summary": rep.summary });
                    match worst {
                        Some(MetricStatus::Critical) if !gates.is_empty() => sr.status = StepStatus::Failed,
                        Some(_) => sr.status = StepStatus::Warning,
                        None => {}
                    }
                }
                Err(e) => fail(&mut sr, e.to_string()),
            },
            "precheck.update" => match self.remote.execute(host, &RemoteRequest { script: PRECHECK_SCRIPT.into(), sudo: false, timeout_secs: 60 }) {
                Ok(r) if r.ok() => {
                    let (st, msg, details) = evaluate_precheck(&r.stdout);
                    sr.status = st;
                    sr.message = msg;
                    sr.details = details.clone();
                    sr.evidence = serde_json::json!({ "checks": details });
                }
                Ok(r) => fail(&mut sr, format!("verificação falhou: {}", if r.message.is_empty() { r.stderr.trim().to_string() } else { r.message })),
                Err(e) => fail(&mut sr, e.to_string()),
            },
            other => fail(&mut sr, format!("operação de etapa desconhecida: '{other}'")),
        }
        sr.duration_ms = t0.elapsed().as_millis() as u64;
        sr
    }

    pub fn execute(&self, host: &Host, id: &str, params: &Params, conf: &Confirmation) -> Res<Done<WorkflowExecution>> {
        let w = self.find(id)?;
        self.resolve_params(&w, params)?;
        let asmt = self.assess(&w, params);
        let op_id = format!("workflow.{}", w.id);
        let cmd = format!("{op_id} {}", params.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "));
        self.guard.authorize(&op_id, Some(host), Some((&w.requires_capability, w.requires_state)), params, &asmt, conf, &host.identity_tokens(), cmd.trim())?;

        let t0 = Instant::now();
        let mut results: Vec<StepResult> = vec![];
        let mut stopped = false;
        for step in &w.steps {
            if stopped {
                results.push(StepResult {
                    order: step.order,
                    operation_id: step.operation_id.clone(),
                    description: subst(&step.description, params),
                    status: StepStatus::NotExecuted,
                    message: "não executada (etapa anterior falhou)".into(),
                    details: vec![],
                    evidence: serde_json::json!({}),
                    duration_ms: 0,
                });
                continue;
            }
            let r = self.run_step(host, step, params);
            if r.status == StepStatus::Failed && w.failure_policy != FailurePolicy::Continue {
                stopped = true;
            }
            results.push(r);
        }
        let failed = results.iter().filter(|r| r.status == StepStatus::Failed).count();
        let passed = results.iter().filter(|r| matches!(r.status, StepStatus::Passed | StepStatus::Warning)).count();
        let skipped = results.iter().filter(|r| r.status == StepStatus::NotExecuted).count();
        let status = if failed == 0 { OperationStatus::Success } else { OperationStatus::Failed };
        let mut summary = format!("{passed} etapa(s) concluída(s), {failed} falha(s), {skipped} não executada(s)");
        if failed > 0 && w.failure_policy == FailurePolicy::Compensate {
            summary.push_str(" — compensação/rollback não é implementado no MVP");
        }
        let exec = WorkflowExecution { workflow_id: w.id.clone(), workflow_name: w.name.clone(), host: host.label(), status, steps: results, summary };
        let steps_json: Vec<_> = exec
            .steps
            .iter()
            .map(|s| serde_json::json!({
                "order": s.order, "operation": s.operation_id, "description": s.description, "status": s.status.as_str(),
                "message": s.message, "details": s.details, "evidence": s.evidence, "duration_ms": s.duration_ms
            }))
            .collect();
        let rec = self.audit.record(
            AuditEntry::new(&op_id, Some(host), asmt.risk, cmd.trim()).status(status).duration(t0.elapsed().as_millis() as u64).evidence(serde_json::json!({
                "workflow": w.id,
                "failure_policy": w.failure_policy.as_str(),
                "params": params,
                "risk_reasons": asmt.reasons,
                "plan_hash": super::risk::plan_hash(&op_id, Some(host.id), params),
                "steps": steps_json,
                "summary": exec.summary,
            })),
        )?;
        Ok(Done { value: exec, audit: Some(rec) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precheck_go_nogo() {
        let ok = "PKG=apt\nLOCK=free\nUPG=3\nSEC=1\nREBOOT=no\nHOLD=0\nAUDIT=0\nFREE_ROOT_KB=9000000\nFREE_VAR_KB=9000000\nLISTS_AGE=1\nFAILED=0\n";
        assert_eq!(evaluate_precheck(ok).0, StepStatus::Passed);
        let warn = ok.replace("REBOOT=no", "REBOOT=yes");
        assert_eq!(evaluate_precheck(&warn).0, StepStatus::Warning);
        let busy = ok.replace("LOCK=free", "LOCK=busy");
        assert_eq!(evaluate_precheck(&busy).0, StepStatus::Failed);
        let full = ok.replace("FREE_ROOT_KB=9000000", "FREE_ROOT_KB=100000");
        assert_eq!(evaluate_precheck(&full).0, StepStatus::Failed);
        assert_eq!(evaluate_precheck("PKG=none\n").0, StepStatus::Failed);
    }

    #[test]
    fn builtin_definitions_are_consistent() {
        for w in builtin_workflows() {
            assert!(!w.steps.is_empty());
            let orders: Vec<u32> = w.steps.iter().map(|s| s.order).collect();
            let mut sorted = orders.clone();
            sorted.sort();
            assert_eq!(orders, sorted);
        }
    }
}
