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

use crate::action::{Action, ModelAction};
use crate::config::AppConfig;
use crate::model::scale::GradeScaleType;
use crate::model::Model;
use crate::ui::exam_result_table::ExamResultTable;
use crate::ui::grading_scale_table::GradingScaleTable;
use crate::ui::report_tab::ExamChart;
use crate::ui::theme::{BLACK, DARK_WHITE, LIGHT_GRAY, THEME};
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
                self.scale_tab.update(self.model.get_scale_data());
                self.update_accent_color();
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
            Action::ExportTo(path_buf) => {
                todo!();
            }
            _ => {}
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
        Block::default().style(THEME.bar_style).render(area, buf);

        // let text = format!(" {} ", self.model.scale.scale_type().text());
        let scale_identifier_text = format!(" {} ", self.model.scale.scale_type().text());
        let point_identifier_text = format!(" {} PTs ", self.model.scale.max_points());
        let half_identifier_text = match self.model.scale.is_using_half_points() {
            true => ".",
            false => "",
        };
        let color = self.model.scale.scale_type().color();

        let [identifier_area, tabs_area, version_area] = Layout::horizontal([
            Constraint::Min(
                (scale_identifier_text.len()
                    + point_identifier_text.len()
                    + half_identifier_text.len()) as u16,
            ),
            Constraint::Percentage(100),
            Constraint::Length(12),
        ])
        .areas(area);

        let scale_identifier =
            Span::from(scale_identifier_text).style(Style::default().fg(BLACK).bg(color));
        let point_identifier =
            Span::from(point_identifier_text).style(Style::default().fg(DARK_WHITE).bg(LIGHT_GRAY));
        let half_identifier =
            Span::from(half_identifier_text).style(Style::default().fg(BLACK).bg(Color::Magenta));

        let identifier =
            Line::default().spans([scale_identifier, point_identifier, half_identifier]);

        let version = Paragraph::new(format!(
            "{}::{}",
            env!("CARGO_PKG_NAME").to_uppercase(),
            env!("CARGO_PKG_VERSION")
        ))
        .right_aligned();

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
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default())
            .divider("»")
            .render(tabs_area, buf);
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

    fn leave_insert_mode(&mut self) {
        self.input_field.reset();
        self.mode = AppMode::Normal;
    }

    fn enter_insert_mode(&mut self) {
        self.status_msg = None;
        self.mode = AppMode::Insert;
    }

    // fn execute_command(&mut self) {
    //     match Commands::parse(self.input_field.value()) {
    //         Ok(Commands::SetMaxPoints(points)) => {
    //             self.status_msg = Some(format!("set max points to {}:", points));
    //             self.set_points(points);
    //         }
    //         Ok(Commands::Export(path_buf)) => {
    //             self.status_msg = Some(format!("export to{}", path_buf.display()));
    //             match export(path_buf.as_path(), &self.model.get_scale_data()) {
    //                 Ok(_) => {
    //                     self.status_msg = Some(format!("exportet to '{}'", path_buf.display()))
    //                 }
    //                 Err(e) => self.status_msg = Some(e.msg()),
    //             }
    //         }
    //         Err(msg) => self.status_msg = Some(msg),
    //     }
    // }

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
                KeyCode::Enter => {
                    // self.execute_command();
                    Some(Action::LeaveInsertMode)
                }
                _ => {
                    self.input_field.handle_event(&Event::Key(key_event));
                    None
                }
            },
            AppMode::Normal => match key_event.code {
                KeyCode::F(1) | KeyCode::Char('1') => {
                    // self.selected_tab = SelectedTab::Scale;
                    Some(Action::SwitchTab(AppTab::Scale))
                }
                KeyCode::F(2) | KeyCode::Char('2') => {
                    // self.selected_tab = SelectedTab::Result;
                    Some(Action::SwitchTab(AppTab::Result))
                }
                KeyCode::F(3) | KeyCode::Char('3') => {
                    // self.selected_tab = SelectedTab::Report;
                    Some(Action::SwitchTab(AppTab::Report))
                }
                KeyCode::Char(':') => Some(Action::EnterInsertMode),
                KeyCode::Char('I') => Some(Action::UpdateModel(ModelAction::SetScale(1))),
                KeyCode::Char('T') => Some(Action::UpdateModel(ModelAction::SetScale(2))),
                KeyCode::Char('L') => Some(Action::UpdateModel(ModelAction::SetScale(3))),
                KeyCode::Char('C') => Some(Action::UpdateModel(ModelAction::SetScale(4))),

                KeyCode::Char('.') => Some(Action::UpdateModel(ModelAction::ToggleHalfPoints)),

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
