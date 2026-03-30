use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::config::{Palette, Theme, DEFAULT_AGENT_COLS};
use crate::rain::RainPanel;

/// Renders the digital rain panel: ambient mode when no agents are present,
/// agent mode with 3-row header + rain body when agents exist.
pub struct RainView<'a> {
    pub rain: &'a RainPanel,
    pub theme: &'a Theme,
}

impl<'a> Widget for RainView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Clear the area with theme background
        let bg_style = Style::default().bg(self.theme.bg);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.set_style(bg_style);
                    cell.set_char(' ');
                }
            }
        }

        if self.rain.agent_order.is_empty() {
            self.render_ambient(area, buf);
        } else {
            self.render_agents(area, buf);
        }
    }
}

impl<'a> RainView<'a> {
    fn render_ambient(&self, area: Rect, buf: &mut Buffer) {
        let cols = &self.rain.ambient_rain;
        if cols.is_empty() {
            return;
        }

        let w = (area.width as usize).min(cols.len());
        let h = area.height as usize;

        for row in 0..h {
            for col_x in 0..w {
                let c = &cols[col_x];
                if row < c.height {
                    let b = c.brights[row];
                    let fg_color = self.theme.brightness_to_color(b, Palette::Green);
                    let style = Style::default().fg(fg_color).bg(self.theme.bg);
                    let pos = Position::new(area.left() + col_x as u16, area.top() + row as u16);
                    if let Some(cell) = buf.cell_mut(pos) {
                        cell.set_char(c.chars[row]);
                        cell.set_style(style);
                    }
                }
            }
        }
    }

    fn render_agents(&self, area: Rect, buf: &mut Buffer) {
        let hov_x = self.rain.hovered_col_x;
        let header_rows: u16 = 3;

        if area.height <= header_rows {
            return;
        }

        // ── Header row 1: agent names ──
        let mut col_x: usize = 0;
        for agent_id in &self.rain.agent_order {
            let aw = *self.rain.agent_widths.get(agent_id.as_str()).unwrap_or(&DEFAULT_AGENT_COLS);
            let name = self
                .rain
                .agent_names
                .get(agent_id.as_str())
                .map(|s| s.as_str())
                .unwrap_or(agent_id.as_str());

            let display = truncate_with_ellipsis(name, aw);
            let padded = center_pad(&display, aw);

            for (i, ch) in padded.chars().enumerate() {
                let x = area.left() + (col_x + i) as u16;
                if x >= area.right() {
                    break;
                }
                let is_hov = col_x + i == hov_x as usize;
                let style = if is_hov {
                    Style::default()
                        .fg(self.theme.fg)
                        .bg(self.theme.sel_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(self.theme.fg)
                        .bg(self.theme.black)
                        .add_modifier(Modifier::BOLD)
                };
                if let Some(cell) = buf.cell_mut(Position::new(x, area.top())) {
                    cell.set_char(ch);
                    cell.set_style(style);
                }
            }
            col_x += aw;
        }

        // ── Header row 2: tool status ──
        let row2_y = area.top() + 1;
        col_x = 0;
        for agent_id in &self.rain.agent_order {
            let aw = *self.rain.agent_widths.get(agent_id.as_str()).unwrap_or(&DEFAULT_AGENT_COLS);
            let (status_text, status_style_name) = self
                .rain
                .agent_status
                .get(agent_id.as_str())
                .map(|(t, s)| (t.as_str(), s.as_str()))
                .unwrap_or(("---", ""));

            let display = &status_text[..status_text.len().min(aw)];
            let padded = center_pad(display, aw);
            let fg = status_name_to_color(status_style_name, self.theme);

            for (i, ch) in padded.chars().enumerate() {
                let x = area.left() + (col_x + i) as u16;
                if x >= area.right() {
                    break;
                }
                let style = Style::default().fg(fg).bg(self.theme.bg);
                if let Some(cell) = buf.cell_mut(Position::new(x, row2_y)) {
                    cell.set_char(ch);
                    cell.set_style(style);
                }
            }
            col_x += aw;
        }

        // ── Header row 3: haiku status ──
        let row3_y = area.top() + 2;
        col_x = 0;
        for agent_id in &self.rain.agent_order {
            let aw = *self.rain.agent_widths.get(agent_id.as_str()).unwrap_or(&DEFAULT_AGENT_COLS);
            let (haiku_text, haiku_style_name) = self
                .rain
                .agent_haiku_status
                .get(agent_id.as_str())
                .map(|(t, s)| (t.as_str(), s.as_str()))
                .unwrap_or(("", ""));

            let display = truncate_with_ellipsis(haiku_text, aw);
            let padded = center_pad(&display, aw);
            let fg = status_name_to_color(haiku_style_name, self.theme);

            for (i, ch) in padded.chars().enumerate() {
                let x = area.left() + (col_x + i) as u16;
                if x >= area.right() {
                    break;
                }
                let style = Style::default().fg(fg).bg(self.theme.bg);
                if let Some(cell) = buf.cell_mut(Position::new(x, row3_y)) {
                    cell.set_char(ch);
                    cell.set_style(style);
                }
            }
            col_x += aw;
        }

        // ── Body: rain columns ──
        // Pre-build flat column reference list for the visible grid
        let mut flat_cols: Vec<Option<usize>> = Vec::new();
        let mut flat_agent_ids: Vec<&str> = Vec::new();
        let mut flat_col_indices: Vec<usize> = Vec::new();

        for agent_id in &self.rain.agent_order {
            let aw = *self.rain.agent_widths.get(agent_id.as_str()).unwrap_or(&DEFAULT_AGENT_COLS);
            let agent_cols = self.rain.agent_rain.get(agent_id.as_str());
            let n = agent_cols.map(|c| c.len()).unwrap_or(0);
            for i in 0..aw {
                if i < n {
                    flat_cols.push(Some(flat_agent_ids.len()));
                    flat_agent_ids.push(agent_id.as_str());
                    flat_col_indices.push(i);
                } else {
                    flat_cols.push(None);
                    flat_agent_ids.push("");
                    flat_col_indices.push(0);
                }
            }
        }

        let rain_h = (area.height - header_rows) as usize;
        let total_cols = flat_cols.len().min(area.width as usize);

        for row in 0..rain_h {
            let y = area.top() + header_rows + row as u16;
            if y >= area.bottom() {
                break;
            }

            for cx in 0..total_cols {
                let x = area.left() + cx as u16;
                if x >= area.right() {
                    break;
                }

                if flat_cols[cx].is_none() {
                    continue; // already cleared to bg
                }

                let agent_id = flat_agent_ids[cx];
                let col_idx = flat_col_indices[cx];

                let agent_cols = match self.rain.agent_rain.get(agent_id) {
                    Some(c) => c,
                    None => continue,
                };
                let c = &agent_cols[col_idx];

                if row >= c.height {
                    continue;
                }

                let mut b = c.brights[row];
                let is_hov = cx as i32 == hov_x;
                if is_hov {
                    b = (b + 0.6).min(1.0);
                }

                let palette = c.colors[row];
                let fg_color = self.theme.brightness_to_color(b, palette);
                let bold = (palette != Palette::None && b > 0.5) || is_hov;

                let mut style = Style::default().fg(fg_color);
                if is_hov {
                    style = style.bg(self.theme.sel_bg);
                } else {
                    style = style.bg(self.theme.bg);
                }
                if bold {
                    style = style.add_modifier(Modifier::BOLD);
                }

                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.set_char(c.chars[row]);
                    cell.set_style(style);
                }
            }
        }
    }
}

/// Truncate a string to fit `max_width`, adding "..." if too long.
fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        return s.to_string();
    }
    if max_width <= 3 {
        return s.chars().take(max_width).collect();
    }
    let mut result: String = s.chars().take(max_width - 3).collect();
    result.push_str("...");
    result
}

/// Center-pad a string to exactly `width` characters.
fn center_pad(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        return s.chars().take(width).collect();
    }
    let left = (width - len) / 2;
    let right = width - len - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

/// Map a style-name string (from agent_status / haiku_status) to a theme color.
/// These names come from the Python side (e.g. "bold green", "bold #7fb4ca", etc.).
/// We do best-effort matching against known color keywords.
fn status_name_to_color(style_name: &str, theme: &Theme) -> ratatui::style::Color {
    let lower = style_name.to_lowercase();
    if lower.contains("green") {
        theme.br_green
    } else if lower.contains("cyan") || lower.contains("blue") {
        theme.br_blue
    } else if lower.contains("yellow") {
        theme.br_yellow
    } else if lower.contains("red") {
        theme.br_red
    } else if lower.contains("magenta") {
        theme.magenta
    } else if lower.contains("orange") {
        theme.orange
    } else if lower.is_empty() {
        theme.br_black
    } else {
        theme.fg
    }
}
