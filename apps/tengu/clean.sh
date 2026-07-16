#!/usr/bin/env bash
set -euo pipefail

readonly GRN='\033[0;32m'
readonly BLU='\033[0;34m'
readonly YLW='\033[1;33m'
readonly RED='\033[0;31m'
readonly NC='\033[0m'

log()  { echo -e "${GRN}[tengu]${NC} $1"; }
info() { echo -e "${BLU}[info]${NC} $1"; }
warn() { echo -e "${YLW}[warn]${NC} $1"; }
err()  { echo -e "${RED}[err]${NC} $1"; }

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"

clean_local_build() {
    info "Cleaning local build artifacts..."
    rm -rf "$PROJECT_ROOT/target" 2>/dev/null && info "  Removed target/"
    rm -rf "$PROJECT_ROOT/frontend/node_modules" 2>/dev/null && info "  Removed frontend/node_modules/"
    rm -rf "$PROJECT_ROOT/frontend/.angular" 2>/dev/null && info "  Removed frontend/.angular/"
    log "Local build cleaned"
}

stop_containers() {
    info "Stopping tengu containers..."
    if [ -f "$PROJECT_ROOT/docker-compose.yml" ]; then
        docker compose -f "$PROJECT_ROOT/docker-compose.yml" down --remove-orphans 2>/dev/null && log "Containers stopped" || warn "No containers were running"
    else
        local running
        running=$(docker ps --filter "name=tengu" -q 2>/dev/null)
        if [ -n "$running" ]; then
            docker stop $running 2>/dev/null && log "Containers stopped" || true
        fi
    fi
}

remove_volumes() {
    info "Removing tengu volumes..."
    local vols
    vols=$(docker volume ls -q --filter "name=tengu" 2>/dev/null)
    if [ -n "$vols" ]; then
        echo "$vols" | while read -r vol; do
            docker volume rm "$vol" 2>/dev/null && info "  Removed volume: $vol" || warn "  Could not remove volume: $vol (may be in use)"
        done
    else
        info "No tengu volumes found"
    fi
}

remove_images() {
    info "Removing tengu images..."
    local images
    images=$(docker images -q --filter "reference=tengu" 2>/dev/null; docker images -q --filter "reference=tengu-*" 2>/dev/null; docker images -q --filter "reference=tengu_*" 2>/dev/null)
    images=$(echo "$images" | sort -u)
    if [ -n "$images" ]; then
        echo "$images" | while read -r img; do
            [ -z "$img" ] && continue
            docker rmi "$img" 2>/dev/null && info "  Removed image: $img" || warn "  Could not remove image: $img (may be in use by a container)"
        done
    else
        info "No tengu images found"
    fi
}

confirm() {
    warn "This will remove ALL tengu containers, volumes, and images."
    read -r -p "Are you sure? [y/N] " reply
    case "$reply" in
        [yY]|[yY][eE][sS]) return 0 ;;
        *) err "Aborted"; exit 1 ;;
    esac
}

show_help() {
    cat <<EOF
${GRN}tengu -- docker cleanup${NC}

Usage: ./clean.sh [options]

Options:
  -a, --all      Remove containers, volumes, images, AND local builds (target, node_modules) — with confirmation
  -f, --force    Same as -a but WITHOUT confirmation
  -v, --volumes  Remove Docker volumes only (retain images)
  -h, --help     Show this help

Without options, removes containers, target/, node_modules/ and .angular/ cache.
EOF
    exit 0
}

CLEAN_ALL=false
CLEAN_FORCE=false
CLEAN_VOLUMES=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -a|--all) CLEAN_ALL=true; shift ;;
        -f|--force) CLEAN_FORCE=true; shift ;;
        -v|--volumes) CLEAN_VOLUMES=true; shift ;;
        -h|--help) show_help ;;
        *) err "Unknown option: $1"; show_help ;;
    esac
done

clean_local_build

if command -v docker &>/dev/null; then
    stop_containers
else
    info "Docker not available, skipping container/image cleanup"
fi

if [ "$CLEAN_ALL" = true ] || [ "$CLEAN_FORCE" = true ]; then
    if [ "$CLEAN_FORCE" != true ]; then
        confirm
    fi
    remove_volumes
    remove_images
    log "Everything removed: containers, volumes, images, and local builds."
elif [ "$CLEAN_VOLUMES" = true ]; then
    remove_volumes
    log "Containers and volumes removed. Images and local builds retained."
else
    log "Containers, local builds removed. Volumes and images retained."
fi
