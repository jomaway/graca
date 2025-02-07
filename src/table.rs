use ratatui::layout::Constraint;
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::{style::{Color, Modifier, Style}, widgets::{StatefulWidget, TableState}};

use crate::grade::Grade;

struct Theme {
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            header_bg: Color::Reset,  //
            header_fg: Color::Reset, //
            row_fg: Color::Rgb(213, 196, 161),
            selected_row_style_fg: Color::Yellow, // Yellow
            normal_row_color: Color::Rgb(60, 56, 54),        //gray
            alt_row_color: Color::Rgb(80, 73, 69),           // gray
        }
    }
}

pub struct GradeTable {
    state: TableState,
    data: Vec<Grade>,
    colors: Theme
}


impl GradeTable {
    pub fn new(data: Vec<Grade>) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            data,
            colors: Theme::default()
        }
    }

    pub fn data(&mut self, data: Vec<Grade>) {
        self.data = data
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % 6,
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    5
                } else {
                    (i - 1) % 6
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg).bold();  
    
        let header = ["GRADE", "FROM", "TO"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
    
        let rows = self.data.iter().enumerate().map(|(i, item)| {
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
        StatefulWidget::render(table, area, buf, &mut self.state);
    }

}



