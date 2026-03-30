pub mod bash;
pub mod json;
pub mod languages;
pub mod path;

use crate::config::{tool_palette, Palette};
use crate::event::HookEvent;

/// A span of text with an associated palette color.
pub type StyledSpan = (String, Palette);
pub type StyledSpans = Vec<StyledSpan>;

/// Dispatch to the appropriate highlighter based on tool name.
pub fn highlight_command(text: &str, tool_name: &str) -> StyledSpans {
    if text.is_empty() {
        return Vec::new();
    }
    match tool_name {
        "Bash" => bash::highlight_bash(text),
        "Read" | "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => path::highlight_path(text),
        "Grep" | "Glob" => path::highlight_pattern(text),
        _ => {
            let palette = tool_palette(tool_name);
            let color = if palette == Palette::None {
                Palette::Cyan
            } else {
                palette
            };
            vec![(text.to_string(), color)]
        }
    }
}

/// Highlight text for the event detail panel (same logic, different entry point).
pub fn highlight_detail(text: &str, tool_name: &str) -> StyledSpans {
    highlight_command(text, tool_name)
}

/// Extract command text, tool name, and palette from a hook event.
///
/// Returns `None` for non-PreToolUse events or unknown tools.
pub fn extract_tool_info(event: &HookEvent) -> Option<(String, String, Palette)> {
    if event.hook_event_type != "PreToolUse" {
        return None;
    }

    let tool_name = event.tool_name()?.to_string();
    let palette = tool_palette(&tool_name);
    if palette == Palette::None {
        return None;
    }

    let tool_input = event.payload.get("tool_input");

    let text = match tool_name.as_str() {
        "Bash" => get_str(tool_input, "command"),
        "Read" | "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => {
            get_str(tool_input, "file_path")
        }
        "Grep" | "Glob" => get_str(tool_input, "pattern"),
        "Task" => get_str(tool_input, "description"),
        "WebFetch" => get_str(tool_input, "url"),
        "WebSearch" => get_str(tool_input, "query"),
        _ => tool_name.clone(),
    };

    let text = text.replace('\n', " ");
    let text = text.trim().to_string();
    let text = if text.len() > 60 {
        text[..60].to_string()
    } else {
        text
    };

    Some((text, tool_name, palette))
}

/// Pull a string from a nested JSON object: tool_input.key
fn get_str(tool_input: Option<&serde_json::Value>, key: &str) -> String {
    tool_input
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
