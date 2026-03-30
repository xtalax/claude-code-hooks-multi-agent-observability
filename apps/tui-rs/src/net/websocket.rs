use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite;

use crate::config::SERVER_PORT;
use crate::event::HookEvent;

/// Messages sent from the websocket task to the main loop.
#[derive(Debug)]
pub enum WsMessage {
    /// Server sent the initial batch of historical events.
    Initial(Vec<HookEvent>),
    /// Server pushed a single new event.
    Event(HookEvent),
    /// WebSocket connection established.
    Connected,
    /// WebSocket connection lost.
    Disconnected,
}

/// Connect to the hook server and forward events over `tx`.
///
/// Reconnects automatically with exponential back-off (1 s → 30 s max).
/// Runs forever; intended to be spawned as a background tokio task.
pub async fn run(tx: mpsc::UnboundedSender<WsMessage>) {
    let url = format!("ws://localhost:{}/stream", SERVER_PORT);
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        match tokio_tungstenite::connect_async(&url).await {
            Ok((ws_stream, _)) => {
                let _ = tx.send(WsMessage::Connected);
                backoff = Duration::from_secs(1);

                let (_write, mut read) = ws_stream.split();
                while let Some(msg) = read.next().await {
                    let text = match msg {
                        Ok(tungstenite::Message::Text(t)) => t,
                        Ok(tungstenite::Message::Close(_)) => break,
                        Err(_) => break,
                        _ => continue,
                    };

                    if let Ok(frame) = serde_json::from_str::<serde_json::Value>(&text) {
                        match frame.get("type").and_then(|t| t.as_str()) {
                            Some("initial") => {
                                if let Some(arr) = frame.get("data").and_then(|d| d.as_array()) {
                                    let events: Vec<HookEvent> = arr
                                        .iter()
                                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                        .collect();
                                    let _ = tx.send(WsMessage::Initial(events));
                                }
                            }
                            Some("event") => {
                                if let Some(data) = frame.get("data") {
                                    if let Ok(ev) =
                                        serde_json::from_value::<HookEvent>(data.clone())
                                    {
                                        let _ = tx.send(WsMessage::Event(ev));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Connection closed / errored
                let _ = tx.send(WsMessage::Disconnected);
            }
            Err(_) => {
                let _ = tx.send(WsMessage::Disconnected);
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(max_backoff);
    }
}
