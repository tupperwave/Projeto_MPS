//! Erros do núcleo. Os "bloqueios" (política, capacidade, confirmação) são erros tipados
//! para que CLI e TUI possam reagir (ex.: pedir confirmação e tentar de novo).
use crate::domain::{ConfirmationLevel, RiskLevel};
use std::fmt;

#[derive(Debug)]
pub enum NhError {
    /// Entrada inválida (parâmetro, alvo, caminho...).
    Invalid(String),
    NotFound(String),
    /// Capacidade insuficiente ou política impede a ação.
    Blocked(String),
    /// A ação exige confirmação do operador antes de executar (RN-01).
    ConfirmationRequired {
        level: ConfirmationLevel,
        risk: RiskLevel,
        expected: String,
        reasons: Vec<String>,
    },
    /// Risco crítico exige dry-run registrado antes (RN-05).
    DryRunRequired { plan_hash: String },
    /// Falha operacional (I/O, banco, processo externo).
    Failed(String),
}

pub type Res<T> = Result<T, NhError>;

impl fmt::Display for NhError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NhError::Invalid(m) => write!(f, "entrada inválida: {m}"),
            NhError::NotFound(m) => write!(f, "não encontrado: {m}"),
            NhError::Blocked(m) => write!(f, "bloqueado: {m}"),
            NhError::ConfirmationRequired { level, risk, expected, .. } => write!(
                f,
                "risco {} exige confirmação ({}). Alvo esperado: {}",
                risk.label(),
                level.describe(),
                expected
            ),
            NhError::DryRunRequired { .. } => {
                write!(f, "risco CRÍTICO exige dry-run registrado antes da execução (RN-05)")
            }
            NhError::Failed(m) => write!(f, "falha: {m}"),
        }
    }
}

impl std::error::Error for NhError {}

impl From<rusqlite::Error> for NhError {
    fn from(e: rusqlite::Error) -> Self {
        NhError::Failed(format!("banco de dados: {e}"))
    }
}
impl From<std::io::Error> for NhError {
    fn from(e: std::io::Error) -> Self {
        NhError::Failed(format!("E/S: {e}"))
    }
}
impl From<serde_json::Error> for NhError {
    fn from(e: serde_json::Error) -> Self {
        NhError::Failed(format!("json: {e}"))
    }
}
impl From<anyhow::Error> for NhError {
    fn from(e: anyhow::Error) -> Self {
        NhError::Failed(format!("{e:#}"))
    }
}
