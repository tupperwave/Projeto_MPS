//! Núcleo da aplicação (AppController do diagrama): raiz de composição e despacho de casos de uso.
//! CLI e TUI falam apenas com `Core`; nenhuma interface executa comandos diretamente.
use crate::adapters::{nmap::NmapScanner, sftp::SFTPAdapter, sqlite::SQLiteRepositories, ssh::OpenSSHExecutor};
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use crate::services::audit::{AuditEntry, AuditService};
use crate::services::capability::CapabilityService;
use crate::services::catalog::ActionCatalogService;
use crate::services::discovery::{scan_params, scan_request, NetworkDiscoveryService};
use crate::services::guard::Guard;
use crate::services::health::HealthCheckService;
use crate::services::inventory::InventoryService;
use crate::services::plan::PlanService;
use crate::services::remote::RemoteCommandService;
use crate::services::risk::{Assessment, RiskPolicyService};
use crate::services::systemd::ServiceManagementService;
use crate::services::transfer::FileTransferService;
use crate::services::workflow::{builtin_workflows, WorkflowService};
use crate::services::Done;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Core {
    pub inventory: Arc<InventoryService>,
    pub audit: Arc<AuditService>,
    pub catalog: Arc<ActionCatalogService>,
    pub risk: Arc<RiskPolicyService>,
    pub workflows: Arc<WorkflowService>,
    capability: Arc<CapabilityService>,
    discovery: Arc<NetworkDiscoveryService>,
    remote: Arc<RemoteCommandService>,
    health: Arc<HealthCheckService>,
    systemd: Arc<ServiceManagementService>,
    transfer: Arc<FileTransferService>,
    plan: Arc<PlanService>,
}

/// O que o operador precisa saber antes de confirmar (sem tocar na rede).
pub struct Preview {
    pub operation: Operation,
    pub host: Option<Host>,
    pub risk: RiskLevel,
    pub reasons: Vec<String>,
    pub level: ConfirmationLevel,
    pub expected: Vec<String>,
}

impl Core {
    pub fn open(db: &Path) -> Res<Core> {
        let repo = Arc::new(SQLiteRepositories::open(db)?);
        Core::with_adapters(repo, Arc::new(NmapScanner::new()), Arc::new(OpenSSHExecutor::new()), Arc::new(SFTPAdapter::new()))
    }

    pub fn with_adapters(
        repo: Arc<SQLiteRepositories>,
        scanner: Arc<dyn INetworkScanner>,
        remote_exec: Arc<dyn IRemoteExecutor>,
        sftp: Arc<dyn IFileTransferClient>,
    ) -> Res<Core> {
        repo.seed_workflows(&builtin_workflows())?;
        let hosts: Arc<dyn IHostRepository> = repo.clone();
        let audit_repo: Arc<dyn IAuditRepository> = repo.clone();
        let wf_repo: Arc<dyn IWorkflowRepository> = repo.clone();

        let audit = Arc::new(AuditService::new(audit_repo));
        let risk = Arc::new(RiskPolicyService::new(audit.clone()));
        let guard = Arc::new(Guard::new(audit.clone(), risk.clone()));
        let capability = Arc::new(CapabilityService::new(hosts.clone(), remote_exec.clone(), sftp.clone(), audit.clone()));
        let catalog = Arc::new(ActionCatalogService::new(wf_repo.clone()));
        let inventory = Arc::new(InventoryService::new(hosts.clone(), audit.clone()));
        let discovery = Arc::new(NetworkDiscoveryService::new(scanner, hosts.clone(), capability.clone(), guard.clone(), risk.clone(), audit.clone()));
        let remote = Arc::new(RemoteCommandService::new(remote_exec.clone(), guard.clone(), risk.clone(), audit.clone()));
        let health = Arc::new(HealthCheckService::new(remote_exec.clone(), guard.clone(), audit.clone()));
        let systemd = Arc::new(ServiceManagementService::new(remote_exec.clone(), guard.clone(), risk.clone(), audit.clone()));
        let transfer = Arc::new(FileTransferService::new(remote_exec.clone(), sftp, guard.clone(), risk.clone(), audit.clone()));
        let workflows = Arc::new(WorkflowService::new(wf_repo, hosts.clone(), remote_exec, health.clone(), systemd.clone(), guard, risk.clone(), audit.clone()));
        let plan = Arc::new(PlanService::new(hosts, capability.clone(), catalog.clone(), workflows.clone(), risk.clone(), audit.clone()));
        Ok(Core { inventory, audit, catalog, risk, workflows, capability, discovery, remote, health, systemd, transfer, plan })
    }

    pub fn scan_action(req: &ScanRequest) -> ActionRequest {
        ActionRequest { operation_id: "discovery.scan".into(), host_id: None, params: scan_params(req) }
    }

    fn host_of(&self, req: &ActionRequest) -> Res<Option<Host>> {
        match req.host_id {
            Some(id) => Ok(Some(self.inventory.get(id)?)),
            None => Ok(None),
        }
    }

    /// Risco, motivos e confirmação exigida — cálculo local, sem acesso à rede.
    pub fn preview(&self, req: &ActionRequest) -> Res<Preview> {
        let operation = self.catalog.find(&req.operation_id)?.ok_or_else(|| NhError::NotFound(format!("operação '{}'", req.operation_id)))?;
        let host = self.host_of(req)?;
        let a: Assessment = if operation.is_workflow {
            let w = self.workflows.find(&operation.id)?;
            self.workflows.assess(&w, &req.params)
        } else {
            self.risk.assess(&operation.id, &req.params)
        };
        let expected = match &host {
            Some(h) => h.identity_tokens(),
            None => vec![req.param("target").to_string()],
        };
        Ok(Preview { operation, host, risk: a.risk, reasons: a.reasons, level: a.risk.confirmation_level(), expected })
    }

    pub fn dry_run(&self, req: &ActionRequest) -> Res<DryRunReport> {
        self.plan.dry_run(req)
    }

    /// Registra que o operador desistiu (UC-03: "Cancelamento → nada é executado").
    pub fn cancel(&self, req: &ActionRequest) -> Res<()> {
        let pv = self.preview(req)?;
        self.audit.record(
            AuditEntry::new(&req.operation_id, pv.host.as_ref(), pv.risk, &format!("{} (cancelado pelo operador)", req.operation_id))
                .status(OperationStatus::Cancelled)
                .evidence(serde_json::json!({ "params": req.params, "risk_reasons": pv.reasons })),
        )?;
        Ok(())
    }

    pub fn execute(&self, req: &ActionRequest, conf: &Confirmation) -> Res<Executed> {
        let host = self.host_of(req)?;
        let need_host = || host.clone().ok_or_else(|| NhError::Invalid("esta operação exige um host".into()));
        let p = |k: &str| req.params.get(k).map(|s| s.trim().to_string()).unwrap_or_default();

        fn wrap<T>(d: Done<T>, make: impl FnOnce(T) -> (Outcome, bool)) -> Executed {
            let (audit_id, audit_hash) = d.audit.as_ref().map(|a| (Some(a.id), Some(a.hash.clone()))).unwrap_or((None, None));
            let (outcome, ok) = make(d.value);
            Executed { outcome, audit_id, audit_hash, ok }
        }

        match req.operation_id.as_str() {
            "capability.inspect" => {
                let h = need_host()?;
                let d = self.capability.inspect(&h)?;
                Ok(wrap(d, |o| (Outcome::Inspect { host: h.label(), capabilities: o.capabilities, facts: o.facts, message: o.message }, o.ok)))
            }
            "health.check" => Ok(wrap(self.health.collect(&need_host()?)?, |r| (Outcome::Health(r), true))),
            "cmd.exec" => {
                let h = need_host()?;
                let timeout = p("timeout").parse().unwrap_or(60);
                let d = self.remote.execute(&h, &p("command"), timeout, conf)?;
                let cmd = p("command");
                Ok(wrap(d, |r| {
                    let ok = r.ok();
                    (Outcome::Exec { host: h.label(), command: cmd, result: r }, ok)
                }))
            }
            "service.list" => Ok(wrap(self.systemd.list(&need_host()?)?, |r| (Outcome::Services(r), true))),
            "service.status" => Ok(wrap(self.systemd.status(&need_host()?, &p("service"))?, |r| (Outcome::ServiceStatus(r), true))),
            "service.restart" => Ok(wrap(self.systemd.restart(&need_host()?, &p("service"), conf)?, |r| {
                let ok = r.status == OperationStatus::Success;
                (Outcome::Restart(r), ok)
            })),
            "file.upload" => Ok(wrap(self.transfer.upload(&need_host()?, &p("local"), &p("remote"), conf)?, |r| {
                let ok = r.status == OperationStatus::Success;
                (Outcome::Transfer(r), ok)
            })),
            "file.download" => Ok(wrap(self.transfer.download(&need_host()?, &p("remote"), &p("local"), p("overwrite") == "true", conf)?, |r| {
                let ok = r.status == OperationStatus::Success;
                (Outcome::Transfer(r), ok)
            })),
            "discovery.scan" => Ok(wrap(self.discovery.discover(&scan_request(&req.params), conf)?, |r| (Outcome::Scan(r), true))),
            id if id.starts_with("workflow.") => Ok(wrap(self.workflows.execute(&need_host()?, id, &req.params, conf)?, |r| {
                let ok = r.status == OperationStatus::Success;
                (Outcome::Workflow(r), ok)
            })),
            other => Err(NhError::NotFound(format!("operação '{other}'"))),
        }
    }
}
