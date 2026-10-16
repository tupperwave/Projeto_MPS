//! Utilitários: tempo, hash, quoting de shell e validação de alvos de rede.
use chrono::{DateTime, Local, SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn iso_minus_secs(secs: i64) -> String {
    (Utc::now() - chrono::Duration::seconds(secs)).to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn fmt_local(ts: &str) -> String {
    match DateTime::parse_from_rfc3339(ts) {
        Ok(dt) => dt.with_timezone(&Local).format("%d/%m %H:%M:%S").to_string(),
        Err(_) => ts.to_string(),
    }
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex(&h.finalize())
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_file(path: &Path) -> std::io::Result<(String, u64)> {
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        total += n as u64;
    }
    Ok((hex(&h.finalize()), total))
}

/// Trunca respeitando limites de caractere. Retorna (texto, foi_truncado).
pub fn truncate(s: &str, max: usize) -> (String, bool) {
    if s.len() <= max {
        return (s.to_string(), false);
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    (s[..end].to_string(), true)
}

pub fn actor() -> String {
    std::env::var("USER").or_else(|_| std::env::var("LOGNAME")).unwrap_or_else(|_| "unknown".into())
}

/// Aspas simples POSIX.
pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Caminho remoto para uso dentro de comandos shell (relativo ao HOME quando não absoluto).
pub fn remote_shell_path(p: &str) -> String {
    if p == "~" {
        "\"$HOME\"".to_string()
    } else if let Some(rest) = p.strip_prefix("~/") {
        format!("\"$HOME\"/{}", sh_quote(rest))
    } else if p.starts_with('/') {
        sh_quote(p)
    } else {
        format!("\"$HOME\"/{}", sh_quote(p))
    }
}

/// Caminho remoto para o sftp (relativo ao HOME quando não absoluto).
pub fn sftp_path(p: &str) -> String {
    if p == "~" {
        ".".to_string()
    } else if let Some(rest) = p.strip_prefix("~/") {
        rest.to_string()
    } else {
        p.to_string()
    }
}

/// Caminhos aceitos nos lotes do sftp: sem aspas, quebras de linha ou barra invertida.
pub fn safe_path_chars(p: &str) -> bool {
    !p.is_empty() && !p.contains(['"', '\n', '\r', '\\', '\0'])
}

pub fn valid_unit_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && !s.starts_with('-')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | ':' | '.' | '_' | '-'))
}

pub fn normalize_unit(s: &str) -> String {
    let known = [".service", ".socket", ".timer", ".target", ".mount", ".path"];
    if known.iter().any(|k| s.ends_with(k)) {
        s.to_string()
    } else {
        format!("{s}.service")
    }
}

// ───────────────────────────── Alvos de rede ─────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetToken {
    Ip(Ipv4Addr),
    Cidr(Ipv4Addr, u8),
    Range(Ipv4Addr, u8), // último octeto: a.b.c.X-Y (X em Ipv4Addr, Y em u8)
    Name(String),
}

/// Valida o alvo da varredura. Rejeita opções (-x), IPv6 e intervalos muito amplos (< /16).
pub fn parse_target(t: &str) -> Result<Vec<TargetToken>, String> {
    let toks: Vec<&str> = t.split_whitespace().collect();
    if toks.is_empty() {
        return Err("informe um alvo (ex.: 192.168.1.0/24)".into());
    }
    let mut out = vec![];
    for tok in toks {
        if tok.starts_with('-') {
            return Err(format!("alvo inválido '{tok}'"));
        }
        if tok.contains(':') {
            return Err("IPv6 não é suportado no MVP".into());
        }
        if let Some((ip, p)) = tok.split_once('/') {
            let ip: Ipv4Addr = ip.parse().map_err(|_| format!("IP inválido em '{tok}'"))?;
            let p: u8 = p.parse().map_err(|_| format!("prefixo inválido em '{tok}'"))?;
            if p > 32 {
                return Err(format!("prefixo inválido em '{tok}'"));
            }
            if p < 16 {
                return Err(format!("intervalo '{tok}' muito amplo (mínimo suportado: /16)"));
            }
            out.push(TargetToken::Cidr(ip, p));
        } else if let Some((ip, end)) = tok.rsplit_once('-') {
            let a: Ipv4Addr = ip.parse().map_err(|_| format!("intervalo inválido '{tok}'"))?;
            let y: u8 = end.parse().map_err(|_| format!("intervalo inválido '{tok}'"))?;
            if y < a.octets()[3] {
                return Err(format!("intervalo inválido '{tok}'"));
            }
            out.push(TargetToken::Range(a, y));
        } else if let Ok(ip) = tok.parse::<Ipv4Addr>() {
            out.push(TargetToken::Ip(ip));
        } else if tok.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
            && tok.chars().any(|c| c.is_ascii_alphabetic())
        {
            out.push(TargetToken::Name(tok.to_string()));
        } else {
            return Err(format!("alvo inválido '{tok}'"));
        }
    }
    Ok(out)
}

pub fn is_private_ip(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_link_local()
}

/// Alvo inteiramente em redes privadas/loopback/link-local? Nomes de host contam como "não privado".
pub fn target_is_private(tokens: &[TargetToken]) -> bool {
    tokens.iter().all(|t| match t {
        TargetToken::Ip(ip) | TargetToken::Cidr(ip, _) | TargetToken::Range(ip, _) => is_private_ip(*ip),
        TargetToken::Name(_) => false,
    })
}

/// `Some(true/false)` se for possível decidir se o IP pertence ao alvo (CIDR/IP/faixa); `None` para nomes.
pub fn ip_in_target(ip: &str, tokens: &[TargetToken]) -> Option<bool> {
    let ip: Ipv4Addr = ip.parse().ok()?;
    let v = u32::from(ip);
    let mut decidable = false;
    for t in tokens {
        match t {
            TargetToken::Ip(a) => {
                decidable = true;
                if *a == ip {
                    return Some(true);
                }
            }
            TargetToken::Cidr(a, p) => {
                decidable = true;
                let mask = if *p == 0 { 0 } else { u32::MAX << (32 - *p as u32) };
                if u32::from(*a) & mask == v & mask {
                    return Some(true);
                }
            }
            TargetToken::Range(a, y) => {
                decidable = true;
                let o = a.octets();
                let io = ip.octets();
                if o[0..3] == io[0..3] && io[3] >= o[3] && io[3] <= *y {
                    return Some(true);
                }
            }
            TargetToken::Name(_) => {}
        }
    }
    if decidable {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting_is_safe() {
        assert_eq!(sh_quote("a'b"), "'a'\\''b'");
        assert_eq!(remote_shell_path("~/x y"), "\"$HOME\"/'x y'");
        assert_eq!(remote_shell_path("/etc/a"), "'/etc/a'");
        assert_eq!(sftp_path("~/x"), "x");
    }

    #[test]
    fn target_validation() {
        assert!(parse_target("192.168.1.0/24").is_ok());
        assert!(parse_target("10.0.0.1-20 10.0.1.5").is_ok());
        assert!(parse_target("-sS").is_err());
        assert!(parse_target("10.0.0.0/8").is_err());
        assert!(parse_target("::1").is_err());
        assert!(parse_target("999.1.1.1").is_err());
        assert!(parse_target("").is_err());
        let t = parse_target("192.168.1.0/24").unwrap();
        assert!(target_is_private(&t));
        assert_eq!(ip_in_target("192.168.1.9", &t), Some(true));
        assert_eq!(ip_in_target("192.168.2.9", &t), Some(false));
        assert!(!target_is_private(&parse_target("8.8.8.8").unwrap()));
        assert!(!target_is_private(&parse_target("example.com").unwrap()));
    }

    #[test]
    fn unit_names() {
        assert!(valid_unit_name("nginx"));
        assert!(valid_unit_name("getty@tty1.service"));
        assert!(!valid_unit_name("ng inx"));
        assert!(!valid_unit_name("x;rm"));
        assert!(!valid_unit_name("-x"));
        assert_eq!(normalize_unit("nginx"), "nginx.service");
        assert_eq!(normalize_unit("cron.service"), "cron.service");
    }
}
