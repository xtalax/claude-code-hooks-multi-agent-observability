use crate::config::SERVER_PORT;
use std::time::Duration;

/// Check whether the hook server is reachable (blocking, 2s timeout).
pub fn server_is_up() -> bool {
    let url = format!("http://localhost:{}/health", SERVER_PORT);
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()
        .and_then(|c| c.get(&url).send().ok())
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
