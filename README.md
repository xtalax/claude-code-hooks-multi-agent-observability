# Multi-Agent Observability System

Real-time monitoring and visualization for Claude Code agents through comprehensive hook event tracking. Watch the [latest deep dive on multi-agent orchestration with Opus 4.6 here](https://youtu.be/RpUTF_U4kiw). With Claude Opus 4.6 and multi-agent orchestration, you can now spin up teams of specialized agents that work in parallel, and this observability system lets you trace every tool call, task handoff, and agent lifecycle event across the entire swarm.

## 🎯 Overview

This system provides complete observability into Claude Code agent behavior by capturing, storing, and visualizing Claude Code [Hook events](https://docs.anthropic.com/en/docs/claude-code/hooks) in real-time. It enables monitoring of multiple concurrent agents with session tracking, event filtering, and live updates. 

<img src="images/app.png" alt="Multi-Agent Observability Dashboard" style="max-width: 800px; width: 100%;">

## 🏗️ Architecture

```
Claude Agents → Hook Scripts → HTTP POST → Bun Server → SQLite → WebSocket → Matrix TUI (Rust)
```

![Agent Data Flow Animation](images/AgentDataFlowV2.gif)

## 📋 Prerequisites

| Tool | Required for | Install |
|------|-------------|---------|
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code) | Everything | `npm install -g @anthropic-ai/claude-code` |
| [Astral uv](https://docs.astral.sh/uv/) | Hooks & status line | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| [Bun](https://bun.sh/) | Event server | `curl -fsSL https://bun.sh/install \| bash` |
| [Rust](https://rustup.rs/) | Matrix TUI | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| [just](https://github.com/casey/just) | Recipe runner (optional) | `cargo install just` |

## 🚀 Full Stack Install (Server + TUI + Hooks)

```bash
# 1. Clone the repo
git clone https://github.com/xtalax/claude-code-hooks-multi-agent-observability.git matrix-observability
cd matrix-observability

# 2. Set up environment variables
cp .env.sample .env
# Edit .env — at minimum set ANTHROPIC_API_KEY
#   ANTHROPIC_API_KEY=sk-ant-...   (required — powers hook summaries + TUI agent naming)
#   ENGINEER_NAME=YourName          (optional — personalizes TTS announcements)
#   ELEVENLABS_API_KEY=...          (optional — high-quality TTS)
#   OPENAI_API_KEY=...              (optional — fallback TTS)

# 3. Install dependencies
just install          # or: cd apps/server && bun install && cd ../tui-rs && cargo build --release

# 4. Start the event server (port 4000)
just start            # or: ./scripts/start-system.sh

# 5. Launch the Matrix TUI
just tui              # or: cargo run --release --manifest-path apps/tui-rs/Cargo.toml

# 6. Open Claude Code in any project with the hooks installed and start working — events stream in real-time
```

## 🔌 Install the Status Line Only

The status line shows context window usage, git branch, and model info directly in Claude Code's footer. It works standalone — no server needed.

### Step 1: Copy the status line script

```bash
# From the matrix-observability repo root:
mkdir -p /path/to/your/project/.claude/status_lines
cp .claude/status_lines/status_line_v6.py /path/to/your/project/.claude/status_lines/
```

### Step 2: Add to your project's `.claude/settings.json`

If you don't have a `settings.json` yet, create one at `.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "uv run $CLAUDE_PROJECT_DIR/.claude/status_lines/status_line_v6.py",
    "padding": 0
  }
}
```

### Step 3: Verify

Restart Claude Code in your project. The status line appears at the bottom:

```
 ᛝ Claude Opus 4.6 ─  main +3 -1 ± ─ ████████░░ 42% of 1.0M ─ ~580k left ─ a1b2c3d4
```

**Theme support**: If you use [omarchy](https://github.com/ArcadeLabsInc/omarchy), colors are loaded from `~/.config/omarchy/current/theme/colors.toml`. Otherwise a built-in Gruvbox palette is used — no config needed.

**Dependencies**: Only `python-dotenv` (resolved automatically by `uv run`).

## 🪝 Install the Hooks Only (Local Logging)

The hooks run standalone and log events to a local `logs/` directory. No server required — useful for auditing tool usage, blocking dangerous commands, and tracking session history.

### Step 1: Copy the hooks and utilities

```bash
# From the matrix-observability repo root:
cp -R .claude/hooks /path/to/your/project/.claude/hooks
```

### Step 2: Add hook configuration to `.claude/settings.json`

Merge this into your project's `.claude/settings.json` (create it if it doesn't exist):

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/pre_tool_use.py"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/post_tool_use.py"
          }
        ]
      }
    ],
    "Notification": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/notification.py"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/stop.py --chat"
          }
        ]
      }
    ],
    "SubagentStop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/subagent_stop.py"
          }
        ]
      }
    ],
    "SubagentStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/subagent_start.py"
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/pre_compact.py"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/user_prompt_submit.py --log-only --store-last-prompt --name-agent"
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/session_start.py"
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/session_end.py"
          }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/permission_request.py"
          }
        ]
      }
    ],
    "PostToolUseFailure": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/post_tool_use_failure.py"
          }
        ]
      }
    ]
  }
}
```

All 12 Claude Code hook events are covered. Each hook logs to `logs/{session_id}/` in your project directory.

### Step 3: Verify

```bash
# Restart Claude Code, run any command, then check for logs:
ls logs/
```

**What the hooks do**:

| Hook | Purpose |
|------|---------|
| `pre_tool_use.py` | Blocks dangerous `rm -rf` commands, prevents `.env` access, logs tool inputs |
| `post_tool_use.py` | Logs tool results with MCP tool detection |
| `post_tool_use_failure.py` | Captures tool execution failures |
| `notification.py` | Tracks notifications with optional TTS (ElevenLabs > OpenAI > pyttsx3) |
| `stop.py` | Records session completion with `--chat` to include transcripts |
| `subagent_start.py` | Tracks subagent spawns (agent_type, agent_id) |
| `subagent_stop.py` | Logs subagent completion with optional TTS |
| `pre_compact.py` | Tracks context compaction, creates transcript backups |
| `user_prompt_submit.py` | Logs user prompts, stores session data, names agents |
| `session_start.py` | Records session start (model, agent_type, source) |
| `session_end.py` | Records session end with reason (clear/logout/exit) |
| `permission_request.py` | Logs permission request events |

**Dependencies**: Python packages are declared inline via `uv` script headers — no `pip install` needed. `uv run` resolves them automatically on first use.

## 🔌 Add Server Observability to Any Project

To stream events from another project to the observability server and TUI, add `send_event.py` alongside the local hooks.

### Step 1: Copy the event sender

```bash
cp .claude/hooks/send_event.py /path/to/your/project/.claude/hooks/
cp -R .claude/hooks/utils /path/to/your/project/.claude/hooks/utils
```

### Step 2: Add `send_event.py` to each hook in `settings.json`

For each hook event, add a second command that sends the event to the server. Replace `YOUR_PROJECT_NAME` with a unique identifier (e.g., `my-api`, `frontend`):

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/pre_tool_use.py"
          },
          {
            "type": "command",
            "command": "uv run $CLAUDE_PROJECT_DIR/.claude/hooks/send_event.py --source-app YOUR_PROJECT_NAME --event-type PreToolUse --summarize"
          }
        ]
      }
    ]
  }
}
```

Apply the same pattern for all 12 event types. The `--summarize` flag uses Claude Haiku to generate one-line event summaries. The `--add-chat` flag (used on `Stop`) includes conversation transcripts.

### Step 3: Ensure the server is running

```bash
# From the matrix-observability directory
just start            # or: ./scripts/start-system.sh
```

The server listens on port 4000 by default (configurable via `SERVER_PORT` env var). Events from all projects stream into the same TUI.

## ⚡ Quick Reference: `just` Recipes

```bash
just              # List all available recipes
just start        # Start event server
just stop         # Stop all processes
just restart      # Stop then start
just server       # Start server only (dev mode)
just tui          # Launch Matrix TUI
just tui-build    # Build TUI (release)
just install      # Install all dependencies
just health       # Check server health
just test-event   # Send a test event
just db-reset     # Reset the database
just hooks        # List all hook scripts
```

## 📁 Project Structure

```
matrix-observability/
│
├── apps/                    # Application components
│   ├── server/             # Bun TypeScript server
│   │   ├── src/
│   │   │   ├── index.ts    # Main server with HTTP/WebSocket endpoints
│   │   │   ├── db.ts       # SQLite database management & migrations
│   │   │   └── types.ts    # TypeScript interfaces
│   │   ├── package.json
│   │   └── events.db       # SQLite database (gitignored)
│   │
│   └── tui-rs/             # Rust Matrix TUI (terminal dashboard)
│       ├── src/
│       │   ├── main.rs     # Entry point, terminal setup, main loop
│       │   ├── app.rs      # App state, event ingestion, haiku naming
│       │   ├── config.rs   # Theme loader (omarchy + fallback), palettes
│       │   ├── event.rs    # HookEvent struct
│       │   ├── ai/         # Haiku API worker (agent naming & status)
│       │   ├── net/        # WebSocket client, health check, server mgmt
│       │   ├── rain/       # Digital rain animation (columns, panels)
│       │   ├── widgets/    # Ratatui widgets (event stream, stats, rain)
│       │   └── highlight/  # Syntax highlighting (bash, JSON, paths)
│       └── Cargo.toml
│
├── .claude/                # Claude Code integration
│   ├── hooks/             # Hook scripts (Python with uv)
│   │   ├── send_event.py          # Universal event sender (all 12 event types)
│   │   ├── pre_tool_use.py        # Tool validation, blocking & summarization
│   │   ├── post_tool_use.py       # Result logging with MCP tool detection
│   │   ├── post_tool_use_failure.py # Tool failure logging
│   │   ├── permission_request.py  # Permission request logging
│   │   ├── notification.py        # User interaction events (type-aware TTS)
│   │   ├── user_prompt_submit.py  # User prompt logging & validation
│   │   ├── stop.py               # Session completion (stop_hook_active guard)
│   │   ├── subagent_stop.py      # Subagent completion with transcript path
│   │   ├── subagent_start.py     # Subagent lifecycle start tracking
│   │   ├── pre_compact.py        # Context compaction with custom instructions
│   │   ├── session_start.py      # Session start with agent type & model
│   │   ├── session_end.py        # Session end with reason tracking
│   │   └── validators/           # Stop hook validators
│   │       ├── validate_new_file.py     # Validate file creation
│   │       └── validate_file_contains.py # Validate file content sections
│   │
│   ├── agents/team/       # Agent team definitions
│   │   ├── builder.md     # Engineering agent with linting hooks
│   │   └── validator.md   # Read-only validation agent
│   │
│   ├── commands/          # Custom slash commands
│   │   └── plan_w_team.md # Team-based planning command
│   │
│   ├── status_lines/      # Status line scripts
│   │   └── status_line_v6.py # Context window usage display
│   │
│   └── settings.json      # Hook configuration (all 12 events)
│
├── justfile               # Task runner recipes (just start, just stop, etc.)
│
├── scripts/               # Utility scripts
│   ├── start-system.sh   # Launch event server
│   └── reset-system.sh   # Stop all processes
│
└── logs/                 # Application logs (gitignored)
```

## 🔧 Component Details

### 1. Hook System (`.claude/hooks/`)

> If you want to master claude code hooks watch [this video](https://github.com/disler/claude-code-hooks-mastery)

The hook system intercepts Claude Code lifecycle events:

- **`send_event.py`**: Core script that sends event data to the observability server
  - Supports all 12 hook event types with event-specific field forwarding
  - Supports `--add-chat` flag for including conversation history
  - Forwards event-specific fields (`tool_name`, `tool_use_id`, `agent_id`, `notification_type`, etc.) as top-level properties for easier querying
  - Validates server connectivity before sending

- **Event-specific hooks** (12 total): Each implements validation and data extraction
  - `pre_tool_use.py`: Blocks dangerous commands, validates tool usage, summarizes tool inputs per tool type
  - `post_tool_use.py`: Captures execution results with MCP tool detection (`mcp_server`, `mcp_tool_name`)
  - `post_tool_use_failure.py`: Logs tool execution failures
  - `permission_request.py`: Logs permission request events
  - `notification.py`: Tracks user interactions with `notification_type`-aware TTS (permission_prompt, idle_prompt, etc.)
  - `user_prompt_submit.py`: Logs user prompts, supports validation with JSON `{"decision": "block"}` pattern
  - `stop.py`: Records session completion with `stop_hook_active` guard to prevent infinite loops
  - `subagent_stop.py`: Monitors subagent task completion with transcript path tracking
  - `subagent_start.py`: Tracks subagent lifecycle start events
  - `pre_compact.py`: Tracks context compaction with custom instructions in backup filenames
  - `session_start.py`: Logs session start with `agent_type`, `model`, and `source` fields
  - `session_end.py`: Logs session end with reason tracking (including `bypass_permissions_disabled`)

### 2. Server (`apps/server/`)

Bun-powered TypeScript server with real-time capabilities:

- **Database**: SQLite with WAL mode for concurrent access
- **Endpoints**:
  - `POST /events` - Receive events from agents
  - `GET /events/recent` - Paginated event retrieval with filtering
  - `GET /events/filter-options` - Available filter values
  - `WS /stream` - Real-time event broadcasting
- **Features**:
  - Automatic schema migrations
  - Event validation
  - WebSocket broadcast to all clients
  - Chat transcript storage

### 3. Matrix TUI (`apps/tui-rs/`)

Rust terminal dashboard with digital rain visualization, built on Ratatui and Crossterm.

- **Digital Rain**: Per-agent rain columns with tool-aware color palettes (Bash=cyan, Read/Write=yellow, Grep=magenta, Task=orange)
- **Agent Naming**: Haiku API generates short memorable names for each agent based on their current task, refreshed on every new user prompt and every 60 seconds
- **Live Event Stream**: Scrollable event log with syntax-highlighted tool output (bash commands, JSON, file paths)
- **Stats Bar**: Real-time event rate, agent count, connection status
- **WebSocket Client**: Connects to the event server on port 4000

```bash
# Build and run
just tui

# Or directly with cargo
cargo run --release --manifest-path apps/tui-rs/Cargo.toml
```

Requires `ANTHROPIC_API_KEY` in `.env` for agent naming (runs without it, names are just omitted).

### Omarchy Compatibility

The Matrix TUI and status line integrate with [omarchy](https://github.com/ArcadeLabsInc/omarchy) for automatic theme matching. When omarchy is installed, colors are loaded from `~/.config/omarchy/current/theme/colors.toml` so the TUI matches your terminal and desktop theme.

**Without omarchy**: A built-in Kanagawa palette is used as the fallback. The full 16-color ANSI palette plus accent, cursor, and selection colors are provided, so the TUI looks good out of the box on any terminal. No configuration needed.

**How it works**: The theme loader starts from the fallback palette and overlays any colors found in the omarchy TOML. This means partial omarchy configs (e.g. only `background` and `accent` defined) work correctly — missing keys fall through to the Kanagawa defaults.

| Component | Fallback Theme | Config Path |
|-----------|---------------|-------------|
| Matrix TUI (Rust) | Kanagawa | `~/.config/omarchy/current/theme/colors.toml` |
| Status Line (Python) | Gruvbox | `~/.config/omarchy/current/theme/colors.toml` |

## 🔄 Data Flow

1. **Event Generation**: Claude Code executes an action (tool use, notification, etc.)
2. **Hook Activation**: Corresponding hook script runs based on `settings.json` configuration
3. **Data Collection**: Hook script gathers context (tool name, inputs, outputs, session ID)
4. **Transmission**: `send_event.py` sends JSON payload to server via HTTP POST
5. **Server Processing**:
   - Validates event structure
   - Stores in SQLite with timestamp
   - Broadcasts to WebSocket clients
6. **TUI Update**: Matrix TUI receives event via WebSocket and updates rain display in real-time

## 🎨 Event Types & Visualization

| Event Type         | Emoji | Purpose                | Color Coding  | Special Display                      |
| ------------------ | ----- | ---------------------- | ------------- | ------------------------------------ |
| PreToolUse         | 🔧     | Before tool execution  | Session-based | Tool name + tool emoji & details     |
| PostToolUse        | ✅     | After tool completion  | Session-based | Tool name + tool emoji & results     |
| PostToolUseFailure | ❌     | Tool execution failed  | Session-based | Error details & interrupt status     |
| PermissionRequest  | 🔐     | Permission requested   | Session-based | Tool name & permission suggestions   |
| Notification       | 🔔     | User interactions      | Session-based | Notification message & type          |
| Stop               | 🛑     | Response completion    | Session-based | Summary & chat transcript            |
| SubagentStart      | 🟢     | Subagent started       | Session-based | Agent ID & type                      |
| SubagentStop       | 👥     | Subagent finished      | Session-based | Agent details & transcript path      |
| PreCompact         | 📦     | Context compaction     | Session-based | Trigger & custom instructions        |
| UserPromptSubmit   | 💬     | User prompt submission | Session-based | Prompt: _"user message"_ (italic)    |
| SessionStart       | 🚀     | Session started        | Session-based | Source, model & agent type           |
| SessionEnd         | 🏁     | Session ended          | Session-based | End reason (clear/logout/exit/other) |

### UserPromptSubmit Event (v1.0.54+)

The `UserPromptSubmit` hook captures every user prompt before Claude processes it. In the UI:
- Displays as `Prompt: "user's message"` in italic text
- Shows the actual prompt content inline (truncated to 100 chars)
- Summary appears on the right side when AI summarization is enabled
- Useful for tracking user intentions and conversation flow

## 🔌 Integration

### For New Projects

1. Copy the event sender:
   ```bash
   cp .claude/hooks/send_event.py YOUR_PROJECT/.claude/hooks/
   ```

2. Add to your `.claude/settings.json`:
   ```json
   {
     "hooks": {
       "PreToolUse": [{
         "matcher": ".*",
         "hooks": [{
           "type": "command",
           "command": "uv run .claude/hooks/send_event.py --source-app YOUR_APP --event-type PreToolUse"
         }]
       }]
     }
   }
   ```

### For This Project

Already integrated! Hooks run both validation and observability:
```json
{
  "type": "command",
  "command": "uv run .claude/hooks/pre_tool_use.py"
},
{
  "type": "command",
  "command": "uv run .claude/hooks/send_event.py --source-app cc-hook-multi-agent-obvs --event-type PreToolUse"
}
```

## 🧪 Testing

```bash
# Quick test event via just
just test-event

# Check server health
just health

# Manual event test
curl -X POST http://localhost:4000/events \
  -H "Content-Type: application/json" \
  -d '{
    "source_app": "test",
    "session_id": "test-123",
    "hook_event_type": "PreToolUse",
    "payload": {"tool_name": "Bash", "tool_input": {"command": "ls"}}
  }'

# Test a hook script directly
just hook-test pre_tool_use
```

## ⚙️ Configuration

### Environment Variables

Copy `.env.sample` to `.env` in the project root and fill in your API keys:

**Application Root** (`.env` file):
- `ANTHROPIC_API_KEY` – Anthropic Claude API key (required)
- `ENGINEER_NAME` – Your name (for logging/identification)
- `OPENAI_API_KEY` – OpenAI API key (optional)
- `ELEVENLABS_API_KEY` – ElevenLabs API key (optional, for TTS)
- `FIRECRAWL_API_KEY` – Firecrawl API key (optional, for web scraping)

### Server Port

- Server: `4000` (HTTP/WebSocket) — the TUI connects here automatically

## 🤖 Agent Teams

This project supports Claude Code Agent Teams for orchestrating multi-agent workflows. Teams are enabled via the `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` environment variable in `.claude/settings.json`.

### Team Agents

- **Builder** (`.claude/agents/team/builder.md`): Engineering agent that executes one task at a time. Includes PostToolUse hooks for `ruff` and `ty` validation on Write/Edit operations.
- **Validator** (`.claude/agents/team/validator.md`): Read-only validation agent that inspects work without modifying files. Cannot use Write, Edit, or NotebookEdit tools.

### Planning with Teams

Use the `/plan_w_team` slash command to create team-based implementation plans:

```bash
/plan_w_team "Add a new feature for X"
```

This generates a spec document in `specs/` with task breakdowns, team member assignments, dependencies, and acceptance criteria. Plans are validated by Stop hook validators that ensure required sections are present.

Execute a plan with:
```bash
/build specs/<plan-name>.md
```

## 🔭 Multi-Agent Orchestration & Observability

[![Multi-Agent Orchestration with Claude Code](images/claude-code-multi-agent-orchestration.png)](https://youtu.be/RpUTF_U4kiw)

The true constraint of agentic engineering is no longer what the models can do — it's our ability to prompt engineer and context engineer the outcomes we need, and build them into reusable systems. Multi-agent orchestration changes the game by letting you spin up teams of specialized agents that each focus on one task extraordinarily well, work in parallel, and shut down when done. See the official [Claude Code Agent Teams documentation](https://code.claude.com/docs/en/agent-teams) for the full reference.

### The Orchestration Workflow

The full multi-agent orchestration lifecycle follows this pattern:

1. **Create a team** — `TeamCreate` sets up the coordination layer
2. **Create tasks** — `TaskCreate` builds the centralized task list that drives all work
3. **Spawn agents** — `Task` deploys specialized agents (builder, validator, etc.) into their own Tmux panes with independent context windows
4. **Work in parallel** — Agents execute their assigned tasks simultaneously, communicating via `SendMessage`
5. **Shut down agents** — Completed agents are gracefully terminated
6. **Delete the team** — `TeamDelete` cleans up all coordination state

### Why Observability Matters

When you have multiple agents running in parallel — each with their own context window, session ID, and task assignments — you need visibility into what's happening across the swarm. Without observability, you're vibe coding at scale. With it, you can:

- **Trace every tool call** across all agents in real-time via the dashboard
- **Filter by agent swim lane** to inspect individual agent behavior
- **Track task lifecycle** — see TaskCreate, TaskUpdate, and SendMessage events flow between agents
- **Spot failures early** — PostToolUseFailure and PermissionRequest events surface issues before they cascade
- **Measure throughput** — the live pulse chart shows activity density across your agent fleet

This is what separates engineers from vibe coders: understanding what's happening underneath the hood so you can scale compute to scale impact with confidence.

## 🛡️ Security Features

- Blocks dangerous `rm -rf` commands via `deny_tool()` JSON pattern (allowed only in specific directories)
- Prevents access to sensitive files (`.env`, private keys)
- `stop_hook_active` guard in `stop.py` and `subagent_stop.py` prevents infinite hook loops
- Stop hook validators ensure plan files contain required sections before completion
- Validates all inputs before execution

## 📊 Technical Stack

- **Server**: Bun, TypeScript, SQLite
- **TUI**: Rust, Ratatui, Crossterm, Tokio, Tungstenite
- **Hooks**: Python 3.11+, Astral uv, TTS (ElevenLabs or OpenAI), LLMs (Claude or OpenAI)
- **Communication**: HTTP REST, WebSocket

## Master AI **Agentic Coding**
> And prepare for the future of software engineering

Learn tactical agentic coding patterns with [Tactical Agentic Coding](https://agenticengineer.com/tactical-agentic-coding?y=opsorch)

Follow the [IndyDevDan YouTube channel](https://www.youtube.com/@indydevdan) to improve your agentic coding advantage.

