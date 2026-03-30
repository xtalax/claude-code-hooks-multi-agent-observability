# Multi-Agent Observability System
# Usage: just <recipe>

set dotenv-load
set quiet

server_port := env("SERVER_PORT", "4000")
project_root := justfile_directory()

# List available recipes
default:
    @just --list

# ─── System ──────────────────────────────────────────────

# Start server + TUI
start:
    ./scripts/start-system.sh

# Stop all processes and clean up
stop:
    ./scripts/reset-system.sh

# Stop then start
restart: stop start

# ─── Server (Bun, port 4000) ────────────────────────────

# Install server dependencies
server-install:
    cd {{project_root}}/apps/server && bun install

# Start server in dev mode (watch)
server:
    cd {{project_root}}/apps/server && SERVER_PORT={{server_port}} bun run dev

# Start server in production mode
server-prod:
    cd {{project_root}}/apps/server && SERVER_PORT={{server_port}} bun run start

# Typecheck server
server-typecheck:
    cd {{project_root}}/apps/server && bun run typecheck

# ─── TUI ─────────────────────────────────────────────────

# Build Matrix TUI (release)
tui-build:
    cargo build --release --manifest-path {{project_root}}/apps/tui-rs/Cargo.toml

# Launch Matrix TUI
tui:
    cargo run --release --manifest-path {{project_root}}/apps/tui-rs/Cargo.toml

# ─── Install ─────────────────────────────────────────────

# Install all dependencies (server + TUI build)
install: server-install tui-build

# ─── Database ────────────────────────────────────────────

# Clear SQLite WAL files
db-clean-wal:
    rm -f {{project_root}}/apps/server/events.db-wal {{project_root}}/apps/server/events.db-shm
    @echo "WAL files removed"

# Delete the entire events database
db-reset:
    rm -f {{project_root}}/apps/server/events.db {{project_root}}/apps/server/events.db-wal {{project_root}}/apps/server/events.db-shm
    @echo "Database reset"

# ─── Testing ─────────────────────────────────────────────

# Send a test event to the server
test-event:
    curl -s -X POST http://localhost:{{server_port}}/events \
      -H "Content-Type: application/json" \
      -d '{"source_app":"test","session_id":"test-1234","hook_event_type":"PreToolUse","payload":{"tool_name":"Bash","tool_input":{"command":"echo hello"}}}' \
      | head -c 200
    @echo ""

# Check server health
health:
    @curl -sf http://localhost:{{server_port}}/health > /dev/null 2>&1 \
      && echo "Server: UP (port {{server_port}})" \
      || echo "Server: DOWN (port {{server_port}})"

# ─── Hooks ───────────────────────────────────────────────

# Test a hook script directly (e.g. just hook-test pre_tool_use)
hook-test name:
    echo '{"session_id":"test-hook","tool_name":"Bash"}' | uv run {{project_root}}/.claude/hooks/{{name}}.py

# List all hook scripts
hooks:
    @ls -1 {{project_root}}/.claude/hooks/*.py | xargs -I{} basename {} .py
