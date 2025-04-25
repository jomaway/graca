use ratatui::style::{Color, Modifier, Style};

// pub struct ColorSchema {
//     accent_color: Color,
//     tab: Style,
//     tab_selected: Style,
//     block_title_focused: Style,
//     block_focused: Style,
//     table_header: Style,
//     table_row_even: Style,
//     table_row_odd: Style,
//     bar_chart: Style,
// }

pub struct Theme {
    pub default_accent_color: Color,
    pub key_binding: KeyBinding,
    pub block_title_style: Style,
    pub table_header_style: Style,
    pub table_row_style: ZebraStyle,
    pub bar_style: Style,
    pub bar_chart_style: BarChartStyle,
}

pub struct KeyBinding {
    pub key: Style,
    pub description: Style,
}

pub struct ZebraStyle {
    pub even: Style,
    pub odd: Style,
    pub selected: Style,
}

pub struct BarChartStyle {
    pub bar: Style,
    pub bar_value: Style,
    pub chart: Style,
    pub label: Style,
}

pub const THEME: Theme = Theme {
    default_accent_color: Color::Yellow,
    key_binding: KeyBinding {
        key: Style::new().fg(BLACK).bg(LIGHT_GRAY),
        description: Style::new().fg(Color::Magenta).bg(BLACK),
    },
    block_title_style: Style::new().add_modifier(Modifier::BOLD),
    table_header_style: Style::new().add_modifier(Modifier::ITALIC).fg(DARK_WHITE),
    table_row_style: ZebraStyle {
        even: Style::new().fg(DARK_WHITE).bg(GRAY),
        odd: Style::new().fg(DARK_WHITE).bg(LIGHT_GRAY),
        selected: Style::new()
            .fg(Color::Yellow)
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD),
    },
    bar_style: Style::new().bg(GRAY).fg(Color::Magenta),
    bar_chart_style: BarChartStyle {
        bar: Style::new().fg(Color::LightYellow),
        bar_value: Style::new().fg(DARK_GRAY).bg(Color::LightYellow),
        chart: Style::new(),
        label: Style::new().fg(DARK_WHITE),
    },
};

pub const DARK_WHITE: Color = Color::Rgb(213, 196, 161);
pub const LIGHT_GRAY: Color = Color::Rgb(80, 73, 69);
pub const GRAY: Color = Color::Rgb(60, 56, 54);
// const MID_GRAY: Color = Color::Rgb(128, 128, 128);
pub const DARK_GRAY: Color = Color::Rgb(68, 68, 68);
pub const BLACK: Color = Color::Rgb(8, 8, 8); // not really black, often #080808
