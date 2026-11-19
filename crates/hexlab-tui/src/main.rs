//! `hexlab-tui` — the local terminal HexLab.
//!
//! Full-screen platform, everything else in floating panels — the web
//! build's layout law, in the terminal: palette panel (left), telemetry
//! panel (right), kinematics + log along the bottom, the hex array in
//! the middle. Agents can drive the same state through `hexlab` CLI;
//! this is the human seat.
//!
//! Keys:
//!   arrows        move cursor (q/r)      Home/End  diagonal (q±1, r∓1)
//!   1..8          brush: H1 FH DD SH RF DS LB PW     0/x erase brush
//!   Enter/Space   place brush at cursor  Backspace erase at cursor
//!   p             cycle preset (carrier → recon → sandbox)
//!   [ / ]         size class             g/G gate −/+    b/B ballast −/+
//!   t i k w       toggles: thrusters, interlocks, stack, waves
//!   :             spoken command line (same grammar as `hexlab spoken`)
//!   M             BOM / Build Bay view   X  export brief view
//!   S             save city.json         q  quit
//!
//! `hexlab-tui --snapshot` renders one frame headlessly and prints it
//! (verification / Omarchy agent screenshot substitute).

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use hexlab_core::axial::Axial;
use hexlab_core::catalog::{asset_by_id, get_facility_bom, FACILITY_BOMS};
use hexlab_core::city::{City, StockedItem};
use hexlab_core::spoken::{parse_spoken, SpokenOp};
use hexlab_core::types::{ModuleType, SizeClass};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Grid,
    Bom,
    Export,
    Help,
}

struct App {
    city: City,
    cursor: Axial,
    brush: Option<ModuleType>, // None = erase
    size: SizeClass,
    view: View,
    preset_idx: usize,
    facility_idx: usize,
    input: Option<String>,
    log: Vec<String>,
    path: String,
}

const PRESETS: [&str; 3] = ["carrier", "recon", "sandbox"];

impl App {
    fn new(path: String) -> Self {
        let city = if std::path::Path::new(&path).exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|t| City::from_json(&t).ok())
                .unwrap_or_else(City::preset_carrier)
        } else {
            City::preset_carrier()
        };
        Self {
            city,
            cursor: Axial::new(0, 0),
            brush: Some(ModuleType::Core),
            size: SizeClass::Standard,
            view: View::Grid,
            preset_idx: 0,
            facility_idx: 0,
            input: None,
            log: vec!["HexLab TUI — ? for help, : for spoken commands".into()],
            path,
        }
    }

    fn log(&mut self, msg: String) {
        self.log.push(msg);
        if self.log.len() > 6 {
            self.log.remove(0);
        }
    }

    fn place_at_cursor(&mut self) {
        match self.brush {
            Some(m) => {
                self.city.place(self.cursor, m, self.size);
                self.log(format!("placed {} ({}) at {}", m.code(), self.size, self.cursor));
            }
            None => {
                if self.city.erase(self.cursor) {
                    self.log(format!("erased {}", self.cursor));
                }
            }
        }
    }

    fn cycle_preset(&mut self) {
        self.preset_idx = (self.preset_idx + 1) % PRESETS.len();
        let name = PRESETS[self.preset_idx];
        self.city = City::preset(name).unwrap_or_default();
        self.log(format!("preset {name} — {} hexes", self.city.len()));
    }

    fn cycle_size(&mut self, dir: i32) {
        let all = SizeClass::ALL;
        let i = all.iter().position(|s| *s == self.size).unwrap_or(1);
        let n = (i as i32 + dir).rem_euclid(4) as usize;
        self.size = all[n];
        self.log(format!("size class {}", self.size));
    }

    fn apply_spoken(&mut self, text: &str) {
        match parse_spoken(text) {
            SpokenOp::Help => self.view = View::Help,
            SpokenOp::ListFacilities => self.log(format!(
                "facilities: {}",
                FACILITY_BOMS.iter().map(|f| f.id).collect::<Vec<_>>().join(", ")
            )),
            SpokenOp::Install { asset_id, .. } => {
                let cell = self.city.preferred_stock_cell();
                let name = asset_by_id(&asset_id).map(|a| a.name).unwrap_or(&asset_id).to_string();
                self.city.stocked.push(StockedItem { asset_id, cell: cell.clone() });
                self.log(format!("installed {name} → {}", cell.as_deref().unwrap_or("no cell")));
            }
            SpokenOp::Bom { facility_id, stock } => {
                if let Some(bom) = get_facility_bom(&facility_id) {
                    self.facility_idx = FACILITY_BOMS.iter().position(|f| f.id == bom.id).unwrap_or(0);
                    self.view = View::Bom;
                    if stock {
                        let cell = self.city.preferred_stock_cell();
                        let mut n = 0;
                        for l in bom.lines {
                            for _ in 0..l.qty {
                                self.city.stocked.push(StockedItem {
                                    asset_id: l.asset_id.to_string(),
                                    cell: cell.clone(),
                                });
                                n += 1;
                            }
                        }
                        self.log(format!("stocked {n} {} items → {}", bom.id, cell.as_deref().unwrap_or("no cell")));
                    } else {
                        self.log(format!("bom {} — M to view", bom.id));
                    }
                }
            }
            SpokenOp::Unknown { raw } => self.log(format!("unknown: \"{raw}\"")),
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> bool {
        // Returns false to quit.
        if let Some(input) = &mut self.input {
            match key.code {
                KeyCode::Enter => {
                    let text = self.input.take().unwrap();
                    self.apply_spoken(&text);
                }
                KeyCode::Esc => self.input = None,
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => input.push(c),
                _ => {}
            }
            return true;
        }
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return false,
            (KeyCode::Char(':'), _) => self.input = Some(String::new()),
            (KeyCode::Left, _) => self.cursor.q -= 1,
            (KeyCode::Right, _) => self.cursor.q += 1,
            (KeyCode::Up, _) => self.cursor.r -= 1,
            (KeyCode::Down, _) => self.cursor.r += 1,
            (KeyCode::Home, _) => {
                self.cursor.q += 1;
                self.cursor.r -= 1;
            }
            (KeyCode::End, _) => {
                self.cursor.q -= 1;
                self.cursor.r += 1;
            }
            (KeyCode::Enter, _) | (KeyCode::Char(' '), _) => self.place_at_cursor(),
            (KeyCode::Backspace, _) | (KeyCode::Delete, _) => {
                if self.city.erase(self.cursor) {
                    self.log(format!("erased {}", self.cursor));
                }
            }
            (KeyCode::Char('p'), _) => self.cycle_preset(),
            (KeyCode::Char('['), _) => self.cycle_size(-1),
            (KeyCode::Char(']'), _) => self.cycle_size(1),
            (KeyCode::Char('g'), _) => {
                self.city.settings.gate_open = (self.city.settings.gate_open - 0.05).max(0.0);
            }
            (KeyCode::Char('G'), _) => {
                self.city.settings.gate_open = (self.city.settings.gate_open + 0.05).min(1.0);
            }
            (KeyCode::Char('b'), _) => {
                self.city.settings.ballast = (self.city.settings.ballast - 0.05).max(0.0);
            }
            (KeyCode::Char('B'), _) => {
                self.city.settings.ballast = (self.city.settings.ballast + 0.05).min(1.0);
            }
            (KeyCode::Char('M'), _) => self.view = if self.view == View::Bom { View::Grid } else { View::Bom },
            (KeyCode::Char('X'), _) => self.view = if self.view == View::Export { View::Grid } else { View::Export },
            (KeyCode::Char('?'), _) => self.view = if self.view == View::Help { View::Grid } else { View::Help },
            (KeyCode::Esc, _) => self.view = View::Grid,
            (KeyCode::Char('t'), _) => {
                self.city.settings.show_thrusters = !self.city.settings.show_thrusters;
            }
            (KeyCode::Char('i'), _) => {
                self.city.settings.show_interlocks = !self.city.settings.show_interlocks;
            }
            (KeyCode::Char('k'), _) => {
                self.city.settings.show_stack = !self.city.settings.show_stack;
            }
            (KeyCode::Char('w'), _) => {
                self.city.settings.show_waves = !self.city.settings.show_waves;
            }
            (KeyCode::Char('S'), _) => {
                match std::fs::write(&self.path, self.city.to_json()) {
                    Ok(()) => self.log(format!("saved → {}", self.path)),
                    Err(e) => self.log(format!("save failed: {e}")),
                }
            }
            (KeyCode::Tab, _) => {
                self.facility_idx = (self.facility_idx + 1) % FACILITY_BOMS.len();
                if self.view != View::Bom {
                    self.view = View::Bom;
                }
            }
            (KeyCode::Char(c), _) if ('1'..='8').contains(&c) => {
                let idx = c as usize - '1' as usize;
                self.brush = Some(ModuleType::ALL[idx]);
            }
            (KeyCode::Char('0'), _) | (KeyCode::Char('x'), _) => self.brush = None,
            _ => {}
        }
        true
    }
}

fn module_color(m: ModuleType) -> Color {
    match m {
        ModuleType::Core => Color::Cyan,
        ModuleType::Flight => Color::Yellow,
        ModuleType::DryDock => Color::Green,
        ModuleType::SubHub => Color::Magenta,
        ModuleType::Refinery => Color::Red,
        ModuleType::Desal => Color::Blue,
        ModuleType::Lab => Color::White,
        ModuleType::Power => Color::LightRed,
    }
}

/// Render the hex array as offset rows of cell blocks. Rows are r,
/// columns q, odd rows indented — a readable axial map in text.
fn grid_lines(app: &App, width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    // Bounds: city extents ± 1, at least radius 2 around origin.
    let (mut qmin, mut qmax, mut rmin, mut rmax) = (-3, 3, -3, 3);
    for (at, _) in app.city.iter() {
        qmin = qmin.min(at.q - 1);
        qmax = qmax.max(at.q + 1);
        rmin = rmin.min(at.r - 1);
        rmax = rmax.max(at.r + 1);
    }
    for r in rmin..=rmax {
        let mut spans: Vec<Span> = Vec::new();
        if r.rem_euclid(2) == 1 {
            spans.push(Span::raw("  "));
        }
        for q in qmin..=qmax {
            let at = Axial::new(q, r);
            let is_cursor = at == app.cursor;
            let (text, mut style) = match app.city.get(at) {
                Some(cell) => (
                    format!("[{}]", cell.module.code()),
                    Style::default().fg(module_color(cell.module)),
                ),
                None => (" ·  ".to_string(), Style::default().fg(Color::DarkGray)),
            };
            if is_cursor {
                style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
            }
            spans.push(Span::styled(text, style));
            spans.push(Span::raw(" "));
            if spans.iter().map(|s| s.content.len()).sum::<usize>() > width {
                break;
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

fn draw(frame: &mut ratatui::Frame, app: &App) {
    let root = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(9),
        ])
        .split(root);

    let t = app.city.telemetry();
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" HEXLAB ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(format!(
            "  modular floating Hex City — {} hexes · cursor {} · brush {} · size {} · {} ",
            t.total,
            app.cursor,
            app.brush.map(|m| m.code()).unwrap_or("ERASE"),
            app.size,
            app.path
        )),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" HEXLAB.SOMACOSF.COM // ANYMACHINE "));
    frame.render_widget(header, rows[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26),
            Constraint::Min(30),
            Constraint::Length(30),
        ])
        .split(rows[1]);

    // Floating panel: palette
    let mut palette: Vec<Line> = vec![Line::from("BRUSH")];
    for (i, m) in ModuleType::ALL.iter().enumerate() {
        let active = app.brush == Some(*m);
        palette.push(Line::from(Span::styled(
            format!(" {} {} {:<10} ×{}", i + 1, m.code(), m.as_str(), app.city.count(*m)),
            Style::default()
                .fg(module_color(*m))
                .add_modifier(if active { Modifier::BOLD | Modifier::UNDERLINED } else { Modifier::empty() }),
        )));
    }
    palette.push(Line::from(" 0 ERASE"));
    palette.push(Line::from(""));
    palette.push(Line::from(format!("PRESET {}", PRESETS[app.preset_idx])));
    palette.push(Line::from("p cycle · [ ] size"));
    frame.render_widget(
        Paragraph::new(palette).block(Block::default().borders(Borders::ALL).title(" 01 // PALETTE ")),
        cols[0],
    );

    // Centre: the platform itself (or an overlay view)
    match app.view {
        View::Grid => {
            frame.render_widget(
                Paragraph::new(grid_lines(app, cols[1].width as usize))
                    .block(Block::default().borders(Borders::ALL).title(" PLATFORM // FULLSCREEN ")),
                cols[1],
            );
        }
        View::Bom => {
            let bom = &FACILITY_BOMS[app.facility_idx];
            let mut lines = vec![Line::from(format!("BOM // {} (Tab = next facility)", bom.label))];
            for l in bom.lines {
                let a = asset_by_id(l.asset_id);
                lines.push(Line::from(format!(
                    " {:>2} × {:<26} {}",
                    l.qty,
                    a.map(|a| a.name).unwrap_or(l.asset_id),
                    a.map(|a| format!("{:.1}×{:.1}×{:.1} m", a.footprint.0, a.footprint.1, a.footprint.2)).unwrap_or_default()
                )));
            }
            frame.render_widget(
                Paragraph::new(lines).wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title(" BUILD BAY // BOM ")),
                cols[1],
            );
        }
        View::Export => {
            frame.render_widget(
                Paragraph::new(hexlab_core::export::x_thread(&app.city))
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title(" EXPORT // X REPLY ")),
                cols[1],
            );
        }
        View::Help => {
            frame.render_widget(
                Paragraph::new(hexlab_core::spoken::help_text() + "\n\nKEYS\n  arrows move · Enter place · 1-8 brush · 0 erase\n  p preset · [ ] size · g/G gate · b/B ballast\n  t/i/k/w toggles · : spoken · M bom · X export · S save · q quit")
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title(" HELP ")),
                cols[1],
            );
        }
    }

    // Floating panel: telemetry + kinematics
    let s = &app.city.settings;
    let tele = vec![
        Line::from(format!("HEX UNITS     {}", t.total)),
        Line::from(format!("DECK AREA     {:.0} m²", t.deck_area_m2)),
        Line::from(format!("INTERLOCKS    {}", t.interlock_nodes)),
        Line::from(format!("TOWERS        {}", t.towers)),
        Line::from(format!("THRUSTERS     {}", t.thrusters)),
        Line::from(format!("DISPLACEMENT  {:.0} t", t.displacement_t)),
        Line::from(format!("BAYS          {} ({} FH/{} DD/{} SH)", t.bays, t.flight, t.drydock, t.subhub)),
        Line::from(format!("STABILITY     {:.1}%", t.stability_pct)),
        Line::from(format!("STOCKED       {}", t.stocked)),
        Line::from(""),
        Line::from(format!("GATE          {:.0}° (g/G)", s.gate_open * 90.0)),
        Line::from(format!("BALLAST       {:.0}% (b/B)", s.ballast * 100.0)),
        Line::from(format!(
            "TOGGLES  T{} I{} K{} W{}",
            on(s.show_thrusters), on(s.show_interlocks), on(s.show_stack), on(s.show_waves)
        )),
    ];
    frame.render_widget(
        Paragraph::new(tele).block(Block::default().borders(Borders::ALL).title(" 03 // TELEMETRY ")),
        cols[2],
    );

    // Bottom: log / spoken input
    let bottom_text = match &app.input {
        Some(input) => vec![Line::from(Span::styled(
            format!("SPOKEN > {input}▌"),
            Style::default().fg(Color::Yellow),
        ))],
        None => app.log.iter().map(|l| Line::from(l.clone())).collect(),
    };
    frame.render_widget(
        Paragraph::new(bottom_text).block(Block::default().borders(Borders::ALL).title(" LOG // : spoken ")),
        rows[2],
    );

    let _ = Clear;
    let _ = Rect::default();
}

fn on(b: bool) -> &'static str {
    if b { "●" } else { "○" }
}

fn snapshot() -> String {
    let app = App::new("city.json".into());
    let backend = ratatui::backend::TestBackend::new(110, 32);
    let mut term = Terminal::new(backend).unwrap();
    term.draw(|f| draw(f, &app)).unwrap();
    let buf = term.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..buf.area.height {
        let mut line = String::new();
        for x in 0..buf.area.width {
            line.push_str(buf[(x, y)].symbol());
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .iter()
        .position(|a| a == "--city")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "city.json".to_string());
    if args.iter().any(|a| a == "--snapshot") {
        print!("{}", snapshot());
        return Ok(());
    }
    let mut app = App::new(path);
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut term = Terminal::new(ratatui::backend::CrosstermBackend::new(stdout()))?;
    loop {
        term.draw(|f| draw(f, &app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if !app.on_key(key) {
                    break;
                }
            }
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }
    }

    #[test]
    fn place_and_spoken_stock() {
        let mut app = App::new("nonexistent-test-city.json".into());
        let before = app.city.len();
        app.cursor = Axial::new(3, 0);
        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.city.len(), before + 1);
        app.on_key(key(KeyCode::Char(':')));
        for c in "stock the machine shop".chars() {
            app.on_key(key(KeyCode::Char(c)));
        }
        app.on_key(key(KeyCode::Enter));
        // machineshop BOM totals 2+2+1+1+1+3 = 10 items
        assert_eq!(app.city.stocked.len(), 10);
    }

    #[test]
    fn snapshot_renders_panels() {
        let s = snapshot();
        assert!(s.contains("HEXLAB"));
        assert!(s.contains("PALETTE"));
        assert!(s.contains("TELEMETRY"));
        assert!(s.contains("[SH]"));
    }
}
