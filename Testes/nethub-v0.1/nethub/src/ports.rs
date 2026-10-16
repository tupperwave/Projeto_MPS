//! Portas (contratos) entre aplicação e infraestrutura — arquitetura hexagonal.
use crate::domain::*;
use crate::error::Res;

pub trait INetworkScanner: Send + Sync {
    fn scan(&self, req: &ScanRequest) -> Res<ScanResult>;
}

pub trait IHostRepository: Send + Sync {
    /// Grava/atualiza hosts descobertos; marca como "down" os hosts do alvo não vistos. Retorna os ids.
    fn upsert_discovered(&self, hosts: &[Host], target: &str) -> Res<Vec<i64>>;
    fn insert_manual(&self, ip: &str, hostname: Option<&str>) -> Res<Host>;
    fn list(&self) -> Res<Vec<Host>>;
    fn get(&self, id: i64) -> Res<Option<Host>>;
    /// Busca por id numérico, IP exato ou hostname.
    fn find(&self, key: &str) -> Res<Option<Host>>;
    fn set_ssh(&self, id: i64, user: Option<&str>, port: Option<u16>, identity: Option<&str>) -> Res<()>;
    fn save_capabilities(&self, id: i64, caps: &[Capability]) -> Res<()>;
    fn set_identity(
        &self,
        id: i64,
        hostname: Option<&str>,
        os: Option<&str>,
        facts: &std::collections::BTreeMap<String, String>,
    ) -> Res<()>;
    fn add_tag(&self, id: i64, tag: &str) -> Res<()>;
    fn remove_tag(&self, id: i64, tag: &str) -> Res<()>;
}

pub struct NewAudit {
    pub ts: String,
    pub actor: String,
    pub host_id: Option<i64>,
    pub host_label: String,
    pub operation_id: String,
    pub risk: RiskLevel,
    pub command: String,
    pub status: OperationStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: i64,
    pub stdout: String,
    pub stdout_sha256: String,
    pub stderr: String,
    pub stderr_sha256: String,
    pub truncated: bool,
    pub evidence: String,
}

pub trait IAuditRepository: Send + Sync {
    /// Anexa um registro encadeando o hash do anterior (append-only, tamper-evident).
    fn append(&self, e: &NewAudit) -> Res<AuditRecord>;
    fn search(&self, f: &AuditFilter) -> Res<Vec<AuditRecord>>;
    fn verify_chain(&self) -> Res<ChainReport>;
    /// Existe dry-run bem-sucedido para este plano desde `since`?
    fn has_dry_run(&self, plan_hash: &str, since: &str) -> Res<bool>;
}

pub struct RemoteRequest {
    pub script: String,
    pub sudo: bool,
    pub timeout_secs: u64,
}

pub trait IRemoteExecutor: Send + Sync {
    fn execute(&self, host: &Host, req: &RemoteRequest) -> Res<OperationResult>;
}

pub trait IFileTransferClient: Send + Sync {
    fn put(&self, host: &Host, local: &str, remote: &str) -> Res<OperationResult>;
    fn get(&self, host: &Host, remote: &str, local: &str) -> Res<OperationResult>;
    /// Verifica se o subsistema SFTP responde (capacidade comprovada).
    fn probe(&self, host: &Host) -> Res<OperationResult>;
}

pub trait IWorkflowRepository: Send + Sync {
    fn find(&self, id: &str) -> Res<Option<Workflow>>;
    fn list(&self) -> Res<Vec<Workflow>>;
}
