use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Row, ScrollbarState, StatefulWidget, Table, TableState,
        Widget,
    },
};

use super::theme::THEME;
use crate::action::{Action, ModelAction};
use tracing::debug;

const ITEM_HEIGHT: usize = 4;

#[derive(Debug, Default, Clone)]
pub struct ExamResultTable {
    title: String,
    accent_color: Color,
    state: TableState,
    scroll_state: ScrollbarState,
    data: Vec<ExamResultTableRowData>,
}

impl ExamResultTable {
    pub fn new() -> Self {
        Self {
            title: "Exam Results".into(),
            accent_color: THEME.default_accent_color,
            state: TableState::new(),
            scroll_state: ScrollbarState::default(),
            data: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.set_title(title);
        self
    }

    pub fn with_data(mut self, data: Vec<ExamResultTableRowData>) -> Self {
        self.set_data(data);
        self
    }

    // // return the selected Students name.
    // pub fn selected(&self) -> Option<&str> {
    //     if let Some(index) = self.state.selected() {
    //         return Some(&self.data[index].name);
    //     }
    //     None
    // }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.into();
    }

    pub fn set_accent_color(&mut self, color: Color) {
        self.accent_color = color;
    }

    pub fn set_data(&mut self, data: Vec<ExamResultTableRowData>) {
        self.data = data;
        self.scroll_state = ScrollbarState::new((self.data.len().saturating_sub(1)) * ITEM_HEIGHT);
    }

    fn scroll_to_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            self.scroll_state = self.scroll_state.position(index * ITEM_HEIGHT);
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> Option<Action> {
        debug!("EVENT: {:?}", key);
        match key.code {
            KeyCode::Up => {
                self.state.select_previous();
                self.scroll_to_selected();
                None
            }
            KeyCode::Down => {
                self.state.select_next();
                self.scroll_to_selected();
                None
            }
            KeyCode::Char('+') => {
                if let Some(index) = self.state.selected() {
                    Some(Action::UpdateModel(ModelAction::IncrementStudentPoints(
                        self.data[index].name.clone(),
                    )))
                } else {
                    None
                }
            }
            KeyCode::Char('-') => {
                if let Some(index) = self.state.selected() {
                    Some(Action::UpdateModel(ModelAction::DecrementStudentPoints(
                        self.data[index].name.clone(),
                    )))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Widget for &mut ExamResultTable {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // exam table
        let block = Block::new()
            .title(Line::raw(format!(" 🚸 {} ", self.title)))
            // .title_style(THEME.table_title_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let header = ["Name", "Points", "Percentage", "Grade"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(THEME.table_header_style)
            .height(1);

        let rows = self.data.iter().enumerate().map(|(i, data)| {
            let row_style = match i % 2 {
                0 => THEME.table_row_style.even,
                _ => THEME.table_row_style.odd,
            };

            let item = data.as_str_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(row_style)
                .height(3)
        });

        let selected_row_style = Style::new()
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD)
            .fg(self.accent_color);

        let selected_cell_style = Style::default()
            .reset()
            .add_modifier(Modifier::BOLD)
            .fg(self.accent_color);

        let bar = " █ ";
        let table = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Fill(1),
                Constraint::Min(1),
                Constraint::Min(1),
                Constraint::Min(1),
            ],
        )
        .block(block)
        .header(header)
        .row_highlight_style(selected_row_style)
        .cell_highlight_style(selected_cell_style)
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]));

        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExamResultTableRowData {
    name: String,
    points: f64,
    percentage: f64,
    grade: u8,
}

impl ExamResultTableRowData {
    pub fn new(name: &str, points: f64, percentage: f64, grade: u8) -> Self {
        Self {
            name: name.to_string(),
            points,
            percentage,
            grade,
        }
    }

    fn as_str_array(&self) -> [String; 4] {
        [
            self.name.clone(),
            self.points.to_string(),
            self.percentage.to_string(),
            self.grade.to_string(),
        ]
    }
}
