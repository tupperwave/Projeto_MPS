//! TUI (Ratatui). Fala apenas com `Core`: a interface nunca executa comandos diretamente.
//! Ações demoradas rodam em threads; a UI recebe o resultado por canal e nunca trava.
use crate::core::Core;
use crate::domain::*;
use crate::error::NhError;
use crate::render::{self, Line as RLine, Tone};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

enum Then {
    Action { op: String, host: i64, keys: Vec<String> },
    Scan,
    Tag(i64),
    HistFilter,
}

enum Mode {
    Normal,
    Menu { ops: Vec<Operation>, sel: usize },
    Input { title: String, labels: Vec<String>, values: Vec<String>, idx: usize, then: Then },
    Confirm { req: ActionRequest, report: DryRunReport, typed: String },
    Result { title: String, lines: Vec<RLine>, scroll: u16 },
    Busy(String),
}

enum Msg {
    Plan(Result<DryRunReport, NhError>, ActionRequest),
    Done(Result<Executed, NhError>, ActionRequest),
}

pub struct App {
    core: Core,
    tab: usize,
    hosts: Vec<Host>,
    sel: usize,
    hist: Vec<AuditRecord>,
    hist_sel: usize,
    hist_filter: String,
    mode: Mode,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    flash: String,
}

fn style(t: Tone) -> Style {
    match t {
        Tone::Normal => Style::default(),
        Tone::Head => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        Tone::Ok => Style::default().fg(Color::Green),
        Tone::Warn => Style::default().fg(Color::Yellow),
        Tone::Bad => Style::default().fg(Color::Red),
        Tone::Dim => Style::default().fg(Color::DarkGray),
    }
}

fn to_text(lines: &[RLine]) -> Text<'static> {
    Text::from(lines.iter().map(|(t, s)| Line::from(Span::styled(s.clone(), style(*t)))).collect::<Vec<_>>())
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h }
}

impl App {
    pub fn new(core: Core) -> App {
        let (tx, rx) = channel();
        let mut a = App { core, tab: 0, hosts: vec![], sel: 0, hist: vec![], hist_sel: 0, hist_filter: String::new(), mode: Mode::Normal, tx, rx, flash: String::new() };
        a.reload();
        a
    }

    fn reload(&mut self) {
        self.hosts = self.core.inventory.list(None, None).unwrap_or_default();
        if self.sel >= self.hosts.len() {
            self.sel = self.hosts.len().saturating_sub(1);
        }
        let f = AuditFilter { text: Some(self.hist_filter.clone()).filter(|s| !s.is_empty()), limit: Some(300), ..Default::default() };
        self.hist = self.core.audit.search(&f).unwrap_or_default();
        if self.hist_sel >= self.hist.len() {
            self.hist_sel = self.hist.len().saturating_sub(1);
        }
    }

    fn host(&self) -> Option<&Host> {
        self.hosts.get(self.sel)
    }

    fn show(&mut self, title: &str, lines: Vec<RLine>) {
        self.mode = Mode::Result { title: title.into(), lines, scroll: 0 };
    }

    fn show_err(&mut self, e: &NhError) {
        let tone = if matches!(e, NhError::Blocked(_) | NhError::ConfirmationRequired { .. } | NhError::DryRunRequired { .. }) { Tone::Warn } else { Tone::Bad };
        self.show("Não executado", vec![(tone, e.to_string())]);
    }

    fn run(&mut self, req: ActionRequest, conf: Confirmation) {
        self.mode = Mode::Busy("Executando…".into());
        let (core, tx) = (self.core.clone(), self.tx.clone());
        std::thread::spawn(move || {
            let r = core.execute(&req, &conf);
            let _ = tx.send(Msg::Done(r, req));
        });
    }

    /// Risco BAIXO executa direto; os demais mostram o dry-run e pedem confirmação proporcional.
    fn start(&mut self, req: ActionRequest) {
        match self.core.preview(&req) {
            Err(e) => self.show_err(&e),
            Ok(pv) if pv.risk == RiskLevel::Low => self.run(req, Confirmation::None),
            Ok(_) => {
                self.mode = Mode::Busy("Gerando dry-run…".into());
                let (core, tx) = (self.core.clone(), self.tx.clone());
                std::thread::spawn(move || {
                    let r = core.dry_run(&req);
                    let _ = tx.send(Msg::Plan(r, req));
                });
            }
        }
    }

    fn on_msg(&mut self, m: Msg) {
        match m {
            Msg::Plan(Ok(rep), req) => {
                if rep.blockers.is_empty() {
                    self.mode = Mode::Confirm { req, report: rep, typed: String::new() };
                } else {
                    self.show("Dry-run: ação bloqueada", render::dry_run(&rep));
                }
            }
            Msg::Plan(Err(e), _) => self.show_err(&e),
            Msg::Done(Ok(ex), req) => {
                let mut lines = render::outcome(&ex.outcome);
                if let (Some(id), Some(h)) = (ex.audit_id, &ex.audit_hash) {
                    lines.push((Tone::Dim, format!("evidência: registro #{id} · hash {}…", render::short_hash(h))));
                }
                let title = if ex.ok { "Concluído" } else { "Concluído com falha" };
                self.reload();
                self.show(&format!("{title} · {}", req.operation_id), lines);
            }
            Msg::Done(Err(e), _) => {
                self.reload();
                self.show_err(&e);
            }
        }
    }

    fn open_menu(&mut self) {
        let Some(h) = self.host().cloned() else { return };
        match self.core.catalog.available_actions(&h) {
            Ok(ops) if !ops.is_empty() => self.mode = Mode::Menu { ops, sel: 0 },
            Ok(_) => self.flash = "nenhuma ação compatível: prove as capacidades com [i]".into(),
            Err(e) => self.show_err(&e),
        }
    }

    fn finish_input(&mut self, values: Vec<String>, then: Then) {
        match then {
            Then::Action { op, host, keys } => {
                let mut req = ActionRequest::new(&op, Some(host));
                for (k, v) in keys.iter().zip(values) {
                    req = req.with(k, v.trim());
                }
                self.start(req);
            }
            Then::Scan => {
                let sr = ScanRequest { target: values[0].trim().to_string(), ports: Some(values[1].trim().to_string()).filter(|s| !s.is_empty()), service_detection: true, assume_up: false };
                self.start(Core::scan_action(&sr));
            }
            Then::Tag(id) => {
                let r = self.core.inventory.get(id).and_then(|h| self.core.inventory.add_tag(&h, values[0].trim()));
                self.mode = Mode::Normal;
                match r {
                    Ok(_) => {
                        self.flash = "tag adicionada".into();
                        self.reload();
                    }
                    Err(e) => self.show_err(&e),
                }
            }
            Then::HistFilter => {
                self.hist_filter = values[0].trim().to_string();
                self.mode = Mode::Normal;
                self.reload();
            }
        }
    }

    /// Retorna true para sair.
    fn key(&mut self, code: KeyCode, mods: KeyModifiers) -> bool {
        if mods.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return true;
        }
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        match mode {
            Mode::Busy(m) => self.mode = Mode::Busy(m),
            Mode::Result { title, lines, mut scroll } => match code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {}
                KeyCode::Down | KeyCode::Char('j') => {
                    scroll = scroll.saturating_add(1);
                    self.mode = Mode::Result { title, lines, scroll };
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    scroll = scroll.saturating_sub(1);
                    self.mode = Mode::Result { title, lines, scroll };
                }
                KeyCode::PageDown => {
                    scroll = scroll.saturating_add(10);
                    self.mode = Mode::Result { title, lines, scroll };
                }
                KeyCode::PageUp => {
                    scroll = scroll.saturating_sub(10);
                    self.mode = Mode::Result { title, lines, scroll };
                }
                _ => self.mode = Mode::Result { title, lines, scroll },
            },
            Mode::Menu { ops, mut sel } => match code {
                KeyCode::Esc => {}
                KeyCode::Down | KeyCode::Char('j') => {
                    sel = (sel + 1).min(ops.len() - 1);
                    self.mode = Mode::Menu { ops, sel };
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    sel = sel.saturating_sub(1);
                    self.mode = Mode::Menu { ops, sel };
                }
                KeyCode::Enter => {
                    let op = ops[sel].clone();
                    let Some(h) = self.host().cloned() else { return false };
                    if op.params.is_empty() {
                        self.start(ActionRequest::new(&op.id, Some(h.id)));
                    } else {
                        self.mode = Mode::Input {
                            title: op.name.clone(),
                            labels: op.params.iter().map(|(_, q)| q.clone()).collect(),
                            values: vec![String::new(); op.params.len()],
                            idx: 0,
                            then: Then::Action { op: op.id.clone(), host: h.id, keys: op.params.iter().map(|(k, _)| k.clone()).collect() },
                        };
                    }
                }
                _ => self.mode = Mode::Menu { ops, sel },
            },
            Mode::Input { title, labels, mut values, mut idx, then } => match code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let optional = matches!(then, Then::Scan | Then::HistFilter);
                    if idx + 1 < labels.len() {
                        idx += 1;
                        self.mode = Mode::Input { title, labels, values, idx, then };
                    } else if !optional && values.iter().any(|v| v.trim().is_empty()) {
                        self.flash = "preencha todos os campos".into();
                        self.mode = Mode::Input { title, labels, values, idx, then };
                    } else if matches!(then, Then::Scan) && values[0].trim().is_empty() {
                        self.flash = "informe o alvo".into();
                        self.mode = Mode::Input { title, labels, values, idx: 0, then };
                    } else {
                        self.finish_input(values, then);
                    }
                }
                KeyCode::Backspace => {
                    values[idx].pop();
                    self.mode = Mode::Input { title, labels, values, idx, then };
                }
                KeyCode::Char(c) => {
                    values[idx].push(c);
                    self.mode = Mode::Input { title, labels, values, idx, then };
                }
                _ => self.mode = Mode::Input { title, labels, values, idx, then },
            },
            Mode::Confirm { req, report, mut typed } => {
                let level = report.confirmation;
                match code {
                    KeyCode::Esc => {
                        let _ = self.core.cancel(&req);
                        self.flash = "cancelado: nada foi executado (registrado)".into();
                        self.reload();
                    }
                    KeyCode::Char('y') if level == ConfirmationLevel::Simple => self.run(req, Confirmation::Yes),
                    KeyCode::Char('n') if level == ConfirmationLevel::Simple => {
                        let _ = self.core.cancel(&req);
                        self.flash = "cancelado: nada foi executado (registrado)".into();
                        self.reload();
                    }
                    KeyCode::Enter if level != ConfirmationLevel::Simple => self.run(req, Confirmation::TypedTarget(typed)),
                    KeyCode::Backspace => {
                        typed.pop();
                        self.mode = Mode::Confirm { req, report, typed };
                    }
                    KeyCode::Char(c) if level != ConfirmationLevel::Simple => {
                        typed.push(c);
                        self.mode = Mode::Confirm { req, report, typed };
                    }
                    _ => self.mode = Mode::Confirm { req, report, typed },
                }
            }
            Mode::Normal => return self.key_normal(code),
        }
        false
    }

    fn key_normal(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('1') => self.tab = 0,
            KeyCode::Char('2') => self.tab = 1,
            KeyCode::Char('3') => self.tab = 2,
            KeyCode::Tab => self.tab = (self.tab + 1) % 3,
            KeyCode::Char('r') => {
                self.reload();
                self.flash = "atualizado".into();
            }
            KeyCode::Down | KeyCode::Char('j') => match self.tab {
                0 => self.sel = (self.sel + 1).min(self.hosts.len().saturating_sub(1)),
                1 => self.hist_sel = (self.hist_sel + 1).min(self.hist.len().saturating_sub(1)),
                _ => {}
            },
            KeyCode::Up | KeyCode::Char('k') => match self.tab {
                0 => self.sel = self.sel.saturating_sub(1),
                1 => self.hist_sel = self.hist_sel.saturating_sub(1),
                _ => {}
            },
            _ if self.tab == 0 => match code {
                KeyCode::Char('i') => {
                    if let Some(h) = self.host() {
                        let req = ActionRequest::new("capability.inspect", Some(h.id));
                        self.start(req);
                    }
                }
                KeyCode::Char('h') => {
                    if let Some(h) = self.host() {
                        let req = ActionRequest::new("health.check", Some(h.id));
                        self.start(req);
                    }
                }
                KeyCode::Char('a') | KeyCode::Enter => self.open_menu(),
                KeyCode::Char('t') => {
                    if let Some(h) = self.host() {
                        self.mode = Mode::Input { title: "Nova tag".into(), labels: vec!["Tag".into()], values: vec![String::new()], idx: 0, then: Then::Tag(h.id) };
                    }
                }
                KeyCode::Char('d') => {
                    self.mode = Mode::Input {
                        title: "Descobrir rede (Nmap)".into(),
                        labels: vec!["Alvo (ex.: 192.168.1.0/24)".into(), "Portas (vazio = padrão)".into()],
                        values: vec![String::new(), String::new()],
                        idx: 0,
                        then: Then::Scan,
                    };
                }
                _ => {}
            },
            _ if self.tab == 1 => match code {
                KeyCode::Enter => {
                    if let Some(r) = self.hist.get(self.hist_sel) {
                        let lines = render::audit_record(r, true);
                        let title = format!("Registro #{}", r.id);
                        self.show(&title, lines);
                    }
                }
                KeyCode::Char('v') => match self.core.audit.verify() {
                    Ok(rep) if rep.ok => self.show("Integridade da trilha", vec![(Tone::Ok, format!("✓ íntegra: {} registro(s)", rep.total)), (Tone::Dim, format!("cabeça: {}", rep.head_hash))]),
                    Ok(rep) => self.show("Integridade da trilha", vec![(Tone::Bad, format!("✗ ADULTERAÇÃO no registro #{}: {}", rep.first_bad_id.unwrap_or(0), rep.detail))]),
                    Err(e) => self.show_err(&e),
                },
                KeyCode::Char('/') => {
                    self.mode = Mode::Input { title: "Filtrar histórico".into(), labels: vec!["Texto (host, operação, comando)".into()], values: vec![self.hist_filter.clone()], idx: 0, then: Then::HistFilter };
                }
                _ => {}
            },
            _ => {}
        }
        false
    }

    // ───────────────────────────── desenho ─────────────────────────────

    pub fn draw(&self, f: &mut Frame) {
        let area = f.area();
        let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(3), Constraint::Length(1)]).split(area);
        let tabs = Tabs::new(vec!["1 Hosts", "2 Histórico", "3 Ajuda"])
            .select(self.tab)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
            .divider(" ");
        f.render_widget(Line::from(vec![Span::styled(" NET-HUB ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD))]), Rect { width: 10, ..rows[0] });
        f.render_widget(tabs, Rect { x: rows[0].x + 11, width: rows[0].width.saturating_sub(11), ..rows[0] });
        match self.tab {
            0 => self.draw_hosts(f, rows[1]),
            1 => self.draw_history(f, rows[1]),
            _ => self.draw_help(f, rows[1]),
        }
        let help = match (&self.mode, self.tab) {
            (Mode::Normal, 0) => "↑↓ navegar · i provar capacidades · h saúde · a/Enter ações · t tag · d descobrir · r atualizar · Tab aba · q sair",
            (Mode::Normal, 1) => "↑↓ navegar · Enter detalhes/evidência · v verificar integridade · / filtrar · Tab aba · q sair",
            (Mode::Normal, _) => "Tab aba · q sair",
            (Mode::Confirm { .. }, _) => "y/n ou digite o alvo + Enter · Esc cancela",
            (Mode::Result { .. }, _) => "↑↓ PgUp/PgDn rolar · Esc fechar",
            _ => "Enter confirma · Esc volta",
        };
        let foot = if self.flash.is_empty() { help.to_string() } else { format!("{}  ·  {}", self.flash, help) };
        f.render_widget(Paragraph::new(foot).style(Style::default().fg(Color::DarkGray)), rows[2]);
        self.draw_popup(f, area);
    }

    fn draw_hosts(&self, f: &mut Frame, area: Rect) {
        let cols = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);
        let items: Vec<ListItem> = self
            .hosts
            .iter()
            .map(|h| {
                let st = h.cap_state(cap::REMOTE_SHELL);
                let dot = if h.status == "up" { "●" } else { "○" };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{dot} "), style(if h.status == "up" { Tone::Ok } else { Tone::Bad })),
                    Span::raw(format!("{:<14} {:<15} ", h.label().chars().take(14).collect::<String>(), h.ip)),
                    Span::styled(st.label().to_string(), style(render::cap_tone(st))),
                ]))
            })
            .collect();
        let mut state = ListState::default();
        if !self.hosts.is_empty() {
            state.select(Some(self.sel));
        }
        let list = List::new(items)
            .block(Block::bordered().title(format!(" Hosts ({}) ", self.hosts.len())))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_stateful_widget(list, cols[0], &mut state);

        let mut lines: Vec<RLine> = vec![];
        match self.host() {
            Some(h) => {
                lines.extend(render::host_summary(h));
                lines.push((Tone::Normal, String::new()));
                lines.extend(render::host_capabilities(h));
                lines.push((Tone::Normal, String::new()));
                if let Ok(items) = self.core.catalog.actions_for(h) {
                    lines.extend(render::host_actions(&items));
                }
            }
            None => lines.push((Tone::Dim, "Inventário vazio. Pressione [d] para descobrir a rede (ou use `nethub scan`).".into())),
        }
        f.render_widget(Paragraph::new(to_text(&lines)).block(Block::bordered().title(" Contexto ")).wrap(Wrap { trim: false }), cols[1]);
    }

    fn draw_history(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .hist
            .iter()
            .map(|r| {
                let l = render::audit_record(r, false);
                ListItem::new(Line::from(Span::styled(l[0].1.clone(), style(l[0].0))))
            })
            .collect();
        let mut state = ListState::default();
        if !self.hist.is_empty() {
            state.select(Some(self.hist_sel));
        }
        let title = if self.hist_filter.is_empty() { " Histórico (evidências) ".to_string() } else { format!(" Histórico · filtro: {} ", self.hist_filter) };
        let list = List::new(items).block(Block::bordered().title(title)).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_stateful_widget(list, area, &mut state);
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let t = [
            (Tone::Head, "NetHub — capacidades verificadas, risco explícito, evidência"),
            (Tone::Normal, ""),
            (Tone::Normal, "Capacidade verificada: alcançável (porta aberta) → autenticada (a chave funciona) → autorizada (sudo/root)."),
            (Tone::Normal, "  A descoberta só prova \"alcançável\". Use [i] para PROVAR autenticação, sudo, systemd e SFTP."),
            (Tone::Normal, "Risco: BAIXO executa direto · MÉDIO pede y/n · ALTO exige digitar o alvo · CRÍTICO exige dry-run + digitar o alvo."),
            (Tone::Normal, "Evidência: toda ação gera um registro com hash encadeado (aba Histórico, [v] verifica a integridade)."),
            (Tone::Normal, ""),
            (Tone::Head, "Fluxo sugerido"),
            (Tone::Normal, "  1) [d] descobrir  2) configure usuário/chave: `nethub host set <host> --user U --key K`  3) [i] provar"),
            (Tone::Normal, "  4) [h] saúde  5) [a] ações: comando, serviços, SFTP, workflows (com dry-run)"),
            (Tone::Normal, ""),
            (Tone::Dim, "Não há senhas armazenadas: apenas ssh-agent / chaves. Teste somente em redes sob sua autorização."),
        ];
        let lines: Vec<RLine> = t.iter().map(|(a, b)| (*a, b.to_string())).collect();
        f.render_widget(Paragraph::new(to_text(&lines)).block(Block::bordered().title(" Ajuda ")).wrap(Wrap { trim: false }), area);
    }

    fn draw_popup(&self, f: &mut Frame, area: Rect) {
        match &self.mode {
            Mode::Normal => {}
            Mode::Busy(m) => {
                let r = centered(area, 34, 3);
                f.render_widget(Clear, r);
                f.render_widget(Paragraph::new(m.clone()).alignment(Alignment::Center).block(Block::bordered()), r);
            }
            Mode::Menu { ops, sel } => {
                let r = centered(area, 70, (ops.len() as u16 + 2).min(area.height));
                f.render_widget(Clear, r);
                let items: Vec<ListItem> = ops
                    .iter()
                    .map(|o| ListItem::new(Line::from(vec![Span::styled(format!("{:<8}", o.base_risk.label()), style(render::risk_tone(o.base_risk))), Span::raw(o.name.clone())])))
                    .collect();
                let mut st = ListState::default();
                st.select(Some(*sel));
                f.render_stateful_widget(List::new(items).block(Block::bordered().title(" Ações disponíveis (risco base) ")).highlight_style(Style::default().add_modifier(Modifier::REVERSED)), r, &mut st);
            }
            Mode::Input { title, labels, values, idx, .. } => {
                let r = centered(area, 72, labels.len() as u16 * 2 + 2);
                f.render_widget(Clear, r);
                let mut lines: Vec<RLine> = vec![];
                for (i, l) in labels.iter().enumerate() {
                    lines.push((if i == *idx { Tone::Head } else { Tone::Dim }, format!("{l}:")));
                    lines.push((Tone::Normal, format!("{}{}", values[i], if i == *idx { "▏" } else { "" })));
                }
                f.render_widget(Paragraph::new(to_text(&lines)).block(Block::bordered().title(format!(" {title} "))), r);
            }
            Mode::Confirm { report, typed, .. } => {
                let mut lines = render::dry_run(report);
                lines.push((Tone::Normal, String::new()));
                match report.confirmation {
                    ConfirmationLevel::Simple => lines.push((Tone::Warn, "Executar? [y] sim   [n] não".into())),
                    _ => {
                        lines.push((Tone::Bad, format!("Digite '{}' para confirmar e pressione Enter:", report.expected_target)));
                        lines.push((Tone::Normal, format!("> {typed}▏")));
                    }
                }
                let r = centered(area, 100, (lines.len() as u16 + 2).min(area.height));
                f.render_widget(Clear, r);
                f.render_widget(Paragraph::new(to_text(&lines)).block(Block::bordered().title(" Confirmar ação ")).wrap(Wrap { trim: false }), r);
            }
            Mode::Result { title, lines, scroll } => {
                let r = centered(area, area.width.saturating_sub(6), area.height.saturating_sub(4));
                f.render_widget(Clear, r);
                f.render_widget(Paragraph::new(to_text(lines)).block(Block::bordered().title(format!(" {title} "))).wrap(Wrap { trim: false }).scroll((*scroll, 0)), r);
            }
        }
    }
}

pub fn run(core: &Core) -> Result<i32, NhError> {
    let mut app = App::new(core.clone());
    let mut terminal = ratatui::init();
    let res = (|| -> std::io::Result<()> {
        loop {
            terminal.draw(|f| app.draw(f))?;
            while let Ok(m) = app.rx.try_recv() {
                app.on_msg(m);
            }
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(k) = event::read()? {
                    if k.kind == KeyEventKind::Press {
                        app.flash.clear();
                        if app.key(k.code, k.modifiers) {
                            return Ok(());
                        }
                    }
                }
            }
        }
    })();
    ratatui::restore();
    res.map_err(|e| NhError::Failed(format!("terminal: {e}")))?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{nmap::NmapScanner, sftp::SFTPAdapter, sqlite::SQLiteRepositories, ssh::OpenSSHExecutor};
    use ratatui::backend::TestBackend;
    use std::sync::Arc;

    fn app() -> App {
        let repo = Arc::new(SQLiteRepositories::open_in_memory().unwrap());
        let core = Core::with_adapters(repo, Arc::new(NmapScanner::new()), Arc::new(OpenSSHExecutor::new()), Arc::new(SFTPAdapter::new())).unwrap();
        // host manual sem SSH provado: nenhuma ação compatível (RN-04)
        core.inventory.add_manual("10.9.9.9", Some("srv-teste")).unwrap();
        App::new(core)
    }

    fn render_all(a: &App) -> String {
        let mut t = Terminal::new(TestBackend::new(110, 32)).unwrap();
        t.draw(|f| a.draw(f)).unwrap();
        t.backend().buffer().content().iter().map(|c| c.symbol()).collect::<String>()
    }

    #[test]
    fn renders_all_tabs_and_popups_without_panic() {
        let mut a = app();
        let s = render_all(&a);
        assert!(s.contains("srv-teste") && s.contains("Capacidades"));
        a.key(KeyCode::Char('2'), KeyModifiers::NONE);
        assert!(render_all(&a).contains("Histórico"));
        a.key(KeyCode::Char('3'), KeyModifiers::NONE);
        assert!(render_all(&a).contains("Fluxo sugerido"));
        a.key(KeyCode::Char('1'), KeyModifiers::NONE);
        // host sem capacidade provada: só "Verificar capacidades" é compatível (RN-04)
        a.key(KeyCode::Char('a'), KeyModifiers::NONE);
        match &a.mode {
            Mode::Menu { ops, .. } => assert_eq!(ops.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(), vec!["capability.inspect"]),
            _ => panic!("menu deveria abrir"),
        }
        assert!(render_all(&a).contains("Verificar capacidades"));
        a.key(KeyCode::Esc, KeyModifiers::NONE);
        a.key(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(render_all(&a).contains("Descobrir rede"));
        a.key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(a.mode, Mode::Normal));
        assert!(a.key(KeyCode::Char('q'), KeyModifiers::NONE));
    }

    #[test]
    fn tag_wizard_adds_tag() {
        let mut a = app();
        a.key(KeyCode::Char('t'), KeyModifiers::NONE);
        for c in "web".chars() {
            a.key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        a.key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(a.hosts[0].tags.contains(&"web".to_string()));
    }
}
