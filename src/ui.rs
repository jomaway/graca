use ratatui::layout::{Alignment, Constraint};
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{StatefulWidget, TableState},
};

use crate::grade::Grade;
use crate::helpers::round_dp;

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
            header_bg: Color::Reset,
            header_fg: Color::Reset,
            row_fg: Color::Rgb(213, 196, 161),
            selected_row_style_fg: Color::Yellow,
            normal_row_color: Color::Rgb(60, 56, 54),
            alt_row_color: Color::Rgb(80, 73, 69),
        }
    }
}

pub struct GradeTable {
    state: TableState,
    data: Vec<Grade>,
    colors: Theme,
}

impl GradeTable {
    pub fn new(data: Vec<Grade>) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            data,
            colors: Theme::default(),
        }
    }

    pub fn update(&mut self, data: Vec<Grade>) {
        self.data = data
    }

    // return the min value of the selected row
    pub fn selected_min(&self) -> Option<f64> {
        match self.state.selected() {
            Some(i) => Some(self.data[i].min()),
            None => None,
        }
    }

    // wrapper for state.selected()
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn set_selected_row_color(&mut self, color: Color) {
        self.colors.selected_row_style_fg = color;
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
            .fg(self.colors.selected_row_style_fg)
            .bold();

        let header = ["GRADE", "MIN", "MAX", "PCT"]
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

            let arr = item.ref_array();
            Row::new(vec![
                Cell::from(Text::from(format!("\n{}\n", item.value()))),
                Cell::from(Text::from(format!("\n{}\n", item.min()))),
                Cell::from(Text::from(format!("\n{}\n", item.max()))),
                Cell::from(Text::from(format!(
                    "\n{}%\n",
                    round_dp(arr[1] / self.data[0].max(), 2)
                ))),
            ])
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
                Constraint::Fill(1),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
        // .bg(self.colors.buffer_bg)
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📋 Point Distribution."),
        );
        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}

pub fn render_help(area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    let instructions = vec![
        Line::from(vec!["< F1 >".magenta().bold(), " Show this popup. ".into()]),
        Line::from(vec!["< q > ".magenta().bold()," Quit the app. ".into()]),
        Line::from(""),
        Line::from(vec!["< p >".yellow().bold()," Open points input. ".into()]),
        Line::from(vec!["< . >".yellow().bold()," Toggle half points. ".into()]),
        Line::from(""),
        Line::from(vec!["< I >".blue().bold()," Change to the IHK scale. ".into()]),
        Line::from(vec!["< T >".blue().bold()," Change to the TECHNIKER scale. ".into()]),
        Line::from(vec!["< L >".blue().bold()," Change to the linear scale. ".into()]),
        Line::from(vec!["< C >".blue().bold()," Change to a custom scale. ".into()]),
        Line::from(""),
        Line::from(vec!["< UP >".green().bold()," Select prev row. ".into()]),
        Line::from(vec!["< DOWN >".green().bold()," Select next row. ".into()]),
        Line::from(vec!["< + >".green().bold()," Increase min point for selected row. ".into()]),
        Line::from(vec!["< - >".green().bold()," Decrease min point for selected row. ".into()]),

    ];


    // Render the popup as a Paragraph
    let popup = Paragraph::new(instructions)
    .block(Block::default().title("Available Shortcuts").borders(Borders::ALL))
    .alignment(Alignment::Left);

    popup.render(area, buf);
}

