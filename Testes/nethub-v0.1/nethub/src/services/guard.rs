//! Guard: porta única de autorização — capacidade suficiente + risco confirmado. Nenhuma ação
//! executa sem passar por aqui (a interface não consegue pular a camada de autorização).
use super::audit::{AuditEntry, AuditService};
use super::catalog::shortfall;
use super::risk::{plan_hash, Assessment, RiskPolicyService};
use crate::domain::*;
use crate::error::{NhError, Res};
use std::sync::Arc;

pub struct Guard {
    audit: Arc<AuditService>,
    risk: Arc<RiskPolicyService>,
}

impl Guard {
    pub fn new(audit: Arc<AuditService>, risk: Arc<RiskPolicyService>) -> Self {
        Guard { audit, risk }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn authorize(
        &self,
        op: &str,
        host: Option<&Host>,
        requires: Option<(&str, CapabilityState)>,
        params: &Params,
        asmt: &Assessment,
        conf: &Confirmation,
        expected: &[String],
        command_desc: &str,
    ) -> Res<()> {
        // 1) capacidade comprovada suficiente? (RN-02/RN-04)
        if let (Some(h), Some((cap, min))) = (host, requires) {
            if let Some(why) = shortfall(h, cap, min) {
                self.audit.record(
                    AuditEntry::new(op, Some(h), asmt.risk, command_desc)
                        .status(OperationStatus::Blocked)
                        .evidence(serde_json::json!({
                            "blocked_by": "capability",
                            "capability": cap,
                            "state": h.cap_state(cap).as_str(),
                            "required": min.as_str(),
                            "reason": why,
                        })),
                )?;
                return Err(NhError::Blocked(why));
            }
        }
        // 2) confirmação proporcional ao risco (RN-01) e dry-run para crítico (RN-05)
        let hash = plan_hash(op, host.map(|h| h.id), params);
        self.risk.require(asmt, expected, conf, &hash)
    }
}
