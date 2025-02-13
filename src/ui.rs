use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Cell, HighlightSpacing, List, ListItem, ListState, Paragraph, Row, Table, Widget};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{StatefulWidget, TableState},
};

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
    colors: Theme,
}

impl GradeTable {
    pub fn new() -> Self {
        Self {
            state: TableState::default().with_selected(0),
            colors: Theme::default(),
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

    pub fn render(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        data: &Vec<Grade>,
    ) {
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

        let rows = data.iter().enumerate().map(|(i, item)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };

            Row::new(vec![
                Cell::from(Text::from(format!("\n{}\n", item.value()))),
                Cell::from(Text::from(format!("\n{}\n", item.min()))),
                Cell::from(Text::from(format!("\n{}\n", item.max()))),
                Cell::from(Text::from(format!("\n{}%\n", item.pct(data[0].max())))),
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
        Line::from(vec!["< q > ".magenta().bold(), " Quit the app. ".into()]),
        Line::from(""),
        Line::from(vec!["< p >".yellow().bold(), " Open points input. ".into()]),
        Line::from(vec![
            "< . >".yellow().bold(),
            " Toggle half points. ".into(),
        ]),
        Line::from(""),
        Line::from(vec![
            "< I >".blue().bold(),
            " Change to the IHK scale. ".into(),
        ]),
        Line::from(vec![
            "< T >".blue().bold(),
            " Change to the TECHNIKER scale. ".into(),
        ]),
        Line::from(vec![
            "< L >".blue().bold(),
            " Change to the linear scale. ".into(),
        ]),
        Line::from(vec![
            "< C >".blue().bold(),
            " Change to a custom scale. ".into(),
        ]),
        Line::from(""),
        Line::from(vec!["< UP >".green().bold(), " Select prev row. ".into()]),
        Line::from(vec!["< DOWN >".green().bold(), " Select next row. ".into()]),
        Line::from(vec![
            "< + >".green().bold(),
            " Increase min point for selected row. ".into(),
        ]),
        Line::from(vec![
            "< - >".green().bold(),
            " Decrease min point for selected row. ".into(),
        ]),
    ];

    // Render the popup as a Paragraph
    let popup = Paragraph::new(instructions)
        .block(
            Block::default()
                .title("Available Shortcuts")
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Left);

    popup.render(area, buf);
}

pub struct NumberInputField {
    input: String,
    character_index: usize,
}

impl NumberInputField {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 0,
        }
    }

    pub fn get_input(&self) -> &str {
        self.input.as_str()
    }

    pub fn get_index(&self) -> usize {
        self.character_index
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        // add guard to max insert 9 digits, to not overflow u32.
        if self.input.len() < 10 {
            let index = self.byte_index();
            self.input.insert(index, new_char);
            self.move_cursor_right();
        }
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    /// return the input as number
    /// todo: split returning the value and converting to a number into seperate things.
    pub fn get_number(&mut self) -> u32 {
        let number: u32 = self.input.parse().expect("Not a valid number");
        self.input.clear();
        self.reset_cursor();
        number
    }
}


pub struct ExportModal
{
    filename: String,
    pub list_state: ListState,
}

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).fg(Color::White).add_modifier(Modifier::BOLD);

impl ExportModal
{
    pub fn new() -> Self {
        let mut modal = Self {
            filename: String::new(),
            list_state: ListState::default(),
        };
        modal.list_state.select(Some(0));
        modal
    }

    pub fn render(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let block = Block::bordered().title("Export").on_magenta().fg(Color::Black);

        let inner = block.inner(area);
        block.render(area, buf);
        self.render_list(inner, buf);
    }

    fn render_list(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let items: Vec<ListItem> = vec![ListItem::new("CSV"), ListItem::new("Excel")];
        
        let list = List::new(items)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.list_state); 
    }
}


/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}