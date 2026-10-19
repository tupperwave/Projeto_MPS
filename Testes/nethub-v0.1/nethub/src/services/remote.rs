//! RemoteCommandService: execução não interativa via SSH, com risco, confirmação e auditoria (RF-05).
use super::audit::{AuditEntry, AuditService};
use super::guard::Guard;
use super::risk::{plan_hash, RiskPolicyService};
use super::Done;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use std::sync::Arc;

pub struct RemoteCommandService {
    remote: Arc<dyn IRemoteExecutor>,
    guard: Arc<Guard>,
    risk: Arc<RiskPolicyService>,
    audit: Arc<AuditService>,
}

impl RemoteCommandService {
    pub fn new(remote: Arc<dyn IRemoteExecutor>, guard: Arc<Guard>, risk: Arc<RiskPolicyService>, audit: Arc<AuditService>) -> Self {
        RemoteCommandService { remote, guard, risk, audit }
    }

    pub fn execute(&self, host: &Host, command: &str, timeout: u64, conf: &Confirmation) -> Res<Done<OperationResult>> {
        let command = command.trim();
        if command.is_empty() {
            return Err(NhError::Invalid("comando vazio".into()));
        }
        if command.len() > 8192 || command.contains('\0') {
            return Err(NhError::Invalid("comando muito longo ou com byte nulo".into()));
        }
        let mut params = Params::new();
        params.insert("command".into(), command.into());
        let asmt = self.risk.assess("cmd.exec", &params);
        self.guard.authorize(
            "cmd.exec",
            Some(host),
            Some((cap::REMOTE_SHELL, CapabilityState::Authenticated)),
            &params,
            &asmt,
            conf,
            &host.identity_tokens(),
            command,
        )?;
        let res = self.remote.execute(host, &RemoteRequest { script: command.into(), sudo: false, timeout_secs: timeout })?;
        let rec = self.audit.record(
            AuditEntry::new("cmd.exec", Some(host), asmt.risk, command).from_result(&res).evidence(serde_json::json!({
                "risk_reasons": asmt.reasons,
                "plan_hash": plan_hash("cmd.exec", Some(host.id), &params),
                "failure": res.failure.map(|f| format!("{f:?}")),
                "message": res.message,
                "confirmation": format!("{:?}", conf),
            })),
        )?;
        Ok(Done { value: res, audit: Some(rec) })
    }
}
