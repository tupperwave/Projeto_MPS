//! NetworkDiscoveryService: Nmap → inventário SQLite (RF-01, UC-01).
use super::audit::{AuditEntry, AuditService};
use super::capability::CapabilityService;
use super::guard::Guard;
use super::risk::{plan_hash, RiskPolicyService};
use super::Done;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use crate::util::parse_target;
use std::sync::Arc;

pub fn scan_params(req: &ScanRequest) -> Params {
    let mut p = Params::new();
    p.insert("target".into(), req.target.trim().into());
    p.insert("ports".into(), req.ports.clone().unwrap_or_default());
    p.insert("detect".into(), req.service_detection.to_string());
    p.insert("assume_up".into(), req.assume_up.to_string());
    p
}

pub fn scan_request(p: &Params) -> ScanRequest {
    let g = |k: &str| p.get(k).cloned().unwrap_or_default();
    ScanRequest {
        target: g("target"),
        ports: Some(g("ports")).filter(|s| !s.is_empty()),
        service_detection: g("detect") != "false",
        assume_up: g("assume_up") == "true",
    }
}

pub struct NetworkDiscoveryService {
    scanner: Arc<dyn INetworkScanner>,
    hosts: Arc<dyn IHostRepository>,
    caps: Arc<CapabilityService>,
    guard: Arc<Guard>,
    risk: Arc<RiskPolicyService>,
    audit: Arc<AuditService>,
}

impl NetworkDiscoveryService {
    pub fn new(scanner: Arc<dyn INetworkScanner>, hosts: Arc<dyn IHostRepository>, caps: Arc<CapabilityService>, guard: Arc<Guard>, risk: Arc<RiskPolicyService>, audit: Arc<AuditService>) -> Self {
        NetworkDiscoveryService { scanner, hosts, caps, guard, risk, audit }
    }

    pub fn discover(&self, req: &ScanRequest, conf: &Confirmation) -> Res<Done<ScanResult>> {
        parse_target(&req.target).map_err(NhError::Invalid)?;
        let params = scan_params(req);
        let asmt = self.risk.assess("discovery.scan", &params);
        let target = req.target.trim().to_string();
        let desc = format!("nmap {target}");
        self.guard.authorize("discovery.scan", None, None, &params, &asmt, conf, &[target.clone()], &desc)?;
        let hash = plan_hash("discovery.scan", None, &params);

        match self.scanner.scan(req) {
            Err(e) => {
                self.audit.record(
                    AuditEntry::new("discovery.scan", None, asmt.risk, &desc)
                        .status(OperationStatus::Failed)
                        .evidence(serde_json::json!({ "target": target, "error": e.to_string(), "plan_hash": hash })),
                )?;
                Err(e)
            }
            Ok(res) if !res.complete => {
                // resultado incompleto não é apresentado nem gravado como se fosse completo
                self.audit.record(
                    AuditEntry::new("discovery.scan", None, asmt.risk, &res.command)
                        .status(OperationStatus::Failed)
                        .evidence(serde_json::json!({ "target": target, "error": "resultado incompleto descartado", "plan_hash": hash })),
                )?;
                Err(NhError::Failed("varredura incompleta: resultado descartado, inventário não alterado".into()))
            }
            Ok(res) => {
                let ids = self.hosts.upsert_discovered(&res.hosts, &target)?;
                for id in &ids {
                    self.caps.derive_baseline(*id)?;
                }
                let rec = self.audit.record(
                    AuditEntry::new("discovery.scan", None, asmt.risk, &res.command).duration((res.elapsed_secs * 1000.0) as u64).evidence(serde_json::json!({
                        "target": target,
                        "hosts_found": res.hosts.iter().map(|h| &h.ip).collect::<Vec<_>>(),
                        "complete": res.complete,
                        "plan_hash": hash,
                        "risk_reasons": asmt.reasons,
                    })),
                )?;
                Ok(Done { value: res, audit: Some(rec) })
            }
        }
    }
}
