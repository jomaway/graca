use ratatui::style::{Color, Modifier, Style, Stylize};

use crate::model::scale::GradeScaleType;

pub trait AppStyle {
    fn scale_color(&self, scale_type: &GradeScaleType) -> Color;
    fn accent_color(&self) -> Color;
    fn text_color(&self, dark: bool) -> Color;
    fn background_color(&self, dark: bool) -> Color;
    fn text(&self) -> Style {
        Style::default().fg(self.text_color(false))
    }
    fn block(&self) -> Style {
        Style::default()
    }
    fn block_title(&self) -> Style {
        Style::default()
    }
    fn background(&self) -> Style {
        Style::default()
    }
    fn table_header(&self) -> Style;
    fn table_row(&self, index: usize) -> Style;
    fn table_row_selected(&self) -> Style;
    fn table_col_selected(&self) -> Style;
    fn tab(&self, selected: bool) -> Style;
    fn tag(&self, colored: bool) -> Style;
    fn indicator(&self, scale_type: Option<&GradeScaleType>) -> Style {
        if let Some(scale) = scale_type {
            Style::default()
                .fg(self.background_color(true))
                .bg(self.scale_color(scale))
        } else {
            Style::default()
                .fg(self.background_color(true))
                .bg(Color::Magenta)
        }
    }
    fn command_indicator_palette(&self) -> Style {
        Style::default()
            .bg(self.background_color(true))
            .fg(self.text_color(false))
    }
    fn top_bar(&self) -> Style;
    fn bottom_bar(&self) -> Style;
    fn bar_chart(&self) -> Style;
}

pub const DARK_WHITE: Color = Color::Rgb(213, 196, 161);
pub const LIGHT_GRAY: Color = Color::Rgb(80, 73, 69);
pub const GRAY: Color = Color::Rgb(60, 56, 54);
pub const BLACK: Color = Color::Rgb(8, 8, 8); // not really black, often #080808

#[derive(Debug, Default)]
pub struct Theme;

impl AppStyle for Theme {
    fn scale_color(&self, scale_type: &GradeScaleType) -> Color {
        match scale_type {
            GradeScaleType::IHK => Color::Yellow,
            GradeScaleType::TECHNIKER => Color::Blue,
            GradeScaleType::LINEAR => Color::Green,
            GradeScaleType::Custom(_) => Color::LightRed,
        }
    }

    fn accent_color(&self) -> Color {
        Color::Cyan
    }

    fn text_color(&self, dark: bool) -> Color {
        match dark {
            true => LIGHT_GRAY,
            false => DARK_WHITE,
        }
    }

    fn background_color(&self, dark: bool) -> Color {
        match dark {
            true => BLACK,
            false => GRAY,
        }
    }

    fn table_header(&self) -> Style {
        Style::default()
            .fg(DARK_WHITE)
            .add_modifier(Modifier::ITALIC)
    }

    fn table_row(&self, index: usize) -> Style {
        match index % 2 {
            0 => Style::default().fg(DARK_WHITE).bg(GRAY),
            _ => Style::default().fg(DARK_WHITE).bg(LIGHT_GRAY),
        }
    }

    fn table_row_selected(&self) -> Style {
        Style::default()
            .fg(self.accent_color())
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD)
    }

    fn table_col_selected(&self) -> Style {
        Style::default()
            .reset()
            .add_modifier(Modifier::BOLD)
            .fg(self.accent_color())
    }

    fn tab(&self, selected: bool) -> Style {
        match selected {
            true => self.tag(true).reversed().bold(),
            false => self.text(),
        }
    }

    fn tag(&self, colored: bool) -> Style {
        match colored {
            true => Style::default().bg(self.accent_color()).fg(LIGHT_GRAY),
            false => Style::default().fg(DARK_WHITE).bg(LIGHT_GRAY),
        }
    }

    fn top_bar(&self) -> Style {
        Style::default().bg(GRAY).fg(Color::Magenta)
    }

    fn bottom_bar(&self) -> Style {
        self.command_indicator_palette()
    }

    fn bar_chart(&self) -> Style {
        Style::default().fg(Color::Cyan)
    }
}

pub const THEME: Theme = Theme {};
