#!/bin/bash

# Define colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== xwa Dashboard Startup Script ===${NC}"

export PYTHONPATH=$(pwd)
export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-"http://localhost:8000"}
export PORT=${PORT:-3000}
export WATCHPACK_POLLING=true
export CHOKIDAR_USEPOLLING=1
export NEXT_TELEMETRY_DISABLED=1

kill_if_listening() {
    local port=$1
    local label=$2
    local pids
    pids=$(lsof -t -i :"${port}" -sTCP:LISTEN 2>/dev/null | tr '\n' ' ')
    if [ -n "${pids}" ]; then
        echo -e "${RED}Found existing ${label} process on port ${port} (PID: ${pids}). Killing to avoid stale builds...${NC}"
        kill ${pids} 2>/dev/null || true
    fi
}

kill_if_listening "${PORT}" "Next.js"
kill_if_listening 8000 "FastAPI"

# 1. Check Python virtual environment
echo -e "\n${GREEN}[1/3] Checking Python Backend Dependencies...${NC}"
if [ ! -d "venv" ]; then
    echo -e "${RED}Virtual environment not found. Creating 'venv'...${NC}"
    python3 -m venv venv
    source venv/bin/activate
    pip install -r requirements.txt
else
    source venv/bin/activate
fi
echo "Python environment Ready."

# 2. Check Node.js Next.js environment
echo -e "\n${GREEN}[2/3] Checking Node.js Frontend Dependencies...${NC}"
cd web
if [ ! -d "node_modules" ]; then
    echo -e "${RED}Node modules not found. Running npm install...${NC}"
    npm install
fi
echo -e "${GREEN}Cleaning stale frontend builds (.next/.turbo/out)...${NC}"
rm -rf .next .turbo out
cd ..
echo "Node.js environment Ready."

# 3. Starting the servers
echo -e "\n${GREEN}[3/3] Starting Servers...${NC}"

# Function to handle graceful shutdown
shutdown() {
    echo -e "\n${RED}Shutting down xwa servers...${NC}"
    kill -TERM $BACKEND_PID $FRONTEND_PID 2>/dev/null
    wait $BACKEND_PID $FRONTEND_PID 2>/dev/null
    deactivate 2>/dev/null || true
    echo -e "Servers gracefully stopped. Virtual environment deactivated."
    exit 0
}

# Trap Ctrl+C (SIGINT) and SIGTERM
trap shutdown SIGINT SIGTERM

# Start Backend
echo -e "Starting FastAPI Backend (Port: 8000)..."
uvicorn api.main:app --port 8000 &
BACKEND_PID=$!

# Start Frontend
echo -e "Starting Next.js Frontend (Port: 3000)..."
cd web && npm run dev &
FRONTEND_PID=$!
cd ..

echo -e "\n${BLUE}================================================${NC}"
echo -e "${GREEN}[DONE] xwa is fully running!${NC}"
echo -e "-> Frontend Dashboard: http://localhost:3000"
echo -e "-> API Swagger Docs:   http://localhost:8000/docs"
echo -e "${RED}Press Ctrl+C at any time to stop everything.${NC}"
echo -e "${BLUE}================================================${NC}\n"

# Wait keeps the script alive until interrupted
wait
