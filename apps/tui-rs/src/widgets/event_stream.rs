use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;

use crate::app::App;
use crate::config::{glyph, tool_palette};
use crate::event::HookEvent;

pub fn render_event_stream(app: &mut App, frame: &mut Frame, area: Rect) {
    let filtered = app.filtered_events();
    let count = filtered.len();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|ev| ListItem::new(format_event_line(ev, app)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(app.theme.bg)),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.sel_bg)
                .add_modifier(Modifier::BOLD),
        );

    // When user hasn't selected anything, auto-scroll to bottom
    if app.selected_idx.is_none() && count > 0 {
        app.list_state.select(Some(count - 1));
    }

    frame.render_stateful_widget(list, area, &mut app.list_state);

    // Scrollbar
    if count > area.height as usize {
        let position = app.list_state.offset();
        let mut scrollbar_state = ScrollbarState::new(count.saturating_sub(area.height as usize))
            .position(position);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(app.theme.sel_bg))
            .track_style(Style::default().fg(app.theme.bg));
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

/// Format a single event into a styled Line for the list.
fn format_event_line<'a>(event: &HookEvent, app: &App) -> Line<'a> {
    let mut spans = Vec::new();

    // Timestamp
    let ts = if event.timestamp > 0 {
        let secs = event.timestamp / 1000;
        let h = (secs / 3600) % 24;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        "??:??:??".to_string()
    };
    spans.push(Span::styled(ts, Style::default().fg(app.theme.br_black)));
    spans.push(Span::raw(" "));

    // Glyph with event-type-based color
    let g = glyph(&event.hook_event_type);
    let glyph_color = glyph_color_for_type(&event.hook_event_type, app);
    spans.push(Span::styled(
        g.to_string(),
        Style::default()
            .fg(glyph_color)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("] ", Style::default().fg(glyph_color)));

    // Agent ID (left-padded to 20)
    let agent_id = event.agent_id();
    let display_name = app
        .rain
        .agent_names
        .get(agent_id.as_str())
        .map(|s| s.as_str())
        .unwrap_or(agent_id.as_str());
    let padded_id = format!("{:<20}", display_name);
    spans.push(Span::styled(padded_id, Style::default().fg(app.theme.br_blue)));

    // Event type
    spans.push(Span::styled(
        event.hook_event_type.clone(),
        Style::default()
            .fg(app.theme.br_green)
            .add_modifier(Modifier::BOLD),
    ));

    // Tool name if present
    if let Some(tool) = event.tool_name() {
        spans.push(Span::styled(
            format!(":{}", tool),
            Style::default().fg(app.theme.green),
        ));
    }

    // Summary with tool palette color
    if !event.summary.is_empty() {
        let summary = if event.summary.len() > 60 {
            format!("  {}...", &event.summary[..57])
        } else {
            format!("  {}", event.summary)
        };
        let summary_color = event
            .tool_name()
            .map(|t| app.theme.palette_style_color(tool_palette(t)))
            .unwrap_or(app.theme.fg);
        spans.push(Span::styled(summary, Style::default().fg(summary_color)));
    }

    Line::from(spans)
}

/// Determine the glyph color based on event type.
fn glyph_color_for_type(event_type: &str, app: &App) -> ratatui::style::Color {
    if event_type.contains("Failure") || event_type == "Stop" {
        app.theme.red
    } else if event_type == "SubagentStart" || event_type == "SubagentStop" {
        app.theme.green
    } else if event_type == "UserPromptSubmit" {
        app.theme.br_yellow
    } else if event_type == "Notification" {
        app.theme.br_blue
    } else {
        app.theme.br_green
    }
}
