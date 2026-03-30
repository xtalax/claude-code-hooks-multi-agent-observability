use ratatui::style::{Color, Style};

use crate::config::{Palette, Theme};

/// A sequence of (character, palette) pairs representing syntax-highlighted text
/// ready for injection into a rain column.
pub type ColoredChars = Vec<(char, Palette)>;

/// Convert a brightness value (0.0..=1.0) and palette into a ratatui Style.
pub fn brightness_to_style(b: f32, palette: Palette, theme: &Theme) -> Style {
    let fg = theme.brightness_to_color(b, palette);
    Style::default().fg(fg).bg(Color::Reset)
}
