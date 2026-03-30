#!/bin/bash

echo "Starting Multi-Agent Observability System"
echo "=========================================="

GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

SERVER_PORT=${SERVER_PORT:-4000}

echo -e "${BLUE}Configuration:${NC}"
echo -e "  Server Port: ${GREEN}$SERVER_PORT${NC}"

# Function to kill processes on a port
kill_port() {
    local port=$1
    local name=$2

    echo -e "\n${YELLOW}Checking for existing $name on port $port...${NC}"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        PIDS=$(lsof -ti :$port 2>/dev/null)
    else
        PIDS=$(lsof -ti :$port 2>/dev/null || fuser -n tcp $port 2>/dev/null | awk '{print $2}')
    fi

    if [ -n "$PIDS" ]; then
        echo -e "${RED}Found existing processes on port $port: $PIDS${NC}"
        for PID in $PIDS; do
            kill -9 $PID 2>/dev/null && echo -e "${GREEN}Killed process $PID${NC}" || echo -e "${RED}Failed to kill process $PID${NC}"
        done
        sleep 1
    else
        echo -e "${GREEN}Port $port is available${NC}"
    fi
}

kill_port $SERVER_PORT "server"

# Start server
echo -e "\n${GREEN}Starting server on port $SERVER_PORT...${NC}"
cd "$PROJECT_ROOT/apps/server"
SERVER_PORT=$SERVER_PORT bun run dev &
SERVER_PID=$!

# Wait for server to be ready
echo -e "${YELLOW}Waiting for server to start...${NC}"
for i in {1..10}; do
    if curl -s http://localhost:$SERVER_PORT/health >/dev/null 2>&1 || curl -s http://localhost:$SERVER_PORT/events/filter-options >/dev/null 2>&1; then
        echo -e "${GREEN}Server is ready!${NC}"
        break
    fi
    sleep 1
done

echo -e "\n${BLUE}============================================${NC}"
echo -e "${GREEN}Multi-Agent Observability Server Started${NC}"
echo -e "${BLUE}============================================${NC}"
echo
echo -e "Server API: ${GREEN}http://localhost:$SERVER_PORT${NC}"
echo -e "WebSocket:  ${GREEN}ws://localhost:$SERVER_PORT/stream${NC}"
echo -e "Server PID: ${YELLOW}$SERVER_PID${NC}"
echo
echo -e "Launch the TUI:  ${YELLOW}just tui${NC}"
echo -e "Stop the server: ${YELLOW}just stop${NC}"
echo
echo -e "${BLUE}Press Ctrl+C to stop${NC}"

cleanup() {
    echo -e "\n${YELLOW}Shutting down...${NC}"
    kill $SERVER_PID 2>/dev/null
    echo -e "${GREEN}Stopped server${NC}"
    exit 0
}

trap cleanup INT

wait $SERVER_PID
