//! InventoryService + TagService: consulta do inventário, credenciais não secretas e tags (RF-02, RF-15).
use super::audit::{AuditEntry, AuditService};
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::IHostRepository;
use std::sync::Arc;

pub struct InventoryService {
    hosts: Arc<dyn IHostRepository>,
    audit: Arc<AuditService>,
}

pub fn normalize_tag(t: &str) -> Res<String> {
    let t = t.trim().trim_start_matches('#').to_lowercase();
    if t.is_empty() || t.len() > 32 || !t.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
        return Err(NhError::Invalid("tag inválida (use letras, números, '-', '_' ou '.'; até 32 caracteres)".into()));
    }
    Ok(t)
}

impl InventoryService {
    pub fn new(hosts: Arc<dyn IHostRepository>, audit: Arc<AuditService>) -> Self {
        InventoryService { hosts, audit }
    }

    /// Lista com filtros opcionais: tag e texto livre (IP/hostname/OS/serviço).
    pub fn list(&self, tag: Option<&str>, text: Option<&str>) -> Res<Vec<Host>> {
        let tag = tag.map(normalize_tag).transpose()?;
        let text = text.map(|t| t.to_lowercase());
        Ok(self
            .hosts
            .list()?
            .into_iter()
            .filter(|h| tag.as_ref().map(|t| h.tags.contains(t)).unwrap_or(true))
            .filter(|h| {
                text.as_ref()
                    .map(|t| {
                        h.ip.contains(t.as_str())
                            || h.label().to_lowercase().contains(t.as_str())
                            || h.os.clone().unwrap_or_default().to_lowercase().contains(t.as_str())
                            || h.open_ports().any(|p| p.service.to_lowercase().contains(t.as_str()))
                    })
                    .unwrap_or(true)
            })
            .collect())
    }

    pub fn find(&self, key: &str) -> Res<Host> {
        self.hosts.find(key)?.ok_or_else(|| NhError::NotFound(format!("host '{key}' (use `nethub hosts` ou `nethub scan`)")))
    }

    pub fn get(&self, id: i64) -> Res<Host> {
        self.hosts.get(id)?.ok_or_else(|| NhError::NotFound(format!("host #{id}")))
    }

    pub fn add_manual(&self, ip: &str, hostname: Option<&str>) -> Res<Host> {
        if ip.parse::<std::net::Ipv4Addr>().is_err() {
            return Err(NhError::Invalid(format!("IP inválido: '{ip}'")));
        }
        self.hosts.insert_manual(ip, hostname)
    }

    /// Usuário/porta/chave são dados de conexão, não segredos. Senhas NUNCA são armazenadas (RN-06).
    pub fn set_ssh(&self, host: &Host, user: Option<&str>, port: Option<u16>, identity: Option<&str>) -> Res<()> {
        if let Some(u) = user {
            if u.is_empty() || !u.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) {
                return Err(NhError::Invalid(format!("usuário SSH inválido: '{u}'")));
            }
        }
        if let Some(k) = identity {
            if !std::path::Path::new(k).is_file() {
                return Err(NhError::Invalid(format!("arquivo de chave não encontrado: '{k}'")));
            }
        }
        self.hosts.set_ssh(host.id, user, port, identity)
    }

    pub fn add_tag(&self, host: &Host, tag: &str) -> Res<()> {
        let t = normalize_tag(tag)?;
        self.hosts.add_tag(host.id, &t)?;
        self.audit.record(AuditEntry::new("host.tag", Some(host), RiskLevel::Low, &format!("tag +{t}")).evidence(serde_json::json!({ "tag": t, "action": "add" })))?;
        Ok(())
    }

    pub fn remove_tag(&self, host: &Host, tag: &str) -> Res<()> {
        let t = normalize_tag(tag)?;
        self.hosts.remove_tag(host.id, &t)?;
        self.audit.record(AuditEntry::new("host.tag", Some(host), RiskLevel::Low, &format!("tag -{t}")).evidence(serde_json::json!({ "tag": t, "action": "remove" })))?;
        Ok(())
    }
}
