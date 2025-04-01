use ratatui::style::{Color, Style};

pub struct Theme {
    pub default_accent_color: Color,
    pub key_binding: KeyBinding,
    pub table_header_style: Style,
    pub table_row_style: ZebraStyle,
    pub bar_style: Style,
}

pub struct KeyBinding {
    pub key: Style,
    pub description: Style,
}

pub struct ZebraStyle {
    pub even: Style,
    pub odd: Style,
}

pub const THEME: Theme = Theme {
    default_accent_color: Color::Yellow,
    key_binding: KeyBinding {
        key: Style::new().fg(BLACK).bg(LIGHT_GRAY),
        description: Style::new().fg(Color::Magenta).bg(BLACK),
    },
    table_header_style: Style::new(),
    table_row_style: ZebraStyle {
        even: Style::new().fg(DARK_WHITE).bg(GRAY),
        odd: Style::new().fg(DARK_WHITE).bg(LIGHT_GRAY),
    },
    bar_style: Style::new().bg(GRAY).fg(Color::Magenta),
};

const DARK_WHITE: Color = Color::Rgb(213, 196, 161);
const LIGHT_GRAY: Color = Color::Rgb(80, 73, 69);
const GRAY: Color = Color::Rgb(60, 56, 54);
// const MID_GRAY: Color = Color::Rgb(128, 128, 128);
// const DARK_GRAY: Color = Color::Rgb(68, 68, 68);
const BLACK: Color = Color::Rgb(8, 8, 8); // not really black, often #080808
