use crate::config::Palette;
use crate::highlight::StyledSpans;

/// Highlight a file path: directories in Orange, filename in Yellow, separators in Magenta.
pub fn highlight_path(text: &str) -> StyledSpans {
    let parts: Vec<&str> = text.split('/').collect();
    let mut result = StyledSpans::new();

    for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
            result.push(("/".into(), Palette::Magenta));
        }
        let is_last = idx == parts.len() - 1;
        let color = if is_last { Palette::Yellow } else { Palette::Orange };
        if !part.is_empty() {
            result.push(((*part).to_string(), color));
        }
    }

    result
}

/// Highlight a regex/glob pattern: special chars in Orange, literals in Magenta.
pub fn highlight_pattern(text: &str) -> StyledSpans {
    let specials = ".*+?[](){}|^$\\";
    let mut result = StyledSpans::new();

    for ch in text.chars() {
        let color = if specials.contains(ch) {
            Palette::Orange
        } else {
            Palette::Magenta
        };
        result.push((ch.to_string(), color));
    }

    result
}
