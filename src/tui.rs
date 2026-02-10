use crate::config::Config;
use crate::killer::{self, KillConfig};
use crate::process::{ProcessInfo, Protocol};
use crate::scanner::create_scanner;
use crate::theme::Theme;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState,
};
use ratatui::Terminal;
use std::collections::HashSet;
use std::io::{self, stdout};
use std::time::{Duration, Instant};

const REFRESH_RATE: Duration = Duration::from_secs(2);
const TICK_RATE: Duration = Duration::from_millis(100);

struct App {
    processes: Vec<ProcessInfo>,
    table_state: TableState,
    selected: HashSet<usize>,
    should_quit: bool,
    last_refresh: Instant,
    status_message: Option<(String, Instant, StatusKind)>,
    show_help: bool,
    sort_column: SortColumn,
    sort_ascending: bool,
    filter_text: String,
    filter_mode: bool,
    config: Config,
    theme: Theme,
    show_confirm_dialog: bool,
    confirm_target_count: usize,
    zapping: Vec<(usize, Instant)>,
}

#[derive(Clone, Copy, PartialEq)]
enum StatusKind {
    Success,
    Error,
    Info,
}

#[derive(Clone, Copy, PartialEq)]
enum SortColumn {
    Port,
    Pid,
    Name,
    Protocol,
}

impl App {
    fn new() -> Self {
        let config = Config::load();
        let theme = match config.theme {
            crate::theme::ThemeVariant::Dark => Theme::dark(),
            crate::theme::ThemeVariant::Light => Theme::light(),
        };

        let mut app = Self {
            processes: Vec::new(),
            table_state: TableState::default(),
            selected: HashSet::new(),
            should_quit: false,
            last_refresh: Instant::now() - REFRESH_RATE,
            status_message: None,
            show_help: false,
            sort_column: SortColumn::Port,
            sort_ascending: true,
            filter_text: String::new(),
            filter_mode: false,
            config,
            theme,
            show_confirm_dialog: false,
            confirm_target_count: 0,
            zapping: Vec::new(),
        };
        app.refresh_processes();
        if !app.processes.is_empty() {
            app.table_state.select(Some(0));
        }
        app
    }

    fn refresh_processes(&mut self) {
        let scanner = create_scanner();
        let old_selection = self.current_process_key();

        if let Ok(mut procs) = scanner.find_all_listening() {
            self.sort_processes(&mut procs);
            self.processes = procs;
        }

        self.last_refresh = Instant::now();
        self.selected.clear();

        // Try to re-select the same process after refresh
        if let Some((pid, port)) = old_selection {
            let filtered = self.filtered_indices();
            for (table_row, &orig_idx) in filtered.iter().enumerate() {
                let p = &self.processes[orig_idx];
                if p.pid == pid && p.port == port {
                    self.table_state.select(Some(table_row));
                    return;
                }
            }
        }

        let filtered = self.filtered_indices();
        if !filtered.is_empty() {
            let sel = self.table_state.selected().unwrap_or(0);
            if sel >= filtered.len() {
                self.table_state.select(Some(filtered.len().saturating_sub(1)));
            }
        } else {
            self.table_state.select(None);
        }
    }

    fn current_process_key(&self) -> Option<(u32, u16)> {
        let filtered = self.filtered_indices();
        self.table_state
            .selected()
            .and_then(|i| filtered.get(i))
            .map(|&idx| {
                let p = &self.processes[idx];
                (p.pid, p.port)
            })
    }

    fn sort_processes(&self, procs: &mut [ProcessInfo]) {
        procs.sort_by(|a, b| {
            let ord = match self.sort_column {
                SortColumn::Port => a.port.cmp(&b.port),
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Protocol => {
                    let pa = matches!(a.protocol, Protocol::Tcp);
                    let pb = matches!(b.protocol, Protocol::Tcp);
                    pa.cmp(&pb)
                }
            };
            if self.sort_ascending {
                ord
            } else {
                ord.reverse()
            }
        });
    }

    fn filtered_indices(&self) -> Vec<usize> {
        if self.filter_text.is_empty() {
            return (0..self.processes.len()).collect();
        }
        let query = self.filter_text.to_lowercase();
        self.processes
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.name.to_lowercase().contains(&query)
                    || p.port.to_string().contains(&query)
                    || p.pid.to_string().contains(&query)
                    || p.command
                        .as_deref()
                        .map(|c| c.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn move_selection(&mut self, delta: i32) {
        let filtered = self.filtered_indices();
        if filtered.is_empty() {
            self.table_state.select(None);
            return;
        }
        let current = self.table_state.selected().unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, filtered.len() as i32 - 1) as usize;
        self.table_state.select(Some(next));
    }

    fn toggle_selection(&mut self) {
        let filtered = self.filtered_indices();
        if let Some(table_row) = self.table_state.selected() {
            if let Some(&orig_idx) = filtered.get(table_row) {
                if self.selected.contains(&orig_idx) {
                    self.selected.remove(&orig_idx);
                } else {
                    self.selected.insert(orig_idx);
                }
            }
        }
    }

    fn select_all(&mut self) {
        let filtered = self.filtered_indices();
        if self.selected.len() == filtered.len() {
            self.selected.clear();
        } else {
            self.selected = filtered.into_iter().collect();
        }
    }

    fn kill_selected(&mut self) {
        let filtered = self.filtered_indices();
        let targets: Vec<usize> = if self.selected.is_empty() {
            // Kill the currently highlighted one
            self.table_state
                .selected()
                .and_then(|row| filtered.get(row).copied())
                .into_iter()
                .collect()
        } else {
            self.selected.iter().copied().collect()
        };

        if targets.is_empty() {
            self.set_status("Nothing selected", StatusKind::Info);
            return;
        }

        let config = KillConfig::default();
        let mut killed = 0;
        let mut failed = 0;

        for &idx in &targets {
            if idx < self.processes.len() {
                let result = killer::kill_process(&self.processes[idx], &config);
                if result.success {
                    killed += 1;
                    // Add to zapping animation
                    self.zapping.push((idx, Instant::now()));
                } else {
                    failed += 1;
                }
            }
        }

        self.selected.clear();

        let msg = if failed == 0 {
            format!("Zapped {} process{}", killed, if killed != 1 { "es" } else { "" })
        } else {
            format!(
                "Zapped {}, {} failed (try sudo)",
                killed, failed
            )
        };
        let kind = if failed == 0 {
            StatusKind::Success
        } else {
            StatusKind::Error
        };
        self.set_status(&msg, kind);
    }

    fn cycle_sort(&mut self) {
        let next = match self.sort_column {
            SortColumn::Port => SortColumn::Pid,
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Protocol,
            SortColumn::Protocol => SortColumn::Port,
        };
        if next == self.sort_column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = next;
            self.sort_ascending = true;
        }
        self.refresh_processes();
    }

    fn set_status(&mut self, msg: &str, kind: StatusKind) {
        self.status_message = Some((msg.to_string(), Instant::now(), kind));
    }

    fn toggle_theme(&mut self) {
        self.config.theme = self.config.theme.toggle();
        self.theme = match self.config.theme {
            crate::theme::ThemeVariant::Dark => Theme::dark(),
            crate::theme::ThemeVariant::Light => Theme::light(),
        };
        if let Err(_) = self.config.save() {
            self.set_status("Failed to save theme preference", StatusKind::Error);
        } else {
            let theme_name = match self.config.theme {
                crate::theme::ThemeVariant::Dark => "Dark",
                crate::theme::ThemeVariant::Light => "Light",
            };
            self.set_status(&format!("Switched to {} theme", theme_name), StatusKind::Info);
        }
    }

    fn request_kill_confirmation(&mut self) {
        let filtered = self.filtered_indices();
        let targets: Vec<usize> = if self.selected.is_empty() {
            self.table_state
                .selected()
                .and_then(|row| filtered.get(row).copied())
                .into_iter()
                .collect()
        } else {
            self.selected.iter().copied().collect()
        };

        if targets.is_empty() {
            self.set_status("Nothing selected", StatusKind::Info);
            return;
        }

        if self.config.skip_confirm_dialog {
            self.kill_selected();
        } else {
            self.show_confirm_dialog = true;
            self.confirm_target_count = targets.len();
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Confirmation dialog captures input
        if self.show_confirm_dialog {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.show_confirm_dialog = false;
                    self.kill_selected();
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_confirm_dialog = false;
                    self.set_status("Kill cancelled", StatusKind::Info);
                }
                _ => {}
            }
            return;
        }

        // Filter mode captures text input
        if self.filter_mode {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.filter_mode = false;
                }
                KeyCode::Backspace => {
                    self.filter_text.pop();
                    self.table_state.select(if self.filtered_indices().is_empty() {
                        None
                    } else {
                        Some(0)
                    });
                }
                KeyCode::Char(c) => {
                    self.filter_text.push(c);
                    self.table_state.select(if self.filtered_indices().is_empty() {
                        None
                    } else {
                        Some(0)
                    });
                }
                _ => {}
            }
            return;
        }

        // Help overlay
        if self.show_help {
            self.show_help = false;
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Home | KeyCode::Char('g') => {
                if !self.filtered_indices().is_empty() {
                    self.table_state.select(Some(0));
                }
            }
            KeyCode::End | KeyCode::Char('G') => {
                let filtered = self.filtered_indices();
                if !filtered.is_empty() {
                    self.table_state.select(Some(filtered.len() - 1));
                }
            }
            KeyCode::Char(' ') => self.toggle_selection(),
            KeyCode::Char('a') => self.select_all(),
            KeyCode::Enter | KeyCode::Char('x') => self.request_kill_confirmation(),
            KeyCode::Char('r') => {
                self.refresh_processes();
                self.set_status("Refreshed", StatusKind::Info);
            }
            KeyCode::Char('s') => self.cycle_sort(),
            KeyCode::Char('/') => {
                self.filter_mode = true;
                self.filter_text.clear();
            }
            KeyCode::Char('t') => self.toggle_theme(),
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let mut app = App::new();
    let animation_duration = Duration::from_millis(app.config.animation_duration_ms);

    loop {
        // Clean up completed animations
        let now = Instant::now();
        let had_zapping = !app.zapping.is_empty();
        app.zapping.retain(|(_, start_time)| now.duration_since(*start_time) < animation_duration);

        // Refresh processes after animations complete
        if had_zapping && app.zapping.is_empty() {
            app.refresh_processes();
        }

        // Auto-refresh
        if app.last_refresh.elapsed() >= REFRESH_RATE && !app.filter_mode {
            app.refresh_processes();
        }

        // Clear expired status messages (after 4 seconds)
        if let Some((_, created, _)) = &app.status_message {
            if created.elapsed() > Duration::from_secs(4) {
                app.status_message = None;
            }
        }

        terminal.draw(|frame| draw(frame, &mut app))?;

        // Poll for events with tick rate
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn draw(frame: &mut ratatui::Frame, app: &mut App) {
    let area = frame.area();

    // Background
    frame.render_widget(
        Block::default().style(Style::default().bg(app.theme.background)),
        area,
    );

    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Length(1), // Filter bar
        Constraint::Min(5),   // Table
        Constraint::Length(1), // Status bar
        Constraint::Length(1), // Key hints
    ])
    .split(area);

    draw_header(frame, chunks[0], app);
    draw_filter_bar(frame, chunks[1], app);
    draw_table(frame, chunks[2], app);
    draw_status_bar(frame, chunks[3], app);
    draw_key_hints(frame, chunks[4], app);

    if app.show_confirm_dialog {
        draw_confirm_dialog(frame, area, app);
    }

    if app.show_help {
        draw_help_overlay(frame, area, app);
    }
}

fn draw_header(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let process_count = app.filtered_indices().len();
    let total = app.processes.len();
    let selected_count = app.selected.len();

    let title_spans = vec![
        Span::styled(
            " PortZap ",
            Style::default()
                .fg(app.theme.header_title)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} processes", process_count),
            Style::default().fg(app.theme.text_secondary),
        ),
        if total != process_count {
            Span::styled(
                format!(" (of {})", total),
                Style::default().fg(app.theme.text_tertiary),
            )
        } else {
            Span::raw("")
        },
        if selected_count > 0 {
            Span::styled(
                format!(" | {} selected", selected_count),
                Style::default()
                    .fg(app.theme.accent_secondary)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
    ];

    let header = Paragraph::new(Line::from(title_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.border))
            .style(Style::default().bg(app.theme.background_secondary)),
    );
    frame.render_widget(header, area);
}

fn draw_filter_bar(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    if app.filter_mode {
        let bar = Paragraph::new(Line::from(vec![
            Span::styled(
                " Filter: ",
                Style::default()
                    .fg(app.theme.accent_secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &app.filter_text,
                Style::default().fg(app.theme.text_default),
            ),
            Span::styled("_", Style::default().fg(app.theme.text_default).add_modifier(Modifier::SLOW_BLINK)),
        ]))
        .style(Style::default().bg(app.theme.background_tertiary));
        frame.render_widget(bar, area);
    } else if !app.filter_text.is_empty() {
        let bar = Paragraph::new(Line::from(vec![
            Span::styled(
                " Filtered: ",
                Style::default().fg(app.theme.text_secondary),
            ),
            Span::styled(
                &app.filter_text,
                Style::default()
                    .fg(app.theme.accent_secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  (/ to edit, Esc to clear)",
                Style::default().fg(app.theme.accent_tertiary),
            ),
        ]))
        .style(Style::default().bg(app.theme.background_tertiary));
        frame.render_widget(bar, area);
    } else {
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(app.theme.background)),
            area,
        );
    }
}

fn draw_table(frame: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let filtered = app.filtered_indices();

    let sort_indicator = |col: SortColumn| -> &str {
        if app.sort_column == col {
            if app.sort_ascending {
                " ▲"
            } else {
                " ▼"
            }
        } else {
            ""
        }
    };

    let header_cells = [
        Cell::from(format!("Port{}", sort_indicator(SortColumn::Port))),
        Cell::from(format!("PID{}", sort_indicator(SortColumn::Pid))),
        Cell::from(format!("Name{}", sort_indicator(SortColumn::Name))),
        Cell::from(format!("Proto{}", sort_indicator(SortColumn::Protocol))),
        Cell::from("Command"),
    ];
    let header = Row::new(header_cells)
        .style(
            Style::default()
                .fg(app.theme.text_default)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    // Build zapping index map for animation
    let mut zapping_indices = std::collections::HashSet::new();
    let animation_duration = Duration::from_millis(app.config.animation_duration_ms);
    let now = Instant::now();
    for (idx, start_time) in &app.zapping {
        let elapsed = now.duration_since(*start_time);
        if elapsed < animation_duration {
            zapping_indices.insert(*idx);
        }
    }

    let rows: Vec<Row> = filtered
        .iter()
        .map(|&orig_idx| {
            let p = &app.processes[orig_idx];
            let is_selected = app.selected.contains(&orig_idx);
            let is_zapping = zapping_indices.contains(&orig_idx);

            let marker = if is_selected { "● " } else { "  " };

            let proto_color = match p.protocol {
                Protocol::Tcp => app.theme.tcp_color,
                Protocol::Udp => app.theme.udp_color,
            };

            let row_fg = if is_selected {
                app.theme.selected_fg
            } else {
                app.theme.text_default
            };

            let cmd = p
                .command
                .as_deref()
                .unwrap_or("-")
                .rsplit('/')
                .next()
                .unwrap_or("-");

            // Apply animation to zapping processes
            if is_zapping {
                let lightning_chars = ["⚡", "✧", "✦"];
                let animation_frame = (now.elapsed().as_millis() / 100) % 3;
                let lightning = lightning_chars[animation_frame as usize];

                Row::new(vec![
                    Cell::from(format!("{}{} {}", marker, p.port, lightning)).style(
                        Style::default().fg(app.theme.accent_secondary)
                    ),
                    Cell::from(p.pid.to_string()).style(Style::default().fg(app.theme.accent_secondary)),
                    Cell::from(p.name.clone()).style(Style::default().fg(app.theme.accent_secondary).add_modifier(Modifier::BOLD)),
                    Cell::from(p.protocol.to_string()).style(Style::default().fg(app.theme.accent_secondary)),
                    Cell::from(cmd.to_string()).style(Style::default().fg(app.theme.accent_secondary)),
                ])
            } else {
                Row::new(vec![
                    Cell::from(format!("{}{}", marker, p.port)).style(Style::default().fg(
                        if is_selected {
                            app.theme.port_selected_fg
                        } else {
                            app.theme.port_fg
                        },
                    )),
                    Cell::from(p.pid.to_string()).style(Style::default().fg(row_fg)),
                    Cell::from(p.name.clone()).style(Style::default().fg(row_fg).add_modifier(Modifier::BOLD)),
                    Cell::from(p.protocol.to_string()).style(Style::default().fg(proto_color)),
                    Cell::from(cmd.to_string()).style(Style::default().fg(app.theme.command_color)),
                ])
            }
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(20),
        Constraint::Length(7),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(app.theme.border))
                .style(Style::default().bg(app.theme.background_secondary)),
        )
        .row_highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_status_bar(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let content = if let Some((msg, _, kind)) = &app.status_message {
        let (icon, color) = match kind {
            StatusKind::Success => ("✓", app.theme.success),
            StatusKind::Error => ("✗", app.theme.error),
            StatusKind::Info => ("●", app.theme.info),
        };
        Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color)),
            Span::styled(msg, Style::default().fg(color)),
        ])
    } else {
        let secs_ago = app.last_refresh.elapsed().as_secs();
        Line::from(vec![Span::styled(
            format!(" Last refreshed {}s ago", secs_ago),
            Style::default().fg(app.theme.accent_tertiary),
        )])
    };

    let bar = Paragraph::new(content).style(Style::default().bg(app.theme.background_secondary));
    frame.render_widget(bar, area);
}

fn draw_key_hints(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let hints = if app.filter_mode {
        vec![
            ("Enter/Esc", "confirm"),
            ("Backspace", "delete"),
        ]
    } else {
        vec![
            ("↑↓/jk", "navigate"),
            ("Space", "select"),
            ("x/Enter", "zap"),
            ("a", "all"),
            ("/", "filter"),
            ("s", "sort"),
            ("r", "refresh"),
            ("t", "theme"),
            ("?", "help"),
            ("q", "quit"),
        ]
    };

    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut s = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(app.theme.highlight_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(*desc, Style::default().fg(app.theme.command_color)),
            ];
            if i < hints.len() - 1 {
                s.push(Span::styled(
                    " │",
                    Style::default().fg(app.theme.border),
                ));
            }
            s
        })
        .collect();

    let bar = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(app.theme.background));
    frame.render_widget(bar, area);
}

fn draw_confirm_dialog(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let width = 40u16.min(area.width.saturating_sub(4));
    let height = 9u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let count_text = if app.confirm_target_count == 1 {
        "Kill 1 process?".to_string()
    } else {
        format!("Kill {} processes?", app.confirm_target_count)
    };

    let dialog_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            &count_text,
            Style::default()
                .fg(app.theme.error)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Y ",
                Style::default()
                    .fg(app.theme.background)
                    .bg(app.theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Confirm  "),
            Span::styled(
                " N ",
                Style::default()
                    .fg(app.theme.background)
                    .bg(app.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ]),
        Line::from(""),
    ];

    let dialog = Paragraph::new(dialog_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.error))
            .style(Style::default().bg(app.theme.background_tertiary)),
    );

    frame.render_widget(dialog, popup_area);
}

fn draw_help_overlay(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let width = 50u16.min(area.width.saturating_sub(4));
    let height = 21u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "PortZap Keyboard Shortcuts",
            Style::default()
                .fg(app.theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓ or j/k  ", Style::default().fg(app.theme.info)),
            Span::raw("Move selection up/down"),
        ]),
        Line::from(vec![
            Span::styled("  g / G        ", Style::default().fg(app.theme.info)),
            Span::raw("Jump to top/bottom"),
        ]),
        Line::from(vec![
            Span::styled("  Space        ", Style::default().fg(app.theme.info)),
            Span::raw("Toggle select process"),
        ]),
        Line::from(vec![
            Span::styled("  a            ", Style::default().fg(app.theme.info)),
            Span::raw("Select/deselect all"),
        ]),
        Line::from(vec![
            Span::styled("  x / Enter    ", Style::default().fg(app.theme.error)),
            Span::raw("Zap selected processes"),
        ]),
        Line::from(vec![
            Span::styled("  /            ", Style::default().fg(app.theme.info)),
            Span::raw("Search/filter processes"),
        ]),
        Line::from(vec![
            Span::styled("  s            ", Style::default().fg(app.theme.info)),
            Span::raw("Cycle sort column"),
        ]),
        Line::from(vec![
            Span::styled("  r            ", Style::default().fg(app.theme.info)),
            Span::raw("Refresh process list"),
        ]),
        Line::from(vec![
            Span::styled("  t            ", Style::default().fg(app.theme.info)),
            Span::raw("Toggle theme"),
        ]),
        Line::from(vec![
            Span::styled("  ?            ", Style::default().fg(app.theme.info)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc      ", Style::default().fg(app.theme.info)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press any key to close",
            Style::default().fg(app.theme.accent_tertiary),
        )),
    ];

    let help = Paragraph::new(help_text).block(
        Block::default()
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.highlight_fg))
            .style(Style::default().bg(app.theme.background_tertiary)),
    );

    frame.render_widget(help, popup_area);
}
