use crate::config::Palette;
use crate::highlight::StyledSpans;

/// Keys whose values should get command-style highlighting.
const CMD_KEYS: &[&str] = &["command", "file_path", "pattern", "query", "url"];

/// Maximum lines to render from a JSON payload.
const MAX_LINES: usize = 30;

/// Highlight a JSON string with basic coloring.
///
/// Keys in Cyan, strings in Yellow, numbers in Magenta, braces in Green,
/// command-value keys get White highlighting.
pub fn highlight_json(text: &str, _tool_name: &str) -> StyledSpans {
    let mut result = StyledSpans::new();

    for (line_idx, line) in text.lines().enumerate() {
        if line_idx >= MAX_LINES {
            result.push(("  ... (truncated)\n".into(), Palette::None));
            break;
        }

        let stripped = line.trim_start();
        let indent = &line[..line.len() - stripped.len()];

        // Structural characters only
        if matches!(
            stripped,
            "{" | "}" | "{," | "}," | "[" | "]" | "[," | "]," | "{}" | "[]"
        ) {
            result.push((format!("  {}\n", line), Palette::Green));
            continue;
        }

        // Lines with a key-value pair: "key": value
        if stripped.starts_with('"') {
            if let Some(colon_pos) = stripped.find("\":") {
                let key_text = &stripped[1..colon_pos];
                let val_part = &stripped[colon_pos + 2..];

                // Check if this is a command key that deserves bright highlighting
                let is_cmd_key = CMD_KEYS.iter().any(|&k| k == key_text);
                let key_color = if is_cmd_key { Palette::White } else { Palette::Cyan };

                result.push((format!("  {}", indent), Palette::None));
                result.push((format!("\"{}\"", key_text), key_color));
                result.push((":".into(), Palette::None));
                highlight_json_value(val_part, &mut result);
                result.push(("\n".into(), Palette::None));
                continue;
            }
        }

        // Fallback: plain green text
        result.push((format!("  {}\n", line), Palette::Green));
    }

    result
}

/// Color a JSON value fragment (everything after the colon).
fn highlight_json_value(val: &str, result: &mut StyledSpans) {
    let trimmed = val.trim();

    if trimmed.starts_with('"') {
        // String value
        result.push((val.to_string(), Palette::Yellow));
    } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
        // Structure opener
        result.push((val.to_string(), Palette::Green));
    } else if trimmed == "true" || trimmed == "false" || trimmed == "null"
        || trimmed == "true," || trimmed == "false," || trimmed == "null,"
    {
        result.push((val.to_string(), Palette::Magenta));
    } else if trimmed.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        // Number
        result.push((val.to_string(), Palette::Magenta));
    } else {
        result.push((val.to_string(), Palette::Green));
    }
}
