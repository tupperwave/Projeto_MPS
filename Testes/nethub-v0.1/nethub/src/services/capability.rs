//! CapabilityService: capacidade verificada em níveis — alcançável → autenticada → autorizada.
//! A descoberta só prova "alcançável"; autenticação e sudo são PROVADOS por teste real (inspect).
use super::audit::{AuditEntry, AuditService};
use super::Done;
use crate::domain::*;
use crate::error::Res;
use crate::ports::*;
use crate::util::now_iso;
use std::collections::BTreeMap;
use std::sync::Arc;

const INSPECT_SCRIPT: &str = r##"
echo "UID=$(id -u)"
echo "USER=$(id -un)"
echo "HOST=$(hostname 2>/dev/null)"
echo "SYSNAME=$(uname -s 2>/dev/null)"
echo "KERNEL=$(uname -sr 2>/dev/null)"
if [ -r /etc/os-release ]; then . /etc/os-release; echo "OS=$PRETTY_NAME"; echo "OSID=$ID"; echo "OSLIKE=$ID_LIKE"; fi
if [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1; then echo "SYSTEMD=yes"; else echo "SYSTEMD=no"; fi
if command -v apt-get >/dev/null 2>&1; then echo "PKG=apt"; elif command -v dnf >/dev/null 2>&1; then echo "PKG=dnf"; elif command -v yum >/dev/null 2>&1; then echo "PKG=yum"; else echo "PKG=none"; fi
if command -v sha256sum >/dev/null 2>&1; then echo "SHA256=yes"; else echo "SHA256=no"; fi
if [ "$(id -u)" = "0" ]; then echo "SUDO=root"; elif command -v sudo >/dev/null 2>&1 && sudo -n true >/dev/null 2>&1; then echo "SUDO=yes"; else echo "SUDO=no"; fi
"##;

pub struct InspectOutcome {
    pub capabilities: Vec<Capability>,
    pub facts: BTreeMap<String, String>,
    pub message: String,
    pub ok: bool,
}

pub struct CapabilityService {
    hosts: Arc<dyn IHostRepository>,
    remote: Arc<dyn IRemoteExecutor>,
    sftp: Arc<dyn IFileTransferClient>,
    audit: Arc<AuditService>,
}

fn parse_facts(out: &str) -> BTreeMap<String, String> {
    out.lines()
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
        .collect()
}

impl CapabilityService {
    pub fn new(hosts: Arc<dyn IHostRepository>, remote: Arc<dyn IRemoteExecutor>, sftp: Arc<dyn IFileTransferClient>, audit: Arc<AuditService>) -> Self {
        CapabilityService { hosts, remote, sftp, audit }
    }

    /// Após a descoberta: porta SSH aberta ⇒ "alcançável". Nunca rebaixa estados já provados
    /// (a menos que a porta SSH tenha deixado de ser observada).
    pub fn derive_baseline(&self, host_id: i64) -> Res<()> {
        let Some(h) = self.hosts.get(host_id)? else { return Ok(()) };
        let mut caps = vec![];
        let ssh_open = h.ssh_port_open();
        let have_ports = !h.ports.is_empty();
        for name in [cap::REMOTE_SHELL, cap::FILE_TRANSFER, cap::SERVICE_MANAGEMENT, cap::HEALTH_CHECK] {
            let cur = h.cap(name);
            let shellish = name == cap::REMOTE_SHELL || name == cap::FILE_TRANSFER;
            let (state, detail) = match (shellish, cur) {
                (true, _) if !ssh_open && have_ports => (CapabilityState::Unknown, "porta SSH não observada na descoberta".to_string()),
                (true, Some(c)) if c.state >= CapabilityState::Reachable => (c.state, c.detail.clone()),
                (true, _) => (CapabilityState::Reachable, format!("porta SSH {} aberta; autenticação ainda não verificada", h.ssh_port())),
                (false, Some(c)) if ssh_open || !have_ports => (c.state, c.detail.clone()),
                (false, _) => (CapabilityState::Unknown, "requer SSH autenticado: execute `nethub inspect`".to_string()),
            };
            caps.push(Capability { name: name.into(), state, detail, verified_at: cur.and_then(|c| c.verified_at.clone()) });
        }
        self.hosts.save_capabilities(host_id, &caps)
    }

    /// Prova as capacidades do host com testes reais e persiste o resultado (com auditoria).
    pub fn inspect(&self, host: &Host) -> Res<Done<InspectOutcome>> {
        let now = Some(now_iso());
        let mk = |name: &str, state: CapabilityState, detail: String| Capability { name: name.into(), state, detail, verified_at: now.clone() };
        let res = self.remote.execute(host, &RemoteRequest { script: INSPECT_SCRIPT.into(), sudo: false, timeout_secs: 25 })?;

        let (caps, facts, message, ok) = if !res.ok() {
            let (st, detail) = match res.failure {
                Some(FailureKind::Auth) | Some(FailureKind::HostKey) => (CapabilityState::Reachable, res.message.clone()),
                _ => (CapabilityState::Unknown, if res.message.is_empty() { res.stderr.clone() } else { res.message.clone() }),
            };
            let blocked = "requer SSH autenticado".to_string();
            (
                vec![
                    mk(cap::REMOTE_SHELL, st, detail.clone()),
                    mk(cap::FILE_TRANSFER, st, detail.clone()),
                    mk(cap::SERVICE_MANAGEMENT, CapabilityState::Unknown, blocked.clone()),
                    mk(cap::HEALTH_CHECK, CapabilityState::Unknown, blocked),
                ],
                host.facts.clone(),
                detail,
                false,
            )
        } else {
            let f = parse_facts(&res.stdout);
            let sudo = f.get("sudo").cloned().unwrap_or_else(|| "no".into());
            let privileged = sudo == "yes" || sudo == "root";
            let user = f.get("user").cloned().unwrap_or_default();
            let shell_state = if privileged { CapabilityState::Authorized } else { CapabilityState::Authenticated };
            let shell_detail = format!(
                "usuário '{}' autenticado por chave; privilégio: {}",
                user,
                match sudo.as_str() {
                    "root" => "root".to_string(),
                    "yes" => "sudo sem senha".to_string(),
                    _ => "nenhum (sem sudo sem senha)".to_string(),
                }
            );
            // SFTP é provado com um comando real, não presumido pela porta
            let sftp = self.sftp.probe(host)?;
            let (ft_state, ft_detail) = if sftp.ok() {
                (CapabilityState::Authenticated, "subsistema SFTP respondeu".to_string())
            } else {
                (CapabilityState::Reachable, format!("SFTP não respondeu: {}", if sftp.message.is_empty() { sftp.stderr.trim().to_string() } else { sftp.message.clone() }))
            };
            let (sm_state, sm_detail) = if f.get("systemd").map(|s| s == "yes").unwrap_or(false) {
                (shell_state, if privileged { "systemd detectado; privilégio para alterar serviços" } else { "systemd detectado; somente leitura (sem sudo)" }.to_string())
            } else {
                (CapabilityState::Unknown, "systemd não detectado neste host".to_string())
            };
            let linux = f.get("sysname").map(|s| s == "Linux").unwrap_or(false);
            let (hc_state, hc_detail) = if linux {
                (CapabilityState::Authenticated, format!("coleta via /proc, df e systemctl ({})", f.get("os").cloned().unwrap_or_else(|| "Linux".into())))
            } else {
                (CapabilityState::Unknown, "verificação de saúde implementada apenas para Linux".to_string())
            };
            let msg = format!("{} capacidades verificadas; privilégio: {}", 4, sudo);
            (
                vec![
                    mk(cap::REMOTE_SHELL, shell_state, shell_detail),
                    mk(cap::FILE_TRANSFER, ft_state, ft_detail),
                    mk(cap::SERVICE_MANAGEMENT, sm_state, sm_detail),
                    mk(cap::HEALTH_CHECK, hc_state, hc_detail),
                ],
                f,
                msg,
                true,
            )
        };

        self.hosts.save_capabilities(host.id, &caps)?;
        if ok {
            let new_name = if host.hostname.is_none() { facts.get("host").map(|s| s.as_str()).filter(|s| !s.is_empty()) } else { None };
            self.hosts.set_identity(host.id, new_name, facts.get("os").map(|s| s.as_str()), &facts)?;
        }
        let rec = self.audit.record(
            AuditEntry::new("capability.inspect", Some(host), RiskLevel::Low, "verificar capacidades (id, sudo -n true, systemd, sftp pwd)")
                .from_result(&res)
                .status(if ok { OperationStatus::Success } else { OperationStatus::Failed })
                .evidence(serde_json::json!({
                    "facts": facts,
                    "capabilities": caps.iter().map(|c| serde_json::json!({"name": c.name, "state": c.state.as_str(), "detail": c.detail})).collect::<Vec<_>>(),
                    "message": message,
                })),
        )?;
        Ok(Done { value: InspectOutcome { capabilities: caps, facts, message, ok }, audit: Some(rec) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn facts_parse() {
        let f = parse_facts("UID=1000\nSUDO=yes\nOS=Ubuntu 24.04 LTS\nlixo sem igual\n");
        assert_eq!(f["uid"], "1000");
        assert_eq!(f["sudo"], "yes");
        assert_eq!(f["os"], "Ubuntu 24.04 LTS");
    }
}
