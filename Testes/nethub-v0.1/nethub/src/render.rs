//! Renderização textual neutra (linhas com "tom") usada pela CLI (ANSI) e pela TUI (estilos).
use crate::domain::*;
use crate::services::catalog::ActionItem;
use crate::util::fmt_local;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Normal,
    Head,
    Ok,
    Warn,
    Bad,
    Dim,
}

pub type Line = (Tone, String);

fn l(t: Tone, s: impl Into<String>) -> Line {
    (t, s.into())
}

pub fn cap_tone(s: CapabilityState) -> Tone {
    match s {
        CapabilityState::Authorized | CapabilityState::Authenticated => Tone::Ok,
        CapabilityState::Reachable => Tone::Warn,
        CapabilityState::Unknown => Tone::Dim,
    }
}

pub fn risk_tone(r: RiskLevel) -> Tone {
    match r {
        RiskLevel::Low => Tone::Ok,
        RiskLevel::Medium => Tone::Warn,
        RiskLevel::High | RiskLevel::Critical => Tone::Bad,
    }
}

pub fn status_tone(s: OperationStatus) -> Tone {
    match s {
        OperationStatus::Success => Tone::Ok,
        OperationStatus::Failed => Tone::Bad,
        OperationStatus::Cancelled => Tone::Dim,
        OperationStatus::Blocked => Tone::Warn,
    }
}

fn metric_tone(s: MetricStatus) -> Tone {
    match s {
        MetricStatus::Ok => Tone::Ok,
        MetricStatus::Unavailable => Tone::Dim,
        MetricStatus::Warn => Tone::Warn,
        MetricStatus::Critical => Tone::Bad,
    }
}

fn step_tone(s: StepStatus) -> Tone {
    match s {
        StepStatus::Passed => Tone::Ok,
        StepStatus::Warning => Tone::Warn,
        StepStatus::Failed => Tone::Bad,
        StepStatus::NotExecuted => Tone::Dim,
    }
}

pub fn short_hash(h: &str) -> &str {
    &h[..12.min(h.len())]
}

pub fn host_summary(h: &Host) -> Vec<Line> {
    let mut v = vec![l(Tone::Head, format!("{}  ({})", h.label(), h.ip))];
    let st = if h.status == "up" { Tone::Ok } else if h.status == "down" { Tone::Bad } else { Tone::Dim };
    v.push(l(st, format!("status: {}   última vez visto: {}", h.status, fmt_local(&h.last_seen))));
    if let Some(os) = &h.os {
        v.push(l(Tone::Normal, format!("sistema: {os}")));
    }
    if let Some(m) = &h.mac {
        v.push(l(Tone::Normal, format!("mac: {m}")));
    }
    v.push(l(
        Tone::Normal,
        format!(
            "ssh: {}@{}:{}{}",
            h.ssh_user.clone().unwrap_or_else(|| "(usuário padrão)".into()),
            h.ip,
            h.ssh_port(),
            h.ssh_identity.as_ref().map(|k| format!("  chave: {k}")).unwrap_or_default()
        ),
    ));
    if !h.tags.is_empty() {
        v.push(l(Tone::Normal, format!("tags: {}", h.tags.iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" "))));
    }
    let open: Vec<String> = h.open_ports().map(|p| format!("{}/{}{}", p.port, if p.service.is_empty() { "?" } else { &p.service }, if p.product.is_empty() { String::new() } else { format!(" ({})", p.product) })).collect();
    v.push(l(Tone::Normal, format!("portas abertas: {}", if open.is_empty() { "nenhuma observada".into() } else { open.join(", ") })));
    v
}

pub fn host_capabilities(h: &Host) -> Vec<Line> {
    let mut v = vec![l(Tone::Head, "Capacidades (alcançável → autenticada → autorizada)")];
    for name in cap::ALL {
        let c = h.cap(name);
        let st = c.map(|c| c.state).unwrap_or(CapabilityState::Unknown);
        let when = c.and_then(|c| c.verified_at.as_ref()).map(|t| format!(" · verificado {}", fmt_local(t))).unwrap_or_else(|| " · não provado".into());
        v.push(l(cap_tone(st), format!("{:<34} {}{}", cap::title(name), st.label().to_uppercase(), when)));
        if let Some(c) = c {
            if !c.detail.is_empty() {
                v.push(l(Tone::Dim, format!("    {}", c.detail)));
            }
        }
    }
    v
}

pub fn host_actions(items: &[ActionItem]) -> Vec<Line> {
    let mut v = vec![l(Tone::Head, "Ações disponíveis (compatíveis com as capacidades provadas)")];
    let avail: Vec<&ActionItem> = items.iter().filter(|a| a.available).collect();
    if avail.is_empty() {
        v.push(l(Tone::Dim, "  nenhuma — verifique/autentique o host (nethub inspect / onboard)"));
    }
    for a in &avail {
        v.push(l(risk_tone(a.op.base_risk), format!("  ✓ {:<44} risco base: {}", a.op.name, a.op.base_risk.label())));
    }
    let blocked: Vec<&ActionItem> = items.iter().filter(|a| !a.available).collect();
    if !blocked.is_empty() {
        v.push(l(Tone::Head, "Bloqueadas (motivo)"));
        for a in blocked {
            v.push(l(Tone::Dim, format!("  ✗ {} — {}", a.op.name, a.reason.clone().unwrap_or_default())));
        }
    }
    v
}

pub fn dry_run(r: &DryRunReport) -> Vec<Line> {
    let mut v = vec![l(Tone::Head, format!("DRY-RUN · {}{}", r.operation_name, r.host.as_ref().map(|h| format!(" · {h}")).unwrap_or_default()))];
    v.push(l(risk_tone(r.highest_risk), format!("risco: {}   confirmação exigida: {}", r.highest_risk.label(), r.confirmation.describe())));
    for x in &r.reasons {
        v.push(l(Tone::Dim, format!("  motivo: {x}")));
    }
    v.push(l(Tone::Normal, ""));
    for s in &r.steps {
        v.push(l(risk_tone(s.risk), format!("{}. {}  [{}]", s.order, s.description, s.risk.label())));
        if let Some(c) = &s.command {
            v.push(l(Tone::Dim, format!("     $ {c}")));
        }
        if let Some(req) = &s.requirement {
            let met = s.requirement_met;
            v.push(l(
                match met { Some(true) => Tone::Ok, Some(false) => Tone::Bad, None => Tone::Dim },
                format!("     requisito: {req} {}", match met { Some(true) => "✓ atendido", Some(false) => "✗ NÃO atendido", None => "" }),
            ));
        }
    }
    for w in &r.warnings {
        v.push(l(Tone::Warn, format!("! {w}")));
    }
    for b in &r.blockers {
        v.push(l(Tone::Bad, format!("✗ BLOQUEIO: {b}")));
    }
    v.push(l(Tone::Dim, format!("plano {} (registrado como evidência)", r.plan_hash)));
    v
}

pub fn outcome(o: &Outcome) -> Vec<Line> {
    match o {
        Outcome::Inspect { host, capabilities, facts, message } => {
            let mut v = vec![l(Tone::Head, format!("Capacidades de {host}")), l(Tone::Normal, message.clone())];
            for c in capabilities {
                v.push(l(cap_tone(c.state), format!("{:<34} {}", cap::title(&c.name), c.state.label().to_uppercase())));
                v.push(l(Tone::Dim, format!("    {}", c.detail)));
            }
            if let Some(os) = facts.get("os") {
                v.push(l(Tone::Dim, format!("sistema: {os}")));
            }
            v
        }
        Outcome::Health(r) => {
            let mut v = vec![l(Tone::Head, format!("Saúde de {} · {}", r.host, fmt_local(&r.collected_at)))];
            for m in &r.metrics {
                let icon = match m.status { MetricStatus::Ok => "●", MetricStatus::Unavailable => "?", MetricStatus::Warn => "▲", MetricStatus::Critical => "✖" };
                v.push(l(metric_tone(m.status), format!("{icon} {:<22} {:<13} {}", m.name, m.status.as_str(), m.detail)));
            }
            v.push(l(metric_tone(r.overall), format!("Resumo: {}", r.summary)));
            v
        }
        Outcome::Exec { host, command, result } => {
            let mut v = vec![l(Tone::Head, format!("{host} $ {command}"))];
            for line in result.stdout.lines() {
                v.push(l(Tone::Normal, line));
            }
            for line in result.stderr.lines() {
                v.push(l(Tone::Warn, line));
            }
            v.push(l(
                status_tone(result.status),
                format!("exit code: {}   duração: {} ms{}", result.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "-".into()), result.duration_ms, if result.message.is_empty() { String::new() } else { format!("   {}", result.message) }),
            ));
            v
        }
        Outcome::Services(list) => {
            let mut v = vec![l(Tone::Head, format!("{} serviços", list.len()))];
            for s in list {
                let t = match s.active_state.as_str() { "active" => Tone::Ok, "failed" => Tone::Bad, _ => Tone::Dim };
                v.push(l(t, format!("{:<34} {:<10} {:<10} {}", s.name, s.active_state, s.sub_state, s.description)));
            }
            v
        }
        Outcome::ServiceStatus(s) => vec![
            l(Tone::Head, s.name.clone()),
            l(if s.is_active() { Tone::Ok } else if s.exists() { Tone::Warn } else { Tone::Bad }, format!("{} / {}   (carregado: {})", s.active_state, s.sub_state, s.load_state)),
            l(Tone::Normal, format!("PID principal: {}   ativado em (monotônico): {}", s.main_pid, s.enter_ts)),
        ],
        Outcome::Restart(r) => {
            let mut v = vec![l(Tone::Head, format!("Reinício de {}", r.service))];
            v.push(l(Tone::Normal, format!("antes : {}/{}  PID {}", r.before.active_state, r.before.sub_state, r.before.main_pid)));
            if let Some(a) = &r.after {
                v.push(l(Tone::Normal, format!("depois: {}/{}  PID {}", a.active_state, a.sub_state, a.main_pid)));
            }
            for c in &r.validation.checks {
                v.push(l(if c.passed { Tone::Ok } else { Tone::Bad }, format!("{} {} — {}", if c.passed { "✓" } else { "✗" }, c.name, c.detail)));
            }
            v.push(l(status_tone(r.status), r.message.clone()));
            v
        }
        Outcome::Transfer(t) => {
            let mut v = vec![l(Tone::Head, format!("{} · {} ⇄ {}", t.direction, t.local, t.remote))];
            v.push(l(Tone::Normal, format!("{} bytes em {} ms", t.bytes, t.duration_ms)));
            if let Some(h) = &t.local_sha256 {
                v.push(l(Tone::Dim, format!("sha256 local : {h}")));
            }
            if let Some(h) = &t.remote_sha256 {
                v.push(l(Tone::Dim, format!("sha256 remoto: {h}")));
            }
            if let Some(h) = &t.overwrote {
                v.push(l(Tone::Warn, format!("sobrescreveu arquivo existente (sha256 anterior {}…)", short_hash(h))));
            }
            v.push(l(status_tone(t.status), t.message.clone()));
            v
        }
        Outcome::Workflow(w) => {
            let mut v = vec![l(Tone::Head, format!("Workflow · {} · {}", w.workflow_name, w.host))];
            for s in &w.steps {
                let icon = match s.status { StepStatus::Passed => "✓", StepStatus::Warning => "!", StepStatus::Failed => "✗", StepStatus::NotExecuted => "·" };
                v.push(l(step_tone(s.status), format!("{icon} {}. {}", s.order, s.description)));
                if !s.message.is_empty() {
                    v.push(l(step_tone(s.status), format!("      {}", s.message)));
                }
                for d in &s.details {
                    v.push(l(Tone::Dim, format!("      {d}")));
                }
            }
            v.push(l(status_tone(w.status), format!("Resultado: {} — {}", w.status.as_str(), w.summary)));
            v
        }
        Outcome::Scan(r) => {
            let mut v = vec![l(Tone::Head, format!("Descoberta de {} · {} host(s) · {:.1}s", r.target, r.hosts.len(), r.elapsed_secs))];
            if r.hosts.is_empty() {
                v.push(l(Tone::Warn, "varredura concluída sem nenhum host ativo"));
            }
            for h in &r.hosts {
                let open: Vec<String> = h.open_ports().map(|p| format!("{}/{}", p.port, p.service)).collect();
                v.push(l(Tone::Ok, format!("{:<15} {:<20} {}", h.ip, h.hostname.clone().unwrap_or_default(), open.join(" "))));
            }
            for w in &r.warnings {
                v.push(l(Tone::Warn, w.clone()));
            }
            v
        }
    }
}

pub fn audit_record(r: &AuditRecord, full: bool) -> Vec<Line> {
    let mut v = vec![l(
        status_tone(r.status),
        format!("#{:<4} {}  {:<22} {:<16} {:<8} {}", r.id, fmt_local(&r.ts), r.operation_id, r.host_label, r.risk.as_str(), r.status.as_str()),
    )];
    if full {
        v.push(l(Tone::Normal, format!("ator: {}   exit: {}   duração: {} ms", r.actor, r.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "-".into()), r.duration_ms)));
        v.push(l(Tone::Normal, format!("comando: {}", r.command)));
        v.push(l(Tone::Dim, format!("hash anterior: {}", r.prev_hash)));
        v.push(l(Tone::Dim, format!("hash         : {}", r.hash)));
        v.push(l(Tone::Dim, format!("sha256(stdout): {}", r.stdout_sha256)));
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&r.evidence) {
            v.push(l(Tone::Head, "evidência:"));
            for line in serde_json::to_string_pretty(&val).unwrap_or_default().lines() {
                v.push(l(Tone::Dim, format!("  {line}")));
            }
        }
        if !r.stdout.is_empty() {
            v.push(l(Tone::Head, "stdout:"));
            for line in r.stdout.lines().take(60) {
                v.push(l(Tone::Normal, format!("  {line}")));
            }
        }
        if !r.stderr.is_empty() {
            v.push(l(Tone::Head, "stderr:"));
            for line in r.stderr.lines().take(30) {
                v.push(l(Tone::Warn, format!("  {line}")));
            }
        }
    }
    v
}
