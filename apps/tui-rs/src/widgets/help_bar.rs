use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

pub fn render_help_bar(app: &App, frame: &mut Frame, area: Rect) {
    let style = Style::default()
        .fg(app.theme.br_black)
        .bg(app.theme.black);

    let line = Line::from(vec![Span::styled(
        " j/k: scroll  s: auto-scroll  Enter: detail  Esc: close  /: filter  c: clear  Tab: focus  q: quit",
        style,
    )]);

    let bar = Paragraph::new(line).style(style);
    frame.render_widget(bar, area);
}
