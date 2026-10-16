//! Camada de aplicação: casos de uso do NetHub.
pub mod audit;
pub mod capability;
pub mod catalog;
pub mod discovery;
pub mod guard;
pub mod health;
pub mod inventory;
pub mod plan;
pub mod remote;
pub mod risk;
pub mod systemd;
pub mod transfer;
pub mod validator;
pub mod workflow;

use crate::domain::AuditRecord;

/// Resultado de um caso de uso junto do registro de auditoria gerado (RN-03).
pub struct Done<T> {
    pub value: T,
    pub audit: Option<AuditRecord>,
}
