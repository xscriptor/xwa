#!/usr/bin/env bash
set -e

readonly GRN='\033[0;32m'
readonly BLU='\033[0;34m'
readonly YLW='\033[1;33m'
readonly RED='\033[0;31m'
readonly CYN='\033[0;36m'
readonly NC='\033[0m'

log()  { echo -e "${GRN}[tengu]${NC} $1"; }
info() { echo -e "${BLU}[info]${NC} $1"; }
warn() { echo -e "${YLW}[warn]${NC} $1"; }
err()  { echo -e "${RED}[err]${NC} $1"; }

RUST_PID=""

cleanup() {
    echo ""
    warn "Shutting down..."
    if [ -n "$RUST_PID" ]; then
        kill "$RUST_PID" 2>/dev/null || true
        wait "$RUST_PID" 2>/dev/null || true
    fi
    if command -v docker &>/dev/null; then
        local running=$(docker ps --filter "name=tengu" -q 2>/dev/null)
        if [ -n "$running" ]; then
            docker stop $running 2>/dev/null || true
        fi
    fi
    info "Bye"
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

build_frontend() {
    if [ ! -d "frontend/node_modules" ]; then
        info "Installing frontend dependencies..."
        (cd frontend && npm install --legacy-peer-deps) || { warn "npm install failed"; return 1; }
    fi
    info "Building frontend..."
    if (cd frontend && npx ng build); then
        info "Frontend built successfully"
    else
        warn "Frontend build had issues"
    fi
}

wait_for_server() {
    local tries=0
    local max=120
    while [ $tries -lt $max ]; do
        if curl -sf http://localhost:${PORT:-8080}/api/health > /dev/null 2>&1; then
            return 0
        fi
        if [ $tries -eq 5 ]; then
            info "Waiting for server to be ready (compiling dependencies)..."
        fi
        if [ $((tries % 15)) -eq 0 ] && [ $tries -gt 0 ]; then
            info "Still waiting... (${tries}s)"
        fi
        sleep 1
        tries=$((tries + 1))
    done
    return 1
}

free_port() {
    local port=${PORT:-8080}
    if command -v fuser &>/dev/null; then
        fuser -k "${port}/tcp" 2>/dev/null || true
    elif command -v lsof &>/dev/null; then
        local pid
        pid=$(lsof -t -i ":$port" 2>/dev/null) && kill "$pid" 2>/dev/null || true
    fi
    sleep 1
}

start_server() {
    free_port
    log "Starting Tengu..."
    RUST_LOG=tengu=info,tower_http=info cargo run --release &
    RUST_PID=$!

    if wait_for_server; then
        log "Server ready"
    else
        err "Timed out waiting for server"
        kill "$RUST_PID" 2>/dev/null || true
        exit 1
    fi
}

show_help() {
    cat <<EOF
${CYN}tengu — web quality auditor${NC}

${BLU}USAGE${NC}
  ./tengu.sh [command] [options]

${BLU}COMMANDS${NC}
  (no command)      Start server (Rust backend + frontend)
  docker            Start via docker-compose
  build             Build frontend only, do not start server
  export [file]     Start server and export audits to file
  import <file>     Start server and import audits from file

${BLU}OPTIONS${NC}
  --rm        With docker: ephemeral container (removed on stop)
  -b          Force frontend rebuild before starting

${BLU}EXAMPLES${NC}
  ./tengu.sh                   Start server at http://localhost:8080
  ./tengu.sh docker            Start with Docker
  ./tengu.sh docker --rm       Ephemeral Docker
  ./tengu.sh build             Build frontend
  ./tengu.sh export            Export audits (saved to exports/)
  ./tengu.sh export res.json   Export audits to res.json
  ./tengu.sh import res.json   Import audits from res.json
EOF
    exit 0
}

# --- Parse arguments ---

COMMAND=""
EPHEMERAL=false
FORCE_BUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        docker) COMMAND="docker"; shift ;;
        build)  COMMAND="build"; shift ;;
        export) COMMAND="export"; shift ;;
        import) COMMAND="import"; shift ;;
        --rm)   EPHEMERAL=true; shift ;;
        -b)     FORCE_BUILD=true; shift ;;
        --help|-h) show_help ;;
        *)
            # If we already have a command, treat as its argument
            if [ -n "$COMMAND" ]; then
                case "$COMMAND" in
                    export|import)
                        ACTION_FILE="$1"; shift ;;
                    *)
                        err "Unknown option: $1"; exit 1 ;;
                esac
            else
                err "Unknown option: $1"; exit 1
            fi
            ;;
    esac
done

trap cleanup SIGINT SIGTERM

# --- Command dispatch ---

case "$COMMAND" in
    docker)
        if [ ! -f docker-compose.yml ]; then
            err "docker-compose.yml not found"
            exit 1
        fi
        log "Launching via docker-compose..."
        if [ "$EPHEMERAL" = true ]; then
            docker compose up --build --rm
        else
            docker compose up --build
        fi
        exit 0
        ;;

    build)
        PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
        cd "$PROJECT_ROOT"
        check_deps
        build_frontend
        log "Build done"
        exit 0
        ;;

    export)
        PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
        cd "$PROJECT_ROOT"
        check_deps

        if [ "$FORCE_BUILD" = true ] || [ ! -d "frontend/node_modules" ]; then
            build_frontend
        fi

        start_server

        mkdir -p exports
        if [ -z "$ACTION_FILE" ]; then
            ACTION_FILE="exports/tengu-export-$(date +%Y%m%d-%H%M%S).json"
        else
            ACTION_FILE="${ACTION_FILE#./}"
            ACTION_FILE="${ACTION_FILE#exports/}"
            ACTION_FILE="exports/$ACTION_FILE"
        fi

        info "Exporting audits to $ACTION_FILE ..."
        if curl -sf "http://localhost:${PORT:-8080}/api/audits/export" -o "$ACTION_FILE"; then
            info "Exported: $ACTION_FILE"
        else
            err "Export failed"
        fi

        info "───────────────────────────────────────────"
        info " Tengu is running on http://localhost:${PORT:-8080}"
        info "───────────────────────────────────────────"
        wait "$RUST_PID"
        ;;

    import)
        PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
        cd "$PROJECT_ROOT"

        if [ -z "$ACTION_FILE" ]; then
            err "Uso: ./tengu.sh import <archivo>"
            exit 1
        fi
        if [ ! -f "$ACTION_FILE" ]; then
            err "File not found: $ACTION_FILE"
            exit 1
        fi

        check_deps

        if [ "$FORCE_BUILD" = true ] || [ ! -d "frontend/node_modules" ]; then
            build_frontend
        fi

        start_server

        info "Importing audits from $ACTION_FILE ..."
        if curl -sf -X POST "http://localhost:${PORT:-8080}/api/audits/import" \
            -H "Content-Type: application/json" \
            -d @"$ACTION_FILE"; then
            info "Import completed"
        else
            err "Import failed"
        fi

        info "───────────────────────────────────────────"
        info " Tengu is running on http://localhost:${PORT:-8080}"
        info "───────────────────────────────────────────"
        wait "$RUST_PID"
        ;;

    "")
        # Default: start server
        PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
        cd "$PROJECT_ROOT"

        check_deps

        if [ "$FORCE_BUILD" = true ] || [ ! -d "frontend/node_modules" ]; then
            build_frontend
        fi

        start_server

        info "───────────────────────────────────────────"
        info " Tengu is running on http://localhost:${PORT:-8080}"
        info "───────────────────────────────────────────"
        echo ""

        wait "$RUST_PID"
        ;;

    *)
        err "Comando desconocido: $COMMAND"
        exit 1
        ;;
esac
