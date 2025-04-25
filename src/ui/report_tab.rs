use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders, Padding, Paragraph, Widget},
};

use super::theme::{LIGHT_GRAY, THEME};

#[derive(Debug, Default, Clone)]
pub struct ExamChart {
    data: [u8; 6],
    accent_color: Color,
    avg: f64,
}

impl ExamChart {
    pub fn new() -> Self {
        let mut s = ExamChart::default();
        s.accent_color = THEME.default_accent_color;
        s
    }

    pub fn set_data(&mut self, values: &[u8; 6], avg: f64) {
        self.data = values.to_owned();
        self.avg = avg;
    }

    pub fn set_accent_color(&mut self, color: Color) {
        self.accent_color = color;
    }
}

impl Widget for &ExamChart {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::new()
            .title(Line::raw(" 📊 Grade Distribution "))
            .title_bottom(
                Line::from(format!(" AVG: {} ", self.avg))
                    .right_aligned()
                    .style(Style::default().bg(LIGHT_GRAY)),
            )
            .title_style(THEME.block_title_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .padding(Padding {
                left: 2,
                right: 3,
                top: 1,
                bottom: 1,
            });

        let bars: Vec<Bar> = self
            .data
            .iter()
            .enumerate()
            .map(|(g, &c)| {
                Bar::default()
                    .value(c as u64)
                    .label(Line::from((g + 1).to_string()))
                    .style(THEME.bar_chart_style.bar.fg(self.accent_color))
                    .value_style(THEME.bar_chart_style.bar_value.bg(self.accent_color))
            })
            .collect();

        let clamped_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: std::cmp::min(30, area.height), // clamp height to 21 if area is bigger.
        };

        BarChart::default()
            .block(block)
            .data(BarGroup::default().bars(&bars))
            .bar_width((area.width - 9) / 6)
            .style(THEME.bar_chart_style.chart)
            .label_style(THEME.bar_chart_style.label)
            .direction(Direction::Vertical)
            .render(clamped_area, buf);
    }
}
