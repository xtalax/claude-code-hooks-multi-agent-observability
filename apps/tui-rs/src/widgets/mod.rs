mod event_detail;
mod event_stream;
mod filter_bar;
mod help_bar;
mod rain_widget;
mod stats_bar;

pub use event_detail::render_event_detail;
pub use event_stream::render_event_stream;
pub use filter_bar::render_filter_bar;
pub use help_bar::render_help_bar;
pub use rain_widget::RainView;
pub use stats_bar::render_stats_bar;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

/// Minimum terminal dimensions for the full layout.
const MIN_WIDTH: u16 = 40;
const MIN_HEIGHT: u16 = 8;

/// Main render function called each frame from the event loop.
///
/// Layout:
///   Top:    stats_bar (1 row, full width)
///   Middle: Horizontal split -- RainPanel (flexible) | right panel (max 150 cols)
///           Right panel: EventStream (flexible) over EventDetail (if visible)
///   Bottom: help_bar (1 row) OR filter_bar (if filter is visible)
pub fn render(app: &mut App, frame: &mut Frame) {
    let size = frame.area();

    // Too-small fallback: show a resize prompt instead of panicking
    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        app.event_stream_area = Rect::default();
        let msg = format!("Terminal too small ({}x{}). Need {}x{}.", size.width, size.height, MIN_WIDTH, MIN_HEIGHT);
        let paragraph = Paragraph::new(Line::from(msg))
            .style(Style::default().fg(app.theme.br_yellow).bg(app.theme.bg));
        frame.render_widget(paragraph, size);
        return;
    }

    // Clamp selection to current filtered count each frame
    app.clamp_selection();

    // Vertical: stats bar (1) | middle (flex) | bottom bar (1)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // stats bar
            Constraint::Min(4),    // middle content
            Constraint::Length(1), // help or filter bar
        ])
        .split(size);

    let stats_area = outer[0];
    let middle_area = outer[1];
    let bottom_area = outer[2];

    // ── Stats bar ──
    render_stats_bar(app, frame, stats_area);

    // ── Middle: rain panel | right panel ──
    // Right panel gets up to 150 columns; rain panel fills the rest
    let right_max = 150u16.min(middle_area.width.saturating_sub(18));
    let rain_min = middle_area.width.saturating_sub(right_max);

    let middle_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(rain_min),       // rain panel (flexible, at least 18 wide)
            Constraint::Max(right_max),      // right panel (up to 150)
        ])
        .split(middle_area);

    let rain_area = middle_split[0];
    let right_area = middle_split[1];

    // ── Sync rain dimensions to actual area ──
    app.on_resize(rain_area.width, rain_area.height);

    // ── Rain panel ──
    let rain_view = RainView {
        rain: &app.rain,
        theme: &app.theme,
    };
    frame.render_widget(rain_view, rain_area);

    // ── Right panel: event stream + optional detail ──
    if app.detail_visible && app.detail_event.is_some() && right_area.height >= 8 {
        // Split right panel: stream on top, detail on bottom (up to 50%)
        let detail_height = right_area.height / 2;
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(4),                    // event stream
                Constraint::Length(detail_height),      // event detail
            ])
            .split(right_area);

        app.event_stream_area = right_split[0];
        render_event_stream(app, frame, right_split[0]);
        render_event_detail(app, frame, right_split[1]);
    } else {
        app.event_stream_area = right_area;
        render_event_stream(app, frame, right_area);
    }

    // ── Bottom bar ──
    if app.filter_visible {
        render_filter_bar(app, frame, bottom_area);
    } else {
        render_help_bar(app, frame, bottom_area);
    }
}
