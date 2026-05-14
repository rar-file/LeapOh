use serde::Deserialize;
use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Vec<u8>> for Color {
    fn from(v: Vec<u8>) -> Self {
        Self {
            r: v.first().copied().unwrap_or(255),
            g: v.get(1).copied().unwrap_or(255),
            b: v.get(2).copied().unwrap_or(255),
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        Ok(Color::from(v))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Always,
    Never,
    Auto,
}

impl ColorMode {
    pub fn resolve(self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => {
                if std::env::var_os("NO_COLOR").is_some() {
                    return false;
                }
                std::io::stdout().is_terminal()
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub enabled: bool,
}

impl Theme {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn fg(&self, c: Color, s: &str) -> String {
        if !self.enabled {
            return s.to_string();
        }
        format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, s)
    }

        pub fn dim(&self, s: &str) -> String {
        if !self.enabled {
            return s.to_string();
        }
        format!("\x1b[2m{}\x1b[0m", s)
    }

    pub fn bold_fg(&self, c: Color, s: &str) -> String {
        if !self.enabled {
            return s.to_string();
        }
        format!("\x1b[1;38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, s)
    }

    pub fn italic_fg(&self, c: Color, s: &str) -> String {
        if !self.enabled {
            return s.to_string();
        }
        format!("\x1b[3;38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, s)
    }
}

// ── Palette ─────────────────────────────────────────────────────────

pub const HEADER:       Color = Color::rgb(245, 245, 240);
pub const HEADER_DIM:   Color = Color::rgb(160, 160, 155);
pub const VOICE:        Color = Color::rgb(220, 200, 170);
pub const SPEAKER:      Color = Color::rgb(255, 170, 90);
// Tag colors — soft, distinct, calm
pub const TAG_GIT:      Color = Color::rgb(120, 200, 130);
pub const TAG_GH:       Color = Color::rgb(190, 140, 240);
pub const TAG_WX:       Color = Color::rgb(120, 200, 230);
pub const TAG_BAT:      Color = Color::rgb(240, 130, 130);
pub const TAG_DSK:      Color = Color::rgb(230, 200, 110);
pub const TAG_CAL:      Color = Color::rgb(200, 170, 240);
pub const TAG_STREAK:   Color = Color::rgb(250, 210, 120);
pub const TAG_DEFAULT:  Color = Color::rgb(170, 170, 170);

pub fn tag_color(tag: &str) -> Color {
    match tag {
        "git" => TAG_GIT,
        "gh" => TAG_GH,
        "wx" => TAG_WX,
        "bat" => TAG_BAT,
        "dsk" => TAG_DSK,
        "cal" => TAG_CAL,
        "◐" => TAG_STREAK,
        _ => TAG_DEFAULT,
    }
}
