use ratatui::style::Color;
use std::collections::HashMap;
use std::path::PathBuf;

// Rain character sets
pub const KATAKANA: &str = "ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍ";
pub const LATIN: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub const DEFAULT_AGENT_COLS: usize = 16;
pub const MIN_AGENT_COLS: usize = 4;
pub const MAX_LOG_LINES: usize = 500;

pub const SERVER_PORT: &str = "4000";

// Event type glyphs
pub fn glyph(event_type: &str) -> &'static str {
    match event_type {
        "PreToolUse" => "[>",
        "PostToolUse" => "[+",
        "PostToolUseFailure" => "[!",
        "UserPromptSubmit" => "[?",
        "Stop" => "[X",
        "SubagentStart" => "[^",
        "SubagentStop" => "[v",
        "Notification" => "[*",
        "SessionStart" => "[=",
        "SessionEnd" => "[~",
        "PermissionRequest" => "[#",
        "PreCompact" => "[%",
        _ => "[·",
    }
}

// Tool → palette mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Palette {
    Green,
    Cyan,
    Yellow,
    Magenta,
    Orange,
    White,
    None,
}

pub fn tool_palette(tool_name: &str) -> Palette {
    match tool_name {
        "Bash" | "WebFetch" | "WebSearch" => Palette::Cyan,
        "Read" | "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => Palette::Yellow,
        "Grep" | "Glob" => Palette::Magenta,
        "Task" => Palette::Orange,
        _ => Palette::None,
    }
}

// ── Theme ──

fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    let (r1, g1, b1) = color_to_rgb(c1);
    let (r2, g2, b2) = color_to_rgb(c2);
    Color::Rgb(
        (r1 as f32 + (r2 as f32 - r1 as f32) * t) as u8,
        (g1 as f32 + (g2 as f32 - g1 as f32) * t) as u8,
        (b1 as f32 + (b2 as f32 - b1 as f32) * t) as u8,
    )
}

fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return Color::Rgb(128, 128, 128);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
    Color::Rgb(r, g, b)
}

pub fn make_shades(bright: Color, base: Color, bg: Color, fg: Color) -> [Color; 7] {
    [
        fg,
        bright,
        base,
        lerp_color(base, bg, 0.33),
        lerp_color(base, bg, 0.55),
        lerp_color(base, bg, 0.77),
        bg,
    ]
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub cursor: Color,
    pub sel_bg: Color,
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub br_black: Color,
    pub br_red: Color,
    pub br_green: Color,
    pub br_yellow: Color,
    pub br_blue: Color,
    pub br_magenta: Color,
    pub br_cyan: Color,
    pub br_white: Color,
    pub orange: Color,
    pub hover_bg: Color,

    // Pre-computed 7-step palettes
    pub green_shades: [Color; 7],
    pub cyan_shades: [Color; 7],
    pub yellow_shades: [Color; 7],
    pub magenta_shades: [Color; 7],
    pub orange_shades: [Color; 7],
    pub white_shades: [Color; 7],
}

impl Theme {
    pub fn load() -> Self {
        let colors = load_omarchy_theme();
        let get = |key: &str, default: &str| -> Color {
            colors
                .get(key)
                .map(|v| hex_to_color(v))
                .unwrap_or_else(|| hex_to_color(default))
        };

        let bg = get("background", "#1f1f28");
        let fg = get("foreground", "#dcd7ba");
        let accent = get("accent", "#7e9cd8");
        let cursor = get("cursor", "#c8c093");
        let sel_bg = get("selection_background", "#2d4f67");
        let black = get("color0", "#090618");
        let red = get("color1", "#c34043");
        let green = get("color2", "#76946a");
        let yellow = get("color3", "#c0a36e");
        let blue = get("color4", "#7e9cd8");
        let magenta = get("color5", "#957fb8");
        let cyan = get("color6", "#6a9589");
        let white = get("color7", "#c8c093");
        let br_black = get("color8", "#727169");
        let br_red = get("color9", "#e82424");
        let br_green = get("color10", "#98bb6c");
        let br_yellow = get("color11", "#e6c384");
        let br_blue = get("color12", "#7fb4ca");
        let br_magenta = get("color13", "#938aa9");
        let br_cyan = get("color14", "#7aa89f");
        let br_white = get("color15", "#dcd7ba");

        let orange = lerp_color(br_yellow, red, 0.25);
        let hover_bg = lerp_color(bg, sel_bg, 0.35);

        let green_shades = make_shades(br_green, green, bg, fg);
        let cyan_shades = make_shades(br_blue, blue, bg, fg);
        let yellow_shades = make_shades(br_yellow, yellow, bg, fg);
        let magenta_shades = make_shades(magenta, br_magenta, bg, fg);
        let orange_shades = make_shades(orange, yellow, bg, fg);
        let white_shades = make_shades(cursor, lerp_color(cursor, bg, 0.3), bg, fg);

        Theme {
            bg, fg, accent, cursor, sel_bg,
            black, red, green, yellow, blue, magenta, cyan, white,
            br_black, br_red, br_green, br_yellow, br_blue, br_magenta, br_cyan, br_white,
            orange, hover_bg,
            green_shades, cyan_shades, yellow_shades, magenta_shades, orange_shades, white_shades,
        }
    }

    pub fn shades(&self, palette: Palette) -> &[Color; 7] {
        match palette {
            Palette::Green | Palette::None => &self.green_shades,
            Palette::Cyan => &self.cyan_shades,
            Palette::Yellow => &self.yellow_shades,
            Palette::Magenta => &self.magenta_shades,
            Palette::Orange => &self.orange_shades,
            Palette::White => &self.white_shades,
        }
    }

    pub fn brightness_to_color(&self, b: f32, palette: Palette) -> Color {
        let shades = self.shades(palette);
        if b > 0.9 {
            return shades[0];
        }
        let idx = ((1.0 - b) * (shades.len() - 1) as f32) as usize;
        shades[idx.min(shades.len() - 1)]
    }

    pub fn palette_style_color(&self, palette: Palette) -> Color {
        match palette {
            Palette::Cyan => self.br_blue,
            Palette::Yellow => self.br_yellow,
            Palette::Magenta => self.magenta,
            Palette::Orange => self.orange,
            Palette::White => self.fg,
            Palette::Green | Palette::None => self.br_green,
        }
    }
}

fn fallback_theme() -> HashMap<String, String> {
    // Kanagawa palette — used when omarchy is not installed
    HashMap::from([
        ("background", "#1f1f28"),
        ("foreground", "#dcd7ba"),
        ("accent", "#7e9cd8"),
        ("cursor", "#c8c093"),
        ("selection_background", "#2d4f67"),
        ("color0", "#090618"),
        ("color1", "#c34043"),
        ("color2", "#76946a"),
        ("color3", "#c0a36e"),
        ("color4", "#7e9cd8"),
        ("color5", "#957fb8"),
        ("color6", "#6a9589"),
        ("color7", "#c8c093"),
        ("color8", "#727169"),
        ("color9", "#e82424"),
        ("color10", "#98bb6c"),
        ("color11", "#e6c384"),
        ("color12", "#7fb4ca"),
        ("color13", "#938aa9"),
        ("color14", "#7aa89f"),
        ("color15", "#dcd7ba"),
    ].map(|(k, v)| (k.to_string(), v.to_string())))
}

fn load_omarchy_theme() -> HashMap<String, String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let path = home.join(".config/omarchy/current/theme/colors.toml");
    let mut colors = fallback_theme();
    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || !line.contains('=') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let val = val.trim().trim_matches('"');
                colors.insert(key.trim().to_string(), val.to_string());
            }
        }
    }
    colors
}
