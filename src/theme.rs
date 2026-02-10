use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ThemeVariant {
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Dark
    }
}

impl ThemeVariant {
    pub fn toggle(&self) -> Self {
        match self {
            ThemeVariant::Dark => ThemeVariant::Light,
            ThemeVariant::Light => ThemeVariant::Dark,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    // Background colors
    pub background: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,

    // Text colors
    pub text_default: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,

    // Accent colors
    pub accent_secondary: Color,
    pub accent_tertiary: Color,

    // Status colors
    pub success: Color,
    pub error: Color,
    pub info: Color,

    // UI element colors
    pub border: Color,
    pub header_title: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub selected_fg: Color,
    pub port_fg: Color,
    pub port_selected_fg: Color,
    pub tcp_color: Color,
    pub udp_color: Color,
    pub command_color: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(15, 15, 25),
            background_secondary: Color::Rgb(20, 20, 35),
            background_tertiary: Color::Rgb(25, 25, 40),

            text_default: Color::Rgb(200, 200, 220),
            text_secondary: Color::Rgb(140, 140, 170),
            text_tertiary: Color::Rgb(100, 100, 120),

            accent_secondary: Color::Rgb(255, 200, 50),
            accent_tertiary: Color::Rgb(80, 80, 100),

            success: Color::Rgb(80, 220, 100),
            error: Color::Rgb(255, 80, 80),
            info: Color::Rgb(100, 180, 255),

            border: Color::Rgb(50, 50, 70),
            header_title: Color::Rgb(255, 100, 50),
            highlight_bg: Color::Rgb(40, 40, 65),
            highlight_fg: Color::Rgb(255, 150, 80),
            selected_fg: Color::Rgb(255, 200, 50),
            port_fg: Color::Rgb(255, 150, 80),
            port_selected_fg: Color::Rgb(255, 200, 50),
            tcp_color: Color::Rgb(100, 200, 255),
            udp_color: Color::Rgb(200, 150, 255),
            command_color: Color::Rgb(120, 120, 150),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Rgb(240, 240, 245),
            background_secondary: Color::Rgb(248, 248, 250),
            background_tertiary: Color::Rgb(235, 235, 240),

            text_default: Color::Rgb(40, 40, 60),
            text_secondary: Color::Rgb(80, 80, 100),
            text_tertiary: Color::Rgb(120, 120, 140),

            accent_secondary: Color::Rgb(200, 100, 0),
            accent_tertiary: Color::Rgb(160, 160, 180),

            success: Color::Rgb(30, 150, 50),
            error: Color::Rgb(200, 30, 30),
            info: Color::Rgb(30, 100, 200),

            border: Color::Rgb(180, 180, 200),
            header_title: Color::Rgb(220, 80, 20),
            highlight_bg: Color::Rgb(220, 230, 245),
            highlight_fg: Color::Rgb(200, 80, 20),
            selected_fg: Color::Rgb(180, 60, 0),
            port_fg: Color::Rgb(200, 80, 20),
            port_selected_fg: Color::Rgb(180, 60, 0),
            tcp_color: Color::Rgb(0, 80, 180),
            udp_color: Color::Rgb(120, 40, 180),
            command_color: Color::Rgb(100, 100, 130),
        }
    }
}
