//! SFTPAdapter: transferências via `sftp -b` (modo lote, sem prompts).
use super::process::LocalProcessRunner;
use super::{base_ssh_opts, ssh_target, to_result};
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::IFileTransferClient;
use crate::util::{safe_path_chars, sftp_path};
use std::time::Duration;

#[derive(Default)]
pub struct SFTPAdapter {
    runner: LocalProcessRunner,
}

impl SFTPAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    fn run_batch(&self, host: &Host, batch: &str, timeout: u64) -> Res<OperationResult> {
        let path = std::env::temp_dir().join(format!(
            "nethub-sftp-{}-{}.batch",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
        ));
        std::fs::write(&path, batch)?;
        let mut a = base_ssh_opts();
        a.push("-P".into());
        a.push(host.ssh_port().to_string());
        if let Some(k) = &host.ssh_identity {
            if !k.is_empty() {
                a.push("-i".into());
                a.push(k.clone());
                a.push("-o".into());
                a.push("IdentitiesOnly=yes".into());
            }
        }
        a.push("-b".into());
        a.push(path.to_string_lossy().into_owned());
        a.push(ssh_target(host));
        let pr = self.runner.run("sftp", &a, Duration::from_secs(timeout));
        let _ = std::fs::remove_file(&path);
        // sftp usa 255 para falhas de conexão, como o ssh
        Ok(to_result(pr, "sftp"))
    }
}

impl IFileTransferClient for SFTPAdapter {
    fn put(&self, host: &Host, local: &str, remote: &str) -> Res<OperationResult> {
        if !safe_path_chars(local) || !safe_path_chars(remote) {
            return Err(NhError::Invalid("caminho contém caractere não suportado (aspas, barra invertida ou quebra de linha)".into()));
        }
        self.run_batch(host, &format!("put \"{}\" \"{}\"\n", local, sftp_path(remote)), 600)
    }
    fn get(&self, host: &Host, remote: &str, local: &str) -> Res<OperationResult> {
        if !safe_path_chars(local) || !safe_path_chars(remote) {
            return Err(NhError::Invalid("caminho contém caractere não suportado (aspas, barra invertida ou quebra de linha)".into()));
        }
        self.run_batch(host, &format!("get \"{}\" \"{}\"\n", sftp_path(remote), local), 600)
    }
    fn probe(&self, host: &Host) -> Res<OperationResult> {
        self.run_batch(host, "pwd\n", 20)
    }
}
