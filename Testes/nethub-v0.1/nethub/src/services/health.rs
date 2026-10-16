//! HealthCheckService: indicadores determinísticos de saúde (RF-11). Sem IA, sem pesos mágicos.
//! Métrica não coletada aparece como INDISPONÍVEL — nunca como normal (UC-04).
use super::audit::{AuditEntry, AuditService};
use super::guard::Guard;
use super::risk::{plan_hash, Assessment};
use super::Done;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use crate::util::now_iso;
use std::sync::Arc;

pub const DISK_WARN: f64 = 80.0;
pub const DISK_CRIT: f64 = 90.0;
pub const MEM_WARN: f64 = 85.0;
pub const MEM_CRIT: f64 = 95.0;
pub const LOAD_WARN: f64 = 1.0; // por CPU
pub const LOAD_CRIT: f64 = 2.0;

pub const HEALTH_SCRIPT: &str = r##"
echo "LOAD=$(cut -d' ' -f1-3 /proc/loadavg 2>/dev/null)"
echo "CPUS=$(nproc 2>/dev/null || grep -c '^processor' /proc/cpuinfo 2>/dev/null)"
echo "MEM=$(awk '/^MemTotal:/{t=$2} /^MemAvailable:/{a=$2} END{if(t>0) print t, a}' /proc/meminfo 2>/dev/null)"
echo "UPTIME=$(cut -d' ' -f1 /proc/uptime 2>/dev/null)"
df -P -k / /var /home 2>/dev/null | awk 'NR>1 && !seen[$6]++ {print "DISK=" $6 " " $2 " " $3}'
if [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1; then
  echo "SYSTEMD=yes"
  echo "SYSTEMD_STATE=$(systemctl is-system-running 2>/dev/null | head -n1)"
  systemctl list-units --type=service --state=failed --no-legend --no-pager --plain 2>/dev/null | awk '{print "FAILED=" $1}'
else
  echo "SYSTEMD=no"
fi
"##;

pub struct HealthCheckService {
    remote: Arc<dyn IRemoteExecutor>,
    guard: Arc<Guard>,
    audit: Arc<AuditService>,
}

fn metric(name: &str, value: Option<f64>, unit: &str, status: MetricStatus, detail: impl Into<String>) -> Metric {
    Metric { name: name.into(), value, unit: unit.into(), status, detail: detail.into() }
}

fn unavailable(name: &str, why: &str) -> Metric {
    metric(name, None, "", MetricStatus::Unavailable, why)
}

fn level(v: f64, warn: f64, crit: f64) -> MetricStatus {
    if v >= crit {
        MetricStatus::Critical
    } else if v >= warn {
        MetricStatus::Warn
    } else {
        MetricStatus::Ok
    }
}

/// Converte a saída do script em relatório. Pura e testável.
pub fn evaluate(label: &str, out: &str) -> HealthReport {
    let mut load: Option<Vec<f64>> = None;
    let mut cpus: Option<f64> = None;
    let mut mem: Option<(f64, f64)> = None;
    let mut uptime: Option<f64> = None;
    let mut disks: Vec<(String, f64, f64)> = vec![];
    let mut failed: Vec<String> = vec![];
    let mut systemd: Option<bool> = None;
    let mut sys_state: Option<String> = None;

    for line in out.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim();
        match k.trim() {
            "LOAD" => {
                let p: Vec<f64> = v.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                if p.len() == 3 {
                    load = Some(p);
                }
            }
            "CPUS" => cpus = v.parse().ok().filter(|c: &f64| *c >= 1.0),
            "MEM" => {
                let p: Vec<f64> = v.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                if p.len() == 2 && p[0] > 0.0 {
                    mem = Some((p[0], p[1]));
                }
            }
            "UPTIME" => uptime = v.parse().ok(),
            "DISK" => {
                let p: Vec<&str> = v.split_whitespace().collect();
                if p.len() == 3 {
                    if let (Ok(t), Ok(u)) = (p[1].parse::<f64>(), p[2].parse::<f64>()) {
                        if t > 0.0 {
                            disks.push((p[0].to_string(), t, u));
                        }
                    }
                }
            }
            "FAILED" => failed.push(v.to_string()),
            "SYSTEMD" => systemd = Some(v == "yes"),
            "SYSTEMD_STATE" => sys_state = Some(v.to_string()).filter(|s| !s.is_empty()),
            _ => {}
        }
    }

    let mut m: Vec<Metric> = vec![];
    match &load {
        Some(l) => {
            let per_cpu = l[0] / cpus.unwrap_or(1.0);
            let st = level(per_cpu, LOAD_WARN, LOAD_CRIT);
            m.push(metric(
                "carga",
                Some(l[0]),
                "",
                st,
                format!("{:.2} {:.2} {:.2} em {} CPU(s){}", l[0], l[1], l[2], cpus.map(|c| c as u32).unwrap_or(1), if cpus.is_none() { " (nº de CPUs desconhecido)" } else { "" }),
            ));
        }
        None => m.push(unavailable("carga", "não foi possível ler /proc/loadavg")),
    }
    match mem {
        Some((t, a)) => {
            let used = (t - a) / t * 100.0;
            m.push(metric("memória", Some(used), "%", level(used, MEM_WARN, MEM_CRIT), format!("{:.0}% usada ({:.1} de {:.1} GiB)", used, (t - a) / 1048576.0, t / 1048576.0)));
        }
        None => m.push(unavailable("memória", "MemAvailable não disponível em /proc/meminfo")),
    }
    if disks.is_empty() {
        m.push(unavailable("disco", "df não retornou dados"));
    }
    for (mount, t, u) in &disks {
        let pct = u / t * 100.0;
        m.push(metric(&format!("disco {mount}"), Some(pct), "%", level(pct, DISK_WARN, DISK_CRIT), format!("{:.0}% usado ({:.1} de {:.1} GiB)", pct, u / 1048576.0, t / 1048576.0)));
    }
    match uptime {
        Some(s) => m.push(metric("uptime", Some(s / 86400.0), "dias", MetricStatus::Ok, format!("{:.1} dias", s / 86400.0))),
        None => m.push(unavailable("uptime", "não foi possível ler /proc/uptime")),
    }
    match systemd {
        Some(true) => {
            let n = failed.len();
            m.push(metric("serviços com falha", Some(n as f64), "", if n == 0 { MetricStatus::Ok } else { MetricStatus::Warn }, if n == 0 { "nenhuma unidade em falha".to_string() } else { failed.join(", ") }));
            match sys_state.as_deref() {
                Some("running") => m.push(metric("estado do systemd", None, "", MetricStatus::Ok, "running")),
                Some(s) => m.push(metric("estado do systemd", None, "", MetricStatus::Warn, s.to_string())),
                None => m.push(unavailable("estado do systemd", "is-system-running sem resposta")),
            }
        }
        _ => m.push(unavailable("serviços systemd", "systemd não detectado neste host")),
    }

    let unavailable_n = m.iter().filter(|x| x.status == MetricStatus::Unavailable).count();
    let overall = m.iter().map(|x| x.status).filter(|s| *s >= MetricStatus::Warn).max().unwrap_or(MetricStatus::Ok);
    let bad: Vec<String> = m.iter().filter(|x| x.status >= MetricStatus::Warn).map(|x| format!("{}: {}", x.name, x.detail)).collect();
    let mut summary = if overall == MetricStatus::Ok {
        "Host operacional: métricas coletadas dentro dos limites.".to_string()
    } else {
        format!("{}: {}", overall.as_str(), bad.join("; "))
    };
    if unavailable_n > 0 {
        summary.push_str(&format!(" {unavailable_n} métrica(s) indisponível(is) — não presumidas normais."));
    }
    HealthReport { host: label.to_string(), collected_at: now_iso(), metrics: m, overall, unavailable: unavailable_n, summary }
}

impl HealthCheckService {
    pub fn new(remote: Arc<dyn IRemoteExecutor>, guard: Arc<Guard>, audit: Arc<AuditService>) -> Self {
        HealthCheckService { remote, guard, audit }
    }

    /// Coleta sem guard/auditoria — usada dentro de workflows (que registram a própria evidência).
    pub fn collect_raw(&self, host: &Host) -> Res<(HealthReport, OperationResult)> {
        let res = self.remote.execute(host, &RemoteRequest { script: HEALTH_SCRIPT.into(), sudo: false, timeout_secs: 30 })?;
        if !res.ok() {
            return Err(NhError::Failed(format!("coleta de saúde falhou: {}", if res.message.is_empty() { res.stderr.trim().to_string() } else { res.message.clone() })));
        }
        Ok((evaluate(&host.label(), &res.stdout), res))
    }

    pub fn collect(&self, host: &Host) -> Res<Done<HealthReport>> {
        let params = Params::new();
        let asmt = Assessment { risk: RiskLevel::Low, reasons: vec!["somente leitura".into()] };
        self.guard.authorize("health.check", Some(host), Some((cap::HEALTH_CHECK, CapabilityState::Authenticated)), &params, &asmt, &Confirmation::None, &host.identity_tokens(), "health check")?;
        match self.collect_raw(host) {
            Ok((report, res)) => {
                let rec = self.audit.record(
                    AuditEntry::new("health.check", Some(host), RiskLevel::Low, "health check (/proc, df, systemctl)")
                        .from_result(&res)
                        .evidence(serde_json::json!({
                            "plan_hash": plan_hash("health.check", Some(host.id), &params),
                            "overall": report.overall.as_str(),
                            "summary": report.summary,
                            "metrics": report.metrics,
                        })),
                )?;
                Ok(Done { value: report, audit: Some(rec) })
            }
            Err(e) => {
                self.audit.record(
                    AuditEntry::new("health.check", Some(host), RiskLevel::Low, "health check")
                        .status(OperationStatus::Failed)
                        .evidence(serde_json::json!({ "error": e.to_string() })),
                )?;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_host() {
        let r = evaluate("h", "LOAD=0.10 0.20 0.30\nCPUS=4\nMEM=8000000 6000000\nUPTIME=86400\nDISK=/ 100000000 40000000\nSYSTEMD=yes\nSYSTEMD_STATE=running\n");
        assert_eq!(r.overall, MetricStatus::Ok);
        assert_eq!(r.unavailable, 0);
    }

    #[test]
    fn thresholds_and_unavailable() {
        let r = evaluate("h", "LOAD=9.0 1 1\nCPUS=2\nMEM=1000 40\nDISK=/ 100 85\nSYSTEMD=yes\nSYSTEMD_STATE=degraded\nFAILED=x.service\n");
        let get = |n: &str| r.metrics.iter().find(|m| m.name == n).unwrap().status;
        assert_eq!(get("carga"), MetricStatus::Critical);
        assert_eq!(get("memória"), MetricStatus::Critical);
        assert_eq!(get("disco /"), MetricStatus::Warn);
        assert_eq!(get("serviços com falha"), MetricStatus::Warn);
        assert_eq!(r.overall, MetricStatus::Critical);
        // sem dados: indisponível, nunca "normal"
        let empty = evaluate("h", "");
        assert!(empty.metrics.iter().all(|m| m.status == MetricStatus::Unavailable));
        assert_eq!(empty.unavailable, empty.metrics.len());
        assert!(empty.summary.contains("indisponível"));
    }
}
