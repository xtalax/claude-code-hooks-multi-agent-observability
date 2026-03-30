# Rust TUI Rewrite — Battle Plan

> **STANDING ORDER: Keep this plan current.**
> Before starting any task, read this file. After completing a task, update its status.
> If the Python TUI (`apps/tui/matrix_tui.py`) changes, diff it against the
> "Source of Truth" section below and update affected tasks/decisions.
> If an architectural decision changes during implementation, update the decision
> record here with the new rationale — do not leave stale decisions in the plan.

## Keeping Intel Current

1. **Before each work session:** Read `apps/tui/matrix_tui.py` and diff against the snapshot below. Note any new features, removed code, or changed behavior. Update affected tasks.
2. **After completing a task:** Mark it `done` in the tracker, note the commit, and update dependent tasks if scope changed.
3. **When Python TUI changes:** Run `wc -l apps/tui/matrix_tui.py` and compare to the snapshot line count. If it differs significantly, re-audit the module structure and task list.
4. **When adding a crate:** Add it to the dependency table with rationale. Check it won't conflict with existing crates.
5. **When an architectural decision is revised:** Strikethrough the old decision, add the new one with date and reason.

### Source of Truth Snapshot

| File | Lines | Last audited |
|------|-------|-------------|
| `apps/tui/matrix_tui.py` | ~1,877 | 2026-03-18 |

**Key Python features at time of snapshot:**
- 3-row rain header (name, tool indicator, Haiku status)
- Dynamic per-agent column widths (sqrt-dampened traffic bonus, 50/50 equal+traffic split)
- Haiku API integration (parallel queue, semaphore(4), name + status generation)
- Agent lifecycle (60s stale timeout, 5s stop grace, periodic reap)
- Omarchy theme loading with Kanagawa fallback
- Syntax highlighting (bash, JSON, path, multi-language keywords)
- WebSocket streaming with auto-reconnect and server auto-start
- dotenv loading from project root
- Tooltip on hover showing full Haiku status
- Tool-colored summaries in event stream

---

## Sailing Orders

**Outcome:** Reimplement the Matrix TUI observability dashboard in Rust with feature parity to the Python/Textual version, gaining native render performance.

**Success metric:** Rust binary runs all features — websocket streaming, rain visualization, agent lifecycle, Haiku API, theme loading, syntax highlighting, keyboard/mouse — at 60+ FPS with <5% CPU.

**Constraints:**
- Prefer mature crates (ratatui, tokio, tokio-tungstenite)
- Preserve omarchy theme loading + Kanagawa fallback
- Wire-compatible with existing Bun server (same WS protocol, same JSON)
- Do not rewrite the server or web client

**Out of scope:** Server changes, web client, new features not in Python version, CI/CD.

---

## Module Structure

```
apps/tui-rs/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry, tokio runtime, terminal setup, main loop
│   ├── app.rs               # App state, event dispatch, tick loop, agent lifecycle
│   ├── config.rs            # Theme loading (.toml), constants, .env, color palettes
│   ├── event.rs             # HookEvent struct, serde deserialization, agent_id
│   ├── net/
│   │   ├── mod.rs
│   │   ├── websocket.rs     # WS connect, reconnect backoff, JSON parsing
│   │   ├── health.rs        # HTTP health check (GET /health)
│   │   └── server.rs        # Subprocess spawn/kill for bun dev server
│   ├── ai/
│   │   ├── mod.rs
│   │   └── haiku.rs         # Anthropic API, name + status gen, queue + semaphore
│   ├── rain/
│   │   ├── mod.rs
│   │   ├── column.rs        # RainColumn: tick, inject_text, pulse, resize, decay
│   │   ├── panel.rs         # Agent layout, widths, ensure/remove/resize, hit test
│   │   └── palette.rs       # 7-step shades, tool palettes, brightness_to_color
│   ├── highlight/
│   │   ├── mod.rs
│   │   ├── bash.rs          # Bash command/flag/string/operator highlighting
│   │   ├── languages.rs     # Keyword sets: Julia/Python/JS/Java/C/C++/C#/Ruby/Perl
│   │   ├── json.rs          # JSON payload highlighting
│   │   └── path.rs          # File path + glob pattern highlighting
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── stats_bar.rs     # Top bar: events, agents, rate, connection status
│   │   ├── event_stream.rs  # Scrollable event list, filtering, format_event
│   │   ├── event_detail.rs  # Full payload panel with syntax-highlighted JSON
│   │   ├── filter_bar.rs    # Search input overlay
│   │   └── rain_widget.rs   # ratatui::Widget impl for RainPanel render
│   └── input.rs             # Crossterm key/mouse events → app actions
```

## Crate Dependencies

| Crate | Purpose | Replaces | Status |
|-------|---------|----------|--------|
| `ratatui` | TUI framework, widgets, layout | textual + rich | done |
| `crossterm` | Terminal backend, raw mode, events | textual.events | done |
| `tokio` | Async runtime, timers, tasks, mpsc channels | asyncio | done |
| `tokio-tungstenite` | WebSocket client | websockets | done |
| `serde` + `serde_json` | JSON (de)serialization | json | done |
| `reqwest` | HTTP (health check + Anthropic API) | urllib.request + anthropic | done |
| `toml` | Omarchy theme parsing | manual TOML parse | done |
| `dotenvy` | .env loading | python-dotenv | done |
| `rand` | RNG for rain chars, sampling | random | done |
| `unicode-width` | Katakana width handling | implicit in Python | done |

## Task Tracker

> Update status as you go: `pending` → `in_progress` → `done`
> Add commit hash when completing a task.

| # | Task | Files | Depends | Status | Commit |
|---|------|-------|---------|--------|--------|
| T1 | Scaffold project | `Cargo.toml`, all `mod.rs` | — | done | |
| T2 | Config + theme | `config.rs` | T1 | done | |
| T3 | HookEvent model | `event.rs` | T1 | done | |
| T4 | Rain column core | `rain/column.rs`, `rain/palette.rs` | T2 | done | |
| T5 | Rain panel layout | `rain/panel.rs` | T3, T4 | done | |
| T6 | Rain widget render | `widgets/rain_widget.rs` | T5 | done | |
| T7 | WebSocket streaming | `net/websocket.rs` | T3 | done | |
| T8 | Health + server mgmt | `net/health.rs`, `net/server.rs` | T1 | done | |
| T9 | App state + dispatch | `app.rs` | T5, T6, T7 | done | |
| T10 | Event stream widget | `widgets/event_stream.rs` | T3 | done | |
| T11 | Stats bar widget | `widgets/stats_bar.rs` | T9 | done | |
| T12 | Event detail widget | `widgets/event_detail.rs` | T3 | done | |
| T13 | Filter bar widget | `widgets/filter_bar.rs` | T10 | done | |
| T14 | Input handling | `input.rs` | T9 | done | |
| T15 | Syntax highlighting | `highlight/*` | T2 | done | |
| T16 | Haiku AI integration | `ai/haiku.rs` | T3, T9 | done | |
| T17 | Main + layout | `main.rs` | All | done | |

## Critical Path

```
T1 → T2 → T4 → T5 → T6 ─┐
T1 → T3 → T7 ────────────┼→ T9 → T14 → T17
T1 → T8 ─────────────────┘
T3 → T10 → T13 ──────────→ T17
T2 → T15 ────────────────→ T17
T3 + T9 → T16 ───────────→ T17
```

**Parallel groups:**
- T2, T3, T8 (after T1)
- T4 + T7 (after T2/T3)
- T10, T11, T12, T15 (independent widgets)
- T16 (after T9)

## Architectural Decisions

> When revising a decision, don't delete the old one — strikethrough it and add the
> replacement below with the date and reason for the change.

### ADR-1: Render Loop
Crossterm raw mode, fixed ~65ms tick (matching Python 0.065s). Ratatui `Frame::render_widget()` per panel. Rain render writes directly to `Buffer` — no per-char heap allocation.

### ADR-2: State Ownership
Single `App` struct owns all state. No `Rc`/`Arc` — tick loop is single-threaded. WebSocket + Haiku workers communicate via `tokio::sync::mpsc` channels.

### ADR-3: Rain Column
`Vec<RainColumn>` per agent. Same tick/decay algorithm. Use fixed-size arrays or `SmallVec` for char/brightness/color arrays to keep columns cache-friendly.

```rust
struct RainColumn {
    chars: Vec<char>,       // KATAKANA + LATIN
    brights: Vec<f32>,      // 0.0–1.0 brightness per cell
    colors: Vec<Palette>,   // enum: Green, Cyan, Yellow, Magenta, Orange, White, None
    head: usize,
    speed: f32,
    accel_ticks: u32,
    height: usize,
    text_queue: VecDeque<(char, Palette)>,
    current_event: Option<i64>,  // event ID, not full struct
}
```

### ADR-4: Event Stream
`VecDeque<HookEvent>` with 500-line cap. `HashMap<i64, usize>` for event ID → display index reverse lookup. O(1) push/pop at both ends.

### ADR-5: Syntax Highlighting
Return `Vec<(String, Style)>` styled spans. Palette as enum:
```rust
enum Palette { Green, Cyan, Yellow, Magenta, Orange, White }
```
Compile-time validity. Each highlighter (bash, json, path) returns the same type.

### ADR-6: Haiku Queue
```rust
// In App
haiku_tx: mpsc::UnboundedSender<String>,  // agent_id

// Background task
async fn haiku_worker(rx, anthropic_client, result_tx) {
    let sem = Arc::new(Semaphore::new(4));
    while let Some(agent_id) = rx.recv().await {
        let permit = sem.clone().acquire_owned().await;
        tokio::spawn(async move {
            // generate name + status via reqwest POST
            // send result back on result_tx
            drop(permit);
        });
    }
}
```

### ADR-7: Agent Lifecycle
Same as Python:
- `agent_last_seen: HashMap<String, Instant>` — monotonic
- Reap after 60s idle or 5s after Stop/SessionEnd
- Rebalance every 2s: `base_share * 0.5 + traffic_share * 0.5` with sqrt dampening
- Resize only if delta >= 2 columns

### ADR-8: Theme System
```rust
struct Theme {
    bg: Color, fg: Color, accent: Color, cursor: Color, sel_bg: Color,
    black: Color, red: Color, green: Color, yellow: Color,
    blue: Color, magenta: Color, cyan: Color, white: Color,
    br_black: Color, br_red: Color, br_green: Color, br_yellow: Color,
    br_blue: Color, br_magenta: Color, br_cyan: Color, br_white: Color,
    orange: Color, hover_bg: Color,
}

impl Theme {
    fn load() -> Self {
        // Try ~/.config/omarchy/current/theme/colors.toml
        // Fall back to Kanagawa defaults
    }
    fn make_shades(bright: Color, base: Color, bg: Color, fg: Color) -> [Color; 7] { ... }
}
```

## Python to Rust Mapping

| Python | Rust |
|--------|------|
| `@dataclass HookEvent` | `#[derive(Deserialize)] struct HookEvent` |
| `dict[str, list[RainColumn]]` | `HashMap<String, Vec<RainColumn>>` |
| `set[str]` | `HashSet<String>` |
| `time.monotonic()` | `std::time::Instant::now()` |
| `asyncio.Queue` | `tokio::sync::mpsc::unbounded_channel` |
| `asyncio.Semaphore` | `tokio::sync::Semaphore` |
| `@work(exclusive=True)` | `tokio::spawn` + channel |
| `self.set_interval(0.065, tick)` | `tokio::time::interval(Duration::from_millis(65))` |
| `random.choice(RAIN_CHARS)` | `rand::thread_rng().gen_range()` |
| `Text.append(ch, style=...)` | `buf.set_string(x, y, ch, style)` |
| `Rich Style.parse(spec)` | `ratatui::style::Style::default().fg(color)` |
| `@lru_cache _style()` | Not needed — Style is Copy in ratatui |
| `subprocess.Popen` | `tokio::process::Command` |
| `os.killpg(SIGTERM)` | `nix::sys::signal::killpg` or libc |

## Estimated Scope

| Area | Est. Lines (Rust) | Notes |
|------|-------------------|-------|
| Config + theme | ~200 | TOML parse, color math, constants |
| Event model | ~80 | Serde struct, agent_id |
| Rain core | ~250 | Column tick, palette, panel layout |
| Rain widget | ~200 | Ratatui render impl |
| Networking | ~200 | WS + health + server mgmt |
| Haiku AI | ~150 | reqwest + queue + semaphore |
| Widgets | ~300 | Stats, stream, detail, filter |
| Highlighting | ~400 | Bash, JSON, path, languages |
| Input | ~100 | Key/mouse dispatch |
| App + main | ~300 | State, dispatch, lifecycle, layout |
| **Total** | **~2,200** | Comparable to Python (1,877) |

## Known Risks

| Risk | Mitigation |
|------|-----------|
| `ratatui` mouse support less mature than Textual | Custom hit testing via `_agent_at_x` equivalent; test early in T6 |
| Katakana double-width in some terminals | `unicode-width` crate; test with multiple terminal emulators |
| No reactive properties (Textual feature) | Always-render approach; ratatui diffs frames automatically |
| Anthropic SDK has no official Rust crate | Use raw `reqwest` POST to messages API; straightforward |

## Change Log

> Record significant plan changes here so future sessions understand what shifted and why.

| Date | Change | Reason |
|------|--------|--------|
| 2026-03-18 | Initial plan created | — |
