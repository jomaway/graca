use ratatui::{
    layout::{Alignment, Constraint},
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, TableState, Widget},
};

use crate::grade::Grade;

use super::theme::THEME;

pub struct GradeTable {
    pub state: TableState,
    accent_color: Color,
    data: Vec<Grade>,
    editable: bool,
}

impl GradeTable {
    pub fn new() -> Self {
        Self {
            state: TableState::default().with_selected(0),
            accent_color: THEME.default_accent_color,
            data: vec![],
            editable: false,
        }
    }

    // wrapper for state.selected()
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn set_accent_color(&mut self, color: Color) {
        self.accent_color = color;
    }

    pub fn update_data(&mut self, data: Vec<Grade>) {
        self.data = data;
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

    pub fn select_col(&mut self, col: Option<usize>) {
        self.state.select_column(col);
    }

    pub fn select_col_min(&mut self) {
        self.state.select_column(Some(1));
    }

    pub fn select_col_max(&mut self) {
        self.state.select_column(Some(2));
    }

    pub fn handle_event(&mut self, event: GradeTableEvent) {
        if self.editable {
            match event {
                GradeTableEvent::IncreasePoints => todo!(),
                GradeTableEvent::DecreasePoints => todo!(),
            }
        }
    }
}

impl Widget for &mut GradeTable {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let selected_row_style = Style::new()
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD)
            .fg(self.accent_color);

        let selected_cell_style = Style::default()
            .reset()
            .add_modifier(Modifier::BOLD)
            .fg(self.accent_color);

        let header = [
            Text::from("GRADE"),
            Text::from("MIN").alignment(Alignment::Center),
            Text::from("MAX").alignment(Alignment::Center),
            Text::from("PCT").alignment(Alignment::Center),
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(THEME.table_header_style)
        .height(1);

        let rows = self.data.iter().enumerate().map(|(i, item)| {
            let row_style = match i % 2 {
                0 => THEME.table_row_style.even,
                _ => THEME.table_row_style.odd,
            };

            Row::new(vec![
                Cell::from(Text::from(format!("\n{}\n", item.value()))),
                Cell::from(Text::from(format!("\n{}\n", item.min())).alignment(Alignment::Center)),
                Cell::from(Text::from(format!("\n{}\n", item.max())).alignment(Alignment::Center)),
                Cell::from(
                    Text::from(format!("\n{}%\n", item.pct(self.data[0].max())))
                        .alignment(Alignment::Center),
                ),
            ])
            .style(row_style)
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
        .cell_highlight_style(selected_cell_style)
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

// pub struct GradeTableState {
//     data: HashMap<usize, String>,
// }

// impl StatefulWidget for GradeTable {
//     type State = GradeTableState;

//     fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         todo!()
//     }
// }

pub enum GradeTableEvent {
    IncreasePoints,
    DecreasePoints,
}
