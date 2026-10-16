//! ActionCatalogService: menu contextual de ações conforme as capacidades verificadas (RF-03, RN-04).
use crate::domain::*;
use crate::error::Res;
use crate::ports::IWorkflowRepository;
use std::sync::Arc;

fn op(id: &str, name: &str, desc: &str, risk: RiskLevel, requires: Option<(&str, CapabilityState)>, params: &[(&str, &str)], needs_host: bool) -> Operation {
    Operation {
        id: id.into(),
        name: name.into(),
        description: desc.into(),
        base_risk: risk,
        requires: requires.map(|(c, s)| (c.to_string(), s)),
        params: params.iter().map(|(k, q)| (k.to_string(), q.to_string())).collect(),
        needs_host,
        is_workflow: false,
    }
}

/// Operações atômicas do MVP.
pub fn operations() -> Vec<Operation> {
    use cap::*;
    use CapabilityState::*;
    use RiskLevel::*;
    vec![
        op("capability.inspect", "Verificar capacidades", "Prova o que é possível no host: autentica, testa sudo, systemd e SFTP", Low, None, &[], true),
        op("health.check", "Verificar saúde", "Disco, memória, carga e serviços systemd (somente leitura)", Low, Some((HEALTH_CHECK, Authenticated)), &[], true),
        op("cmd.exec", "Executar comando remoto", "Comando não interativo via SSH; risco calculado pelo comando", Medium, Some((REMOTE_SHELL, Authenticated)), &[("command", "Comando")], true),
        op("service.list", "Listar serviços systemd", "Lista unidades .service e seus estados", Low, Some((SERVICE_MANAGEMENT, Authenticated)), &[], true),
        op("service.status", "Consultar serviço", "Estado detalhado de um serviço systemd", Low, Some((SERVICE_MANAGEMENT, Authenticated)), &[("service", "Nome do serviço")], true),
        op("service.restart", "Reiniciar serviço", "Reinicia e valida (ativo + PID/instante mudou)", Medium, Some((SERVICE_MANAGEMENT, Authorized)), &[("service", "Nome do serviço")], true),
        op("file.upload", "Enviar arquivo (SFTP)", "Upload com verificação SHA-256", Medium, Some((FILE_TRANSFER, Authenticated)), &[("local", "Arquivo local"), ("remote", "Destino remoto")], true),
        op("file.download", "Baixar arquivo (SFTP)", "Download com verificação SHA-256", Medium, Some((FILE_TRANSFER, Authenticated)), &[("remote", "Arquivo remoto"), ("local", "Destino local")], true),
        op("discovery.scan", "Descobrir rede", "Varredura Nmap de um intervalo autorizado", Medium, None, &[("target", "Alvo (ex.: 192.168.1.0/24)")], false),
    ]
}

pub fn base_risk(id: &str) -> RiskLevel {
    operations().into_iter().find(|o| o.id == id).map(|o| o.base_risk).unwrap_or(RiskLevel::Low)
}

fn workflow_op(w: &Workflow) -> Operation {
    let risk = w.steps.iter().map(|s| base_risk(&s.operation_id)).max().unwrap_or(RiskLevel::Low);
    Operation {
        id: format!("workflow.{}", w.id),
        name: format!("Workflow: {}", w.name),
        description: w.description.clone(),
        base_risk: risk,
        requires: Some((w.requires_capability.clone(), w.requires_state)),
        params: w.params.clone(),
        needs_host: true,
        is_workflow: true,
    }
}

/// Texto explicando por que uma capacidade não atende ao mínimo (None = atende).
pub fn shortfall(host: &Host, capability: &str, min: CapabilityState) -> Option<String> {
    let c = host.cap(capability);
    let st = c.map(|c| c.state).unwrap_or(CapabilityState::Unknown);
    if st >= min {
        return None;
    }
    let hint = match st {
        CapabilityState::Unknown => "ainda não verificada: execute `nethub inspect <host>`",
        CapabilityState::Reachable => "alcançável, mas não autenticada",
        _ if min == CapabilityState::Authorized => "falta privilégio (sudo sem senha ou root)",
        _ => "",
    };
    let detail = c.map(|c| c.detail.as_str()).filter(|d| !d.is_empty()).unwrap_or("");
    let mut s = format!("{} está {} (exigido: {}) — {}", cap::title(capability), st.label(), min.label(), hint);
    if !detail.is_empty() {
        s.push_str(&format!(" [{detail}]"));
    }
    Some(s)
}

pub struct ActionItem {
    pub op: Operation,
    pub available: bool,
    pub reason: Option<String>,
}

pub struct ActionCatalogService {
    workflows: Arc<dyn IWorkflowRepository>,
}

impl ActionCatalogService {
    pub fn new(workflows: Arc<dyn IWorkflowRepository>) -> Self {
        ActionCatalogService { workflows }
    }

    pub fn all_operations(&self) -> Res<Vec<Operation>> {
        let mut v = operations();
        for w in self.workflows.list()? {
            v.push(workflow_op(&w));
        }
        Ok(v)
    }

    pub fn find(&self, id: &str) -> Res<Option<Operation>> {
        Ok(self.all_operations()?.into_iter().find(|o| o.id == id))
    }

    /// Todas as ações do host: compatíveis (available) e bloqueadas com o motivo.
    pub fn actions_for(&self, host: &Host) -> Res<Vec<ActionItem>> {
        let mut out = vec![];
        for o in self.all_operations()?.into_iter().filter(|o| o.needs_host) {
            let reason = o.requires.as_ref().and_then(|(c, min)| shortfall(host, c, *min));
            out.push(ActionItem { available: reason.is_none(), reason, op: o });
        }
        Ok(out)
    }

    /// RN-04: somente ações compatíveis com as capacidades detectadas.
    pub fn available_actions(&self, host: &Host) -> Res<Vec<Operation>> {
        Ok(self.actions_for(host)?.into_iter().filter(|a| a.available).map(|a| a.op).collect())
    }
}
