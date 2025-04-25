use ratatui::{
    buffer::Buffer,
    layout::{Direction, Rect},
    style::Color,
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders, Widget},
};

use super::theme::THEME;

#[derive(Debug, Default, Clone)]
pub struct ExamChart {
    data: [u8; 6],
    accent_color: Color,
}

impl ExamChart {
    pub fn new() -> Self {
        let mut s = ExamChart::default();
        s.accent_color = THEME.default_accent_color;
        s
    }

    pub fn set_data(&mut self, values: &[u8; 6]) {
        self.data = values.to_owned();
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
            .title_style(THEME.block_title_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

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

        BarChart::default()
            .block(block)
            .data(BarGroup::default().bars(&bars))
            .bar_width((area.width - 4) / 6)
            .style(THEME.bar_chart_style.chart)
            .label_style(THEME.bar_chart_style.label)
            .direction(Direction::Vertical)
            .render(area, buf);
    }
}
