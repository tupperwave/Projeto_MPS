//! NmapScanner: executa o nmap (conexão TCP, sem privilégios) e interpreta o XML.
use super::process::LocalProcessRunner;
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::INetworkScanner;
use crate::util::{now_iso, parse_target};
use std::time::Duration;

pub const DEFAULT_PORTS: &str = "21,22,23,25,53,80,110,139,143,443,445,631,993,995,1433,2049,3306,3389,5432,5900,8000,8080,8443,9100";

pub struct NmapScanner {
    runner: LocalProcessRunner,
    binary: String,
}

impl NmapScanner {
    pub fn new() -> Self {
        NmapScanner { runner: LocalProcessRunner, binary: std::env::var("NETHUB_NMAP").unwrap_or_else(|_| "nmap".into()) }
    }
}

impl Default for NmapScanner {
    fn default() -> Self {
        Self::new()
    }
}

fn valid_ports(p: &str) -> bool {
    !p.is_empty() && p.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '-')
}

impl INetworkScanner for NmapScanner {
    fn scan(&self, req: &ScanRequest) -> Res<ScanResult> {
        parse_target(&req.target).map_err(NhError::Invalid)?;
        let ports = req.ports.clone().unwrap_or_else(|| DEFAULT_PORTS.to_string());
        if !valid_ports(&ports) {
            return Err(NhError::Invalid(format!("lista de portas inválida: '{ports}'")));
        }
        let mut args: Vec<String> = ["-sT", "-oX", "-", "-T4", "--max-retries", "1", "--host-timeout", "90s"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        if req.assume_up {
            args.push("-Pn".into());
        } else {
            args.push("-PS22,80,443,445".into());
        }
        args.push("-p".into());
        args.push(ports);
        if req.service_detection {
            args.push("-sV".into());
            args.push("--version-light".into());
        }
        args.extend(req.target.split_whitespace().map(|s| s.to_string()));

        let pr = self.runner.run(&self.binary, &args, Duration::from_secs(600));
        if let Some(e) = &pr.spawn_error {
            return Err(NhError::Failed(format!("Nmap indisponível ({e}). Instale o nmap; nenhum resultado foi gravado.")));
        }
        if pr.timed_out {
            return Err(NhError::Failed("Nmap excedeu o tempo limite; resultado descartado (incompleto).".into()));
        }
        if pr.exit_code != Some(0) {
            return Err(NhError::Failed(format!(
                "Nmap falhou (exit {:?}): {}",
                pr.exit_code,
                pr.stderr.lines().next().unwrap_or("sem detalhes")
            )));
        }
        let mut res = parse_nmap_xml(&pr.stdout)?;
        res.target = req.target.clone();
        res.command = format!("{} {}", self.binary, args.join(" "));
        res.elapsed_secs = pr.duration_ms as f64 / 1000.0;
        Ok(res)
    }
}

/// Interpreta o XML do Nmap. Somente hosts "up" entram; `complete` exige `finished exit="success"`.
pub fn parse_nmap_xml(xml: &str) -> Res<ScanResult> {
    // o Nmap emite <!DOCTYPE>; o roxmltree não resolve entidades externas, então permitir DTD é seguro
    let opts = roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
    let doc = roxmltree::Document::parse_with_options(xml, opts).map_err(|e| NhError::Failed(format!("XML do Nmap inválido: {e}")))?;
    let root = doc.root_element();
    if root.tag_name().name() != "nmaprun" {
        return Err(NhError::Failed("XML não é uma saída do Nmap".into()));
    }
    let complete = root
        .descendants()
        .find(|n| n.has_tag_name("finished"))
        .map(|n| n.attribute("exit") == Some("success"))
        .unwrap_or(false);
    let now = now_iso();
    let mut hosts = vec![];
    for h in root.children().filter(|n| n.has_tag_name("host")) {
        let up = h
            .children()
            .find(|n| n.has_tag_name("status"))
            .map(|n| n.attribute("state") == Some("up"))
            .unwrap_or(false);
        if !up {
            continue;
        }
        let mut ip = None;
        let mut mac = None;
        for a in h.children().filter(|n| n.has_tag_name("address")) {
            match a.attribute("addrtype") {
                Some("ipv4") | Some("ipv6") if ip.is_none() => ip = a.attribute("addr").map(|s| s.to_string()),
                Some("mac") => mac = a.attribute("addr").map(|s| s.to_string()),
                _ => {}
            }
        }
        let Some(ip) = ip else { continue };
        let hostname = h
            .descendants()
            .find(|n| n.has_tag_name("hostname"))
            .and_then(|n| n.attribute("name"))
            .map(|s| s.to_string());
        let mut ports = vec![];
        for p in h.descendants().filter(|n| n.has_tag_name("port")) {
            let st = p.children().find(|n| n.has_tag_name("state"));
            let sv = p.children().find(|n| n.has_tag_name("service"));
            ports.push(NetworkPort {
                port: p.attribute("portid").and_then(|s| s.parse().ok()).unwrap_or(0),
                protocol: p.attribute("protocol").unwrap_or("tcp").to_string(),
                state: st.and_then(|n| n.attribute("state")).unwrap_or("unknown").to_string(),
                service: sv.and_then(|n| n.attribute("name")).unwrap_or("").to_string(),
                product: sv
                    .map(|n| {
                        format!("{} {}", n.attribute("product").unwrap_or(""), n.attribute("version").unwrap_or(""))
                            .trim()
                            .to_string()
                    })
                    .unwrap_or_default(),
            });
        }
        hosts.push(Host {
            ip,
            hostname,
            mac,
            status: "up".into(),
            ports,
            first_seen: now.clone(),
            last_seen: now.clone(),
            ..Default::default()
        });
    }
    let mut warnings = vec![];
    if !complete {
        warnings.push("Nmap não reportou término com sucesso: resultado pode estar incompleto".to_string());
    }
    Ok(ScanResult { complete, hosts, warnings, ..Default::default() })
}

#[cfg(test)]
mod tests {
    use super::*;
    const XML: &str = r#"<?xml version="1.0"?>
<nmaprun scanner="nmap" args="nmap -sT">
<host><status state="up"/><address addr="192.168.1.20" addrtype="ipv4"/><address addr="AA:BB:CC:DD:EE:FF" addrtype="mac"/>
<hostnames><hostname name="server01" type="PTR"/></hostnames>
<ports><port protocol="tcp" portid="22"><state state="open"/><service name="ssh" product="OpenSSH" version="9.6"/></port>
<port protocol="tcp" portid="80"><state state="closed"/><service name="http"/></port></ports></host>
<host><status state="down"/><address addr="192.168.1.21" addrtype="ipv4"/></host>
<runstats><finished exit="success"/></runstats></nmaprun>"#;

    #[test]
    fn parses_hosts_and_ports() {
        let r = parse_nmap_xml(XML).unwrap();
        assert!(r.complete);
        assert_eq!(r.hosts.len(), 1);
        let h = &r.hosts[0];
        assert_eq!(h.ip, "192.168.1.20");
        assert_eq!(h.hostname.as_deref(), Some("server01"));
        assert_eq!(h.mac.as_deref(), Some("AA:BB:CC:DD:EE:FF"));
        assert_eq!(h.ports.len(), 2);
        assert_eq!(h.ports[0].product, "OpenSSH 9.6");
        assert_eq!(h.ssh_port(), 22);
    }

    #[test]
    fn incomplete_is_flagged_and_garbage_rejected() {
        let r = parse_nmap_xml(r#"<nmaprun><host><status state="up"/><address addr="10.0.0.1" addrtype="ipv4"/></host></nmaprun>"#).unwrap();
        assert!(!r.complete);
        assert!(parse_nmap_xml("not xml").is_err());
    }
}
