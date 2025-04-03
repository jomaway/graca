use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::Block;
use std::io;
use strum::IntoEnumIterator;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use ratatui::prelude::*;
use ratatui::{text::Line, widgets::Paragraph, DefaultTerminal, Frame};
use tracing::debug;

use crate::command::Commands;
use crate::config::AppConfig;
use crate::export::export;
use crate::grade::*;
use crate::grade_table::GradeTable;
use crate::theme::THEME;

#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    View,
    Command,
    // Help,
    Exited,
}

pub struct App {
    config: AppConfig,
    // state: AppState,
    mode: AppMode,
    calculator: GradeCalculator,
    table: GradeTable,
    input_field: Input,
    status_msg: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            config: AppConfig::new(),
            // state: AppState::Running,
            mode: AppMode::View,
            calculator: GradeCalculator::new(),
            table: GradeTable::new(),
            input_field: Input::default(),
            status_msg: None,
        }
    }

    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.change_scale(config.get_default_scale());
        self.config = config;
        self
    }

    pub fn with_points(mut self, points: u32) -> Self {
        self.set_points(points);
        self
    }

    pub fn set_points(&mut self, points: u32) {
        self.calculator.total_points = points;
    }

    fn change_scale(&mut self, scale: GradeScale) {
        self.table.set_accent_color(scale.color());
        self.calculator.scale = scale;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.mode != AppMode::Exited {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // main layout.
        let [header_area, _, main_area, _, command_area, help_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(21),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        // HEADER
        self.render_header_bar(header_area, frame.buffer_mut());

        // MAIN AREA
        let [_, table_area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Max(80),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        self.table.update_data(self.calculator.calc());
        self.table.render(table_area, frame.buffer_mut());

        // BOTTOM
        self.render_command_bar(command_area, frame.buffer_mut());
        App::render_help_bar(help_area, frame.buffer_mut());
    }

    fn render_header_bar(&self, area: Rect, buf: &mut Buffer) {
        Block::default().style(THEME.bar_style).render(area, buf);

        let text = format!(" {} ", self.calculator.scale.text());
        let color = self.calculator.scale.color();

        let [identifier_area, _, version_area] = Layout::horizontal([
            Constraint::Min(text.len() as u16),
            Constraint::Percentage(100),
            Constraint::Length(12),
        ])
        .areas(area);

        let identifier = Paragraph::new(text).style(Style::default().fg(Color::Black).bg(color));
        let version = Paragraph::new(format!(
            "{} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .right_aligned();

        identifier.render(identifier_area, buf);
        version.render(version_area, buf);
    }

    fn render_command_bar(&self, area: Rect, buf: &mut Buffer) {
        let text = if self.mode == AppMode::Command {
            format!(">>> {}", self.input_field.value())
        } else if let Some(msg) = &self.status_msg {
            format!("Status: {}", msg)
        } else {
            "".into()
        };

        Paragraph::new(text)
            .style(THEME.bar_style)
            .render(area, buf);
    }

    fn render_help_bar(area: Rect, buf: &mut Buffer) {
        let mut keys: Vec<(&str, &str, Color)> = GradeScale::iter()
            .map(|s| (s.key_binding(), s.text(), s.color()))
            .collect();

        keys.push(("Q", "Quit", Color::Magenta));

        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|(key, desc, color)| {
                let key = Span::styled(
                    format!(" {key} "),
                    THEME.key_binding.key.bg(color.to_owned()),
                );
                let desc = Span::styled(
                    format!(" {desc} "),
                    THEME.key_binding.description.fg(color.to_owned()),
                );
                [key, desc]
            })
            .collect();

        Line::from(spans)
            .centered()
            .style((Color::Indexed(236), Color::Indexed(232)))
            .render(area, buf);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn leave_command_mode(&mut self) {
        self.input_field.reset();
        self.mode = AppMode::View;
    }

    fn enter_command_mode(&mut self) {
        self.status_msg = None;
        self.mode = AppMode::Command;
    }

    fn execute_command(&mut self) {
        match Commands::parse(self.input_field.value()) {
            Ok(Commands::SetMaxPoints(points)) => {
                self.status_msg = Some(format!("set max points to {}:", points));
                self.calculator.total_points = points
            }
            Ok(Commands::Export(path_buf)) => {
                self.status_msg = Some(format!("export to{}", path_buf.display()));
                match export(path_buf.as_path(), &self.calculator.calc()) {
                    Ok(_) => {
                        self.status_msg = Some(format!("exportet to '{}'", path_buf.display()))
                    }
                    Err(e) => self.status_msg = Some(e.msg()),
                }
            }
            Err(msg) => self.status_msg = Some(msg),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        // Terminate with CTRL+C
        if key_event.modifiers == KeyModifiers::CONTROL {
            if key_event.code == KeyCode::Char('c') {
                debug!("Should exit");
                self.exit();
            }
        }

        match self.mode {
            AppMode::Command => match key_event.code {
                KeyCode::Esc => self.leave_command_mode(),
                KeyCode::Enter => {
                    self.execute_command();
                    self.leave_command_mode();
                }
                _ => {
                    self.input_field.handle_event(&Event::Key(key_event));
                }
            },
            AppMode::View => match key_event.code {
                KeyCode::Char(':') => self.enter_command_mode(),
                KeyCode::Char('p') => {
                    self.input_field = "set-points ".into();
                    self.enter_command_mode();
                }
                KeyCode::Char('e') => {
                    self.input_field = "export-to ".into();
                    self.enter_command_mode();
                }

                KeyCode::Char('I') => self.change_scale(GradeScale::IHK),
                KeyCode::Char('T') => self.change_scale(GradeScale::TECHNIKER),
                KeyCode::Char('L') => self.change_scale(GradeScale::LINEAR),
                KeyCode::Char('C') => self.change_scale(self.calculator.scale.to_custom()),

                KeyCode::Down | KeyCode::Char('j') => self.table.next_row(),
                KeyCode::Up | KeyCode::Char('k') => self.table.previous_row(),
                KeyCode::Left | KeyCode::Char('h') => self.table.select_col_min(),
                KeyCode::Right | KeyCode::Char('l') => self.table.select_col_max(),
                KeyCode::Esc => self.table.state.select_column(None),
                KeyCode::Char('.') => self.calculator.toggle_steps(),
                KeyCode::PageUp | KeyCode::Char('+') => self.increase_points(),
                KeyCode::PageDown | KeyCode::Char('-') => self.decrease_points(),

                KeyCode::Char('q') => self.exit(),
                _ => {}
            },
            _ => {}
        }
    }

    fn increase_points(&mut self) {
        self.change_scale(self.calculator.scale.to_custom());
        match self.table.selected() {
            Some(i) => {
                // get current min point
                if let Some(min) = self.calculator.min_for(i as u32 + 1) {
                    self.calculator.scale.change(
                        i,
                        round_dp((min + 1.0) / self.calculator.total_points as f64, 2),
                    );
                };
            }
            None => {}
        }
    }

    fn decrease_points(&mut self) {
        self.change_scale(self.calculator.scale.to_custom());

        match self.table.selected() {
            Some(i) => {
                // get current min point
                if let Some(min) = self.calculator.min_for(i as u32 + 1) {
                    self.calculator.scale.change(
                        i,
                        round_dp((min - 1.0) / self.calculator.total_points as f64, 2),
                    );
                };
            }
            None => {}
        }
    }

    fn exit(&mut self) {
        self.mode = AppMode::Exited;
    }
}
