#!/usr/bin/env bash
set -e

readonly GRN='\033[0;32m'
readonly BLU='\033[0;34m'
readonly YLW='\033[1;33m'
readonly RED='\033[0;31m'
readonly CYN='\033[0;36m'
readonly NC='\033[0m'

log()  { echo -e "${GRN}[shinobi]${NC} $1"; }
info() { echo -e "${BLU}[info]${NC} $1"; }
warn() { echo -e "${YLW}[warn]${NC} $1"; }
err()  { echo -e "${RED}[err]${NC} $1"; }

cleanup() {
    echo ""
    warn "Shutting down..."
    if [ -n "$RUST_PID" ]; then kill "$RUST_PID" 2>/dev/null || true; fi
    if [ -n "$PY_PID" ]; then kill "$PY_PID" 2>/dev/null || true; fi
    if command -v docker &>/dev/null; then
        local running=$(docker ps --filter "name=shinobi" --filter "name=extractor" -q 2>/dev/null)
        if [ -n "$running" ]; then
            info "Stopping docker containers..."
            docker stop $running 2>/dev/null || true
        fi
    fi
    info "All processes stopped"
    exit 0
}

check_deps() {
    if ! command -v cargo &>/dev/null; then
        err "Rust (cargo) not found. Install: https://rustup.rs"
        exit 1
    fi
    if ! command -v node &>/dev/null; then
        err "Node.js not found. Install: https://nodejs.org"
        exit 1
    fi
}

setup_frontend() {
    if [ ! -d "frontend/node_modules" ]; then
        info "Installing frontend dependencies..."
        (cd frontend && npm install --legacy-peer-deps) || warn "npm install failed"
    fi
    if [ ! -d "static/browser" ]; then
        info "Building frontend..."
        if command -v npx &>/dev/null; then
            (cd frontend && npx ng build) || warn "ng build failed"
        else
            warn "npx not found, frontend may not render"
        fi
    fi
}

setup_python() {
    if ! command -v python3 &>/dev/null; then
        err "Python 3 not found. Install: https://python.org"
        exit 1
    fi
    if [ ! -d "extractor/venv" ]; then
        info "Creating Python virtual environment..."
        python3 -m venv extractor/venv
        info "Installing Python dependencies (may take a while)..."
        extractor/venv/bin/pip install -q -r extractor/requirements.txt 2>/dev/null || {
            warn "pip install failed, retrying with --break-system-packages..."
            extractor/venv/bin/pip install -q --break-system-packages -r extractor/requirements.txt 2>/dev/null || {
                warn "Some deps failed, extractor may have limited functionality"
            }
        }
    fi
    if ! command -v httrack &>/dev/null; then
        if command -v pacman &>/dev/null; then
            info "Installing httrack via pacman..."
            sudo pacman -S --noconfirm httrack 2>/dev/null || warn "Could not install httrack via pacman"
        elif command -v apt-get &>/dev/null; then
            info "Installing httrack via apt..."
            sudo apt-get install -y -qq httrack 2>/dev/null || warn "Could not install httrack via apt"
        elif command -v dnf &>/dev/null; then
            info "Installing httrack via dnf..."
            sudo dnf install -y httrack 2>/dev/null || warn "Could not install httrack via dnf"
        elif command -v brew &>/dev/null; then
            info "Installing httrack via brew..."
            brew install httrack 2>/dev/null || warn "Could not install httrack via brew"
        else
            warn "httrack not found. Python crawl mode will fail."
            warn "Install: pacman -S httrack | apt install httrack | dnf install httrack | brew install httrack"
        fi
    fi
}

show_help() {
    cat <<EOF
${CYN}shinobi — launch control${NC}

Usage: ./shinobi.sh [options]

Options:
  -f, --fast         Launch Rust backend + frontend (Fast Test mode)
  -d, --deep         Launch Rust + Python (Fast Test + Deep Research)
  -p, --python-only  Launch only Python extractor (for development)
  -b, --build        Force rebuild frontend before launching
  -D, --docker       Use docker-compose instead of native processes
  -h, --help         Show this help

Examples:
  ./shinobi.sh              # Fast Test mode (default)
  ./shinobi.sh --deep        # Full stack: Fast Test + Deep Research
  ./shinobi.sh --docker      # Launch all services via docker-compose
  ./shinobi.sh --deep -b     # Full stack with fresh frontend build
EOF
    exit 0
}

MODE="fast"
BUILD=false
USE_DOCKER=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -f|--fast|--rust-only) MODE="fast"; shift ;;
        -d|--deep) MODE="deep"; shift ;;
        -p|--python-only) MODE="python"; shift ;;
        -b|--build) BUILD=true; shift ;;
        -D|--docker) USE_DOCKER=true; shift ;;
        -h|--help) show_help ;;
        *) err "Unknown option: $1"; show_help ;;
    esac
done

trap cleanup SIGINT SIGTERM

if [ "$USE_DOCKER" = true ]; then
    if [ ! -f docker-compose.yml ]; then
        err "docker-compose.yml not found"
        exit 1
    fi
    log "Launching via docker-compose..."
    if [ "$MODE" = "fast" ]; then
        docker compose up --build shinobi
    else
        docker compose up --build
    fi
    exit 0
fi

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

check_deps
setup_frontend

if [ "$BUILD" = true ]; then
    log "Rebuilding frontend..."
    if command -v npx &>/dev/null; then
        (cd frontend && npx ng build) || warn "ng build failed"
    fi
fi

if [ "$MODE" = "fast" ] || [ "$MODE" = "deep" ]; then
    log "Starting Rust backend..."
    RUST_LOG=shinobi=info,tower_http=info cargo run --release &
    RUST_PID=$!
    sleep 2
    if kill -0 "$RUST_PID" 2>/dev/null; then
        info "Rust backend running (PID: $RUST_PID) — http://localhost:8080"
    else
        err "Rust backend failed to start. Check logs above for details."
        exit 1
    fi
fi

if [ "$MODE" = "deep" ]; then
    setup_python
    log "Starting Python extractor on port 9090..."
    extractor/venv/bin/python extractor/main.py 9090 &
    PY_PID=$!
    sleep 2
    if kill -0 "$PY_PID" 2>/dev/null; then
        info "Python extractor running (PID: $PY_PID) — http://localhost:9090"
    else
        err "Python extractor failed to start. Run manually to debug:"
        err "  extractor/venv/bin/python extractor/main.py 9090"
        exit 1
    fi
fi

if [ "$MODE" = "python" ]; then
    setup_python
    extractor/venv/bin/python extractor/main.py 9090 &
    PY_PID=$!
    wait $PY_PID
    exit 0
fi

echo ""
info "───────────────────────────────────────"
info " Shinobi is running"
if [ "$MODE" = "fast" ]; then
    info " Mode: Fast Test"
    info " URL:  http://localhost:8080"
elif [ "$MODE" = "deep" ]; then
    info " Mode: Fast Test + Deep Research"
    info " URL:  http://localhost:8080"
    info " Extractor: http://localhost:9090"
fi
info " Press Ctrl+C to stop all services"
info "───────────────────────────────────────"
echo ""

wait
