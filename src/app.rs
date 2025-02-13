use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::prelude::*;
use ratatui::widgets::Clear;
use ratatui::{
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

use crate::export::{CsvExporter, ExcelExporter, Exporter};
use crate::grade::*;
use crate::helpers::round_dp;
use crate::ui::{popup_area, render_help, ExportModal, GradeTable, NumberInputField};

#[derive(Debug, PartialEq)]
pub enum AppState {
    Running,
    RunningEditPoints,
    RunningShowHelp,
    Exporting,
    Exited,
}


const OUTPUT_PATH: &'static str = "/home/jonas/RDF/graca";

pub struct App {
    state: AppState,
    calculator: GradeCalculator,
    table: GradeTable,
    modal: ExportModal,
    point_edit_field: NumberInputField,
}

impl App {
    pub fn new() -> Self {

        Self {
            state: AppState::Running,
            calculator: GradeCalculator::new(),
            table: GradeTable::new(),
            modal: ExportModal::new(),
            point_edit_field: NumberInputField::new(),
        }
    }

    pub fn set_points(&mut self, points: u32) {
        self.calculator.total_points = points;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.state != AppState::Exited {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let instructions = Line::from(vec![
            " Help ".into(),
            "<F1>".magenta().bold(),
            " Quit ".into(),
            "<q> ".magenta().bold(),
            " Set Points ".into(),
            "<p>".blue().bold(),
        ]);
        let block = Block::bordered()
            .title_bottom(instructions.centered())
            .border_set(border::EMPTY);

        // render block around everything else.
        let inner = block.inner(area);

        frame.render_widget(block, area);

        // main layout.
        let [header_area, _, main_area, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(21),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let [_, table_area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Max(80),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        // self.render_header(header_area, frame.buffer_mut());
        let text = if self.state == AppState::RunningShowHelp {
            format!(" HELP ")
        } else {
            format!(" {} ", self.calculator.scale.text())
        };

        let color = if self.state == AppState::RunningShowHelp {
            Color::Magenta
        } else {
            scale_color(&self.calculator.scale)
        };

        let [scale_identifier_area, input_area, version_area] = Layout::horizontal([
            Constraint::Min(text.len() as u16),
            Constraint::Percentage(100),
            Constraint::Length(12),
        ])
        .areas(header_area);

        let scale_identifier =
            Paragraph::new(text).style(Style::default().fg(Color::Black).bg(color));

        let bar_style = Style::default().bg(Color::Rgb(60, 56, 54));

        let version = Paragraph::new("graca v0.1")
            .right_aligned()
            .style(bar_style);

        frame.render_widget(scale_identifier, scale_identifier_area);
        frame.render_widget(version, version_area);

        if self.state == AppState::RunningEditPoints {
            let input = Paragraph::new(format!(" max:{}", self.point_edit_field.get_input()))
                .style(bar_style.fg(Color::Yellow));

            frame.render_widget(input, input_area);

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after rendering
            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                // the plus 5 comes from the max: which is printed before.
                input_area.x + self.point_edit_field.get_index() as u16 + 5,
                input_area.y,
            ))
        } else {
            frame.render_widget(Paragraph::new("").style(bar_style), input_area);
        }

        if self.state == AppState::RunningShowHelp {
            render_help(table_area, frame.buffer_mut());
        } else {
            self.table
                .render(table_area, frame.buffer_mut(), &self.calculator.calc());
        }

        if self.state == AppState::Exporting {
            let modal_area = popup_area(area, 60, 20);

            frame.render_widget(Clear, modal_area);
            self.modal.render(modal_area, frame.buffer_mut());
        }
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

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.state {
            AppState::Running => match key_event.code {
                KeyCode::Char('p') => self.state = AppState::RunningEditPoints,
                KeyCode::Char('.') => self.calculator.toggle_steps(),

                KeyCode::Char('I') => self.change_scale(GradeScale::IHK),
                KeyCode::Char('T') => self.change_scale(GradeScale::TECHNIKER),
                KeyCode::Char('L') => self.change_scale(GradeScale::LINEAR),
                KeyCode::Char('C') => self.change_scale(self.calculator.scale.to_custom()),

                KeyCode::Down | KeyCode::Char('j') => self.table.next_row(),
                KeyCode::Up | KeyCode::Char('k') => self.table.previous_row(),
                KeyCode::PageUp | KeyCode::Char('+') => {
                    if self.calculator.scale.is_custom() {
                        match self.table.selected() {
                            Some(i) => {
                                // get current min point
                                if let Some(min) = self.calculator.min_for(i as u32 + 1) {
                                    self.calculator.scale.change(
                                        i,
                                        round_dp(
                                            (min + 1.0) / self.calculator.total_points as f64,
                                            2,
                                        ),
                                    );
                                };
                            }
                            None => {}
                        }
                    }
                }

                KeyCode::PageDown | KeyCode::Char('-') => {
                    if self.calculator.scale.is_custom() {
                        match self.table.selected() {
                            Some(i) => {
                                // get current min point
                                if let Some(min) = self.calculator.min_for(i as u32 + 1) {
                                    self.calculator.scale.change(
                                        i,
                                        round_dp(
                                            (min - 1.0) / self.calculator.total_points as f64,
                                            2,
                                        ),
                                    );
                                };
                            }
                            None => {}
                        }
                    }
                }

                KeyCode::Char('e') => self.state = AppState::Exporting,
                KeyCode::F(1) => self.state = AppState::RunningShowHelp,
                KeyCode::Char('q') => self.exit(),
                _ => {}
            },
            AppState::RunningEditPoints => match key_event.code {
                KeyCode::Char(c) if c.is_digit(10) => self.point_edit_field.enter_char(c),
                KeyCode::Backspace => self.point_edit_field.delete_char(),
                KeyCode::Left => self.point_edit_field.move_cursor_left(),
                KeyCode::Right => self.point_edit_field.move_cursor_right(),
                KeyCode::Esc => self.state = AppState::Running,
                KeyCode::Enter => {
                    let points = self.point_edit_field.get_number();
                    self.set_points(points);
                    self.state = AppState::Running;
                }
                _ => {}
            },
            AppState::RunningShowHelp => match key_event.code {
                KeyCode::Esc => self.state = AppState::Running,
                KeyCode::Char('q') => self.exit(),
                _ => {}
            },
            AppState::Exporting => match key_event.code {
                KeyCode::Esc => self.state = AppState::Running,
                KeyCode::Up => self.modal.list_state.select(Some(0)),
                KeyCode::Down => self.modal.list_state.select(Some(1)),
                KeyCode::Enter => {
                    if let Some(selected) = self.modal.list_state.selected() {
                        let data = self.calculator.calc();
                        if 0 == selected {
                            CsvExporter::new(OUTPUT_PATH).export(&data).expect("Export csv file.");
                        } else if 1 == selected {
                            ExcelExporter::new(OUTPUT_PATH).export(&data).expect("Export excel file.");
                        }
                    }
                    self.state = AppState::Running;
                }
                _ => {}
            },
            AppState::Exited => {}
        }
    }

    fn exit(&mut self) {
        self.state = AppState::Exited
    }

    fn change_scale(&mut self, scale: GradeScale) {
        self.table.set_selected_row_color(scale_color(&scale));
        self.calculator.scale = scale;
    }
}

/// helper function to get scale colors
fn scale_color(scale: &GradeScale) -> Color {
    match scale {
        GradeScale::IHK => Color::Yellow,
        GradeScale::TECHNIKER => Color::Blue,
        GradeScale::LINEAR => Color::Green,
        GradeScale::Custom(_) => Color::LightRed,
    }
}
