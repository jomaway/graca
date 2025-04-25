use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use layout::Flex;
use ratatui::widgets::{Block, Tabs};
use std::io;
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use ratatui::prelude::*;
use ratatui::{text::Line, widgets::Paragraph, DefaultTerminal, Frame};
use tracing::debug;

use crate::action::{Action, UserEvent};
use crate::command::Commands;
use crate::config::AppConfig;
use crate::export::export;
use crate::model::scale::{Grade, GradeScaleType, GradingScale};
use crate::model::students::StudentList;
use crate::model::Model;
use crate::ui::exam_result_table::{ExamResultTable, ExamResultTableRowData};
use crate::ui::exam_stats_chart::ExamChart;
use crate::ui::grading_scale_table::GradingScaleTable;
use crate::ui::theme::THEME;

#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    View,
    Edit,
    Command,
    // Help,
    Exited,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum SelectedTab {
    #[default]
    Scale,
    Result,
    Report,
}

impl SelectedTab {
    pub fn to_string(&self) -> String {
        match self {
            SelectedTab::Scale => "Scale [1]".into(),
            SelectedTab::Result => "Result [2]".into(),
            SelectedTab::Report => "Report [3]".into(),
        }
    }
}

pub struct App {
    config: AppConfig,
    mode: AppMode,
    model: Model,
    // calculator: GradeCalculator,
    scale_table: GradingScaleTable,
    result_table: ExamResultTable,
    exam_chart: ExamChart,
    input_field: Input,
    status_msg: Option<String>,
    selected_tab: SelectedTab,
}

impl App {
    pub fn new() -> Self {
        // only for testing
        // todo: remove this
        let mut m = Model::new();
        m.load_student_data(PathBuf::from("data/TestStudents.csv").as_path());
        let restab = ExamResultTable::new(&format!("{} 📔 MCR 📎 KA1", m.get_class_name()))
            .with_data(m.get_student_data());

        Self {
            config: AppConfig::new(),
            mode: AppMode::View,
            model: m,
            scale_table: GradingScaleTable::new(),
            result_table: restab,
            exam_chart: ExamChart::default(),
            input_field: Input::default(),
            status_msg: None,
            selected_tab: SelectedTab::default(),
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
        // self.calculator.total_points = points;
        self.model.scale.set_total_points(points as f64);
    }

    fn change_scale(&mut self, scale: GradeScaleType) {
        self.model.scale.change_scale_type(scale);
        self.update_accent_color();
    }

    fn update_accent_color(&mut self) {
        let scale = self.model.scale.scale_type();
        self.scale_table.set_accent_color(scale.color());
        self.result_table.set_accent_color(scale.color());
        self.exam_chart.set_accent_color(scale.color());
    }

    fn update(&mut self, action: Action) {
        // debug!("AVG: {}", self.model.grade_average());
        let mut chart_data = [0u8; 6];
        for (grade, count) in self.model.grade_distribution() {
            if (1..=6).contains(&grade) {
                chart_data[(grade - 1) as usize] = count as u8;
            }
        }
        self.exam_chart.set_data(&chart_data);
        self.result_table.set_data(self.model.get_student_data());
        self.scale_table.update(self.model.get_scale_data());
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
            SelectedTab::Scale => self.scale_table.render(table_area, frame.buffer_mut()),
            SelectedTab::Result => self.result_table.render(table_area, frame.buffer_mut()),
            SelectedTab::Report => self.exam_chart.render(table_area, frame.buffer_mut()),
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
        let titles = SelectedTab::iter().map(|tab| tab.to_string());
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .select(selected_tab_index)
            .divider("»")
            .render(area, buf);
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
                self.handle_key_event(key_event);
                self.update(Action::User(UserEvent::SwitchTab(SelectedTab::Report)));
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

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        // Terminate with CTRL+C
        if key_event.modifiers == KeyModifiers::CONTROL {
            if key_event.code == KeyCode::Char('c') {
                debug!("Should exit");
                self.exit();
            }
        }

        match key_event.code {
            KeyCode::F(1) => {
                self.selected_tab = SelectedTab::Scale;
            }
            KeyCode::F(2) => {
                self.selected_tab = SelectedTab::Result;
            }
            KeyCode::F(3) => {
                self.selected_tab = SelectedTab::Report;
            }
            _ => {}
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

                KeyCode::Char('I') => self.change_scale(GradeScaleType::IHK),
                KeyCode::Char('T') => self.change_scale(GradeScaleType::TECHNIKER),
                KeyCode::Char('L') => self.change_scale(GradeScaleType::LINEAR),
                KeyCode::Char('C') => self.change_scale(self.model.scale.scale_type().to_custom()),

                KeyCode::Char('.') => self.model.scale.toggle_half_points(),

                KeyCode::Char('q') => self.exit(),
                _ => match self.selected_tab {
                    SelectedTab::Scale => match key_event.code {
                        KeyCode::Down | KeyCode::Char('j') => self.scale_table.state.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.scale_table.state.select_previous()
                        }
                        KeyCode::Left | KeyCode::Char('h') => self.scale_table.select_col_min(),
                        KeyCode::Right | KeyCode::Char('l') => self.scale_table.select_col_max(),
                        KeyCode::Esc => self.scale_table.state.select_column(None),
                        KeyCode::PageUp => {
                            self.set_points(self.model.scale.max_points() as u32 + 1)
                        }

                        KeyCode::Char('+') => self.increase_points(),
                        KeyCode::PageDown => {
                            self.set_points(self.model.scale.max_points() as u32 - 1)
                        }
                        KeyCode::Char('-') => self.decrease_points(),
                        _ => {}
                    },
                    SelectedTab::Result => {
                        self.result_table.handle_event(key_event);
                    }
                    SelectedTab::Report => {}
                },
            },
            _ => {}
        }
    }

    fn increase_points(&mut self) {
        match self.scale_table.selected() {
            Some(grade) => {
                self.model
                    .scale
                    .increment_points_for_grade(Grade::try_from(grade).unwrap());
                self.update_accent_color();
            }
            None => {}
        }
    }

    fn decrease_points(&mut self) {
        match self.scale_table.selected() {
            Some(grade) => {
                self.model
                    .scale
                    .decrement_points_for_grade(Grade::try_from(grade).unwrap());
                self.update_accent_color();
            }
            None => {}
        }
    }

    fn exit(&mut self) {
        self.mode = AppMode::Exited;
    }
}
