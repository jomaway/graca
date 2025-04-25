use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use layout::Flex;
use ratatui::widgets::{Block, Tabs};
use std::io;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use ratatui::prelude::*;
use ratatui::{text::Line, widgets::Paragraph, DefaultTerminal, Frame};
use tracing::debug;

use crate::action::{Action, ScaleAction};
use crate::command::Commands;
use crate::config::AppConfig;
use crate::export::export;
use crate::model::scale::{Grade, GradeScaleType};
use crate::model::Model;
use crate::ui::exam_result_table::ExamResultTable;
use crate::ui::exam_stats_chart::ExamChart;
use crate::ui::grading_scale_table::GradingScaleTable;
use crate::ui::theme::THEME;
use crate::ui::AppTab;

#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Insert,
    // Help,
    Exited,
}

pub struct App {
    config: AppConfig,
    mode: AppMode,
    model: Model,
    scale_tab: GradingScaleTable,
    results_tab: ExamResultTable,
    report_tab: ExamChart,
    input_field: Input,
    status_msg: Option<String>,
    selected_tab: AppTab,
}

impl App {
    pub fn new() -> Self {
        // only for testing
        // todo: remove this
        let mut m = Model::new();
        m.load_student_data(PathBuf::from("data/TestStudents.csv").as_path());
        // &format!("{} 📔 MCR 📎 KA1", m.get_class_name())
        let restab = ExamResultTable::new()
            .with_title(&m.get_class_name())
            .with_data(m.get_student_data());

        Self {
            config: AppConfig::new(),
            mode: AppMode::Normal,
            model: m,
            scale_tab: GradingScaleTable::new(),
            results_tab: restab,
            report_tab: ExamChart::default(),
            input_field: Input::default(),
            status_msg: None,
            selected_tab: AppTab::default(),
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

    pub fn init(mut self) -> Self {
        self.update(Action::UpdateView);
        self
    }

    pub fn set_points(&mut self, points: u32) {
        // self.calculator.total_points = points;
        self.model.scale.set_max_points(points as f64);
    }

    fn change_scale(&mut self, scale: GradeScaleType) {
        self.model.scale.change_scale_type(scale);
        self.update_accent_color();
    }

    fn update_accent_color(&mut self) {
        let scale = self.model.scale.scale_type();
        self.scale_tab.set_accent_color(scale.color());
        self.results_tab.set_accent_color(scale.color());
        self.report_tab.set_accent_color(scale.color());
    }

    fn update(&mut self, action: Action) {
        debug!("ACTION: {}", action);

        match action {
            Action::Quit => self.exit(),
            Action::ProcessCommand(_) => todo!(),
            Action::EnterCommandMode => self.enter_command_mode(),
            Action::LeaveCommandMode => self.leave_command_mode(),
            Action::SwitchTab(selected_tab) => {
                self.selected_tab = selected_tab;
                self.update(Action::UpdateView);
            }
            Action::UpdateView => {
                let mut chart_data = [0u8; 6];
                for (grade, count) in self.model.grade_distribution() {
                    if (1..=6).contains(&grade) {
                        chart_data[(grade - 1) as usize] = count as u8;
                    }
                }
                self.report_tab.set_data(&chart_data);
                self.results_tab.set_data(self.model.get_student_data());
                self.scale_tab.update(self.model.get_scale_data());
                self.update_accent_color();
            }
            _ => {
                self.model.update(action);
                self.update(Action::UpdateView);
            }
        }
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
            Constraint::Percentage(100),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        // HEADER
        self.render_header_bar(header_area, frame.buffer_mut());

        // MAIN AREA
        let [table_area] = Layout::horizontal([Constraint::Max(80)])
            .flex(Flex::Center)
            .areas(main_area);

        match self.selected_tab {
            AppTab::Scale => self.scale_tab.render(table_area, frame.buffer_mut()),
            AppTab::Result => self.results_tab.render(table_area, frame.buffer_mut()),
            AppTab::Report => self.report_tab.render(table_area, frame.buffer_mut()),
        }

        // BOTTOM
        self.render_command_bar(command_area, frame.buffer_mut());
        App::render_help_bar(help_area, frame.buffer_mut());
    }

    fn render_header_bar(&self, area: Rect, buf: &mut Buffer) {
        Block::default().style(THEME.bar_style).render(area, buf);

        let text = format!(" {} ", self.model.scale.scale_type().text());
        let color = self.model.scale.scale_type().color();

        let [identifier_area, _, tabs_area, version_area] = Layout::horizontal([
            Constraint::Min(text.len() as u16),
            Constraint::Length(1),
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
        self.render_tabs(tabs_area, buf);
        version.render(version_area, buf);
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = AppTab::iter().map(|tab| tab.to_string());
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .select(selected_tab_index)
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .divider("»")
            .render(area, buf);
    }

    fn render_command_bar(&self, area: Rect, buf: &mut Buffer) {
        let text = if self.mode == AppMode::Insert {
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
        let mut keys: Vec<(&str, &str, Color)> = GradeScaleType::iter()
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
                if let Some(action) = self.handle_key_event(key_event) {
                    self.update(action);
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn leave_command_mode(&mut self) {
        self.input_field.reset();
        self.mode = AppMode::Normal;
    }

    fn enter_command_mode(&mut self) {
        self.status_msg = None;
        self.mode = AppMode::Insert;
    }

    fn execute_command(&mut self) {
        match Commands::parse(self.input_field.value()) {
            Ok(Commands::SetMaxPoints(points)) => {
                self.status_msg = Some(format!("set max points to {}:", points));
                self.set_points(points);
            }
            Ok(Commands::Export(path_buf)) => {
                self.status_msg = Some(format!("export to{}", path_buf.display()));
                match export(path_buf.as_path(), &self.model.get_scale_data()) {
                    Ok(_) => {
                        self.status_msg = Some(format!("exportet to '{}'", path_buf.display()))
                    }
                    Err(e) => self.status_msg = Some(e.msg()),
                }
            }
            Err(msg) => self.status_msg = Some(msg),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        // Terminate with CTRL+C
        if key_event.modifiers == KeyModifiers::CONTROL {
            if key_event.code == KeyCode::Char('c') {
                debug!("Should exit");
                self.exit();
            }
        }

        match self.mode {
            AppMode::Insert => match key_event.code {
                KeyCode::Esc => Some(Action::LeaveCommandMode),
                KeyCode::Enter => {
                    self.execute_command();
                    Some(Action::LeaveCommandMode)
                }
                _ => {
                    self.input_field.handle_event(&Event::Key(key_event));
                    None
                }
            },
            AppMode::Normal => match key_event.code {
                KeyCode::F(1) => {
                    // self.selected_tab = SelectedTab::Scale;
                    Some(Action::SwitchTab(AppTab::Scale))
                }
                KeyCode::F(2) => {
                    // self.selected_tab = SelectedTab::Result;
                    Some(Action::SwitchTab(AppTab::Result))
                }
                KeyCode::F(3) => {
                    // self.selected_tab = SelectedTab::Report;
                    Some(Action::SwitchTab(AppTab::Report))
                }
                KeyCode::Char(':') => Some(Action::EnterCommandMode),
                KeyCode::Char('I') => Some(Action::ChangeScale(ScaleAction::SetScale(1))),
                KeyCode::Char('T') => Some(Action::ChangeScale(ScaleAction::SetScale(2))),
                KeyCode::Char('L') => Some(Action::ChangeScale(ScaleAction::SetScale(3))),
                KeyCode::Char('C') => Some(Action::ChangeScale(ScaleAction::SetScale(4))),

                KeyCode::Char('.') => Some(Action::ChangeScale(ScaleAction::ToggleHalfPoints)),

                KeyCode::Char('q') => Some(Action::Quit),

                _ => match self.selected_tab {
                    AppTab::Scale => self.scale_tab.handle_event(key_event),
                    AppTab::Result => self.results_tab.handle_event(key_event),
                    AppTab::Report => None,
                },
            },
            _ => None,
        }
    }

    fn exit(&mut self) {
        self.mode = AppMode::Exited;
    }
}
