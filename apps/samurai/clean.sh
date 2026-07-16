#!/usr/bin/env bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║         Samurai Cleanup Script          ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════╝${NC}"
echo ""

# ── 1. Kill background processes ────────────────────────────────────
echo -e "${CYAN}[+] Killing background processes...${NC}"
for proc in "uvicorn app.main" "ng serve" "npm start"; do
    if pgrep -f "$proc" &>/dev/null; then
        echo "    Killing: $proc"
        pkill -f "$proc" 2>/dev/null || true
    fi
done
echo -e "${GREEN}[+] Processes cleaned.${NC}"
echo ""

# ── 2. Docker cleanup ────────────────────────────────────────────────
if command -v docker &>/dev/null; then
    echo -e "${CYAN}[+] Cleaning Docker resources...${NC}"

    # Try docker compose (v2 plugin) or docker-compose (standalone)
    DC=""
    if docker compose version &>/dev/null; then
        DC="docker compose"
    elif docker-compose --version &>/dev/null; then
        DC="docker-compose"
    fi

    if [ -n "$DC" ] && [ -f "$ROOT_DIR/docker-compose.yml" ]; then
        echo "    Stopping project containers, removing volumes & orphans..."
        $DC -f "$ROOT_DIR/docker-compose.yml" down -v --remove-orphans 2>/dev/null || true
    fi

    echo "    Pruning dangling images..."
    docker image prune -f 2>/dev/null || true

    echo "    Pruning unused networks..."
    docker network prune -f 2>/dev/null || true

    echo "    Pruning unused volumes..."
    docker volume prune -f 2>/dev/null || true

    echo "    Pruning build cache..."
    docker builder prune -f 2>/dev/null || true

    echo -e "${GREEN}[+] Docker cleanup done.${NC}"
else
    echo -e "    ${YELLOW}Docker not found, skipping.${NC}"
fi

echo ""

# ── 3. Python artifacts ──────────────────────────────────────────────
echo -e "${CYAN}[+] Cleaning Python artifacts...${NC}"
if [ -d "$ROOT_DIR/backend/.venv" ]; then
    echo "    Removing virtual environment (.venv)..."
    rm -rf "$ROOT_DIR/backend/.venv"
fi

COUNT=$(find "$ROOT_DIR/backend" -type d -name "__pycache__" 2>/dev/null | wc -l)
if [ "$COUNT" -gt 0 ]; then
    echo "    Removing $COUNT __pycache__ directories..."
    find "$ROOT_DIR/backend" -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
fi

find "$ROOT_DIR/backend" -type f \( -name "*.pyc" -o -name "*.pyo" \) -delete 2>/dev/null || true

# ── 4. Node / frontend artifacts ─────────────────────────────────────
echo -e "${CYAN}[+] Cleaning frontend artifacts...${NC}"
if [ -d "$ROOT_DIR/frontend/node_modules" ]; then
    echo "    Removing node_modules/..."
    rm -rf "$ROOT_DIR/frontend/node_modules"
fi
if [ -f "$ROOT_DIR/frontend/package-lock.json" ]; then
    echo "    Removing package-lock.json..."
    rm -f "$ROOT_DIR/frontend/package-lock.json"
fi
if [ -d "$ROOT_DIR/frontend/dist" ]; then
    echo "    Removing dist/..."
    rm -rf "$ROOT_DIR/frontend/dist"
fi
if [ -d "$ROOT_DIR/frontend/.angular" ]; then
    echo "    Removing .angular/ cache..."
    rm -rf "$ROOT_DIR/frontend/.angular"
fi

# ── 5. macOS / general junk ──────────────────────────────────────────
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${CYAN}[+] Cleaning macOS artifacts...${NC}"
    find "$ROOT_DIR" -name ".DS_Store" -delete 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}[+] Cleanup complete.${NC}"
echo -e "    Run ${CYAN}./samurai.sh${NC} or ${CYAN}./samurai.sh --native${NC} for a fresh start."
