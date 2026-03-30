#!/bin/bash

echo "Stopping Multi-Agent Observability System"
echo "=========================================="

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

SERVER_PORT=${SERVER_PORT:-4000}

# Function to kill processes on a port
kill_port() {
    local port=$1
    local name=$2

    echo -e "\n${YELLOW}Checking for $name on port $port...${NC}"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        PIDS=$(lsof -ti :$port 2>/dev/null)
    else
        PIDS=$(lsof -ti :$port 2>/dev/null || fuser -n tcp $port 2>/dev/null | awk '{print $2}')
    fi

    if [ -n "$PIDS" ]; then
        echo -e "${RED}Found processes on port $port: $PIDS${NC}"
        for PID in $PIDS; do
            kill -9 $PID 2>/dev/null && echo -e "${GREEN}Killed process $PID${NC}" || echo -e "${RED}Failed to kill process $PID${NC}"
        done
    else
        echo -e "${GREEN}No processes found on port $port${NC}"
    fi
}

kill_port $SERVER_PORT "server"

# Kill any remaining bun processes related to the server
echo -e "\n${YELLOW}Checking for remaining bun processes...${NC}"
ps aux | grep -E "bun.*apps/server" | grep -v grep | awk '{print $2}' | while read PID; do
    if [ -n "$PID" ]; then
        kill -9 $PID 2>/dev/null && echo -e "${GREEN}Killed bun process $PID${NC}"
    fi
done

# Clean SQLite WAL files
echo -e "\n${YELLOW}Cleaning up SQLite WAL files...${NC}"
if [ -f "$PROJECT_ROOT/apps/server/events.db-wal" ]; then
    rm -f "$PROJECT_ROOT/apps/server/events.db-wal" "$PROJECT_ROOT/apps/server/events.db-shm"
    echo -e "${GREEN}Removed SQLite WAL files${NC}"
else
    echo -e "${GREEN}No WAL files to clean${NC}"
fi

echo -e "\n${GREEN}System stopped.${NC}"
echo -e "\nTo start again: ${YELLOW}just start${NC}"
