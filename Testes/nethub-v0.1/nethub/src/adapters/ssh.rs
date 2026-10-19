//! OpenSSHExecutor: executa scripts não interativos via `ssh` do sistema.
use super::process::LocalProcessRunner;
use super::{base_ssh_opts, ssh_target, to_result};
use crate::domain::*;
use crate::error::Res;
use crate::ports::{IRemoteExecutor, RemoteRequest};
use base64::Engine;
use std::time::Duration;

#[derive(Default)]
pub struct OpenSSHExecutor {
    runner: LocalProcessRunner,
}

impl OpenSSHExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn args_for(host: &Host, remote: &str) -> Vec<String> {
        let mut a = base_ssh_opts();
        a.push("-p".into());
        a.push(host.ssh_port().to_string());
        if let Some(k) = &host.ssh_identity {
            if !k.is_empty() {
                a.push("-i".into());
                a.push(k.clone());
                a.push("-o".into());
                a.push("IdentitiesOnly=yes".into());
            }
        }
        a.push(ssh_target(host));
        a.push(remote.to_string());
        a
    }
}

impl IRemoteExecutor for OpenSSHExecutor {
    fn execute(&self, host: &Host, req: &RemoteRequest) -> Res<OperationResult> {
        // O script viaja em base64: evita qualquer problema de quoting/injeção no shell remoto.
        let b64 = base64::engine::general_purpose::STANDARD.encode(req.script.as_bytes());
        let sudo = if req.sudo && !host.is_root() { "sudo -n " } else { "" };
        let remote = format!("{sudo}sh -c \"$(echo '{b64}' | base64 -d)\"");
        let args = Self::args_for(host, &remote);
        let pr = self.runner.run("ssh", &args, Duration::from_secs(req.timeout_secs.max(1)));
        Ok(to_result(pr, "ssh"))
    }
}
