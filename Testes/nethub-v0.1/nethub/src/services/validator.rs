//! PostActionValidator: confirma o resultado prático da ação no ambiente (RF-08).
use crate::domain::*;

#[derive(Default)]
pub struct PostActionValidator;

impl PostActionValidator {
    pub fn validate_restart(&self, before: &ServiceState, after: &ServiceState, exec: &OperationResult) -> ValidationResult {
        let restarted = before.main_pid != after.main_pid || before.enter_ts != after.enter_ts || !before.is_active();
        ValidationResult::from_checks(vec![
            Check {
                name: "systemctl restart retornou sucesso".into(),
                passed: exec.ok(),
                detail: format!("exit code {}", exec.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "-".into())),
            },
            Check {
                name: "serviço está ativo depois do reinício".into(),
                passed: after.is_active(),
                detail: format!("{}/{}", after.active_state, after.sub_state),
            },
            Check {
                name: "o serviço realmente reiniciou (PID ou instante de ativação mudou)".into(),
                passed: restarted,
                detail: format!("PID {}→{}, ativação {}→{}", before.main_pid, after.main_pid, before.enter_ts, after.enter_ts),
            },
        ])
    }

    pub fn validate_transfer(&self, expected: &str, actual: Option<&str>) -> ValidationResult {
        ValidationResult::from_checks(vec![Check {
            name: "SHA-256 de origem e destino são idênticos".into(),
            passed: actual == Some(expected),
            detail: format!("origem {}…, destino {}", &expected[..12.min(expected.len())], actual.map(|a| format!("{}…", &a[..12.min(a.len())])).unwrap_or_else(|| "indisponível".into())),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(pid: u64, ts: &str, active: &str) -> ServiceState {
        ServiceState { name: "x".into(), load_state: "loaded".into(), active_state: active.into(), sub_state: "running".into(), main_pid: pid, enter_ts: ts.into() }
    }
    fn ok() -> OperationResult {
        OperationResult { status: OperationStatus::Success, exit_code: Some(0), stdout: String::new(), stderr: String::new(), duration_ms: 1, failure: None, message: String::new() }
    }

    #[test]
    fn restart_requires_real_restart() {
        let v = PostActionValidator;
        assert!(v.validate_restart(&st(10, "1", "active"), &st(11, "2", "active"), &ok()).passed);
        // comando "ok" mas nada mudou → não afirmamos sucesso
        assert!(!v.validate_restart(&st(10, "1", "active"), &st(10, "1", "active"), &ok()).passed);
        // reiniciou mas ficou inativo
        assert!(!v.validate_restart(&st(10, "1", "active"), &st(0, "2", "failed"), &ok()).passed);
        // estava parado e subiu
        assert!(v.validate_restart(&st(0, "0", "inactive"), &st(12, "3", "active"), &ok()).passed);
    }

    #[test]
    fn transfer_hash_compare() {
        let v = PostActionValidator;
        assert!(v.validate_transfer("abc", Some("abc")).passed);
        assert!(!v.validate_transfer("abc", Some("abd")).passed);
        assert!(!v.validate_transfer("abc", None).passed);
    }
}
