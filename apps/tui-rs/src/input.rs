use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use crate::app::App;

/// Handle a key event. Returns `true` if the app should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // When the filter bar is active, capture text-editing keys first.
    if app.filter_visible {
        match key.code {
            KeyCode::Esc => {
                app.filter_visible = false;
                return false;
            }
            KeyCode::Enter => {
                app.filter_visible = false;
                return false;
            }
            KeyCode::Backspace => {
                app.filter_text.pop();
                return false;
            }
            KeyCode::Char(c) => {
                app.filter_text.push(c);
                return false;
            }
            _ => return false,
        }
    }

    // Global keybindings — mirrors Python BINDINGS list exactly.
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('j') => app.scroll_down(),
        KeyCode::Char('k') => app.scroll_up(),
        KeyCode::Char('/') => {
            app.filter_visible = true;
        }
        KeyCode::Char('c') => {
            app.filter_text.clear();
        }
        KeyCode::Char('s') => app.toggle_scroll(),
        KeyCode::Enter => app.show_detail(),
        KeyCode::Esc => app.close_overlay(),
        KeyCode::End => app.scroll_to_bottom(),
        KeyCode::Home => app.scroll_to_top(),
        KeyCode::Tab => { /* focus_next — no-op in ratatui single-focus model */ }
        _ => {}
    }
    false
}

/// Check if a mouse position is inside a Rect.
fn in_rect(area: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
}

/// Handle mouse events (clicks, scroll, and hover).
pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    let stream_area = app.event_stream_area;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if in_rect(stream_area, mouse.column, mouse.row) {
                // Click in event stream: select the clicked row and show detail
                let row_offset = (mouse.row - stream_area.y) as usize;
                let count = app.filtered_events().len();
                if count > 0 {
                    let scroll_offset = app.list_state.offset();
                    let clicked_idx = scroll_offset + row_offset;
                    if clicked_idx < count {
                        app.selected_idx = Some(clicked_idx);
                        app.list_state.select(Some(clicked_idx));
                        app.show_detail();
                    }
                }
            } else {
                // Click on the rain panel area: filter to the clicked agent.
                let (agent_id, _col_idx) = app.rain.agent_at_x(mouse.column as usize);
                if let Some(id) = agent_id {
                    app.filter_text = id.to_string();
                    app.filter_visible = true;
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if in_rect(stream_area, mouse.column, mouse.row) {
                app.scroll_down();
            }
        }
        MouseEventKind::ScrollUp => {
            if in_rect(stream_area, mouse.column, mouse.row) {
                app.scroll_up();
            }
        }
        MouseEventKind::Moved => {
            let x = mouse.column as usize;
            let agent_name = {
                let (agent_id, _) = app.rain.agent_at_x(x);
                agent_id.map(|s| s.to_string())
            };
            app.rain.hovered_col_x = x as i32;
            app.rain.hovered_agent = agent_name;
        }
        _ => {}
    }
}
