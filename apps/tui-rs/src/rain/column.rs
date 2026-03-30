use std::collections::VecDeque;

use rand::Rng;

use crate::config::{Palette, KATAKANA, LATIN};

use super::palette::ColoredChars;

/// Lazily-built combined rain character set (Katakana + Latin).
fn rain_chars() -> &'static [char] {
    use std::sync::OnceLock;
    static CHARS: OnceLock<Vec<char>> = OnceLock::new();
    CHARS.get_or_init(|| {
        KATAKANA.chars().chain(LATIN.chars()).collect()
    })
}

fn random_rain_char() -> char {
    let chars = rain_chars();
    let idx = rand::thread_rng().gen_range(0..chars.len());
    chars[idx]
}

/// One 1-char-wide rain stream. Can be taken over by tool text in colour.
pub struct RainColumn {
    pub chars: Vec<char>,
    pub brights: Vec<f32>,
    pub colors: Vec<Palette>,
    pub head: usize,
    pub speed: f32,
    pub accel_ticks: u32,
    pub height: usize,
    text_queue: VecDeque<(char, Palette)>,
    current_event_id: Option<i64>,
}

impl RainColumn {
    /// How busy this column is (queue length + coloured cells). Lower = freer.
    pub fn is_busy(&self) -> usize {
        self.text_queue.len() + self.colors.iter().filter(|c| **c != Palette::None).count()
    }

    /// Current event id being displayed, if any.
    pub fn current_event_id(&self) -> Option<i64> {
        self.current_event_id
    }

    /// Advance the rain column by one tick.
    pub fn tick(&mut self) {
        let mut rng = rand::thread_rng();

        // Occasional speed drift (5% chance per tick, stay within 0.3–1.8)
        if rng.gen::<f32>() < 0.05 {
            self.speed += rng.gen_range(-0.15..0.15);
            self.speed = self.speed.clamp(0.3, 1.8);
        }

        let effective_speed = self.speed * if self.accel_ticks > 0 { 2.5 } else { 1.0 };
        if self.accel_ticks > 0 {
            self.accel_ticks -= 1;
        }

        let is_printing = !self.text_queue.is_empty();

        if rand::thread_rng().gen::<f32>() < effective_speed * 0.45 {
            let next_head = (self.head + 1) % self.height;

            if let Some((ch, palette)) = self.text_queue.pop_front() {
                self.head = next_head;
                self.chars[self.head] = ch;
                self.colors[self.head] = palette;
                self.brights[self.head] = 1.0;
            } else {
                self.head = next_head;
                self.chars[self.head] = random_rain_char();
                self.colors[self.head] = Palette::None;
                self.brights[self.head] = 1.0;
            }
        }

        // Decay brightness for all non-head cells
        for i in 0..self.height {
            if i != self.head {
                if is_printing && self.colors[i] != Palette::None {
                    // Tool text stays bright while command is still printing
                    self.brights[i] = (self.brights[i] - 0.005).max(0.5);
                } else if self.colors[i] != Palette::None {
                    // Tool text fades out slowly (4x slower than rain)
                    self.brights[i] = (self.brights[i] - 0.0175).max(0.0);
                    if self.brights[i] == 0.0 {
                        self.colors[i] = Palette::None;
                    }
                } else {
                    self.brights[i] = (self.brights[i] - 0.07).max(0.0);
                }
            }
        }

        // Clear event reference once all tool text has fully faded
        if self.current_event_id.is_some()
            && self.text_queue.is_empty()
            && !self.colors.iter().any(|c| *c != Palette::None)
        {
            self.current_event_id = None;
        }
    }

    /// Take over this column with pre-colored text.
    pub fn inject_text(&mut self, colored_chars: &ColoredChars, event_id: Option<i64>) {
        self.text_queue.clear();
        for &(ch, palette) in colored_chars {
            let ch = if ch.is_whitespace() && ch != ' ' { ' ' } else if ch == ' ' { ' ' } else { ch };
            let palette = if palette == Palette::None { Palette::Cyan } else { palette };
            self.text_queue.push_back((ch, palette));
        }
        self.accel_ticks = self.accel_ticks.max(12);
        self.current_event_id = event_id;
    }

    /// Briefly accelerate this column.
    pub fn pulse(&mut self) {
        self.accel_ticks = 15;
    }

    /// Grow or shrink to new height.
    pub fn resize(&mut self, h: usize) {
        if h == self.height {
            return;
        }
        let old = self.height;
        self.height = h;
        if h > old {
            let extra = h - old;
            for _ in 0..extra {
                self.chars.push(random_rain_char());
                self.brights.push(0.0);
                self.colors.push(Palette::None);
            }
        } else {
            self.chars.truncate(h);
            self.brights.truncate(h);
            self.colors.truncate(h);
        }
        self.head = self.head % h.max(1);
    }
}

/// Create a new rain column with random initial state.
pub fn make_rain_column(height: usize) -> RainColumn {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = (0..height).map(|_| random_rain_char()).collect();
    let brights: Vec<f32> = (0..height).map(|_| rng.gen::<f32>() * 0.3).collect();
    let colors: Vec<Palette> = vec![Palette::None; height];
    let head = if height > 0 { rng.gen_range(0..height) } else { 0 };

    // Randomize speed: 0.4–1.6 so columns fall at different rates
    let speed = 0.4 + rng.gen::<f32>() * 1.2;

    RainColumn {
        chars,
        brights,
        colors,
        head,
        speed,
        accel_ticks: 0,
        height,
        text_queue: VecDeque::new(),
        current_event_id: None,
    }
}
