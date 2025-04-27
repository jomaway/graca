use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Row, ScrollbarState, StatefulWidget, Table, TableState,
        Widget,
    },
};

use super::theme::{AppStyle, THEME};
use crate::action::{Action, ModelAction};
use tracing::debug;

const ITEM_HEIGHT: usize = 4;

#[derive(Debug, Default, Clone)]
pub struct ExamResultTable {
    title: String,
    state: TableState,
    scroll_state: ScrollbarState,
    data: Vec<ExamResultTableRowData>,
}

impl ExamResultTable {
    pub fn new() -> Self {
        Self {
            title: "Exam Results".into(),
            state: TableState::default()
                .with_selected(0)
                .with_selected_column(1),
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

    pub fn set_title(&mut self, title: &str) {
        self.title = title.into();
    }

    pub fn set_data(&mut self, data: Vec<ExamResultTableRowData>) {
        self.data = data;
        self.scroll_state = ScrollbarState::new((self.data.len().saturating_sub(1)) * ITEM_HEIGHT);
    }

    fn scroll_to_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            tracing::debug!("IDX: {index}");
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
            .title_style(THEME.block_title())
            .style(THEME.block())
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let header = [
            Text::from("Name"),
            Text::from("Points").alignment(Alignment::Center),
            Text::from("Percentage").alignment(Alignment::Center),
            Text::from("Grade").alignment(Alignment::Center),
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(THEME.table_header())
        .height(1);

        let rows = self.data.iter().enumerate().map(|(index, data)| {
            let item = data.as_str_array();
            item.into_iter()
                .enumerate()
                .map(|(idx, content)| {
                    let text = format!("\n{content}\n");
                    let mut align = Alignment::Left;

                    if idx != 0 {
                        align = Alignment::Center
                    }

                    let grade_style = match data.grade {
                        5 | 6 => Style::new().bg(Color::Red).add_modifier(Modifier::BOLD),
                        3 | 4 => Style::new().bg(Color::Yellow).add_modifier(Modifier::BOLD),
                        1 | 2 => Style::new().bg(Color::Green).add_modifier(Modifier::BOLD),
                        _ => Style::new().add_modifier(Modifier::BOLD),
                    };

                    let mut text = Text::from(text).alignment(align);

                    if idx == 3 {
                        text = text.patch_style(grade_style);
                    }

                    Cell::from(text)
                })
                .collect::<Row>()
                .style(THEME.table_row(index))
                .height(3)
        });

        let bar = " █ ";
        let table = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Min(2),
                Constraint::Min(1),
                Constraint::Min(1),
                Constraint::Min(1),
            ],
        )
        .block(block)
        .header(header)
        // .row_highlight_style(THEME.table_row_selected())
        .cell_highlight_style(THEME.table_row_selected())
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
