use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Tabs};
use ratatui::{text::Line, Frame};
use std::io;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use tracing::debug;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::action::{Action, ModelAction};
use crate::config::AppConfig;
use crate::model::scale::GradeScaleType;
use crate::model::Model;
use crate::tui::Tui;
use crate::ui::report_tab::ExamChart;
use crate::ui::scale_tab::GradingScaleTable;
use crate::ui::students_tab::ExamResultTable;
use crate::ui::theme::{AppStyle, THEME};
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
    student_data_file_path: Option<PathBuf>,
    scale_tab: GradingScaleTable,
    results_tab: ExamResultTable,
    report_tab: ExamChart,
    input_field: Input,
    selected_tab: AppTab,
}

impl App {
    pub fn new() -> Self {
        let config = if let Ok(config) = AppConfig::read_config() {
            config
        } else {
            AppConfig::default()
        };

        Self {
            config,
            mode: AppMode::Normal,
            model: Model::new(),
            student_data_file_path: None,
            scale_tab: GradingScaleTable::new(GradeScaleType::IHK),
            results_tab: ExamResultTable::new(),
            report_tab: ExamChart::default(),
            input_field: Input::default(),
            selected_tab: AppTab::default(),
        }
    }

    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.model
            .scale
            .change_scale_type(config.get_default_scale());
        self.config = config;
        self
    }

    pub fn with_points(mut self, points: u32) -> Self {
        self.model.scale.set_max_points(points as f64);
        self
    }

    pub fn with_course(mut self, course_file_path: Option<PathBuf>) -> Self {
        if let Some(path_buf) = course_file_path {
            if let Err(e) = self.model.load_student_data(path_buf.as_path()) {
                debug!("{e}")
            } else {
                self.student_data_file_path = Some(path_buf);
            }
        };
        self
    }

    pub fn init(mut self) -> Self {
        self.model
            .scale
            .change_scale_type(self.config.get_default_scale());
        self.update(Action::UpdateView);
        self
    }

    fn update(&mut self, action: Action) {
        debug!("ACTION: {}", action);

        match action {
            Action::Quit => self.exit(),
            Action::EnterInsertMode => self.enter_insert_mode(),
            Action::LeaveInsertMode => self.leave_insert_mode(),
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
                self.report_tab
                    .set_data(&chart_data, self.model.grade_average());
                self.results_tab.set_data(self.model.get_student_data());
                self.scale_tab
                    .update(*self.model.scale.scale_type(), self.model.get_scale_data());
            }
            Action::LoadStudentList(path_buf) => {
                self.model
                    .load_student_data(path_buf.as_path())
                    .expect(&format!(
                        "Could not load student data from path '{}'",
                        path_buf.display()
                    ));
            }
            Action::UpdateModel(act) => {
                self.model.update(act);
                self.update(Action::UpdateView);
            }
            Action::ExportTo(_) => {
                if let Some(file_path) = self.student_data_file_path.clone() {
                    if let Err(e) = self.model.save_student_data(file_path.as_path()) {
                        tracing::error!("{e}")
                    }
                }
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        while self.mode != AppMode::Exited {
            tui.terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        tui.exit()?;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // main layout.
        let [header_area, main_area, help_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(1),
        ])
        .areas(area);

        // HEADER
        self.render_header_bar(header_area, frame.buffer_mut());

        // MAIN AREA
        let [table_area] = Layout::horizontal([Constraint::Max(80)])
            .margin(1)
            .flex(Flex::Center)
            .areas(main_area);

        match self.selected_tab {
            AppTab::Scale => self.scale_tab.render(table_area, frame.buffer_mut()),
            AppTab::Result => self.results_tab.render(table_area, frame.buffer_mut()),
            AppTab::Report => self.report_tab.render(table_area, frame.buffer_mut()),
        }

        // BOTTOM
        App::render_help_bar(help_area, frame.buffer_mut());
    }

    fn render_header_bar(&self, area: Rect, buf: &mut Buffer) {
        Block::default().style(THEME.top_bar()).render(area, buf);

        let scale_identifier_text = format!(" {} ", self.model.scale.scale_type().text());
        let point_identifier_text = format!(" {} PTs ", self.model.scale.max_points());
        let half_identifier_text = match self.model.scale.is_using_half_points() {
            true => ".",
            false => "",
        };

        let [identifier_area, tabs_area, version_area] = Layout::horizontal([
            Constraint::Min(
                (scale_identifier_text.len()
                    + point_identifier_text.len()
                    + half_identifier_text.len()) as u16,
            ),
            Constraint::Percentage(100),
            Constraint::Length(7),
        ])
        .areas(area);

        let scale_identifier = Span::from(scale_identifier_text)
            .style(THEME.indicator(Some(self.model.scale.scale_type())));
        let point_identifier = Span::from(point_identifier_text).style(THEME.tag(false));
        let half_identifier = Span::from(half_identifier_text).style(THEME.indicator(None));

        let identifier =
            Line::default().spans([scale_identifier, point_identifier, half_identifier]);

        let version = Span::from(format!(" {} ", env!("CARGO_PKG_NAME").to_uppercase()))
            .style(THEME.indicator(None));

        identifier.render(identifier_area, buf);
        self.render_tabs(tabs_area, buf);
        version.render(version_area, buf);
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let [tabs_area] = Layout::horizontal([Constraint::Length(37)])
            .flex(Flex::Center)
            .areas(area);

        let titles = AppTab::iter().map(|tab| tab.to_string());
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .select(selected_tab_index)
            .highlight_style(THEME.tab(true))
            .style(THEME.tab(false))
            .divider("»")
            .render(tabs_area, buf);
    }

    fn render_help_bar(area: Rect, buf: &mut Buffer) {
        let mut spans: Vec<Span> = GradeScaleType::iter()
            .flat_map(|scale_type| {
                [
                    Span::styled(
                        format!(" {} ", scale_type.key_binding()),
                        THEME.indicator(Some(&scale_type)),
                    ),
                    Span::styled(
                        format!(" {} ", scale_type.text()),
                        THEME.indicator(Some(&scale_type)).reversed(),
                    ),
                ]
            })
            .collect();

        spans.push(Span::styled(" Q ", THEME.indicator(None)));

        spans.push(Span::styled(" Quit ", THEME.indicator(None).reversed()));

        Line::from(spans)
            .centered()
            .style(THEME.bottom_bar())
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

    fn leave_insert_mode(&mut self) {
        self.input_field.reset();
        self.mode = AppMode::Normal;
    }

    fn enter_insert_mode(&mut self) {
        self.mode = AppMode::Insert;
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
                KeyCode::Esc => Some(Action::LeaveInsertMode),
                KeyCode::Enter => Some(Action::LeaveInsertMode),
                _ => {
                    self.input_field.handle_event(&Event::Key(key_event));
                    None
                }
            },
            AppMode::Normal => match key_event.code {
                KeyCode::F(1) | KeyCode::Char('1') => Some(Action::SwitchTab(AppTab::Scale)),
                KeyCode::F(2) | KeyCode::Char('2') => Some(Action::SwitchTab(AppTab::Result)),
                KeyCode::F(3) | KeyCode::Char('3') => Some(Action::SwitchTab(AppTab::Report)),
                KeyCode::Char(':') => Some(Action::EnterInsertMode),
                KeyCode::Char('I') => Some(Action::UpdateModel(ModelAction::SetScale(1))),
                KeyCode::Char('T') => Some(Action::UpdateModel(ModelAction::SetScale(2))),
                KeyCode::Char('L') => Some(Action::UpdateModel(ModelAction::SetScale(3))),
                KeyCode::Char('C') => Some(Action::UpdateModel(ModelAction::SetScale(4))),

                KeyCode::Char('.') => Some(Action::UpdateModel(ModelAction::ToggleHalfPoints)),

                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('e') => Some(Action::ExportTo(None)),

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
