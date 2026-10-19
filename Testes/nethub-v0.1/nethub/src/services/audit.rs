//! AuditService: toda ação gera evidência encadeada por hash (RN-03, RNF-06).
use crate::domain::*;
use crate::error::Res;
use crate::ports::{IAuditRepository, NewAudit};
use crate::util::*;
use std::sync::Arc;

const MAX_STORED_OUTPUT: usize = 64 * 1024;
pub const DRY_RUN_TTL_SECS: i64 = 15 * 60;

pub struct AuditEntry {
    pub host_id: Option<i64>,
    pub host_label: String,
    pub operation_id: String,
    pub risk: RiskLevel,
    pub command: String,
    pub status: OperationStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub evidence: serde_json::Value,
}

impl AuditEntry {
    pub fn new(op: &str, host: Option<&Host>, risk: RiskLevel, command: &str) -> Self {
        AuditEntry {
            host_id: host.map(|h| h.id),
            host_label: host.map(|h| h.label()).unwrap_or_else(|| "-".into()),
            operation_id: op.to_string(),
            risk,
            command: command.to_string(),
            status: OperationStatus::Success,
            exit_code: None,
            duration_ms: 0,
            stdout: String::new(),
            stderr: String::new(),
            evidence: serde_json::json!({}),
        }
    }
    pub fn status(mut self, s: OperationStatus) -> Self {
        self.status = s;
        self
    }
    pub fn evidence(mut self, v: serde_json::Value) -> Self {
        self.evidence = v;
        self
    }
    pub fn duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
    pub fn from_result(mut self, r: &OperationResult) -> Self {
        self.status = r.status;
        self.exit_code = r.exit_code;
        self.duration_ms = r.duration_ms;
        self.stdout = r.stdout.clone();
        self.stderr = r.stderr.clone();
        self
    }
}

pub struct AuditService {
    repo: Arc<dyn IAuditRepository>,
}

impl AuditService {
    pub fn new(repo: Arc<dyn IAuditRepository>) -> Self {
        AuditService { repo }
    }

    pub fn record(&self, e: AuditEntry) -> Res<AuditRecord> {
        let (stdout, t1) = truncate(&e.stdout, MAX_STORED_OUTPUT);
        let (stderr, t2) = truncate(&e.stderr, MAX_STORED_OUTPUT);
        self.repo.append(&NewAudit {
            ts: now_iso(),
            actor: actor(),
            host_id: e.host_id,
            host_label: e.host_label,
            operation_id: e.operation_id,
            risk: e.risk,
            command: e.command,
            status: e.status,
            exit_code: e.exit_code,
            duration_ms: e.duration_ms as i64,
            stdout,
            // o hash cobre a saída COMPLETA, mesmo que o texto armazenado seja truncado
            stdout_sha256: sha256_hex(e.stdout.as_bytes()),
            stderr,
            stderr_sha256: sha256_hex(e.stderr.as_bytes()),
            truncated: t1 || t2,
            evidence: e.evidence.to_string(),
        })
    }

    pub fn search(&self, f: &AuditFilter) -> Res<Vec<AuditRecord>> {
        self.repo.search(f)
    }

    pub fn verify(&self) -> Res<ChainReport> {
        self.repo.verify_chain()
    }

    pub fn has_recent_dry_run(&self, plan_hash: &str) -> Res<bool> {
        self.repo.has_dry_run(plan_hash, &iso_minus_secs(DRY_RUN_TTL_SECS))
    }
}
