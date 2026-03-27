#!/bin/bash

# Define colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== xwa Dashboard Startup Script ===${NC}"

export PYTHONPATH=$(pwd)
export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-"http://localhost:8000"}
export PORT=${PORT:-3000}
export WATCHPACK_POLLING=true
export CHOKIDAR_USEPOLLING=1
export NEXT_TELEMETRY_DISABLED=1

# Detect OS type
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS_ID="$ID"
        OS_ID_LIKE="$ID_LIKE"
    elif [ -f /etc/arch-release ]; then
        OS_ID="arch"
    else
        OS_ID="unknown"
    fi
}

# Install system dependencies based on OS
install_system_deps() {
    detect_os
    echo -e "${YELLOW}Detected OS: ${OS_ID}${NC}"
    
    # Get Python version for version-specific packages
    local py_version
    py_version=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")' 2>/dev/null || echo "3")

    case "$OS_ID" in
        ubuntu|debian)
            echo -e "${YELLOW}Installing system dependencies for Ubuntu/Debian (Python ${py_version})...${NC}"
            sudo apt update
            sudo apt install -y "python${py_version}-venv" python3-full lsof nodejs npm || \
            sudo apt install -y python3-venv python3-full lsof nodejs npm
            ;;
        arch|manjaro)
            echo -e "${YELLOW}Installing system dependencies for Arch Linux...${NC}"
            sudo pacman -Sy --noconfirm python python-pip lsof nodejs npm
            ;;
        *)
            if [[ "$OS_ID_LIKE" == *"debian"* ]] || [[ "$OS_ID_LIKE" == *"ubuntu"* ]]; then
                echo -e "${YELLOW}Installing system dependencies for Debian-based system (Python ${py_version})...${NC}"
                sudo apt update
                sudo apt install -y "python${py_version}-venv" python3-full lsof nodejs npm || \
                sudo apt install -y python3-venv python3-full lsof nodejs npm
            elif [[ "$OS_ID_LIKE" == *"arch"* ]]; then
                echo -e "${YELLOW}Installing system dependencies for Arch-based system...${NC}"
                sudo pacman -Sy --noconfirm python python-pip lsof nodejs npm
            else
                echo -e "${RED}Unknown OS. Please install python3-venv, lsof, nodejs, and npm manually.${NC}"
                exit 1
            fi
            ;;
    esac
}

# Check if required commands exist
check_requirements() {
    local missing=0
    
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}python3 not found${NC}"
        missing=1
    fi
    
    if ! command -v node &> /dev/null; then
        echo -e "${RED}node not found${NC}"
        missing=1
    fi
    
    if ! command -v npm &> /dev/null; then
        echo -e "${RED}npm not found${NC}"
        missing=1
    fi
    
    # Check if python3-venv is available
    if ! python3 -c "import ensurepip" &> /dev/null; then
        echo -e "${RED}python3-venv not available${NC}"
        missing=1
    fi
    
    return $missing
}

# Initial dependency check
echo -e "\n${GREEN}[0/3] Checking System Dependencies...${NC}"
if ! check_requirements; then
    echo -e "${YELLOW}Some system dependencies are missing. Attempting to install...${NC}"
    install_system_deps
    if ! check_requirements; then
        echo -e "${RED}Failed to install system dependencies. Please install them manually.${NC}"
        exit 1
    fi
fi
echo "System dependencies OK."

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

# 1. Clean and recreate Python virtual environment
echo -e "\n${GREEN}[1/3] Setting Up Python Backend Dependencies...${NC}"
if [ -d "venv" ]; then
    echo -e "${YELLOW}Removing existing virtual environment...${NC}"
    rm -rf venv
fi
echo -e "${YELLOW}Creating fresh virtual environment 'venv'...${NC}"
python3 -m venv venv
if [ ! -f "venv/bin/activate" ]; then
    echo -e "${RED}Failed to create virtual environment. The venv was not created properly.${NC}"
    echo -e "${RED}Try running: sudo apt install python3.12-venv python3-full${NC}"
    exit 1
fi
source venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt
echo "Python environment Ready."

# 2. Clean and reinstall Node.js dependencies
echo -e "\n${GREEN}[2/3] Setting Up Node.js Frontend Dependencies...${NC}"
cd web
if [ -d "node_modules" ]; then
    echo -e "${YELLOW}Removing existing node_modules...${NC}"
    rm -rf node_modules
fi
echo -e "${YELLOW}Installing Node.js dependencies...${NC}"
npm install
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
