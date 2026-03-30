use crate::config::Palette;
use crate::highlight::languages::{is_lang_command, is_keyword, lang_mode_for};
use crate::highlight::StyledSpans;

/// Syntax-highlight a bash command string.
///
/// Colors: commands (White), flags (Magenta), strings (Yellow),
/// operators (Orange), paths (Yellow), arguments (Cyan).
pub fn highlight_bash(text: &str) -> StyledSpans {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut result = StyledSpans::new();
    let mut i = 0;
    let mut expect_cmd = true;
    let mut lang_mode: &str = "";
    let mut last_flag = String::new();

    while i < len {
        let ch = chars[i];

        // Whitespace
        if ch == ' ' {
            result.push((" ".into(), Palette::Cyan));
            i += 1;
            continue;
        }

        // Operators: | || && ; > >> < <<
        if ch == '|' || ch == ';' {
            result.push((ch.to_string(), Palette::Orange));
            if ch == '|' && i + 1 < len && chars[i + 1] == '|' {
                i += 1;
                result.push(("|".into(), Palette::Orange));
            }
            expect_cmd = true;
            lang_mode = "";
            i += 1;
            continue;
        }
        if ch == '&' {
            result.push(("&".into(), Palette::Orange));
            if i + 1 < len && chars[i + 1] == '&' {
                i += 1;
                result.push(("&".into(), Palette::Orange));
                expect_cmd = true;
                lang_mode = "";
            }
            i += 1;
            continue;
        }
        if ch == '>' || ch == '<' {
            result.push((ch.to_string(), Palette::Orange));
            if i + 1 < len && chars[i + 1] == ch {
                i += 1;
                result.push((ch.to_string(), Palette::Orange));
            }
            i += 1;
            continue;
        }

        // Quoted strings
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let mut raw = String::new();
            let mut j = i + 1;
            while j < len && chars[j] != quote {
                if chars[j] == '\\' && j + 1 < len {
                    raw.push(chars[j]);
                    raw.push(chars[j + 1]);
                    j += 2;
                } else {
                    raw.push(chars[j]);
                    j += 1;
                }
            }

            // If lang_mode active and last flag was -e/-c, highlight as code
            if !lang_mode.is_empty()
                && matches!(last_flag.as_str(), "-e" | "-c" | "--eval" | "--eval=")
            {
                result.push((quote.to_string(), Palette::Yellow));
                highlight_inline_code(&raw, lang_mode, &mut result);
                if j < len {
                    result.push((chars[j].to_string(), Palette::Yellow));
                    j += 1;
                }
            } else {
                result.push((quote.to_string(), Palette::Yellow));
                for k in (i + 1)..j {
                    if chars[k] == '\\' && k + 1 < j {
                        result.push((chars[k].to_string(), Palette::Orange));
                    } else {
                        result.push((chars[k].to_string(), Palette::Yellow));
                    }
                }
                if j < len {
                    result.push((chars[j].to_string(), Palette::Yellow));
                    j += 1;
                }
            }
            i = j;
            last_flag.clear();
            continue;
        }

        // Flags: -x, --long-flag
        if ch == '-' && (i == 0 || chars[i - 1] == ' ') {
            let flag_start = i;
            let mut flag_str = String::new();
            while i < len && !is_word_break(chars[i]) {
                result.push((chars[i].to_string(), Palette::Magenta));
                flag_str.push(chars[i]);
                i += 1;
            }
            last_flag = text[flag_start..flag_start + flag_str.len()].to_string();
            expect_cmd = false;
            continue;
        }

        // Collect a word
        let ws = i;
        while i < len && !is_word_break(chars[i]) {
            i += 1;
        }
        let word: String = chars[ws..i].iter().collect();

        if word.is_empty() {
            result.push((chars.get(i).map_or("?".into(), |c| c.to_string()), Palette::Cyan));
            i += 1;
            continue;
        }

        // Classify the word
        let color = if word.starts_with('/') || word.starts_with("./") || word.starts_with("~/") {
            Palette::Yellow // path
        } else if expect_cmd && is_lang_command(&word) {
            lang_mode = lang_mode_for(&word);
            Palette::White // known command
        } else {
            if expect_cmd {
                lang_mode = "";
            }
            Palette::Cyan // argument or unknown command
        };

        result.push((word, color));
        expect_cmd = false;
    }

    result
}

fn is_word_break(ch: char) -> bool {
    matches!(ch, ' ' | '|' | '&' | ';' | '>' | '<' | '"' | '\'')
}

/// Highlight inline code within a quoted string (e.g. `python -c '...'`).
fn highlight_inline_code(code: &str, lang: &str, result: &mut StyledSpans) {
    let chars: Vec<char> = code.chars().collect();
    let len = chars.len();
    let mut i = 0;

    // We only need keyword checking — use the unified is_keyword check
    let _ = lang; // lang could refine keyword set, but is_keyword covers all

    while i < len {
        let ch = chars[i];

        // Strings
        if ch == '"' || ch == '\'' || ch == '`' {
            let quote = ch;
            result.push((ch.to_string(), Palette::Yellow));
            i += 1;
            while i < len && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < len {
                    result.push((chars[i].to_string(), Palette::Orange));
                    i += 1;
                }
                result.push((chars[i].to_string(), Palette::Yellow));
                i += 1;
            }
            if i < len {
                result.push((chars[i].to_string(), Palette::Yellow));
                i += 1;
            }
            continue;
        }

        // Numbers
        if ch.is_ascii_digit() {
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                result.push((chars[i].to_string(), Palette::Orange));
                i += 1;
            }
            continue;
        }

        // Operators and punctuation
        if "+-*/%=!<>&|^~@#$?:;,".contains(ch) {
            result.push((ch.to_string(), Palette::Orange));
            i += 1;
            continue;
        }

        // Brackets
        if "()[]{}".contains(ch) {
            result.push((ch.to_string(), Palette::Magenta));
            i += 1;
            continue;
        }

        // Whitespace
        if ch == ' ' {
            result.push((" ".into(), Palette::Cyan));
            i += 1;
            continue;
        }

        // Words
        if ch.is_alphabetic() || ch == '_' {
            let ws = i;
            while i < len && (chars[i].is_alphanumeric() || matches!(chars[i], '_' | '!' | '?')) {
                i += 1;
            }
            let word: String = chars[ws..i].iter().collect();
            let color = if is_keyword(&word) {
                Palette::White
            } else {
                Palette::Cyan
            };
            result.push((word, color));
            continue;
        }

        // Anything else
        result.push((ch.to_string(), Palette::Cyan));
        i += 1;
    }
}
