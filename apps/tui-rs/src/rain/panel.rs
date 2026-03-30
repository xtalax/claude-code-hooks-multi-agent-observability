use std::collections::HashMap;

use rand::seq::SliceRandom;

use crate::config::DEFAULT_AGENT_COLS;

use super::column::{make_rain_column, RainColumn};
use super::palette::ColoredChars;

/// Digital rain panel: each agent gets a section of green rain columns.
/// Tool calls take over individual columns in their own colour.
pub struct RainPanel {
    pub agent_order: Vec<String>,
    pub agent_rain: HashMap<String, Vec<RainColumn>>,
    pub agent_widths: HashMap<String, usize>,
    pub agent_names: HashMap<String, String>,
    /// agent_id -> (text, style_name)
    pub agent_status: HashMap<String, (String, String)>,
    /// agent_id -> (display_text, style_name) from haiku
    pub agent_haiku_status: HashMap<String, (String, String)>,
    /// agent_id -> full untruncated status for tooltip
    pub agent_full_status: HashMap<String, String>,
    pub ambient_rain: Vec<RainColumn>,
    pub hovered_agent: Option<String>,
    pub hovered_col_x: i32,
    pub hovered_event_id: Option<i64>,
}

impl RainPanel {
    pub fn new() -> Self {
        Self {
            agent_order: Vec::new(),
            agent_rain: HashMap::new(),
            agent_widths: HashMap::new(),
            agent_names: HashMap::new(),
            agent_status: HashMap::new(),
            agent_haiku_status: HashMap::new(),
            agent_full_status: HashMap::new(),
            ambient_rain: Vec::new(),
            hovered_agent: None,
            hovered_col_x: -1,
            hovered_event_id: None,
        }
    }

    /// Ensure an agent has rain columns allocated.
    pub fn ensure_agent(&mut self, agent_id: &str, height: usize) {
        if !self.agent_rain.contains_key(agent_id) {
            let w = DEFAULT_AGENT_COLS;
            let h = height.max(5);
            let cols: Vec<RainColumn> = (0..w).map(|_| make_rain_column(h)).collect();
            self.agent_rain.insert(agent_id.to_string(), cols);
            self.agent_widths.insert(agent_id.to_string(), w);
            self.agent_order.push(agent_id.to_string());
        }
    }

    /// Remove an agent's rain columns and all associated state.
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.agent_rain.remove(agent_id);
        self.agent_widths.remove(agent_id);
        self.agent_names.remove(agent_id);
        self.agent_status.remove(agent_id);
        self.agent_haiku_status.remove(agent_id);
        self.agent_full_status.remove(agent_id);
        self.agent_order.retain(|id| id != agent_id);
        if self.hovered_agent.as_deref() == Some(agent_id) {
            self.hovered_agent = None;
            self.hovered_col_x = -1;
            self.hovered_event_id = None;
        }
    }

    /// Grow or shrink an agent's rain column count. Skip if delta < 2.
    pub fn resize_agent(&mut self, agent_id: &str, new_width: usize, height: usize) {
        let cols = match self.agent_rain.get_mut(agent_id) {
            Some(c) => c,
            None => return,
        };
        let old_width = cols.len();
        if (new_width as isize - old_width as isize).unsigned_abs() < 2 {
            return;
        }
        let h = height.max(5);
        if new_width > old_width {
            for _ in 0..(new_width - old_width) {
                cols.push(make_rain_column(h));
            }
        } else {
            cols.truncate(new_width);
        }
        self.agent_widths.insert(agent_id.to_string(), new_width);
    }

    /// Set the status text for an agent.
    pub fn set_agent_status(&mut self, agent_id: &str, text: String, style: String) {
        self.agent_status
            .insert(agent_id.to_string(), (text, style));
    }

    /// Briefly accelerate a random sample of an agent's columns.
    pub fn pulse_agent(&mut self, agent_id: &str, height: usize) {
        self.ensure_agent(agent_id, height);
        if let Some(cols) = self.agent_rain.get_mut(agent_id) {
            let mut rng = rand::thread_rng();
            let count = cols.len().min(4);
            let indices: Vec<usize> = {
                let mut idx: Vec<usize> = (0..cols.len()).collect();
                idx.shuffle(&mut rng);
                idx.into_iter().take(count).collect()
            };
            for i in indices {
                cols[i].pulse();
            }
        }
    }

    /// Spread tool text across the agent's columns in word-sized chunks.
    pub fn inject_tool(
        &mut self,
        agent_id: &str,
        colored_chars: &ColoredChars,
        event_id: Option<i64>,
        height: usize,
    ) {
        self.ensure_agent(agent_id, height);
        if let Some(cols) = self.agent_rain.get_mut(agent_id) {
            let n = cols.len();
            if n == 0 || colored_chars.is_empty() {
                return;
            }

            // Split colored_chars into chunks on space boundaries
            let mut chunks: Vec<ColoredChars> = Vec::new();
            let mut current: ColoredChars = Vec::new();
            for &(ch, palette) in colored_chars {
                if ch == ' ' && !current.is_empty() {
                    chunks.push(std::mem::take(&mut current));
                } else if ch != ' ' {
                    current.push((ch, palette));
                }
            }
            if !current.is_empty() {
                chunks.push(current);
            }

            if chunks.is_empty() {
                return;
            }

            // Assign each chunk to a random column, preferring less-busy ones
            let mut rng = rand::thread_rng();
            let mut indices: Vec<usize> = (0..n).collect();
            indices.shuffle(&mut rng);

            // Sort by busyness so least-busy columns get text first
            indices.sort_by_key(|&i| cols[i].is_busy());

            for (ci, chunk) in chunks.iter().enumerate() {
                let col_idx = indices[ci % n];
                cols[col_idx].inject_text(chunk, event_id);
            }
        }
    }

    /// Return (agent_id, local_col_index) for a given screen x coordinate.
    pub fn agent_at_x(&self, x: usize) -> (Option<&str>, usize) {
        let mut offset = 0usize;
        for agent_id in &self.agent_order {
            let w = *self.agent_widths.get(agent_id.as_str()).unwrap_or(&DEFAULT_AGENT_COLS);
            if x < offset + w {
                return (Some(agent_id.as_str()), x - offset);
            }
            offset += w;
        }
        (None, 0)
    }

    /// Tick all rain columns (both agent and ambient).
    pub fn tick(&mut self) {
        if !self.agent_rain.is_empty() {
            for cols in self.agent_rain.values_mut() {
                for col in cols.iter_mut() {
                    col.tick();
                }
            }
        } else {
            for col in self.ambient_rain.iter_mut() {
                col.tick();
            }
        }
    }

    /// Highlight the rain column holding a given event (reverse lookup from stream).
    /// Sets hovered_col_x to the screen x of the matching column, or -1 if not found.
    pub fn highlight_event(&mut self, event_id: Option<i64>) {
        if event_id.is_none() {
            self.hovered_col_x = -1;
            return;
        }
        let target = event_id.unwrap();
        let mut offset = 0usize;
        for agent_id in &self.agent_order {
            let cols = match self.agent_rain.get(agent_id.as_str()) {
                Some(c) => c,
                None => continue,
            };
            for (ci, col) in cols.iter().enumerate() {
                if col.current_event_id() == Some(target) {
                    self.hovered_col_x = (offset + ci) as i32;
                    return;
                }
            }
            offset += *self.agent_widths.get(agent_id.as_str()).unwrap_or(&DEFAULT_AGENT_COLS);
        }
        // No column found -- clear highlight
        self.hovered_col_x = -1;
    }

    /// Initialize ambient rain columns to fill a given width and height.
    pub fn init_ambient(&mut self, width: usize, height: usize) {
        let h = height.max(5);
        self.ambient_rain = (0..width.max(2)).map(|_| make_rain_column(h)).collect();
    }

    /// Resize all columns (agent + ambient) after a terminal resize.
    pub fn on_resize(&mut self, width: usize, height: usize) {
        let h = height.max(5);
        for cols in self.agent_rain.values_mut() {
            for col in cols.iter_mut() {
                col.resize(h);
            }
        }
        // Grow/shrink ambient columns to match width (preserves existing rain state)
        while self.ambient_rain.len() < width {
            self.ambient_rain.push(make_rain_column(h));
        }
        if self.ambient_rain.len() > width {
            self.ambient_rain.truncate(width);
        }
        for col in self.ambient_rain.iter_mut() {
            col.resize(h);
        }
    }
}
