//! FileTransferService: upload/download via SFTP com verificação SHA-256 de ponta a ponta (RF-13).
use super::audit::{AuditEntry, AuditService};
use super::guard::Guard;
use super::risk::{plan_hash, RiskPolicyService};
use super::validator::PostActionValidator;
use super::Done;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use crate::util::{remote_shell_path, safe_path_chars, sha256_file};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

enum RemoteFile {
    Absent,
    Sha(String),
    NoTool,
}

pub struct FileTransferService {
    remote: Arc<dyn IRemoteExecutor>,
    sftp: Arc<dyn IFileTransferClient>,
    guard: Arc<Guard>,
    risk: Arc<RiskPolicyService>,
    audit: Arc<AuditService>,
    validator: PostActionValidator,
}

impl FileTransferService {
    pub fn new(remote: Arc<dyn IRemoteExecutor>, sftp: Arc<dyn IFileTransferClient>, guard: Arc<Guard>, risk: Arc<RiskPolicyService>, audit: Arc<AuditService>) -> Self {
        FileTransferService { remote, sftp, guard, risk, audit, validator: PostActionValidator }
    }

    fn remote_file(&self, host: &Host, path: &str) -> Res<RemoteFile> {
        let script = format!(
            "p={}; if [ -f \"$p\" ]; then if command -v sha256sum >/dev/null 2>&1; then sha256sum -- \"$p\" | cut -d' ' -f1; else echo NOSHA; fi; else echo ABSENT; fi",
            remote_shell_path(path)
        );
        let r = self.remote.execute(host, &RemoteRequest { script, sudo: false, timeout_secs: 30 })?;
        if !r.ok() {
            return Err(NhError::Failed(format!("não foi possível consultar o arquivo remoto: {}", if r.message.is_empty() { r.stderr.trim().to_string() } else { r.message })));
        }
        let line = r.stdout.trim();
        Ok(match line {
            "ABSENT" => RemoteFile::Absent,
            "NOSHA" => RemoteFile::NoTool,
            s if s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()) => RemoteFile::Sha(s.to_lowercase()),
            other => return Err(NhError::Failed(format!("resposta inesperada ao calcular SHA-256 remoto: '{other}'"))),
        })
    }

    fn finish(&self, host: &Host, op: &str, risk: RiskLevel, cmd: &str, tr: &TransferResult, evidence: serde_json::Value) -> Res<AuditRecord> {
        self.audit.record(AuditEntry::new(op, Some(host), risk, cmd).status(tr.status).duration(tr.duration_ms).evidence(evidence))
    }

    fn evidence(tr: &TransferResult, asmt_reasons: &[String], hash: String, validation: &ValidationResult) -> serde_json::Value {
        serde_json::json!({
            "direction": tr.direction, "local": tr.local, "remote": tr.remote, "bytes": tr.bytes,
            "local_sha256": tr.local_sha256, "remote_sha256": tr.remote_sha256,
            "integrity_verified": tr.integrity_verified, "overwrote_sha256": tr.overwrote,
            "message": tr.message, "risk_reasons": asmt_reasons, "plan_hash": hash,
            "validation": validation.to_json(),
        })
    }

    pub fn upload(&self, host: &Host, local: &str, remote: &str, conf: &Confirmation) -> Res<Done<TransferResult>> {
        if !safe_path_chars(local) || !safe_path_chars(remote) {
            return Err(NhError::Invalid("caminho contém caractere não suportado (aspas, barra invertida ou quebra de linha)".into()));
        }
        let lp = Path::new(local);
        let md = std::fs::metadata(lp).map_err(|e| NhError::Invalid(format!("arquivo local '{local}': {e}")))?;
        if !md.is_file() {
            return Err(NhError::Invalid(format!("'{local}' não é um arquivo regular")));
        }
        let (lsha, bytes) = sha256_file(lp)?;
        let mut params = Params::new();
        params.insert("local".into(), local.into());
        params.insert("remote".into(), remote.into());
        let asmt = self.risk.assess("file.upload", &params);
        let cmd = format!("sftp put {local} → {}:{remote}", host.label());
        self.guard.authorize("file.upload", Some(host), Some((cap::FILE_TRANSFER, CapabilityState::Authenticated)), &params, &asmt, conf, &host.identity_tokens(), &cmd)?;
        let hash = plan_hash("file.upload", Some(host.id), &params);
        let t0 = Instant::now();
        let mut tr = TransferResult {
            direction: "upload".into(), local: local.into(), remote: remote.into(), status: OperationStatus::Failed, bytes,
            local_sha256: Some(lsha.clone()), remote_sha256: None, integrity_verified: false, overwrote: None, message: String::new(), duration_ms: 0,
        };

        let before = match self.remote_file(host, remote) {
            Ok(b) => b,
            Err(e) => {
                tr.message = e.to_string();
                tr.duration_ms = t0.elapsed().as_millis() as u64;
                let rec = self.finish(host, "file.upload", asmt.risk, &cmd, &tr, Self::evidence(&tr, &asmt.reasons, hash, &ValidationResult::default()))?;
                return Ok(Done { value: tr, audit: Some(rec) });
            }
        };
        if let RemoteFile::Sha(s) = &before {
            tr.overwrote = Some(s.clone());
        }
        let put = self.sftp.put(host, local, remote)?;
        let validation;
        if !put.ok() {
            tr.message = format!("transferência falhou: {}", if put.stderr.trim().is_empty() { put.message.clone() } else { put.stderr.trim().to_string() });
            validation = ValidationResult::default();
        } else {
            match self.remote_file(host, remote) {
                Ok(RemoteFile::Sha(s)) => {
                    validation = self.validator.validate_transfer(&lsha, Some(&s));
                    tr.remote_sha256 = Some(s);
                    tr.integrity_verified = validation.passed;
                    if validation.passed {
                        tr.status = OperationStatus::Success;
                        tr.message = "enviado e verificado (SHA-256 idêntico)".into();
                    } else {
                        tr.message = "INTEGRIDADE NÃO CONFIRMADA: SHA-256 remoto diverge do local".into();
                    }
                }
                Ok(RemoteFile::NoTool) => {
                    validation = ValidationResult::default();
                    tr.status = OperationStatus::Success;
                    tr.message = "enviado, mas a integridade NÃO pôde ser verificada (sha256sum ausente no host)".into();
                }
                Ok(RemoteFile::Absent) => {
                    validation = ValidationResult::default();
                    tr.message = "o arquivo não aparece no destino após o envio".into();
                }
                Err(e) => {
                    validation = ValidationResult::default();
                    tr.message = format!("não foi possível verificar o destino: {e}");
                }
            }
        }
        tr.duration_ms = t0.elapsed().as_millis() as u64;
        let rec = self.finish(host, "file.upload", asmt.risk, &cmd, &tr, Self::evidence(&tr, &asmt.reasons, hash, &validation))?;
        Ok(Done { value: tr, audit: Some(rec) })
    }

    pub fn download(&self, host: &Host, remote: &str, local: &str, overwrite: bool, conf: &Confirmation) -> Res<Done<TransferResult>> {
        if !safe_path_chars(local) || !safe_path_chars(remote) {
            return Err(NhError::Invalid("caminho contém caractere não suportado (aspas, barra invertida ou quebra de linha)".into()));
        }
        let lp = Path::new(local);
        if lp.exists() && !overwrite {
            return Err(NhError::Invalid(format!("'{local}' já existe; use --overwrite para sobrescrever")));
        }
        let mut params = Params::new();
        params.insert("local".into(), local.into());
        params.insert("remote".into(), remote.into());
        let asmt = self.risk.assess("file.download", &params);
        let cmd = format!("sftp get {}:{remote} → {local}", host.label());
        self.guard.authorize("file.download", Some(host), Some((cap::FILE_TRANSFER, CapabilityState::Authenticated)), &params, &asmt, conf, &host.identity_tokens(), &cmd)?;
        let hash = plan_hash("file.download", Some(host.id), &params);
        let t0 = Instant::now();
        let mut tr = TransferResult {
            direction: "download".into(), local: local.into(), remote: remote.into(), status: OperationStatus::Failed, bytes: 0,
            local_sha256: None, remote_sha256: None, integrity_verified: false,
            overwrote: if lp.exists() { sha256_file(lp).ok().map(|x| x.0) } else { None }, message: String::new(), duration_ms: 0,
        };
        let mut validation = ValidationResult::default();
        match self.remote_file(host, remote) {
            Err(e) => tr.message = e.to_string(),
            Ok(RemoteFile::Absent) => tr.message = "arquivo remoto inexistente (ou não é um arquivo regular)".into(),
            Ok(rf) => {
                if let RemoteFile::Sha(s) = &rf {
                    tr.remote_sha256 = Some(s.clone());
                }
                let got = self.sftp.get(host, remote, local)?;
                if !got.ok() {
                    tr.message = format!("transferência falhou: {}", if got.stderr.trim().is_empty() { got.message.clone() } else { got.stderr.trim().to_string() });
                } else {
                    match sha256_file(lp) {
                        Ok((lsha, bytes)) => {
                            tr.bytes = bytes;
                            tr.local_sha256 = Some(lsha.clone());
                            match &tr.remote_sha256 {
                                Some(rs) => {
                                    validation = self.validator.validate_transfer(rs, Some(&lsha));
                                    tr.integrity_verified = validation.passed;
                                    if validation.passed {
                                        tr.status = OperationStatus::Success;
                                        tr.message = "baixado e verificado (SHA-256 idêntico)".into();
                                    } else {
                                        tr.message = "INTEGRIDADE NÃO CONFIRMADA: SHA-256 local diverge do remoto".into();
                                    }
                                }
                                None => {
                                    tr.status = OperationStatus::Success;
                                    tr.message = "baixado, mas a integridade NÃO pôde ser verificada (sha256sum ausente no host)".into();
                                }
                            }
                        }
                        Err(e) => tr.message = format!("não foi possível ler o arquivo baixado: {e}"),
                    }
                }
            }
        }
        tr.duration_ms = t0.elapsed().as_millis() as u64;
        let rec = self.finish(host, "file.download", asmt.risk, &cmd, &tr, Self::evidence(&tr, &asmt.reasons, hash, &validation))?;
        Ok(Done { value: tr, audit: Some(rec) })
    }
}
