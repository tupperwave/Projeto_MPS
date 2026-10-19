//! LocalProcessRunner: executa programas locais com timeout e captura limitada de saída.
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const MAX_CAPTURE: usize = 2 * 1024 * 1024;

#[derive(Debug, Default, Clone)]
pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub duration_ms: u64,
    pub spawn_error: Option<String>,
}

#[derive(Default, Clone, Copy)]
pub struct LocalProcessRunner;

fn read_capped<R: Read>(mut r: R) -> String {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match r.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if buf.len() < MAX_CAPTURE {
                    let take = n.min(MAX_CAPTURE - buf.len());
                    buf.extend_from_slice(&chunk[..take]);
                }
            }
        }
    }
    String::from_utf8_lossy(&buf).into_owned()
}

impl LocalProcessRunner {
    pub fn run(&self, program: &str, args: &[String], timeout: Duration) -> ProcessResult {
        let start = Instant::now();
        let mut cmd = Command::new(program);
        cmd.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return ProcessResult {
                    spawn_error: Some(format!("{program}: {e}")),
                    duration_ms: start.elapsed().as_millis() as u64,
                    ..Default::default()
                }
            }
        };
        let out = child.stdout.take().expect("stdout piped");
        let err = child.stderr.take().expect("stderr piped");
        let t_out = thread::spawn(move || read_capped(out));
        let t_err = thread::spawn(move || read_capped(err));

        let mut timed_out = false;
        let mut exit_code = None;
        loop {
            match child.try_wait() {
                Ok(Some(st)) => {
                    exit_code = st.code();
                    break;
                }
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        timed_out = true;
                        break;
                    }
                    thread::sleep(Duration::from_millis(15));
                }
                Err(_) => break,
            }
        }
        ProcessResult {
            exit_code,
            stdout: t_out.join().unwrap_or_default(),
            stderr: t_err.join().unwrap_or_default(),
            timed_out,
            duration_ms: start.elapsed().as_millis() as u64,
            spawn_error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs_and_times_out() {
        let r = LocalProcessRunner;
        let ok = r.run("sh", &["-c".into(), "echo hi; echo err >&2; exit 3".into()], Duration::from_secs(5));
        assert_eq!(ok.exit_code, Some(3));
        assert_eq!(ok.stdout.trim(), "hi");
        assert_eq!(ok.stderr.trim(), "err");
        let t = r.run("sh", &["-c".into(), "sleep 5".into()], Duration::from_millis(200));
        assert!(t.timed_out);
        let m = r.run("definitely-not-a-binary", &[], Duration::from_secs(1));
        assert!(m.spawn_error.is_some());
    }
}
