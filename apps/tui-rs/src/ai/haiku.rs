use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::{mpsc, Semaphore};

/// Request sent from App to the Haiku worker.
pub struct HaikuRequest {
    pub agent_id: String,
    /// Whether to generate a name for this agent.
    pub needs_name: bool,
    /// Context for name generation (e.g. first user prompt).
    pub name_context: String,
    /// Context for status generation (recent events summary).
    pub status_context: String,
    /// When true, forget that this agent was named so it can be re-named.
    pub forget: bool,
}

/// Result sent back from the Haiku worker to App.
pub struct HaikuResult {
    pub agent_id: String,
    pub name: Option<String>,
    pub status: Option<String>,
}

const CONCURRENCY: usize = 4;

/// Background worker that processes Haiku API requests.
///
/// Reads `HaikuRequest`s from `rx`, calls the Haiku API with bounded
/// concurrency, and sends `HaikuResult`s back on `result_tx`.
///
/// If `ANTHROPIC_API_KEY` is not set, drains the channel silently (no-op mode).
pub async fn worker(
    mut rx: mpsc::UnboundedReceiver<HaikuRequest>,
    result_tx: mpsc::UnboundedSender<HaikuResult>,
) {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            // No API key -- drain requests so the channel never backs up.
            while rx.recv().await.is_some() {}
            return;
        }
    };

    let client = reqwest::Client::new();
    let sem = Arc::new(Semaphore::new(CONCURRENCY));
    let mut named: HashSet<String> = HashSet::new();

    while let Some(req) = rx.recv().await {
        // Forget signal: clear naming state so the agent can be re-named
        // when it becomes active again after being reaped.
        if req.forget {
            named.remove(&req.agent_id);
            continue;
        }

        let permit = Arc::clone(&sem);
        let client = client.clone();
        let key = api_key.clone();
        let tx = result_tx.clone();

        // Track naming locally so we only generate a name once per agent,
        // even if the App sends needs_name=true multiple times before the
        // first result arrives.
        let should_name = req.needs_name && named.insert(req.agent_id.clone());

        tokio::spawn(async move {
            let _permit = permit.acquire().await.expect("semaphore closed");

            let name = if should_name && !req.name_context.is_empty() {
                let prompt = format!(
                    "Give this AI coding agent a short memorable name (1-3 words) based on \
                     what it was asked to do. Lowercase, no punctuation.\n\n{}",
                    req.name_context
                );
                call_haiku(&client, &key, &prompt, 15).await
            } else {
                None
            };

            let status = if !req.status_context.is_empty() {
                let prompt = format!(
                    "In 3-8 words, describe what this AI coding agent is currently doing \
                     based on its recent activity. Be specific and concise. No punctuation. \
                     Lowercase.\n\n{}",
                    req.status_context
                );
                call_haiku(&client, &key, &prompt, 30).await
            } else {
                None
            };

            let _ = tx.send(HaikuResult {
                agent_id: req.agent_id,
                name,
                status,
            });
        });
    }
}

/// Call the Anthropic Messages API with the given prompt.
/// Returns `None` on any error (network, parse, bad status, etc.).
async fn call_haiku(
    client: &reqwest::Client,
    api_key: &str,
    prompt: &str,
    max_tokens: u32,
) -> Option<String> {
    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": max_tokens,
        "messages": [{
            "role": "user",
            "content": prompt,
        }],
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let text = json
        .get("content")?
        .as_array()?
        .first()?
        .get("text")?
        .as_str()?
        .trim()
        .to_string();

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
