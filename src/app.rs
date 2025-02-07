use ratatui::layout::Flex;
use ratatui::widgets::{Clear, TableState};
use ratatui::widgets::{BorderType, Cell, Row, Table};

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal, Frame,};


use crate::grade::*;

struct Theme {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
}

impl Theme {
    const fn new() -> Self {
        Self {
            buffer_bg: Color::Rgb(28, 28, 28),    // Dark
            header_bg: Color::Reset,  //
            header_fg: Color::Reset, //
            row_fg: Color::Rgb(213, 196, 161),
            selected_row_style_fg: Color::Yellow, // Yellow
            normal_row_color: Color::Rgb(60, 56, 54),        //gray
            alt_row_color: Color::Rgb(80, 73, 69),           // gray
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum AppState {
    Running,
    RunningEditPoints,
    RunningShowHelp,
    Exited,
}

pub struct App {
    state: AppState, 
    table_state: TableState,
    colors: Theme,
    data: GradeCalculator
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Running,
            table_state: TableState::default().with_selected(0),
            colors: Theme::new(),
            data: GradeCalculator::default()
        }
    }

    pub fn set_points(&mut self, points: u32) {
        self.data.points = points;
        // self.items = calculate_grade_ranges(self.points, &self.algo);
    }

    pub fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => (i + 1) % 6,
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    5
                } else {
                    (i - 1) % 6
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
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

        let [header_area,_, main_area, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(21),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let [table_area, _] = Layout::horizontal([
            Constraint::Max(80),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        self.render_header(header_area, frame.buffer_mut());
        self.render_table(table_area, frame.buffer_mut());

        
        if self.state == AppState::RunningEditPoints {
            let point_block = Paragraph::new(format!("{}", self.data.points))
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Max Points:"),
            );

            let area = popup_area(area, 60, 20);
            frame.render_widget(Clear, area);
            frame.render_widget(point_block, area);
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
        match key_event.code {
            KeyCode::Char(c) if c.is_digit(10) => {
                if self.state == AppState::RunningEditPoints && self.data.points < 100 {
                    self.set_points(self.data.points * 10 + c.to_digit(10).unwrap());
                }
            }
            KeyCode::Backspace => {
                if self.state == AppState::RunningEditPoints {
                    self.set_points(self.data.points / 10 );
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.next_row(),
            KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
            KeyCode::Char('p') => self.state = AppState::RunningEditPoints,
            KeyCode::Char('I') => self.data.scale = GradeScale::IHK,
            KeyCode::Char('T') => self.data.scale = GradeScale::TECHNIKER,
            KeyCode::Char('L') => self.data.scale = GradeScale::LINEAR,
            KeyCode::Char('C') => {
                // open dialog to add a custom scale
                todo!()
            },
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc | KeyCode::Enter => self.state = AppState::Running,

            _ => {}
        }
    }

    fn render_header(&mut self, area: Rect, buf: &mut Buffer) {
        let text = format!(" {} ", self.data.scale.text());

        let color = scale_color(&self.data.scale);

        let [scale_identifier_area, bar_area] =
        Layout::horizontal([Constraint::Min(text.len() as u16), Constraint::Percentage(100)])
            .areas(area);

        let scale_identifier = Paragraph::new(text)
            .style(Style::default().fg(Color::Black).bg(color));

        let bar = Paragraph::new("graca v0.1").right_aligned().style(Style::default().bg(Color::Rgb(60, 56, 54)));
        scale_identifier.render(scale_identifier_area, buf);
        bar.render(bar_area, buf);


    }

    fn render_table(&mut self, area: Rect, buf: &mut Buffer) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(scale_color(&self.data.scale)).bold();  // todo!()

        let header = ["GRADE", "FROM", "TO"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let items = self.data.calc(); //calc_ranges(self.data.points, &self.data.scale.values().to_vec());
        let rows = items.iter().enumerate().map(|(i, item)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };

            item.ref_array()
                .into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(3)
        });
        let bar = " █ ";
        let table = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Fill(2),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
        // .bg(self.colors.buffer_bg)
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
        .block(Block::default().borders(Borders::ALL).title("📋 Point Distribution."));
        StatefulWidget::render(table, area, buf, &mut self.table_state);
    }


    fn exit(&mut self) {
        self.state = AppState::Exited
    }

}


/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
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