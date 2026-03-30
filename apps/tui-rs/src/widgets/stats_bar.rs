use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

pub fn render_stats_bar(app: &App, frame: &mut Frame, area: Rect) {
    let event_count = app.events.len();
    let agent_count = app.agents.len();
    let rate = app.rate();

    let (status_text, status_style) = if app.connected {
        (
            "\u{25cf} LIVE",
            Style::default()
                .fg(app.theme.br_green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            "\u{25cb} DISCONNECTED",
            Style::default()
                .fg(app.theme.br_red)
                .add_modifier(Modifier::BOLD),
        )
    };

    let sep = Style::default().fg(app.theme.sel_bg);
    let stat = Style::default().fg(app.theme.br_green);
    let title_style = Style::default()
        .fg(app.theme.fg)
        .bg(app.theme.sel_bg)
        .add_modifier(Modifier::BOLD);

    let mut spans = vec![
        Span::styled(" \u{2593} MATRIX OBSERVABILITY \u{2593}  ", title_style),
        Span::styled(format!("Events: {}", event_count), stat),
        Span::styled(" | ", sep),
        Span::styled(format!("Agents: {}", agent_count), stat),
        Span::styled(" | ", sep),
        Span::styled(format!("{:.1}/s", rate), stat),
        Span::styled(" | ", sep),
        Span::styled(status_text, status_style),
    ];

    if app.scroll_mode {
        spans.push(Span::styled(" | ", sep));
        spans.push(Span::styled(
            "\u{25bc} SCROLL",
            Style::default()
                .fg(app.theme.br_yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::raw(" "));

    let bar = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(app.theme.black));
    frame.render_widget(bar, area);
}
