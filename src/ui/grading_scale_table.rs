use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint},
    prelude::{Buffer, Rect},
    style::Color,
    text::Text,
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, TableState, Widget},
};

use super::theme::{AppStyle, THEME};
use crate::{
    action::{Action, ModelAction},
    model::scale::GradeScaleType,
};
use tracing::debug;

pub struct GradingScaleTable {
    pub state: TableState,
    accent_color: Color,
    scale_type: GradeScaleType,
    data: Vec<GradingScaleTableRowData>,
}

impl GradingScaleTable {
    pub fn new(scale_type: GradeScaleType) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            accent_color: Color::Cyan,
            scale_type: scale_type,
            data: vec![],
        }
    }

    // return the selected grade as u8 if a row is selected
    pub fn selected(&self) -> Option<u8> {
        if let Some(index) = self.state.selected() {
            return Some(self.data[index].grade);
        }
        None
    }

    pub fn set_accent_color(&mut self, color: Color) {
        self.accent_color = color;
    }

    pub fn update(&mut self, data: Vec<GradingScaleTableRowData>) {
        self.data = data;
    }

    pub fn select_col_min(&mut self) {
        self.state.select_column(Some(1));
    }

    pub fn select_col_max(&mut self) {
        self.state.select_column(Some(2));
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> Option<Action> {
        debug!("EVENT: {:?}", key);
        match key.code {
            KeyCode::Up | KeyCode::Char('j') => {
                self.state.select_previous();
                None
            }
            KeyCode::Down | KeyCode::Char('k') => {
                self.state.select_next();
                None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.select_col_min();
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.select_col_max();
                None
            }
            KeyCode::Esc => {
                self.state.select_column(None);
                None
            }
            KeyCode::Char('+') => {
                if let Some(index) = self.state.selected() {
                    Some(Action::UpdateModel(ModelAction::IncrementThreshold(
                        self.data[index].grade,
                    )))
                } else {
                    None
                }
            }
            KeyCode::Char('-') => {
                if let Some(index) = self.state.selected() {
                    Some(Action::UpdateModel(ModelAction::DecrementThreshold(
                        self.data[index].grade,
                    )))
                } else {
                    None
                }
            }
            KeyCode::PageUp => Some(Action::UpdateModel(ModelAction::IncrementMaxPoints)),
            KeyCode::PageDown => Some(Action::UpdateModel(ModelAction::DecrementMaxPoints)),
            _ => None,
        }
    }
}

impl Widget for &mut GradingScaleTable {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let header = [
            Text::from("GRADE"),
            Text::from("MIN").alignment(Alignment::Center),
            Text::from("MAX").alignment(Alignment::Center),
            Text::from("PCT").alignment(Alignment::Center),
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(THEME.table_header())
        .height(1);

        let rows = self.data.iter().enumerate().map(|(i, data)| {
            let item = data.as_str_array();
            item.into_iter()
                .enumerate()
                .map(|(idx, content)| {
                    let text = if idx == 0 {
                        Text::from(format!("\n{content}\n"))
                    } else {
                        Text::from(format!("\n{content}\n")).alignment(Alignment::Center)
                    };
                    Cell::from(text)
                })
                .collect::<Row>()
                .style(THEME.table_row(i))
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
        .row_highlight_style(THEME.table_row_selected().fg(self.accent_color))
        .cell_highlight_style(THEME.table_col_selected().fg(self.accent_color))
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 💯 Grading Scale ")
                .style(THEME.block())
                .title_style(THEME.block_title()),
        );

        let clamped_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: std::cmp::min(21, area.height), // clamp height to 21 if area is bigger.
        };
        StatefulWidget::render(table, clamped_area, buf, &mut self.state);
    }
}

#[derive(Debug, Default, Clone)]
pub struct GradingScaleTableRowData {
    grade: u8,
    min: f64,
    max: f64,
    pct: f64,
}

impl GradingScaleTableRowData {
    pub fn new(grade: u8, min: f64, max: f64, pct: f64) -> Self {
        Self {
            grade,
            min,
            max,
            pct,
        }
    }

    pub fn as_str_array(&self) -> [String; 4] {
        [
            self.grade.to_string(),
            self.min.to_string(),
            self.max.to_string(),
            format!("{}%", (self.pct * 100.0).round()),
        ]
    }
}
