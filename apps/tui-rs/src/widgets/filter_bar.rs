use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render_filter_bar(app: &App, frame: &mut Frame, area: Rect) {
    let label_style = Style::default()
        .fg(app.theme.br_green)
        .add_modifier(Modifier::BOLD);

    let input_style = Style::default().fg(app.theme.br_green);

    let cursor = if app.filter_visible { "\u{2588}" } else { "" };

    let line = Line::from(vec![
        Span::styled(" Filter: ", label_style),
        Span::styled(app.filter_text.clone(), input_style),
        Span::styled(cursor, input_style),
    ]);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(app.theme.sel_bg))
        .style(Style::default().bg(app.theme.bg));

    let bar = Paragraph::new(line).block(block);
    frame.render_widget(bar, area);
}
