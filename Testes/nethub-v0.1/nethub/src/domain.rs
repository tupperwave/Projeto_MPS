//! Camada de domínio: entidades e valores, sem dependência de infraestrutura.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Params = BTreeMap<String, String>;

// ───────────────────────────── Risco e confirmação ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            RiskLevel::Low => "BAIXO",
            RiskLevel::Medium => "MÉDIO",
            RiskLevel::High => "ALTO",
            RiskLevel::Critical => "CRÍTICO",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "LOW" | "BAIXO" => Some(RiskLevel::Low),
            "MEDIUM" | "MEDIO" | "MÉDIO" => Some(RiskLevel::Medium),
            "HIGH" | "ALTO" => Some(RiskLevel::High),
            "CRITICAL" | "CRITICO" | "CRÍTICO" => Some(RiskLevel::Critical),
            _ => None,
        }
    }
    /// Confirmação proporcional ao risco (RF-04 / RN-01 / RN-05).
    pub fn confirmation_level(self) -> ConfirmationLevel {
        match self {
            RiskLevel::Low => ConfirmationLevel::None,
            RiskLevel::Medium => ConfirmationLevel::Simple,
            RiskLevel::High => ConfirmationLevel::TypeTarget,
            RiskLevel::Critical => ConfirmationLevel::TypeTargetAndDryRun,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmationLevel {
    None,
    Simple,
    TypeTarget,
    TypeTargetAndDryRun,
}

impl ConfirmationLevel {
    pub fn describe(self) -> &'static str {
        match self {
            ConfirmationLevel::None => "sem confirmação",
            ConfirmationLevel::Simple => "confirmação simples (--yes)",
            ConfirmationLevel::TypeTarget => "digitar o nome/IP do alvo (--confirm <alvo>)",
            ConfirmationLevel::TypeTargetAndDryRun => {
                "dry-run registrado + digitar o nome/IP do alvo (--confirm <alvo>)"
            }
        }
    }
}

/// O que o operador efetivamente informou para autorizar a ação.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confirmation {
    None,
    Yes,
    TypedTarget(String),
}

// ───────────────────────────── Capacidades ─────────────────────────────

/// Níveis ordenados: Unknown < Reachable < Authenticated < Authorized.
/// Alcançável (porta aberta) → autenticável (chave funciona) → autorizado (sudo/root).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CapabilityState {
    Unknown,
    Reachable,
    Authenticated,
    Authorized,
}

impl CapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityState::Unknown => "UNKNOWN",
            CapabilityState::Reachable => "REACHABLE",
            CapabilityState::Authenticated => "AUTHENTICATED",
            CapabilityState::Authorized => "AUTHORIZED",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            CapabilityState::Unknown => "desconhecida",
            CapabilityState::Reachable => "alcançável",
            CapabilityState::Authenticated => "autenticada",
            CapabilityState::Authorized => "autorizada",
        }
    }
    pub fn parse(s: &str) -> CapabilityState {
        match s.to_ascii_uppercase().as_str() {
            "REACHABLE" => CapabilityState::Reachable,
            "AUTHENTICATED" => CapabilityState::Authenticated,
            "AUTHORIZED" => CapabilityState::Authorized,
            _ => CapabilityState::Unknown,
        }
    }
}

pub mod cap {
    pub const REMOTE_SHELL: &str = "remote_shell";
    pub const FILE_TRANSFER: &str = "file_transfer";
    pub const SERVICE_MANAGEMENT: &str = "service_management";
    pub const HEALTH_CHECK: &str = "health_check";
    pub const ALL: [&str; 4] = [REMOTE_SHELL, FILE_TRANSFER, SERVICE_MANAGEMENT, HEALTH_CHECK];

    pub fn title(name: &str) -> &'static str {
        match name {
            REMOTE_SHELL => "Shell remoto (SSH)",
            FILE_TRANSFER => "Transferência de arquivos (SFTP)",
            SERVICE_MANAGEMENT => "Gestão de serviços (systemd)",
            HEALTH_CHECK => "Verificação de saúde",
            _ => "Capacidade",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub state: CapabilityState,
    pub detail: String,
    pub verified_at: Option<String>,
}

// ───────────────────────────── Host ─────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetworkPort {
    pub port: u16,
    pub protocol: String,
    pub state: String,
    pub service: String,
    pub product: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Host {
    pub id: i64,
    pub ip: String,
    pub hostname: Option<String>,
    pub mac: Option<String>,
    pub os: Option<String>,
    /// "up", "down" ou "unknown"
    pub status: String,
    pub ssh_user: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_identity: Option<String>,
    /// Fatos coletados na inspeção (uid, sudo, systemd, kernel...).
    pub facts: BTreeMap<String, String>,
    pub first_seen: String,
    pub last_seen: String,
    pub ports: Vec<NetworkPort>,
    pub capabilities: Vec<Capability>,
    pub tags: Vec<String>,
}

impl Host {
    pub fn label(&self) -> String {
        match &self.hostname {
            Some(h) if !h.is_empty() => h.clone(),
            _ => self.ip.clone(),
        }
    }
    pub fn cap(&self, name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.name == name)
    }
    pub fn cap_state(&self, name: &str) -> CapabilityState {
        self.cap(name).map(|c| c.state).unwrap_or(CapabilityState::Unknown)
    }
    pub fn open_ports(&self) -> impl Iterator<Item = &NetworkPort> {
        self.ports.iter().filter(|p| p.state == "open")
    }
    /// Porta SSH: override do operador > porta com serviço "ssh" > 22.
    pub fn ssh_port(&self) -> u16 {
        if let Some(p) = self.ssh_port {
            return p;
        }
        self.open_ports()
            .find(|p| p.service.eq_ignore_ascii_case("ssh"))
            .map(|p| p.port)
            .unwrap_or(22)
    }
    pub fn ssh_port_open(&self) -> bool {
        let p = self.ssh_port();
        self.open_ports().any(|x| x.port == p || x.service.eq_ignore_ascii_case("ssh"))
    }
    pub fn is_root(&self) -> bool {
        self.facts.get("sudo").map(|s| s == "root").unwrap_or(false)
    }
    pub fn is_systemd(&self) -> bool {
        self.facts.get("systemd").map(|s| s == "yes").unwrap_or(false)
    }
    /// Textos aceitos como "digitar o alvo" na confirmação.
    pub fn identity_tokens(&self) -> Vec<String> {
        let mut v = vec![self.ip.clone()];
        if let Some(h) = &self.hostname {
            if !h.is_empty() {
                v.push(h.clone());
            }
        }
        v
    }
}

// ───────────────────────────── Operações ─────────────────────────────

#[derive(Clone, Debug)]
pub struct Operation {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_risk: RiskLevel,
    pub requires: Option<(String, CapabilityState)>,
    /// (chave, texto de pergunta)
    pub params: Vec<(String, String)>,
    pub needs_host: bool,
    pub is_workflow: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    Success,
    Failed,
    Cancelled,
    /// Tentativa impedida por capacidade/política (registrada como evidência).
    Blocked,
}

impl OperationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            OperationStatus::Success => "SUCCESS",
            OperationStatus::Failed => "FAILED",
            OperationStatus::Cancelled => "CANCELLED",
            OperationStatus::Blocked => "BLOCKED",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "SUCCESS" => Some(OperationStatus::Success),
            "FAILED" => Some(OperationStatus::Failed),
            "CANCELLED" => Some(OperationStatus::Cancelled),
            "BLOCKED" => Some(OperationStatus::Blocked),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureKind {
    Connection,
    Auth,
    HostKey,
    Timeout,
    Remote,
    Local,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub status: OperationStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub failure: Option<FailureKind>,
    pub message: String,
}

impl OperationResult {
    pub fn ok(&self) -> bool {
        self.status == OperationStatus::Success
    }
}

#[derive(Clone, Debug)]
pub struct Check {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Default)]
pub struct ValidationResult {
    pub passed: bool,
    pub checks: Vec<Check>,
}

impl ValidationResult {
    pub fn from_checks(checks: Vec<Check>) -> Self {
        ValidationResult { passed: !checks.is_empty() && checks.iter().all(|c| c.passed), checks }
    }
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "passed": self.passed,
            "checks": self.checks.iter().map(|c| serde_json::json!({"name": c.name, "passed": c.passed, "detail": c.detail})).collect::<Vec<_>>()
        })
    }
}

// ───────────────────────────── Saúde ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MetricStatus {
    Ok,
    Unavailable,
    Warn,
    Critical,
}

impl MetricStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MetricStatus::Ok => "OK",
            MetricStatus::Unavailable => "INDISPONÍVEL",
            MetricStatus::Warn => "ATENÇÃO",
            MetricStatus::Critical => "CRÍTICO",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: Option<f64>,
    pub unit: String,
    pub status: MetricStatus,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthReport {
    pub host: String,
    pub collected_at: String,
    pub metrics: Vec<Metric>,
    pub overall: MetricStatus,
    pub unavailable: usize,
    pub summary: String,
}

// ───────────────────────────── systemd ─────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SystemdService {
    pub name: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServiceState {
    pub name: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub main_pid: u64,
    pub enter_ts: String,
}

impl ServiceState {
    pub fn exists(&self) -> bool {
        !self.load_state.is_empty() && self.load_state != "not-found"
    }
    pub fn is_active(&self) -> bool {
        self.active_state == "active"
    }
}

#[derive(Clone, Debug)]
pub struct RestartOutcome {
    pub service: String,
    pub before: ServiceState,
    pub after: Option<ServiceState>,
    pub exec: Option<OperationResult>,
    pub validation: ValidationResult,
    pub status: OperationStatus,
    pub message: String,
}

// ───────────────────────────── Transferência ─────────────────────────────

#[derive(Clone, Debug)]
pub struct TransferResult {
    pub direction: String,
    pub local: String,
    pub remote: String,
    pub status: OperationStatus,
    pub bytes: u64,
    pub local_sha256: Option<String>,
    pub remote_sha256: Option<String>,
    pub integrity_verified: bool,
    pub overwrote: Option<String>,
    pub message: String,
    pub duration_ms: u64,
}

// ───────────────────────────── Workflows ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailurePolicy {
    Stop,
    Continue,
    /// Reservado: compensação/rollback não é implementado no MVP.
    Compensate,
}

impl FailurePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            FailurePolicy::Stop => "STOP",
            FailurePolicy::Continue => "CONTINUE",
            FailurePolicy::Compensate => "COMPENSATE",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub order: u32,
    pub operation_id: String,
    pub description: String,
    #[serde(default)]
    pub params: Params,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub failure_policy: FailurePolicy,
    /// (chave, pergunta)
    pub params: Vec<(String, String)>,
    pub requires_capability: String,
    pub requires_state: CapabilityState,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepStatus {
    Passed,
    Warning,
    Failed,
    NotExecuted,
}

impl StepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            StepStatus::Passed => "PASSED",
            StepStatus::Warning => "WARNING",
            StepStatus::Failed => "FAILED",
            StepStatus::NotExecuted => "NOT_EXECUTED",
        }
    }
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub order: u32,
    pub operation_id: String,
    pub description: String,
    pub status: StepStatus,
    pub message: String,
    pub details: Vec<String>,
    pub evidence: serde_json::Value,
    pub duration_ms: u64,
}

#[derive(Clone, Debug)]
pub struct WorkflowExecution {
    pub workflow_id: String,
    pub workflow_name: String,
    pub host: String,
    pub status: OperationStatus,
    pub steps: Vec<StepResult>,
    pub summary: String,
}

// ───────────────────────────── Dry-run ─────────────────────────────

#[derive(Clone, Debug)]
pub struct DryRunStep {
    pub order: u32,
    pub operation_id: String,
    pub description: String,
    pub command: Option<String>,
    pub risk: RiskLevel,
    pub requirement: Option<String>,
    pub requirement_met: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct DryRunReport {
    pub operation_id: String,
    pub operation_name: String,
    pub host: Option<String>,
    pub steps: Vec<DryRunStep>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub highest_risk: RiskLevel,
    pub reasons: Vec<String>,
    pub plan_hash: String,
    pub confirmation: ConfirmationLevel,
    pub expected_target: String,
}

// ───────────────────────────── Auditoria / evidência ─────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: i64,
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
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Clone, Debug, Default)]
pub struct AuditFilter {
    pub id: Option<i64>,
    pub host: Option<String>,
    pub operation: Option<String>,
    pub status: Option<OperationStatus>,
    pub since: Option<String>,
    pub text: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct ChainReport {
    pub total: usize,
    pub ok: bool,
    pub first_bad_id: Option<i64>,
    pub detail: String,
    pub head_hash: String,
}

// ───────────────────────────── Descoberta ─────────────────────────────

#[derive(Clone, Debug)]
pub struct ScanRequest {
    pub target: String,
    pub ports: Option<String>,
    pub service_detection: bool,
    pub assume_up: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ScanResult {
    pub complete: bool,
    pub target: String,
    pub command: String,
    pub hosts: Vec<Host>,
    pub elapsed_secs: f64,
    pub warnings: Vec<String>,
}

// ───────────────────────────── Requisição unificada de ação ─────────────────────────────

/// Pedido de ação independente da interface (CLI/TUI) — o "caso de uso" despachado ao núcleo.
#[derive(Clone, Debug)]
pub struct ActionRequest {
    pub operation_id: String,
    pub host_id: Option<i64>,
    pub params: Params,
}

impl ActionRequest {
    pub fn new(op: &str, host_id: Option<i64>) -> Self {
        ActionRequest { operation_id: op.to_string(), host_id, params: Params::new() }
    }
    pub fn with(mut self, k: &str, v: &str) -> Self {
        self.params.insert(k.to_string(), v.to_string());
        self
    }
    pub fn param(&self, k: &str) -> &str {
        self.params.get(k).map(|s| s.as_str()).unwrap_or("")
    }
}

#[derive(Clone, Debug)]
pub enum Outcome {
    Inspect { host: String, capabilities: Vec<Capability>, facts: BTreeMap<String, String>, message: String },
    Health(HealthReport),
    Exec { host: String, command: String, result: OperationResult },
    Services(Vec<SystemdService>),
    ServiceStatus(ServiceState),
    Restart(RestartOutcome),
    Transfer(TransferResult),
    Workflow(WorkflowExecution),
    Scan(ScanResult),
}

#[derive(Clone, Debug)]
pub struct Executed {
    pub outcome: Outcome,
    pub audit_id: Option<i64>,
    pub audit_hash: Option<String>,
    pub ok: bool,
}
