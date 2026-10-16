//! Adapters de infraestrutura: implementam as portas usando ferramentas do sistema.
pub mod nmap;
pub mod process;
pub mod sftp;
pub mod sqlite;
pub mod ssh;

use crate::domain::*;
use process::ProcessResult;

/// Opções comuns de ssh/sftp: sem prompts (BatchMode), TOFU para host keys, timeouts curtos.
/// `NETHUB_SSH_OPTS` permite opções extras (ex.: "-oUserKnownHostsFile=/tmp/kh").
pub fn base_ssh_opts() -> Vec<String> {
    let mut v: Vec<String> = [
        "-o", "BatchMode=yes",
        "-o", "ConnectTimeout=5",
        "-o", "StrictHostKeyChecking=accept-new",
        "-o", "LogLevel=ERROR",
        "-o", "ServerAliveInterval=5",
        "-o", "ServerAliveCountMax=3",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    if let Ok(extra) = std::env::var("NETHUB_SSH_OPTS") {
        v.extend(extra.split_whitespace().map(|s| s.to_string()));
    }
    v
}

pub fn ssh_target(host: &Host) -> String {
    match &host.ssh_user {
        Some(u) if !u.is_empty() => format!("{}@{}", u, host.ip),
        _ => host.ip.clone(),
    }
}

/// Converte o resultado bruto do processo em resultado de operação, classificando a falha
/// (autenticação, conexão, timeout...) sem expor credenciais.
pub fn to_result(pr: ProcessResult, what: &str) -> OperationResult {
    let mut failure = None;
    let mut message = String::new();
    let mut status = OperationStatus::Success;

    if let Some(e) = &pr.spawn_error {
        failure = Some(FailureKind::Local);
        message = format!("não foi possível executar o programa local: {e}");
        status = OperationStatus::Failed;
    } else if pr.timed_out {
        failure = Some(FailureKind::Timeout);
        message = format!("{what}: tempo limite excedido");
        status = OperationStatus::Failed;
    } else {
        match pr.exit_code {
            Some(0) => {}
            Some(255) if what.starts_with("ssh") || what.starts_with("sftp") => {
                status = OperationStatus::Failed;
                let e = pr.stderr.to_lowercase();
                if e.contains("permission denied") || e.contains("authentication failed") || e.contains("too many authentication") {
                    failure = Some(FailureKind::Auth);
                    message = "autenticação por chave falhou (sem senha/prompt). Configure a chave: `nethub onboard <host>`".into();
                } else if e.contains("host key verification failed") || e.contains("remote host identification has changed") {
                    failure = Some(FailureKind::HostKey);
                    message = "verificação da chave do host falhou (possível mudança/MITM). Verifique ~/.ssh/known_hosts".into();
                } else {
                    failure = Some(FailureKind::Connection);
                    message = format!("falha de conexão: {}", pr.stderr.lines().next().unwrap_or("sem detalhes"));
                }
            }
            Some(c) => {
                status = OperationStatus::Failed;
                failure = Some(FailureKind::Remote);
                message = format!("comando terminou com exit code {c}");
            }
            None => {
                status = OperationStatus::Failed;
                failure = Some(FailureKind::Local);
                message = "processo encerrado por sinal".into();
            }
        }
    }
    OperationResult {
        status,
        exit_code: pr.exit_code,
        stdout: pr.stdout,
        stderr: pr.stderr,
        duration_ms: pr.duration_ms,
        failure,
        message,
    }
}
