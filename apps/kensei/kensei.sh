#!/usr/bin/env bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

MODE=""
BACKEND_PID=""
FRONTEND_PID=""
DOCKER_COMPOSE=""
COMPOSE_FILE="$ROOT_DIR/docker-compose.yml"

pkg_manager() {
    if [[ "$OSTYPE" == "darwin"* ]]; then echo "brew"
    elif command -v apt-get &>/dev/null; then echo "apt"
    elif command -v dnf &>/dev/null; then echo "dnf"
    elif command -v pacman &>/dev/null; then echo "pacman"
    elif command -v zypper &>/dev/null; then echo "zypper"
    else echo "unknown"; fi
}
PKG_MANAGER=$(pkg_manager)

find_compose() {
    if docker compose version &>/dev/null; then
        DOCKER_COMPOSE="docker compose"; return 0
    fi
    if docker-compose --version &>/dev/null; then
        DOCKER_COMPOSE="docker-compose"; return 0
    fi
    return 1
}

docker_available() { command -v docker &>/dev/null && docker info &>/dev/null; }

arch_map() {
    case "$1" in
        python3|python3-venv) echo "python" ;;
        python3-pip) echo "python-pip" ;;
        nodejs)  echo "nodejs" ;;
        npm)     echo "npm" ;;
        docker.io) echo "docker" ;;
        docker-compose-v2) echo "docker-compose" ;;
        *)       echo "$1" ;;
    esac
}

install_pkg() {
    local pkg="$1"
    [ "$PKG_MANAGER" = "pacman" ] && pkg=$(arch_map "$pkg")
    echo -e "${YELLOW}[~] Installing $pkg...${NC}"
    case "$PKG_MANAGER" in
        brew)   brew install "$pkg" ;;
        apt)    sudo apt-get install -y "$pkg" ;;
        dnf)    sudo dnf install -y "$pkg" ;;
        pacman) sudo pacman -S --noconfirm "$pkg" ;;
        zypper) sudo zypper --non-interactive install "$pkg" ;;
        *)      echo -e "${RED}[!] Install $pkg manually.${NC}"; return 1 ;;
    esac
}

ensure_docker() {
    if ! command -v docker &>/dev/null; then
        echo -e "${YELLOW}[~] Docker not found. Installing...${NC}"
        case "$PKG_MANAGER" in
            brew)  brew install --cask docker ;;
            apt)   sudo apt-get install -y docker.io && sudo usermod -aG docker "$USER"
                   echo -e "${YELLOW}[!] You may need to log out and back in for Docker group.${NC}" ;;
            dnf)   sudo dnf install -y docker docker-compose
                   sudo systemctl enable --now docker
                   sudo usermod -aG docker "$USER" ;;
            pacman) sudo pacman -S --noconfirm docker docker-compose
                    sudo systemctl enable --now docker
                    sudo usermod -aG docker "$USER" ;;
            *)     echo -e "${RED}[!] Install Docker manually: https://docker.com${NC}"; exit 1 ;;
        esac
    fi
    if ! find_compose; then
        echo -e "${YELLOW}[~] Docker Compose not found. Installing...${NC}"
        install_pkg docker-compose-v2
        find_compose || { echo -e "${RED}[!] Failed to setup Docker Compose.${NC}"; exit 1; }
    fi
}

ensure_code_deps() {
    if ! command -v python3 &>/dev/null; then install_pkg python3; fi
    if ! python3 -m pip --version &>/dev/null; then install_pkg python3-pip; fi
    if ! command -v node &>/dev/null; then
        [ "$PKG_MANAGER" = "brew" ] && brew install node || install_pkg nodejs
    fi
    if ! command -v npm &>/dev/null; then
        [ "$PKG_MANAGER" = "brew" ] && brew install node || install_pkg npm
    fi
}

port_open() {
    local host="$1" port="$2"
    if command -v nc &>/dev/null; then nc -z "$host" "$port" 2>/dev/null
    elif [ -e /dev/tcp ]; then timeout 2 bash -c "echo >/dev/tcp/$host/$port" 2>/dev/null
    else return 1; fi
}

dc_up() {
    [ -n "$DOCKER_COMPOSE" ] && $DOCKER_COMPOSE -f "$COMPOSE_FILE" up -d "$1" 2>/dev/null
}

dc_down() {
    [ -n "$DOCKER_COMPOSE" ] && $DOCKER_COMPOSE -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
}

docker_run_pg() {
    docker rm -f kensei-db 2>/dev/null || true
    docker run -d --name kensei-db -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgres \
        -e POSTGRES_DB=kensei -p 5432:5432 postgres:15-alpine 2>/dev/null
}

docker_run_redis() {
    docker rm -f kensei-redis 2>/dev/null || true
    docker run -d --name kensei-redis -p 6379:6379 redis:alpine 2>/dev/null
}

docker_infra_stop() { docker rm -f kensei-db kensei-redis 2>/dev/null || true; }

ensure_infra() {
    local host="$1" port="$2" name="$3"
    if port_open "$host" "$port"; then
        echo -e "  ${GREEN}$name already running at $host:$port${NC}"
        return 0
    fi
    if ! docker_available; then
        echo -e "${RED}[!] Docker is not available. Cannot start $name.${NC}"
        echo -e "${YELLOW}    Install Docker, or use --native-no-infra and manage PG/Redis yourself.${NC}"
        return 1
    fi
    echo -e "${YELLOW}[~] $name not found locally. Starting via Docker...${NC}"
    local started=false
    if find_compose; then
        local svc; case "$name" in PostgreSQL) svc="db" ;; Redis) svc="redis" ;; esac
        dc_up "$svc" && started=true
    fi
    if [ "$started" = false ]; then
        case "$name" in PostgreSQL) docker_run_pg && started=true ;; Redis) docker_run_redis && started=true ;; esac
    fi
    if [ "$started" = true ]; then
        echo -n -e "${CYAN}[~] Waiting for $name...${NC}"
        for _ in $(seq 1 30); do
            if port_open "$host" "$port"; then
                INFRA_STARTED="$INFRA_STARTED $name"
                echo -e " ${GREEN}ready${NC}"
                return 0
            fi
            echo -n "."; sleep 2
        done
        echo -e " ${RED}timeout${NC}"
        return 1
    fi
    echo -e "${RED}[!] Failed to start $name via Docker.${NC}"
    return 1
}

cleanup() {
    echo -e "\n${RED}[!] Shutting down...${NC}"
    if [ "$MODE" = "docker" ]; then
        dc_down
    else
        [ -n "$BACKEND_PID" ] && kill "$BACKEND_PID" 2>/dev/null || true
        [ -n "$FRONTEND_PID" ] && kill "$FRONTEND_PID" 2>/dev/null || true
        wait 2>/dev/null || true
        if [ -n "$INFRA_STARTED" ]; then
            echo -e "${CYAN}[~] Stopping infra containers...${NC}"
            if echo "$INFRA_STARTED" | grep -q "PostgreSQL\|Redis"; then
                docker_infra_stop
            else
                [ -n "$DOCKER_COMPOSE" ] && $DOCKER_COMPOSE -f "$COMPOSE_FILE" stop $INFRA_STARTED 2>/dev/null || true
            fi
        fi
    fi
    echo -e "${GREEN}[+] Done.${NC}"
    exit 0
}

usage() {
    echo "Usage: $0 [--docker|--sqlite|--native|--native-no-infra]"
    echo ""
    echo "  --docker            Everything with Docker Compose (default)"
    echo "  --sqlite            Code natively with SQLite (zero infra — fast dev)"
    echo "  --native            Code natively, PG & Redis via Docker"
    echo "  --native-no-infra   Code natively, you manage PG & Redis"
    echo ""
    echo "Examples:"
    echo "  $0                  # full Docker"
    echo "  $0 --sqlite         # dev mode with SQLite (no Docker needed)"
    echo "  $0 --native         # dev mode with PG via Docker"
    echo "  $0 --native-no-infra # dev mode, you run PG/Redis"
    exit 1
}

INFRA_STARTED=""
case "${1:---docker}" in
    --docker)          MODE="docker" ;;
    --sqlite)          MODE="sqlite" ;;
    --native)          MODE="native" ;;
    --native-no-infra) MODE="native"; NO_INFRA=true ;;
    *) usage ;;
esac

trap cleanup SIGINT SIGTERM

if [ "$MODE" = "docker" ]; then
    ensure_docker
    echo -e "${CYAN}[+] Starting Kensei with Docker Compose...${NC}"
    $DOCKER_COMPOSE -f "$COMPOSE_FILE" up --build
    exit $?
fi

# ── SQLite mode — zero infra, just Python + Node ─────────────────────
if [ "$MODE" = "sqlite" ]; then
    echo -e "${CYAN}[+] Starting Kensei with SQLite (zero infra)...${NC}"
    ensure_code_deps

    echo -e "${CYAN}[+] Setting up Python environment...${NC}"
    cd "$ROOT_DIR/backend"
    if [ ! -d ".venv" ]; then python3 -m venv .venv; fi
    source .venv/bin/activate
    pip install --quiet -r requirements.txt 2>/dev/null || pip install --quiet fastapi uvicorn SQLAlchemy httpx beautifulsoup4 aiohttp cryptography requests

    export DB_DRIVER=sqlite
    export DB_PATH="${DB_PATH:-$ROOT_DIR/kensei.db}"

    echo -e "${CYAN}[+] Starting backend on :8000...${NC}"
    uvicorn app.main:app --host 0.0.0.0 --port 8000 &
    BACKEND_PID=$!

    echo -e "${CYAN}[+] Setting up frontend...${NC}"
    cd "$ROOT_DIR/frontend"
    if [ ! -d "node_modules" ]; then
        npm install
    fi
    echo -e "${CYAN}[+] Starting frontend on :4200...${NC}"
    npm start &
    FRONTEND_PID=$!

    echo ""
    echo -e "${GREEN}[+] Kensei is running (SQLite):${NC}"
    echo -e "    Frontend:  ${CYAN}http://localhost:4200${NC}"
    echo -e "    Backend:   ${CYAN}http://localhost:8000${NC}"
    echo -e "    Press ${RED}Ctrl+C${NC} to stop both."
    echo ""
    wait
    exit 0
fi

# ── Native mode (PG) ─────────────────────────────────────────────────
echo -e "${CYAN}[+] Starting Kensei natively...${NC}"
ensure_code_deps

DB_HOST="${DB_HOST:-localhost}"
REDIS_HOST="${REDIS_HOST:-localhost}"

if [ "$NO_INFRA" = true ]; then
    if ! port_open "$DB_HOST" 5432; then
        echo -e "${RED}[!] PostgreSQL is not reachable at $DB_HOST:5432.${NC}"
        echo -e "${YELLOW}    Use --native (without -no-infra) to auto-start it.${NC}"
        exit 1
    fi
else
    ensure_infra "$DB_HOST" 5432 "PostgreSQL"
fi

echo -e "${CYAN}[+] Setting up Python environment...${NC}"
cd "$ROOT_DIR/backend"
if [ ! -d ".venv" ]; then python3 -m venv .venv; fi
source .venv/bin/activate
echo -e "${CYAN}[+] Installing Python dependencies...${NC}"
pip install --quiet -r requirements.txt

export DB_HOST="${DB_HOST:-localhost}"
export REDIS_URL="${REDIS_URL:-redis://localhost:6379/0}"
export REDIS_HOST="${REDIS_HOST:-localhost}"

echo -e "${CYAN}[+] Starting backend on :8000...${NC}"
uvicorn app.main:app --host 0.0.0.0 --port 8000 &
BACKEND_PID=$!

echo -e "${CYAN}[+] Setting up frontend...${NC}"
cd "$ROOT_DIR/frontend"
if [ ! -d "node_modules" ]; then
    echo -e "${CYAN}[+] Installing npm dependencies...${NC}"
    npm install
fi
echo -e "${CYAN}[+] Starting frontend on :4200...${NC}"
npm start &
FRONTEND_PID=$!

echo ""
echo -e "${GREEN}[+] Kensei is running:${NC}"
echo -e "    Frontend:  ${CYAN}http://localhost:4200${NC}"
echo -e "    Backend:   ${CYAN}http://localhost:8000${NC}"
echo -e "    Press ${RED}Ctrl+C${NC} to stop both."
echo ""

wait