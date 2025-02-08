use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::prelude::*;
use ratatui::{
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block,  Paragraph},
    DefaultTerminal, Frame,
};

use crate::fields::NumberInputField;
use crate::grade::*;
use crate::table::GradeTable;


#[derive(Debug, PartialEq)]
pub enum AppState {
    Running,
    RunningEditPoints,
    RunningShowHelp,
    Exited,
}

pub struct App {
    state: AppState,
    table: GradeTable,
    point_edit_field: NumberInputField,
    data: GradeCalculator,
}

impl App {
    pub fn new() -> Self {
        let data = GradeCalculator::default();
        Self {
            state: AppState::Running,
            table: GradeTable::new(data.calc()),
            point_edit_field: NumberInputField::new(),
            data,
        }
    }

    pub fn set_points(&mut self, points: u32) {
        self.data.points = points;
        self.table.update(self.data.calc());
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
            "<F1>".blue().bold(),
            " Set Points ".into(),
            "<p>".blue().bold(),
            " Quit ".into(),
            "<q> ".blue().bold(),
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

        let [_, table_area, _] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Max(80), Constraint::Fill(1)]).areas(main_area);

        // self.render_header(header_area, frame.buffer_mut());
        let text = format!(" {} ", self.data.scale.text());

        let color = scale_color(&self.data.scale);

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
            let input = Paragraph::new(format!(" max:{}",self.point_edit_field.get_input()))
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

        self.table.render(table_area, frame.buffer_mut());

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
                KeyCode::Char('j') | KeyCode::Down => self.table.next_row(),
                KeyCode::Char('k') | KeyCode::Up => self.table.previous_row(),
                KeyCode::Char('p') => self.state = AppState::RunningEditPoints,
                KeyCode::Char('I') => self.change_scale(GradeScale::IHK),
                KeyCode::Char('T') => self.change_scale(GradeScale::TECHNIKER),
                KeyCode::Char('L') => self.change_scale(GradeScale::LINEAR),
                KeyCode::Char('C') => {
                    // open dialog to add a custom scale
                    todo!()
                }
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
            AppState::RunningShowHelp => todo!(),
            AppState::Exited => todo!(),
        }
    }

    fn exit(&mut self) {
        self.state = AppState::Exited
    }

    fn change_scale(&mut self, scale: GradeScale) {
        self.table.set_selected_row_color(scale_color(&scale));
        self.data.scale = scale;
        self.table.update(self.data.calc());
    }

}



/// helper function to get scale colors
fn scale_color(scale: &GradeScale) -> Color {
    match scale {
        GradeScale::IHK => Color::Yellow,
        GradeScale::TECHNIKER => Color::Blue,
        GradeScale::LINEAR => Color::Green,
        GradeScale::CUSTOM(_) => Color::Red,
    }
}
