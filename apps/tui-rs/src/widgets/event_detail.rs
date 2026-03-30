use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::highlight::{extract_tool_info, highlight_detail};
use crate::highlight::json::highlight_json;

pub fn render_event_detail(app: &App, frame: &mut Frame, area: Rect) {
    let event = match &app.detail_event {
        Some(ev) => ev,
        None => return,
    };

    let label_style = Style::default().fg(app.theme.br_blue);
    let value_style = Style::default().fg(app.theme.fg);
    let accent = Style::default()
        .fg(app.theme.accent)
        .add_modifier(Modifier::BOLD);
    let sep_style = Style::default().fg(app.theme.sel_bg);

    let mut lines: Vec<Line> = Vec::new();

    // Header
    let header_dashes = "\u{2500}".repeat(40);
    lines.push(Line::from(vec![
        Span::styled("\u{2500}\u{2500} EVENT DETAIL ", accent),
        Span::styled(header_dashes, sep_style),
    ]));

    // Type
    lines.push(Line::from(vec![
        Span::styled("Type: ", label_style),
        Span::styled(event.hook_event_type.clone(), value_style),
    ]));

    // Agent
    lines.push(Line::from(vec![
        Span::styled("Agent: ", label_style),
        Span::styled(event.agent_id(), value_style),
    ]));

    // Haiku status (if available)
    let haiku_status = app
        .rain
        .agent_haiku_status
        .get(&event.agent_id())
        .map(|(text, _)| text.clone())
        .unwrap_or_default();
    if !haiku_status.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Status: ", label_style),
            Span::styled(
                haiku_status,
                Style::default().fg(app.theme.br_green),
            ),
        ]));
    }

    // Time
    let ts_str = if event.timestamp > 0 {
        let secs = event.timestamp / 1000;
        let h = (secs / 3600) % 24;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        "?".to_string()
    };
    lines.push(Line::from(vec![
        Span::styled("Time: ", label_style),
        Span::styled(ts_str, value_style),
    ]));

    // Model
    if !event.model_name.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Model: ", label_style),
            Span::styled(event.model_name.clone(), value_style),
        ]));
    }

    // Command (syntax-highlighted)
    if let Some((cmd_text, tool_name, _palette)) = extract_tool_info(event) {
        if !cmd_text.is_empty() {
            let highlighted = highlight_detail(&cmd_text, &tool_name);
            let mut cmd_spans = vec![Span::styled(
                "Command: ",
                Style::default()
                    .fg(app.theme.br_blue)
                    .add_modifier(Modifier::BOLD),
            )];
            for (text, palette) in &highlighted {
                let color = app.theme.palette_style_color(*palette);
                cmd_spans.push(Span::styled(text.clone(), Style::default().fg(color)));
            }
            lines.push(Line::from(cmd_spans));
        }
    }

    // Summary
    if !event.summary.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Summary: ", label_style),
            Span::styled(
                event.summary.clone(),
                Style::default().fg(app.theme.br_green),
            ),
        ]));
    }

    // Payload (JSON highlighted)
    lines.push(Line::from(vec![Span::styled("Payload:", label_style)]));

    let tool_name_for_json = extract_tool_info(event)
        .map(|(_, t, _)| t)
        .unwrap_or_default();
    match serde_json::to_string_pretty(&event.payload) {
        Ok(json_str) => {
            let highlighted = highlight_json(&json_str, &tool_name_for_json);
            // Collect spans into lines, splitting on newlines within spans.
            // Multiple spans between newlines belong on the same Line.
            let mut current_spans: Vec<Span> = Vec::new();
            for (text, palette) in &highlighted {
                let color = app.theme.palette_style_color(*palette);
                let parts: Vec<&str> = text.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if !part.is_empty() {
                        current_spans.push(Span::styled(
                            part.to_string(),
                            Style::default().fg(color),
                        ));
                    }
                    // A newline boundary (not the last fragment) flushes the current line
                    if i < parts.len() - 1 && !current_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                    }
                }
            }
            if !current_spans.is_empty() {
                lines.push(Line::from(current_spans));
            }
        }
        Err(_) => {
            lines.push(Line::from(vec![Span::styled(
                format!("  {:?}", event.payload),
                Style::default().fg(app.theme.green),
            )]));
        }
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(app.theme.sel_bg))
        .style(Style::default().bg(app.theme.bg));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
