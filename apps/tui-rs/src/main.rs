mod app;
mod config;
mod event;
mod input;

mod ai;
mod highlight;
mod net;
mod rain;
mod widgets;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self as ct_event, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use app::App;
use net::websocket::WsMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(".env"),
    )
    .ok();

    let theme = config::Theme::load();

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Channels
    let (ws_tx, ws_rx) = mpsc::unbounded_channel::<WsMessage>();
    let (haiku_req_tx, haiku_req_rx) = mpsc::unbounded_channel::<ai::haiku::HaikuRequest>();
    let (haiku_res_tx, haiku_res_rx) = mpsc::unbounded_channel::<ai::haiku::HaikuResult>();

    // Spawn background tasks
    tokio::spawn(net::websocket::run(ws_tx));
    tokio::spawn(ai::haiku::worker(haiku_req_rx, haiku_res_tx));

    let mut app = App::new(theme, ws_rx, haiku_req_tx, haiku_res_rx);

    // Auto-start server if needed
    app.ensure_server();

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| widgets::render(&mut app, f))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if ct_event::poll(timeout)? {
            match ct_event::read()? {
                ct_event::Event::Key(key) => {
                    if input::handle_key(&mut app, key) {
                        break;
                    }
                }
                ct_event::Event::Mouse(mouse) => {
                    input::handle_mouse(&mut app, mouse);
                }
                ct_event::Event::Resize(_w, _h) => {
                    // Clamp selected index so it stays valid after resize
                    app.clamp_selection();
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = std::time::Instant::now();
        }
    }

    // Cleanup
    app.shutdown();
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
