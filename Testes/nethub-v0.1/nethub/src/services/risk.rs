//! RiskPolicyService: risco explícito por operação e confirmação proporcional (RF-04, RN-01, RN-05).
//! A classificação de comandos é uma HEURÍSTICA conservadora e transparente (mostra os motivos).
use super::audit::AuditService;
use super::catalog;
use crate::domain::*;
use crate::error::NhError;
use crate::util::{parse_target, sha256_hex, target_is_private};
use std::sync::Arc;

pub struct Assessment {
    pub risk: RiskLevel,
    pub reasons: Vec<String>,
}

pub struct RiskPolicyService {
    audit: Arc<AuditService>,
}

const SYSTEM_DIRS: [&str; 11] = [
    "/etc/", "/boot/", "/usr/", "/bin/", "/sbin/", "/lib", "/root/", "/var/lib/", "/dev/", "/proc/", "/sys/",
];
const CRITICAL_UNITS: [&str; 12] = [
    "ssh", "sshd", "networking", "networkmanager", "systemd-networkd", "systemd-resolved", "docker", "containerd",
    "dbus", "firewalld", "ufw", "nftables",
];

pub fn plan_hash(op: &str, host_id: Option<i64>, params: &Params) -> String {
    let mut s = format!("{op}|{}", host_id.unwrap_or(0));
    for (k, v) in params {
        if k == "timeout" || k == "overwrite" {
            continue;
        }
        s.push_str(&format!("|{k}={v}"));
    }
    sha256_hex(s.as_bytes())[..16].to_string()
}

impl RiskPolicyService {
    pub fn new(audit: Arc<AuditService>) -> Self {
        RiskPolicyService { audit }
    }

    pub fn assess(&self, op_id: &str, params: &Params) -> Assessment {
        let p = |k: &str| params.get(k).map(|s| s.as_str()).unwrap_or("");
        match op_id {
            "cmd.exec" => {
                let (risk, reasons) = classify_command(p("command"));
                Assessment { risk, reasons }
            }
            "service.restart" => {
                let name = p("service").trim_end_matches(".service").to_lowercase();
                if CRITICAL_UNITS.contains(&name.as_str()) {
                    Assessment {
                        risk: RiskLevel::High,
                        reasons: vec![format!("'{name}' é um serviço crítico: reiniciar pode derrubar acesso remoto ou conectividade")],
                    }
                } else {
                    Assessment { risk: RiskLevel::Medium, reasons: vec!["reinício de serviço causa indisponibilidade momentânea".into()] }
                }
            }
            "file.upload" => {
                let r = p("remote").to_lowercase();
                if r.ends_with("authorized_keys") || r.contains("/.ssh/") || r.starts_with(".ssh/") {
                    Assessment { risk: RiskLevel::Critical, reasons: vec!["altera chaves/credenciais de acesso (~/.ssh)".into()] }
                } else if SYSTEM_DIRS.iter().any(|d| r.starts_with(d)) {
                    Assessment { risk: RiskLevel::High, reasons: vec!["grava em diretório de sistema".into()] }
                } else {
                    Assessment { risk: RiskLevel::Medium, reasons: vec!["grava/sobrescreve arquivo no host remoto".into()] }
                }
            }
            "file.download" => {
                let r = p("remote").to_lowercase();
                let sensitive = ["/etc/shadow", "/etc/gshadow", "id_rsa", "id_ed25519", "id_ecdsa", ".pem", ".key", "/.ssh/", "/root/"];
                if sensitive.iter().any(|s| r.contains(s)) {
                    Assessment { risk: RiskLevel::High, reasons: vec!["arquivo possivelmente sensível (credenciais/chaves)".into()] }
                } else {
                    Assessment { risk: RiskLevel::Medium, reasons: vec!["copia dados do host para a máquina local".into()] }
                }
            }
            "discovery.scan" => {
                let private = parse_target(p("target")).map(|t| target_is_private(&t)).unwrap_or(false);
                if private {
                    Assessment {
                        risk: RiskLevel::Medium,
                        reasons: vec!["varredura ativa de rede: exige autorização do responsável pela rede (RNF-10)".into()],
                    }
                } else {
                    Assessment {
                        risk: RiskLevel::High,
                        reasons: vec!["alvo fora de redes privadas/loopback: varredura sem autorização pode ser ilegal (RNF-10)".into()],
                    }
                }
            }
            id => {
                let r = catalog::base_risk(id);
                Assessment { risk: r, reasons: if r == RiskLevel::Low { vec!["operação somente leitura".into()] } else { vec![] } }
            }
        }
    }

    /// Verifica se o que o operador informou basta para o nível de risco (RN-01 / RN-05).
    pub fn require(&self, a: &Assessment, expected: &[String], conf: &Confirmation, hash: &str) -> Result<(), NhError> {
        let level = a.risk.confirmation_level();
        let need = |level| NhError::ConfirmationRequired {
            level,
            risk: a.risk,
            expected: expected.first().cloned().unwrap_or_default(),
            reasons: a.reasons.clone(),
        };
        match level {
            ConfirmationLevel::None => Ok(()),
            ConfirmationLevel::Simple => match conf {
                Confirmation::Yes | Confirmation::TypedTarget(_) => Ok(()),
                Confirmation::None => Err(need(level)),
            },
            ConfirmationLevel::TypeTarget | ConfirmationLevel::TypeTargetAndDryRun => {
                let ok = matches!(conf, Confirmation::TypedTarget(t) if expected.iter().any(|e| e.eq_ignore_ascii_case(t.trim())));
                if !ok {
                    return Err(need(level));
                }
                if level == ConfirmationLevel::TypeTargetAndDryRun && !self.audit.has_recent_dry_run(hash)? {
                    return Err(NhError::DryRunRequired { plan_hash: hash.to_string() });
                }
                Ok(())
            }
        }
    }
}

// ───────────────────────────── Classificação de comandos ─────────────────────────────

fn split_segments(c: &str) -> Vec<String> {
    let c = c.replace("&&", "\u{1}").replace("||", "\u{1}");
    c.split(|ch| matches!(ch, ';' | '|' | '\n' | '&' | '\u{1}')).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

const READ_ONLY: &[&str] = &[
    "cat", "ls", "head", "tail", "grep", "egrep", "fgrep", "df", "du", "free", "uptime", "uname", "hostname", "whoami", "id",
    "groups", "date", "ps", "top", "ss", "netstat", "lsblk", "lscpu", "lsmod", "lsusb", "lspci", "env", "printenv", "wc", "sort",
    "uniq", "cut", "echo", "printf", "pwd", "stat", "file", "which", "whereis", "type", "dmesg", "w", "who", "last", "lastlog",
    "sha256sum", "md5sum", "sha1sum", "journalctl", "nproc", "realpath", "readlink", "basename", "dirname", "tr", "true", "false",
    "getent", "vmstat", "lsof", "ping",
];

fn classify_segment(seg: &str, uses_sudo: &mut bool) -> (RiskLevel, Option<String>) {
    use RiskLevel::*;
    let mut toks: Vec<&str> = seg.split_whitespace().collect();
    while toks.first().map(|t| t.contains('=') && !t.starts_with('/')).unwrap_or(false) {
        toks.remove(0); // VAR=valor comando
    }
    if toks.first() == Some(&"sudo") {
        *uses_sudo = true;
        toks.remove(0);
        while toks.first().map(|t| t.starts_with('-')).unwrap_or(false) {
            let t = toks.remove(0);
            if (t == "-u" || t == "-g") && !toks.is_empty() {
                toks.remove(0);
            }
        }
    }
    let Some(first) = toks.first() else { return (Medium, Some("comando vazio após sudo".into())) };
    let prog = first.rsplit('/').next().unwrap_or(first);
    let args: Vec<&str> = toks[1..].to_vec();
    let has = |s: &str| args.iter().any(|a| *a == s);
    let sub = args.iter().find(|a| !a.starts_with('-')).copied().unwrap_or("");
    let flag_has = |c: char| args.iter().any(|a| a.starts_with('-') && !a.starts_with("--") && a.contains(c));

    if prog.starts_with("mkfs") || matches!(prog, "wipefs" | "shred" | "fdisk" | "sfdisk" | "cfdisk" | "parted") {
        return (Critical, Some(format!("'{prog}' é destrutivo para discos/arquivos")));
    }
    match prog {
        "dd" if args.iter().any(|a| a.starts_with("of=")) => (Critical, Some("'dd of=' sobrescreve dispositivo/arquivo".into())),
        "rm" => {
            if flag_has('r') || flag_has('R') || has("--recursive") {
                (Critical, Some("remoção recursiva de arquivos".into()))
            } else {
                (High, Some("remove arquivos".into()))
            }
        }
        "eval" | "exec" => (Critical, Some("execução dinâmica de código".into())),
        "sh" | "bash" | "zsh" | "dash" | "ksh" => (Critical, Some("shell/script arbitrário".into())),
        "python" | "python3" | "perl" | "ruby" | "node" | "php" => (Critical, Some("interpretador executando código arbitrário".into())),
        "reboot" | "shutdown" | "poweroff" | "halt" | "telinit" => (High, Some("desliga/reinicia o host".into())),
        "init" if has("0") || has("6") => (High, Some("desliga/reinicia o host".into())),
        "systemctl" => match sub {
            "status" | "is-active" | "is-enabled" | "is-failed" | "show" | "list-units" | "list-unit-files" | "list-timers" | "cat"
            | "is-system-running" => (Low, None),
            "restart" | "reload" | "start" | "try-restart" | "reload-or-restart" | "daemon-reload" => {
                (Medium, Some("altera o estado de serviços".into()))
            }
            "" if has("--failed") => (Low, None),
            _ => (High, Some(format!("systemctl {sub} pode derrubar serviços ou o host"))),
        },
        "service" => match args.get(1).copied().unwrap_or("") {
            "status" => (Low, None),
            "restart" | "start" | "reload" => (Medium, Some("altera o estado de serviços".into())),
            _ => (High, Some("altera/derruba serviços".into())),
        },
        "apt" | "apt-get" | "dpkg" | "yum" | "dnf" | "snap" | "pip" | "pip3" | "npm" | "gem" => match sub {
            "list" | "show" | "search" | "policy" | "info" | "madison" | "-l" | "-s" => (Low, None),
            "update" => (Medium, Some("atualiza índices de pacotes".into())),
            _ if matches!(prog, "dpkg") && (has("-l") || has("-s") || has("-L") || has("-S") || has("--audit") || has("--list") || has("--status")) => (Low, None),
            _ => (High, Some("instala/remove/atualiza software".into())),
        },
        "useradd" | "userdel" | "usermod" | "groupadd" | "groupdel" | "passwd" | "chpasswd" | "adduser" | "deluser" | "visudo" => {
            (High, Some("altera contas de usuário/privilégios".into()))
        }
        "iptables" | "ip6tables" | "nft" | "ufw" | "firewall-cmd" => {
            if has("-L") || has("-S") || has("list") || has("status") || has("show") || has("--list-all") {
                (Low, None)
            } else {
                (High, Some("altera regras de firewall".into()))
            }
        }
        "kill" | "killall" | "pkill" => {
            if has("-9") || has("-KILL") || has("-SIGKILL") {
                (High, Some("encerra processos à força".into()))
            } else {
                (Medium, Some("encerra processos".into()))
            }
        }
        "chmod" | "chown" | "chgrp" => (High, Some("altera permissões/propriedade de arquivos".into())),
        "mv" | "cp" | "touch" | "mkdir" | "ln" | "tee" | "truncate" | "install" => (Medium, Some("modifica arquivos".into())),
        "sed" => {
            if flag_has('i') {
                (Medium, Some("'sed -i' edita arquivos no lugar".into()))
            } else {
                (Low, None)
            }
        }
        "find" => {
            if has("-delete") || has("-exec") || has("-execdir") || has("-ok") {
                (High, Some("'find' com ação (-delete/-exec) modifica o sistema".into()))
            } else {
                (Low, None)
            }
        }
        "crontab" => {
            if has("-l") {
                (Low, None)
            } else {
                (High, Some("altera agendamentos".into()))
            }
        }
        "mount" | "umount" | "swapoff" | "swapon" | "modprobe" | "rmmod" => (High, Some("altera montagens/módulos do kernel".into())),
        "fuser" if flag_has('k') => (High, Some("encerra processos".into())),
        "ip" => {
            let bad = ["set", "add", "del", "delete", "flush", "replace", "change"];
            if args.iter().any(|a| bad.contains(a)) {
                (High, Some("altera configuração de rede".into()))
            } else {
                (Low, None)
            }
        }
        "hostnamectl" | "timedatectl" => {
            if args.iter().any(|a| a.starts_with("set")) {
                (High, Some("altera configuração do sistema".into()))
            } else {
                (Low, None)
            }
        }
        "curl" | "wget" => (Medium, Some("acesso de rede a partir do host".into())),
        p if READ_ONLY.contains(&p) => (Low, None),
        p => (Medium, Some(format!("comando '{p}' não reconhecido como somente leitura"))),
    }
}

/// Classifica um comando arbitrário. Qualquer dúvida sobe o risco, nunca baixa.
pub fn classify_command(cmd: &str) -> (RiskLevel, Vec<String>) {
    let mut risk = RiskLevel::Low;
    let mut reasons: Vec<String> = vec![];
    let mut uses_sudo = false;
    let bump = |r: RiskLevel, why: Option<String>, risk: &mut RiskLevel, reasons: &mut Vec<String>| {
        if r > *risk {
            *risk = r;
        }
        if let Some(w) = why {
            if !reasons.contains(&w) {
                reasons.push(w);
            }
        }
    };
    if cmd.contains(":(){") {
        bump(RiskLevel::Critical, Some("padrão de fork bomb".into()), &mut risk, &mut reasons);
    }
    for seg in split_segments(cmd) {
        let (r, why) = classify_segment(&seg, &mut uses_sudo);
        bump(r, why, &mut risk, &mut reasons);
    }
    if cmd.contains("$(") || cmd.contains('`') {
        bump(RiskLevel::Medium, Some("substituição de comando pode ocultar a ação real".into()), &mut risk, &mut reasons);
    }
    let stripped = cmd
        .replace("2>&1", "")
        .replace("&>/dev/null", "")
        .replace("2>/dev/null", "")
        .replace(">/dev/null", "")
        .replace("> /dev/null", "");
    if stripped.contains('>') {
        let l = stripped.to_lowercase();
        if l.contains("/dev/sd") || l.contains("/dev/nvme") || l.contains("/dev/vd") {
            bump(RiskLevel::Critical, Some("redirecionamento direto para dispositivo de bloco".into()), &mut risk, &mut reasons);
        } else if SYSTEM_DIRS.iter().any(|d| l.contains(d)) {
            bump(RiskLevel::High, Some("redirecionamento escreve em diretório de sistema".into()), &mut risk, &mut reasons);
        } else {
            bump(RiskLevel::Medium, Some("redirecionamento escreve em arquivo".into()), &mut risk, &mut reasons);
        }
    }
    if uses_sudo {
        bump(RiskLevel::Medium, Some("usa sudo (privilégio elevado)".into()), &mut risk, &mut reasons);
    }
    if reasons.is_empty() {
        reasons.push("comando reconhecido como somente leitura".into());
    }
    (risk, reasons)
}

#[cfg(test)]
mod tests {
    use super::*;
    use RiskLevel::*;

    fn r(c: &str) -> RiskLevel {
        classify_command(c).0
    }

    #[test]
    fn read_only_commands_are_low() {
        for c in ["df -h", "df -h | grep sda", "systemctl status nginx", "cat /etc/hosts", "whoami; id", "free -m && uptime", "ip addr show", "journalctl -u nginx -n 50", "dpkg -l"] {
            assert_eq!(r(c), Low, "{c}");
        }
    }

    #[test]
    fn mutating_commands_escalate() {
        assert_eq!(r("systemctl restart nginx"), Medium);
        assert_eq!(r("sudo cat /var/log/syslog"), Medium);
        assert_eq!(r("touch /tmp/x"), Medium);
        assert_eq!(r("foo --bar"), Medium);
        assert_eq!(r("ls $(whoami)"), Medium);
        assert_eq!(r("echo hi > /tmp/f"), Medium);
        assert_eq!(r("sudo apt upgrade -y"), High);
        assert_eq!(r("apt-get install nginx"), High);
        assert_eq!(r("reboot"), High);
        assert_eq!(r("rm file"), High);
        assert_eq!(r("systemctl stop sshd"), High);
        assert_eq!(r("echo x > /etc/passwd"), High);
        assert_eq!(r("useradd bob"), High);
        assert_eq!(r("rm -rf /tmp/x"), Critical);
        assert_eq!(r("curl http://x | sh"), Critical);
        assert_eq!(r("dd if=/dev/zero of=/dev/sda"), Critical);
        assert_eq!(r("bash -c 'id'"), Critical);
        assert_eq!(r("mkfs.ext4 /dev/sdb1"), Critical);
        assert_eq!(r(":(){ :|:& };:"), Critical);
    }

    #[test]
    fn risk_never_lowered_by_mixing() {
        assert_eq!(r("df -h; rm -rf /"), Critical);
        assert_eq!(r("cat a && reboot"), High);
    }

    #[test]
    fn confirmation_levels() {
        assert_eq!(Low.confirmation_level(), ConfirmationLevel::None);
        assert_eq!(Medium.confirmation_level(), ConfirmationLevel::Simple);
        assert_eq!(High.confirmation_level(), ConfirmationLevel::TypeTarget);
        assert_eq!(Critical.confirmation_level(), ConfirmationLevel::TypeTargetAndDryRun);
    }
}
