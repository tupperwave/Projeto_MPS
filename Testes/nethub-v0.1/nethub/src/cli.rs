//! Interface de linha de comando: cada subcomando é um caso de uso despachado ao `Core`.
use crate::core::Core;
use crate::domain::*;
use crate::error::NhError;
use crate::render::{self, Line, Tone};
use crate::util::fmt_local;
use clap::{Args, Parser, Subcommand};
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nethub", version, about = "NetHub: administração de infraestrutura orientada a capacidades verificadas, com risco explícito e evidência de execução",
    long_about = "Sem subcomando, abre a TUI.\n\nCódigos de saída: 0 sucesso · 1 operação falhou · 2 uso/entrada inválida · 3 bloqueado por capacidade · 4 falta confirmação/dry-run")]
pub struct Cli {
    /// Caminho do banco SQLite (padrão: ~/.nethub/nethub.db)
    #[arg(long, global = true, env = "NETHUB_DB")]
    pub db: Option<PathBuf>,
    /// Desativa cores ANSI
    #[arg(long, global = true)]
    pub no_color: bool,
    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Args, Clone, Default)]
pub struct Act {
    /// Confirmação simples (suficiente apenas para risco MÉDIO)
    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Digite o nome/IP do alvo para confirmar riscos ALTO/CRÍTICO
    #[arg(long, value_name = "ALVO")]
    pub confirm: Option<String>,
    /// Apenas simular: mostra o plano, o risco e os comandos, e registra o dry-run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Abre a interface TUI
    Tui,
    /// Descobre hosts (Nmap) e grava no inventário
    Scan {
        /// Alvo: CIDR, IP, faixa 10.0.0.1-20 ou nome (ex.: 192.168.1.0/24)
        target: String,
        /// Portas (ex.: 22,80,443 ou 1-1024); padrão: lista curta de serviços comuns
        #[arg(long)]
        ports: Option<String>,
        #[arg(long)]
        no_service_detect: bool,
        /// Não depende de ping (-Pn): trata todos os IPs como ativos
        #[arg(long)]
        assume_up: bool,
        #[command(flatten)]
        act: Act,
    },
    /// Lista o inventário
    Hosts {
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        search: Option<String>,
    },
    /// Detalhes e configuração de um host
    Host {
        #[command(subcommand)]
        cmd: HostCmd,
    },
    /// Orienta (ou executa ssh-copy-id) para autenticar por chave
    Onboard {
        host: String,
        /// Executa ssh-copy-id interativamente (a senha vai só para o ssh-copy-id, nunca ao NetHub)
        #[arg(long)]
        copy_id: bool,
    },
    /// Prova as capacidades do host: autenticação, sudo, systemd e SFTP
    Inspect { host: String },
    /// Menu contextual: o que dá para fazer neste host (e por que o resto está bloqueado)
    Actions { host: String },
    /// Verificação de saúde (disco, memória, carga, systemd)
    Health { host: String },
    /// Executa um comando não interativo via SSH
    Exec {
        host: String,
        #[arg(long)]
        timeout: Option<u64>,
        #[command(flatten)]
        act: Act,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        command: Vec<String>,
    },
    /// Serviços systemd
    Service {
        #[command(subcommand)]
        cmd: ServiceCmd,
    },
    /// Envia um arquivo por SFTP (com verificação SHA-256)
    Upload {
        host: String,
        local: String,
        remote: String,
        #[command(flatten)]
        act: Act,
    },
    /// Baixa um arquivo por SFTP (com verificação SHA-256)
    Download {
        host: String,
        remote: String,
        local: String,
        #[arg(long)]
        overwrite: bool,
        #[command(flatten)]
        act: Act,
    },
    /// Workflows com dry-run e validação
    Workflow {
        #[command(subcommand)]
        cmd: WorkflowCmd,
    },
    /// Histórico de operações (evidências)
    History {
        #[arg(long)]
        host: Option<String>,
        /// Prefixo da operação (ex.: service, file, workflow)
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long, default_value_t = 30)]
        limit: usize,
        /// Mostra um registro completo, com evidências
        #[arg(long)]
        id: Option<i64>,
    },
    /// Integridade da trilha de auditoria
    Audit {
        #[command(subcommand)]
        cmd: AuditCmd,
    },
    /// Tags de organização
    Tag {
        #[command(subcommand)]
        cmd: TagCmd,
    },
}

#[derive(Subcommand)]
pub enum HostCmd {
    Show { host: String },
    /// Adiciona manualmente (sem descoberta)
    Add { ip: String, #[arg(long)] name: Option<String> },
    /// Define usuário/porta/chave SSH (nenhuma senha é armazenada)
    Set {
        host: String,
        #[arg(long)]
        user: Option<String>,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        key: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ServiceCmd {
    List { host: String },
    Status { host: String, name: String },
    Restart {
        host: String,
        name: String,
        #[command(flatten)]
        act: Act,
    },
}

#[derive(Subcommand)]
pub enum WorkflowCmd {
    List,
    Show { id: String },
    /// Simula o workflow e registra o dry-run
    DryRun {
        id: String,
        host: String,
        #[arg(long = "param", value_name = "CHAVE=VALOR")]
        params: Vec<String>,
    },
    /// Executa (mostra o plano e pede confirmação proporcional ao risco)
    Run {
        id: String,
        host: String,
        #[arg(long = "param", value_name = "CHAVE=VALOR")]
        params: Vec<String>,
        #[command(flatten)]
        act: Act,
    },
}

#[derive(Subcommand)]
pub enum AuditCmd {
    /// Recalcula a cadeia de hashes e acusa adulteração
    Verify,
    /// Mostra o hash da cabeça da cadeia (guarde fora da máquina para detectar truncamento)
    Head,
}

#[derive(Subcommand)]
pub enum TagCmd {
    Add { host: String, tag: String },
    Rm { host: String, tag: String },
    List,
}

// ───────────────────────────── saída ─────────────────────────────

pub fn ansi(t: Tone, color: bool) -> (&'static str, &'static str) {
    if !color {
        return ("", "");
    }
    let c = match t {
        Tone::Normal => return ("", ""),
        Tone::Head => "\x1b[1;36m",
        Tone::Ok => "\x1b[32m",
        Tone::Warn => "\x1b[33m",
        Tone::Bad => "\x1b[31m",
        Tone::Dim => "\x1b[90m",
    };
    (c, "\x1b[0m")
}

fn print_lines(lines: &[Line], color: bool) {
    for (t, s) in lines {
        let (a, b) = ansi(*t, color);
        println!("{a}{s}{b}");
    }
}

fn say(t: Tone, s: &str, color: bool) {
    print_lines(&[(t, s.to_string())], color);
}

fn prompt(msg: &str) -> String {
    print!("{msg}");
    let _ = std::io::stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
    s.trim().to_string()
}

fn parse_params(v: &[String]) -> Result<Params, NhError> {
    let mut p = Params::new();
    for kv in v {
        let (k, val) = kv.split_once('=').ok_or_else(|| NhError::Invalid(format!("--param espera CHAVE=VALOR, recebi '{kv}'")))?;
        p.insert(k.trim().to_string(), val.trim().to_string());
    }
    Ok(p)
}

fn exit_code(e: &NhError) -> i32 {
    match e {
        NhError::Failed(_) => 1,
        NhError::Invalid(_) | NhError::NotFound(_) => 2,
        NhError::Blocked(_) => 3,
        NhError::ConfirmationRequired { .. } | NhError::DryRunRequired { .. } => 4,
    }
}

/// Fluxo comum de ação: dry-run opcional → confirmação proporcional ao risco → execução → evidência.
fn run_action(core: &Core, req: ActionRequest, act: &Act, color: bool) -> Result<i32, NhError> {
    if act.dry_run {
        let r = core.dry_run(&req)?;
        print_lines(&render::dry_run(&r), color);
        return Ok(if r.blockers.is_empty() { 0 } else { 3 });
    }
    let tty = std::io::stdin().is_terminal();
    let mut conf = match (&act.confirm, act.yes) {
        (Some(t), _) => Confirmation::TypedTarget(t.clone()),
        (None, true) => Confirmation::Yes,
        _ => Confirmation::None,
    };
    for _ in 0..3 {
        match core.execute(&req, &conf) {
            Ok(done) => {
                print_lines(&render::outcome(&done.outcome), color);
                if let (Some(id), Some(h)) = (done.audit_id, &done.audit_hash) {
                    say(Tone::Dim, &format!("evidência: registro #{id} · hash {}…", render::short_hash(h)), color);
                }
                return Ok(if done.ok { 0 } else { 1 });
            }
            Err(NhError::ConfirmationRequired { level, risk, expected, reasons }) => {
                if !tty {
                    say(Tone::Bad, &format!("risco {} exige confirmação: {}", risk.label(), level.describe()), color);
                    for r in &reasons {
                        say(Tone::Dim, &format!("  motivo: {r}"), color);
                    }
                    if level == ConfirmationLevel::TypeTargetAndDryRun {
                        say(Tone::Dim, "  risco CRÍTICO: rode antes com --dry-run (RN-05) e depois com --confirm <alvo>", color);
                    } else if level == ConfirmationLevel::TypeTarget {
                        say(Tone::Dim, &format!("  use: --confirm {expected}"), color);
                    } else {
                        say(Tone::Dim, "  use: --yes", color);
                    }
                    return Ok(4);
                }
                // terminal interativo: mostra o plano (registrado) e pergunta
                let report = core.dry_run(&req)?;
                print_lines(&render::dry_run(&report), color);
                if !report.blockers.is_empty() {
                    return Ok(3);
                }
                let ok = match level {
                    ConfirmationLevel::Simple => {
                        let a = prompt("Executar? [s/N] ");
                        if matches!(a.to_lowercase().as_str(), "s" | "sim" | "y" | "yes") {
                            conf = Confirmation::Yes;
                            true
                        } else {
                            false
                        }
                    }
                    _ => {
                        let a = prompt(&format!("Risco {}. Digite '{}' para confirmar (vazio cancela): ", risk.label(), expected));
                        if a.is_empty() {
                            false
                        } else {
                            conf = Confirmation::TypedTarget(a);
                            true
                        }
                    }
                };
                if !ok {
                    core.cancel(&req)?;
                    say(Tone::Dim, "Cancelado: nada foi executado (registrado como CANCELLED).", color);
                    return Ok(2);
                }
            }
            Err(NhError::DryRunRequired { .. }) => {
                if !tty {
                    say(Tone::Bad, "risco CRÍTICO exige dry-run registrado antes: rode com --dry-run e depois repita com --confirm <alvo>", color);
                    return Ok(4);
                }
                let report = core.dry_run(&req)?;
                print_lines(&render::dry_run(&report), color);
                if !report.blockers.is_empty() {
                    return Ok(3);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(NhError::Failed("não foi possível concluir a confirmação".into()))
}

fn host_table(hosts: &[Host], color: bool) {
    println!("{:<4} {:<15} {:<18} {:<8} {:<14} {:<22} {}", "ID", "IP", "HOST", "STATUS", "SSH", "SISTEMA", "TAGS");
    for h in hosts {
        let st = h.cap_state(cap::REMOTE_SHELL);
        let (a, b) = ansi(render::cap_tone(st), color);
        println!(
            "{:<4} {:<15} {:<18} {:<8} {a}{:<14}{b} {:<22} {}",
            h.id,
            h.ip,
            h.hostname.clone().unwrap_or_default(),
            h.status,
            st.label(),
            h.os.clone().unwrap_or_default().chars().take(22).collect::<String>(),
            h.tags.iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" ")
        );
    }
    if hosts.is_empty() {
        println!("(inventário vazio — use `nethub scan <alvo>`)");
    }
}

pub fn default_db() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".nethub").join("nethub.db")
}

pub fn main() -> i32 {
    let cli = Cli::parse();
    let color = !cli.no_color && std::env::var("NO_COLOR").is_err() && std::io::stdout().is_terminal();
    let db = cli.db.clone().unwrap_or_else(default_db);
    let core = match Core::open(&db) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("erro ao abrir o banco {}: {e}", db.display());
            return 1;
        }
    };
    let cmd = cli.cmd.unwrap_or(Cmd::Tui);
    match dispatch(&core, cmd, color) {
        Ok(c) => c,
        Err(e) => {
            let (a, b) = ansi(Tone::Bad, color);
            eprintln!("{a}erro: {e}{b}");
            exit_code(&e)
        }
    }
}

fn dispatch(core: &Core, cmd: Cmd, color: bool) -> Result<i32, NhError> {
    let find = |k: &str| core.inventory.find(k);
    match cmd {
        Cmd::Tui => crate::tui::run(core),
        Cmd::Scan { target, ports, no_service_detect, assume_up, act } => {
            let req = ScanRequest { target, ports, service_detection: !no_service_detect, assume_up };
            run_action(core, Core::scan_action(&req), &act, color)
        }
        Cmd::Hosts { tag, search } => {
            host_table(&core.inventory.list(tag.as_deref(), search.as_deref())?, color);
            Ok(0)
        }
        Cmd::Host { cmd } => match cmd {
            HostCmd::Show { host } => {
                let h = find(&host)?;
                print_lines(&render::host_summary(&h), color);
                println!();
                print_lines(&render::host_capabilities(&h), color);
                println!();
                print_lines(&render::host_actions(&core.catalog.actions_for(&h)?), color);
                Ok(0)
            }
            HostCmd::Add { ip, name } => {
                let h = core.inventory.add_manual(&ip, name.as_deref())?;
                println!("host #{} adicionado: {}", h.id, h.ip);
                Ok(0)
            }
            HostCmd::Set { host, user, port, key } => {
                let h = find(&host)?;
                core.inventory.set_ssh(&h, user.as_deref(), port, key.as_deref())?;
                println!("configuração SSH de {} atualizada (nenhuma senha é armazenada). Rode `nethub inspect {}`.", h.label(), host);
                Ok(0)
            }
        },
        Cmd::Onboard { host, copy_id } => {
            let h = find(&host)?;
            let user = h.ssh_user.clone();
            println!("Host {} — SSH {}:{} — {}", h.label(), h.ip, h.ssh_port(), h.cap_state(cap::REMOTE_SHELL).label());
            println!("O NetHub usa apenas autenticação por chave (ssh-agent / ~/.ssh) e nunca armazena senhas.\n");
            println!("  1) ssh-keygen -t ed25519                 # se ainda não tiver chave");
            println!("  2) ssh-copy-id -p {} {}@{}", h.ssh_port(), user.clone().unwrap_or_else(|| "USUARIO".into()), h.ip);
            println!("  3) nethub host set {host} --user USUARIO [--key ~/.ssh/id_ed25519]");
            println!("  4) nethub inspect {host}");
            if copy_id {
                let Some(u) = user else {
                    return Err(NhError::Invalid(format!("defina o usuário antes: nethub host set {host} --user USUARIO")));
                };
                let mut c = std::process::Command::new("ssh-copy-id");
                c.arg("-p").arg(h.ssh_port().to_string());
                if let Some(k) = &h.ssh_identity {
                    c.arg("-i").arg(k);
                }
                c.arg(format!("{u}@{}", h.ip));
                let ok = c.status().map(|s| s.success()).unwrap_or(false);
                if !ok {
                    return Err(NhError::Failed("ssh-copy-id falhou".into()));
                }
                return run_action(core, ActionRequest::new("capability.inspect", Some(h.id)), &Act::default(), color);
            }
            Ok(0)
        }
        Cmd::Inspect { host } => run_action(core, ActionRequest::new("capability.inspect", Some(find(&host)?.id)), &Act::default(), color),
        Cmd::Actions { host } => {
            let h = find(&host)?;
            print_lines(&render::host_actions(&core.catalog.actions_for(&h)?), color);
            Ok(0)
        }
        Cmd::Health { host } => run_action(core, ActionRequest::new("health.check", Some(find(&host)?.id)), &Act::default(), color),
        Cmd::Exec { host, timeout, act, command } => {
            let h = find(&host)?;
            let mut req = ActionRequest::new("cmd.exec", Some(h.id)).with("command", &command.join(" "));
            if let Some(t) = timeout {
                req = req.with("timeout", &t.to_string());
            }
            run_action(core, req, &act, color)
        }
        Cmd::Service { cmd } => match cmd {
            ServiceCmd::List { host } => run_action(core, ActionRequest::new("service.list", Some(find(&host)?.id)), &Act::default(), color),
            ServiceCmd::Status { host, name } => run_action(core, ActionRequest::new("service.status", Some(find(&host)?.id)).with("service", &name), &Act::default(), color),
            ServiceCmd::Restart { host, name, act } => run_action(core, ActionRequest::new("service.restart", Some(find(&host)?.id)).with("service", &name), &act, color),
        },
        Cmd::Upload { host, local, remote, act } => {
            run_action(core, ActionRequest::new("file.upload", Some(find(&host)?.id)).with("local", &local).with("remote", &remote), &act, color)
        }
        Cmd::Download { host, remote, local, overwrite, act } => {
            let mut req = ActionRequest::new("file.download", Some(find(&host)?.id)).with("remote", &remote).with("local", &local);
            if overwrite {
                req = req.with("overwrite", "true");
            }
            run_action(core, req, &act, color)
        }
        Cmd::Workflow { cmd } => match cmd {
            WorkflowCmd::List => {
                for w in core.workflows.list()? {
                    println!("{:<28} {}", w.id, w.name);
                    println!("{:<28} {}", "", w.description);
                }
                Ok(0)
            }
            WorkflowCmd::Show { id } => {
                let w = core.workflows.find(&id)?;
                println!("{} — {}\n{}", w.id, w.name, w.description);
                println!("política de falha: {}   requer: {} ≥ {}   parâmetros: {}", w.failure_policy.as_str(), cap::title(&w.requires_capability), w.requires_state.label(), if w.params.is_empty() { "nenhum".into() } else { w.params.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>().join(", ") });
                for s in &w.steps {
                    println!("  {}. [{}] {}", s.order, s.operation_id, s.description);
                }
                Ok(0)
            }
            WorkflowCmd::DryRun { id, host, params } => {
                let mut req = ActionRequest::new(&format!("workflow.{}", id.trim_start_matches("workflow.")), Some(find(&host)?.id));
                req.params = parse_params(&params)?;
                run_action(core, req, &Act { dry_run: true, ..Default::default() }, color)
            }
            WorkflowCmd::Run { id, host, params, act } => {
                let mut req = ActionRequest::new(&format!("workflow.{}", id.trim_start_matches("workflow.")), Some(find(&host)?.id));
                req.params = parse_params(&params)?;
                run_action(core, req, &act, color)
            }
        },
        Cmd::History { host, op, status, text, limit, id } => {
            let status = match status {
                Some(s) => Some(OperationStatus::parse(&s).ok_or_else(|| NhError::Invalid("status: SUCCESS, FAILED, CANCELLED ou BLOCKED".into()))?),
                None => None,
            };
            let recs = core.audit.search(&AuditFilter { id, host, operation: op, status, text, limit: Some(limit), ..Default::default() })?;
            if recs.is_empty() {
                println!("nenhum registro encontrado");
            }
            for r in &recs {
                print_lines(&render::audit_record(r, id.is_some()), color);
            }
            Ok(0)
        }
        Cmd::Audit { cmd } => {
            let rep = core.audit.verify()?;
            match cmd {
                AuditCmd::Verify => {
                    if rep.ok {
                        say(Tone::Ok, &format!("✓ trilha íntegra: {} registro(s), cadeia de hashes consistente", rep.total), color);
                        println!("cabeça: {}", rep.head_hash);
                        Ok(0)
                    } else {
                        say(Tone::Bad, &format!("✗ ADULTERAÇÃO DETECTADA no registro #{}: {}", rep.first_bad_id.unwrap_or(0), rep.detail), color);
                        Ok(1)
                    }
                }
                AuditCmd::Head => {
                    println!("{} registro(s)\ncabeça: {}", rep.total, rep.head_hash);
                    Ok(0)
                }
            }
        }
        Cmd::Tag { cmd } => match cmd {
            TagCmd::Add { host, tag } => {
                core.inventory.add_tag(&find(&host)?, &tag)?;
                println!("tag adicionada");
                Ok(0)
            }
            TagCmd::Rm { host, tag } => {
                core.inventory.remove_tag(&find(&host)?, &tag)?;
                println!("tag removida");
                Ok(0)
            }
            TagCmd::List => {
                let mut all: std::collections::BTreeMap<String, Vec<String>> = Default::default();
                for h in core.inventory.list(None, None)? {
                    for t in &h.tags {
                        all.entry(t.clone()).or_default().push(h.label());
                    }
                }
                if all.is_empty() {
                    println!("(sem tags)");
                }
                for (t, hs) in all {
                    println!("#{t:<16} {}", hs.join(", "));
                }
                Ok(0)
            }
        },
    }
}

#[allow(dead_code)]
fn _unused(ts: &str) -> String {
    fmt_local(ts)
}
