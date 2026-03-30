use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use crate::ai::haiku::{HaikuRequest, HaikuResult};
use crate::config::{self, Palette, Theme, MAX_LOG_LINES, MIN_AGENT_COLS};
use crate::event::HookEvent;
use crate::highlight::{extract_tool_info, highlight_command};
use crate::net::health::server_is_up;
use crate::net::server::ServerProcess;
use crate::net::websocket::WsMessage;
use crate::rain::{ColoredChars, RainPanel};

// Timer intervals
const REBALANCE_INTERVAL: Duration = Duration::from_secs(2);
const REAP_INTERVAL: Duration = Duration::from_secs(5);
const HAIKU_INTERVAL: Duration = Duration::from_secs(8);

// Agent lifecycle
const AGENT_STALE_TIMEOUT: Duration = Duration::from_secs(60);
const AGENT_STOP_GRACE: Duration = Duration::from_secs(5);

// Rate window
const RATE_WINDOW: Duration = Duration::from_secs(5);
// Traffic rebalance window
const TRAFFIC_WINDOW: Duration = Duration::from_secs(10);

// Haiku status TTL
const HAIKU_STATUS_TTL: Duration = Duration::from_secs(15);

/// Default rain panel height used when we don't know terminal size yet.
const DEFAULT_RAIN_HEIGHT: usize = 24;

pub struct App {
    pub theme: Theme,
    pub rain: RainPanel,
    pub events: Vec<HookEvent>,
    pub agents: HashSet<String>,
    pub connected: bool,
    pub filter_text: String,
    pub filter_visible: bool,
    pub scroll_mode: bool,
    pub selected_idx: Option<usize>,
    pub list_state: ratatui::widgets::ListState,
    pub detail_event: Option<HookEvent>,
    pub detail_visible: bool,
    pub event_stream_area: ratatui::layout::Rect,

    // Agent lifecycle
    pub agent_last_seen: HashMap<String, Instant>,
    pub agent_stopped: HashMap<String, Instant>,
    pub agent_event_times: HashMap<String, Vec<Instant>>,
    pub agent_last_tool: HashMap<String, (String, Instant)>,
    pub agent_status_expires: HashMap<String, Instant>,

    // Rate tracking
    pub rate_timestamps: Vec<Instant>,

    // Channels
    ws_rx: mpsc::UnboundedReceiver<WsMessage>,
    haiku_tx: mpsc::UnboundedSender<HaikuRequest>,
    haiku_rx: mpsc::UnboundedReceiver<HaikuResult>,

    // Server process
    server_proc: Option<ServerProcess>,

    // Timers
    last_rebalance: Instant,
    last_reap: Instant,
    last_haiku: Instant,

    // Terminal dimensions (updated by resize events)
    pub rain_height: usize,
    pub rain_width: usize,
}

impl App {
    pub fn new(
        theme: Theme,
        ws_rx: mpsc::UnboundedReceiver<WsMessage>,
        haiku_tx: mpsc::UnboundedSender<HaikuRequest>,
        haiku_rx: mpsc::UnboundedReceiver<HaikuResult>,
    ) -> Self {
        let now = Instant::now();
        let mut rain = RainPanel::new();
        rain.init_ambient(80, DEFAULT_RAIN_HEIGHT);

        Self {
            theme,
            rain,
            events: Vec::new(),
            agents: HashSet::new(),
            connected: false,
            filter_text: String::new(),
            filter_visible: false,
            scroll_mode: false,
            selected_idx: None,
            list_state: ratatui::widgets::ListState::default(),
            detail_event: None,
            detail_visible: false,
            event_stream_area: ratatui::layout::Rect::default(),

            agent_last_seen: HashMap::new(),
            agent_stopped: HashMap::new(),
            agent_event_times: HashMap::new(),
            agent_last_tool: HashMap::new(),
            agent_status_expires: HashMap::new(),

            rate_timestamps: Vec::new(),

            ws_rx,
            haiku_tx,
            haiku_rx,

            server_proc: None,

            last_rebalance: now,
            last_reap: now,
            last_haiku: now,

            rain_height: DEFAULT_RAIN_HEIGHT,
            rain_width: 80,
        }
    }

    // ── Tick (called every 65ms from main loop) ──

    pub fn tick(&mut self) {
        // 1. Drain all pending websocket messages
        self.drain_ws();

        // 2. Drain all pending haiku results
        self.drain_haiku();

        // 3. Advance rain animation
        self.rain.tick();

        let now = Instant::now();

        // 4. Rebalance columns every 2s
        if now.duration_since(self.last_rebalance) >= REBALANCE_INTERVAL {
            self.rebalance_columns();
            self.update_tool_status();
            self.last_rebalance = now;
        }

        // 5. Reap dead agents every 5s
        if now.duration_since(self.last_reap) >= REAP_INTERVAL {
            self.reap_dead_agents();
            self.last_reap = now;
        }

        // 6. Trigger haiku refresh every 8s
        if now.duration_since(self.last_haiku) >= HAIKU_INTERVAL {
            self.trigger_haiku_refresh();
            self.last_haiku = now;
        }
    }

    // ── WebSocket drain ──

    fn drain_ws(&mut self) {
        while let Ok(msg) = self.ws_rx.try_recv() {
            match msg {
                WsMessage::Connected => {
                    self.connected = true;
                }
                WsMessage::Disconnected => {
                    self.connected = false;
                }
                WsMessage::Initial(events) => {
                    for event in events {
                        self.ingest_event(event, true);
                    }
                }
                WsMessage::Event(event) => {
                    self.ingest_event(event, false);
                }
            }
        }
    }

    // ── Haiku drain ──

    fn drain_haiku(&mut self) {
        while let Ok(result) = self.haiku_rx.try_recv() {
            if let Some(name) = result.name {
                self.rain
                    .agent_names
                    .insert(result.agent_id.clone(), name);
            }
            if let Some(status) = result.status {
                self.rain
                    .agent_full_status
                    .insert(result.agent_id.clone(), status.clone());
                self.agent_status_expires
                    .insert(result.agent_id.clone(), Instant::now() + HAIKU_STATUS_TTL);

                // Truncate for display
                let aw = *self
                    .rain
                    .agent_widths
                    .get(&result.agent_id)
                    .unwrap_or(&config::DEFAULT_AGENT_COLS);
                let display = if status.len() <= aw {
                    status.clone()
                } else if aw > 3 {
                    format!("{}...", &status[..aw - 3])
                } else {
                    status[..aw].to_string()
                };

                // Pick style from the agent's last tool
                let style = if let Some((ref tool_name, _)) = self.agent_last_tool.get(&result.agent_id) {
                    let palette = config::tool_palette(tool_name);
                    palette_style_name(palette)
                } else {
                    "cyan".to_string()
                };

                self.rain
                    .agent_haiku_status
                    .insert(result.agent_id.clone(), (display, style));
            }
        }
    }

    // ── Event ingestion ──

    pub fn ingest_event(&mut self, event: HookEvent, batch: bool) {
        let agent_id = event.agent_id();
        let now = Instant::now();

        // Add to event log
        self.events.push(event.clone());
        if self.events.len() > MAX_LOG_LINES {
            let drain = self.events.len() - MAX_LOG_LINES;
            self.events.drain(..drain);
        }

        // Track agent
        self.agents.insert(agent_id.clone());
        self.agent_last_seen.insert(agent_id.clone(), now);
        self.agent_event_times
            .entry(agent_id.clone())
            .or_default()
            .push(now);

        // Track last tool call -- invalidate haiku cache on tool change
        if event.hook_event_type == "PreToolUse" {
            if let Some(tool_name) = event.tool_name() {
                let tool_name = tool_name.to_string();
                let prev = self.agent_last_tool.get(&agent_id);
                let changed = prev.map_or(true, |(prev_tool, _)| prev_tool != &tool_name);
                self.agent_last_tool
                    .insert(agent_id.clone(), (tool_name, now));
                if changed {
                    self.agent_status_expires.remove(&agent_id);
                }
            }
        }

        // Mark agent for removal on terminal events
        if event.hook_event_type == "Stop" || event.hook_event_type == "SessionEnd" {
            self.agent_stopped.insert(agent_id.clone(), now);
        }

        // Update rain panel
        self.rain.ensure_agent(&agent_id, self.rain_height);

        if !batch {
            self.rain.pulse_agent(&agent_id, self.rain_height);
            self.rate_timestamps.push(now);

            // Inject syntax-highlighted tool text into rain column
            if let Some((text, tool_name, _palette)) = extract_tool_info(&event) {
                let spans = highlight_command(&text, &tool_name);
                let colored = spans_to_colored_chars(&spans);
                self.rain
                    .inject_tool(&agent_id, &colored, event.id, self.rain_height);
            }
        }
    }

    // ── Column rebalancing ──
    // 50% equal share + 50% proportional to sqrt(events_in_last_10s)

    pub fn rebalance_columns(&mut self) {
        if self.agents.is_empty() {
            return;
        }

        let agents: Vec<String> = self.rain.agent_order.clone();
        let n = agents.len();
        if n == 0 {
            return;
        }

        let total_width = self.rain_width;
        if total_width < n * MIN_AGENT_COLS {
            return;
        }

        let now = Instant::now();
        let cutoff = now - TRAFFIC_WINDOW;

        // Prune old timestamps and compute rates
        let mut rates: HashMap<&str, usize> = HashMap::new();
        for aid in &agents {
            if let Some(times) = self.agent_event_times.get_mut(aid.as_str()) {
                times.retain(|t| *t > cutoff);
                rates.insert(aid.as_str(), times.len());
            } else {
                rates.insert(aid.as_str(), 0);
            }
        }

        // Allocate: equal base share + gentle traffic bonus (sqrt)
        let base_share = total_width as f64 / n as f64;
        let bonuses: HashMap<&str, f64> = agents
            .iter()
            .map(|aid| {
                let r = *rates.get(aid.as_str()).unwrap_or(&0) as f64;
                (aid.as_str(), r.sqrt())
            })
            .collect();
        let total_bonus: f64 = bonuses.values().sum::<f64>().max(1.0);

        let mut new_widths: HashMap<&str, usize> = HashMap::new();
        for aid in &agents {
            let bonus = bonuses.get(aid.as_str()).copied().unwrap_or(0.0);
            let traffic_share = (total_width as f64 * 0.5) * bonus / total_bonus;
            let raw = base_share * 0.5 + traffic_share;
            new_widths.insert(aid.as_str(), raw.floor().max(MIN_AGENT_COLS as f64) as usize);
        }

        // Distribute leftover columns to highest-traffic agents
        let assigned: usize = new_widths.values().sum();
        let leftover = total_width as isize - assigned as isize;

        if leftover > 0 {
            let mut by_rate: Vec<&str> = agents.iter().map(|s| s.as_str()).collect();
            by_rate.sort_by(|a, b| {
                rates.get(b).unwrap_or(&0).cmp(rates.get(a).unwrap_or(&0))
            });
            for i in 0..leftover as usize {
                let aid = by_rate[i % n];
                *new_widths.get_mut(aid).unwrap() += 1;
            }
        } else if leftover < 0 {
            let mut by_rate: Vec<&str> = agents.iter().map(|s| s.as_str()).collect();
            by_rate.sort_by(|a, b| {
                rates.get(a).unwrap_or(&0).cmp(rates.get(b).unwrap_or(&0))
            });
            for i in 0..(-leftover) as usize {
                let aid = by_rate[i % n];
                if let Some(w) = new_widths.get_mut(aid) {
                    if *w > MIN_AGENT_COLS {
                        *w -= 1;
                    }
                }
            }
        }

        // Apply new widths
        for aid in &agents {
            if let Some(&w) = new_widths.get(aid.as_str()) {
                self.rain.resize_agent(aid, w, self.rain_height);
            }
        }
    }

    // ── Tool status update (row 2 per agent) ──

    pub fn update_tool_status(&mut self) {
        let now = Instant::now();
        let agents: Vec<String> = self.agents.iter().cloned().collect();

        for aid in &agents {
            if self.agent_stopped.contains_key(aid) {
                self.rain
                    .set_agent_status(aid, "END".to_string(), "red".to_string());
                continue;
            }

            if let Some((ref tool_name, ref tool_time)) = self.agent_last_tool.get(aid) {
                if now.duration_since(*tool_time) < Duration::from_secs(10) {
                    let style = palette_style_name(config::tool_palette(tool_name));
                    self.rain
                        .set_agent_status(aid, tool_name.clone(), style);
                    continue;
                }
            }

            if let Some(last_seen) = self.agent_last_seen.get(aid) {
                if now.duration_since(*last_seen) < Duration::from_secs(20) {
                    self.rain
                        .set_agent_status(aid, ">>>".to_string(), "green".to_string());
                    continue;
                }
            }

            self.rain
                .set_agent_status(aid, "idle".to_string(), "dim".to_string());
        }
    }

    // ── Reap dead agents ──

    pub fn reap_dead_agents(&mut self) {
        let now = Instant::now();
        let mut to_remove: Vec<String> = Vec::new();

        for agent_id in self.agents.iter() {
            // Agent sent Stop/SessionEnd -- remove after grace period
            if let Some(stopped_at) = self.agent_stopped.get(agent_id) {
                if now.duration_since(*stopped_at) >= AGENT_STOP_GRACE {
                    to_remove.push(agent_id.clone());
                }
                continue;
            }

            // Agent hasn't sent events recently -- stale
            if let Some(last_seen) = self.agent_last_seen.get(agent_id) {
                if now.duration_since(*last_seen) >= AGENT_STALE_TIMEOUT {
                    to_remove.push(agent_id.clone());
                }
            }
        }

        for agent_id in &to_remove {
            self.agents.remove(agent_id);
            self.agent_last_seen.remove(agent_id);
            self.agent_stopped.remove(agent_id);
            self.agent_event_times.remove(agent_id);
            self.agent_last_tool.remove(agent_id);
            self.agent_status_expires.remove(agent_id);
            self.rain.remove_agent(agent_id);
            // Tell haiku worker to forget this agent was named,
            // so naming fires again if the agent becomes active later.
            let _ = self.haiku_tx.send(HaikuRequest {
                agent_id: agent_id.clone(),
                needs_name: false,
                name_context: String::new(),
                status_context: String::new(),
                forget: true,
            });
        }
    }

    // ── Haiku refresh ──

    pub fn trigger_haiku_refresh(&mut self) {
        if self.agents.is_empty() {
            return;
        }

        let now = Instant::now();

        for agent_id in self.agents.iter() {
            let needs_name = !self.rain.agent_names.contains_key(agent_id);

            // Determine if status refresh is eligible.
            // Stopped, stale, or TTL-unexpired agents skip status — but
            // naming must still be attempted so short-lived agents get named.
            let skip_status = self.agent_stopped.contains_key(agent_id)
                || self
                    .agent_last_seen
                    .get(agent_id)
                    .map_or(true, |ls| now.duration_since(*ls) > Duration::from_secs(20))
                || self
                    .agent_status_expires
                    .get(agent_id)
                    .map_or(false, |exp| now < *exp);

            if skip_status && !needs_name {
                continue;
            }

            // Build context from recent events for this agent
            let agent_events: Vec<&HookEvent> = self
                .events
                .iter()
                .filter(|e| e.agent_id() == *agent_id)
                .collect();
            let recent: Vec<&HookEvent> = agent_events.iter().rev().take(12).copied().collect();

            if recent.is_empty() {
                continue;
            }

            // Build name context from first user prompt
            let name_context = if needs_name {
                agent_events
                    .iter()
                    .find(|e| e.hook_event_type == "UserPromptSubmit" && !e.summary.is_empty())
                    .or_else(|| agent_events.iter().find(|e| !e.summary.is_empty()))
                    .map(|e| {
                        let s = &e.summary;
                        if s.len() > 200 {
                            s[..200].to_string()
                        } else {
                            s.clone()
                        }
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            };

            // Build status context (empty if status refresh was skipped)
            let status_context = if skip_status {
                String::new()
            } else {
                recent
                    .iter()
                    .map(|e| {
                        let mut line = e.hook_event_type.clone();
                        if let Some(tool) = e.tool_name() {
                            line.push(':');
                            line.push_str(tool);
                        }
                        if !e.summary.is_empty() {
                            line.push(' ');
                            let summary = if e.summary.len() > 80 {
                                &e.summary[..80]
                            } else {
                                &e.summary
                            };
                            line.push_str(summary);
                        }
                        line
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            // Send request to haiku worker
            let _ = self.haiku_tx.send(HaikuRequest {
                agent_id: agent_id.clone(),
                needs_name,
                name_context,
                status_context,
                forget: false,
            });
        }
    }

    // ── Server management ──

    pub fn ensure_server(&mut self) {
        if self.server_proc.is_some() {
            return;
        }

        if server_is_up() {
            return;
        }

        // Start the server
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let server_dir = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join("apps/server"));

        if let Some(dir) = server_dir {
            if dir.is_dir() {
                self.server_proc = Some(ServerProcess::start(&dir));
            }
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(ref mut proc) = self.server_proc {
            proc.shutdown();
        }
        self.server_proc = None;
    }

    // ── Rate computation ──

    pub fn rate(&self) -> f32 {
        let now = Instant::now();
        let count = self
            .rate_timestamps
            .iter()
            .filter(|t| now.duration_since(**t) < RATE_WINDOW)
            .count();
        if count > 0 {
            count as f32 / RATE_WINDOW.as_secs_f32()
        } else {
            0.0
        }
    }

    // ── Filtered events ──

    pub fn filtered_events(&self) -> Vec<&HookEvent> {
        if self.filter_text.is_empty() {
            return self.events.iter().collect();
        }
        let ft = self.filter_text.to_lowercase();
        self.events
            .iter()
            .filter(|e| {
                let searchable = format!(
                    "{} {} {}",
                    e.agent_id(),
                    e.hook_event_type,
                    e.summary
                )
                .to_lowercase();
                searchable.contains(&ft)
            })
            .collect()
    }

    // ── Resize handling ──

    // ── UI actions (called from input.rs) ──

    pub fn scroll_down(&mut self) {
        self.scroll_mode = false;
        let count = self.filtered_events().len();
        if count == 0 { return; }
        let idx = match self.selected_idx {
            Some(i) => (i + 1).min(count - 1),
            None => 0,
        };
        self.selected_idx = Some(idx);
        self.list_state.select(Some(idx));
    }

    pub fn scroll_up(&mut self) {
        self.scroll_mode = false;
        let count = self.filtered_events().len();
        if count == 0 { return; }
        let idx = match self.selected_idx {
            Some(i) => i.saturating_sub(1),
            None => count.saturating_sub(1),
        };
        self.selected_idx = Some(idx);
        self.list_state.select(Some(idx));
    }

    pub fn toggle_scroll(&mut self) {
        self.scroll_mode = !self.scroll_mode;
        if self.scroll_mode {
            self.selected_idx = Some(0);
            self.list_state.select(Some(0));
        }
    }

    pub fn clamp_selection(&mut self) {
        let count = self.filtered_events().len();
        if let Some(idx) = self.selected_idx {
            if count == 0 {
                self.selected_idx = None;
                self.list_state.select(None);
            } else if idx >= count {
                self.selected_idx = Some(count - 1);
                self.list_state.select(Some(count - 1));
            }
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        // Clear selection so auto-scroll-to-bottom resumes
        self.selected_idx = None;
        self.list_state.select(None);
    }

    pub fn scroll_to_top(&mut self) {
        let count = self.filtered_events().len();
        if count > 0 {
            self.selected_idx = Some(0);
            self.list_state.select(Some(0));
        }
    }

    pub fn show_detail(&mut self) {
        let filtered = self.filtered_events();
        if let Some(idx) = self.selected_idx {
            if let Some(ev) = filtered.get(idx) {
                self.detail_event = Some((*ev).clone());
                self.detail_visible = true;
            }
        } else if let Some(ev) = filtered.last() {
            self.detail_event = Some((*ev).clone());
            self.detail_visible = true;
        }
    }

    pub fn close_overlay(&mut self) {
        if self.filter_visible {
            self.filter_visible = false;
        } else if self.detail_visible {
            self.detail_visible = false;
        }
    }

    pub fn on_resize(&mut self, width: u16, height: u16) {
        let rain_w = (width as usize).max(2);
        let rain_h = (height as usize).max(5);
        self.rain_width = rain_w;
        self.rain_height = rain_h;
        self.rain.on_resize(rain_w, rain_h);
    }
}

// ── Helpers ──

/// Convert `StyledSpans` (highlight output) into `ColoredChars` (rain input).
fn spans_to_colored_chars(spans: &[(String, Palette)]) -> ColoredChars {
    spans
        .iter()
        .flat_map(|(text, palette)| text.chars().map(move |ch| (ch, *palette)))
        .collect()
}

/// Map a Palette variant to a style name string (used by status display).
fn palette_style_name(palette: Palette) -> String {
    match palette {
        Palette::Cyan => "cyan".to_string(),
        Palette::Yellow => "yellow".to_string(),
        Palette::Magenta => "magenta".to_string(),
        Palette::Orange => "orange".to_string(),
        Palette::White => "white".to_string(),
        Palette::Green | Palette::None => "green".to_string(),
    }
}
