use ratatui::{
    buffer::Buffer,
    layout::{Direction, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders, Padding, Widget},
};

use super::theme::{AppStyle, THEME};

#[derive(Debug, Default, Clone)]
pub struct ExamChart {
    data: [u8; 6],
    accent_color: Color,
    avg: f64,
}

impl ExamChart {
    pub fn new() -> Self {
        ExamChart::default()
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
                Line::from(vec![
                    Span::from(" AVG ").style(THEME.tag(true)),
                    Span::from(format!(" {} ", self.avg)).style(THEME.tag(true).reversed().bold()),
                ])
                .right_aligned(),
            )
            .title_style(THEME.block_title())
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
                    .style(THEME.bar_chart())
                    .value_style(THEME.bar_chart().reversed())
            })
            .collect();

        let clamped_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: std::cmp::min(30, area.height), // clamp height to 21 if area is bigger.
        };

        tracing::info!("AREA WIDTH {}", area.width);
        BarChart::default()
            .block(block.padding(Padding {
                left: 4,
                right: 4,
                top: 1,
                bottom: 1,
            }))
            .data(BarGroup::default().bars(&bars))
            .bar_width((area.width - 25) / 6)
            .bar_gap(3)
            .style(THEME.block())
            .label_style(THEME.text().italic())
            .direction(Direction::Vertical)
            .render(clamped_area, buf);
    }
}
