# xwa - Local Development Setup Guide

This guide provides step-by-step instructions for deploying and running the **xwa** Web Analysis Dashboard on your local machine. Supports **Ubuntu/Debian**, **Arch Linux**, and **WSL** environments.

---

## System Requirements

- **Python 3.10+**
- **Node.js 18+** with npm
- **Git**

---

## Quick Start (Recommended)

The easiest way to run xwa is using the automated startup script:

```bash
# Clone the repository
git clone https://github.com/xscriptor/xwa.git
cd xwa

# Run the startup script (handles everything automatically)
./start.sh
```

The script will:
1. Detect your operating system (Ubuntu/Debian/Arch/WSL)
2. Install missing system dependencies (requires sudo)
3. Create a Python virtual environment
4. Install Python dependencies
5. Install Node.js dependencies
6. Start both backend and frontend servers

Once complete, access:
- **Frontend Dashboard:** http://localhost:3000
- **API Documentation:** http://localhost:8000/docs

---

## Manual Installation

If you prefer to set things up manually or the automated script doesn't work for your system, follow these steps:

### Step 1: Install System Dependencies

#### Ubuntu / Debian / WSL (Ubuntu-based)

```bash
sudo apt update
sudo apt install -y python3 python3-venv python3-full lsof nodejs npm
```

#### Arch Linux / Manjaro / WSL (Arch-based)

```bash
sudo pacman -Sy --noconfirm python python-pip lsof nodejs npm
```

### Step 2: Clone the Repository

```bash
git clone https://github.com/xscriptor/xwa.git
cd xwa
```

### Step 3: Set Up Python Backend

```bash
# Create virtual environment
python3 -m venv venv

# Activate virtual environment
source venv/bin/activate

# Install Python dependencies
pip install --upgrade pip
pip install -r requirements.txt
```

### Step 4: Set Up Node.js Frontend

```bash
cd web
npm install
cd ..
```

### Step 5: Start the Servers

You need two terminal windows/tabs:

**Terminal 1 - Backend (FastAPI):**

```bash
cd xwa
source venv/bin/activate
uvicorn api.main:app --port 8000
```

**Terminal 2 - Frontend (Next.js):**

```bash
cd xwa/web
npm run dev
```

---

## Stopping the Application

### If Started with `./start.sh`

Press **Ctrl+C** in the terminal where `start.sh` is running. The script will gracefully shut down both servers.

### If Started Manually

**Stop the Backend:**
- Press **Ctrl+C** in the terminal running `uvicorn`

**Stop the Frontend:**
- Press **Ctrl+C** in the terminal running `npm run dev`

### Force Stop (if servers are unresponsive)

Find and kill processes by port:

```bash
# Find and kill process on port 8000 (Backend)
lsof -ti:8000 | xargs kill -9

# Find and kill process on port 3000 (Frontend)
lsof -ti:3000 | xargs kill -9
```

Or kill by process name:

```bash
# Kill uvicorn (Backend)
pkill -f uvicorn

# Kill node (Frontend)
pkill -f "next dev"
```

---

## Environment Variables

The following environment variables can be customized:

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | Frontend port |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8000` | Backend API URL |

Example:

```bash
PORT=4000 NEXT_PUBLIC_API_URL=http://localhost:8080 ./start.sh
```

---

## Troubleshooting

### "python3-venv not available" Error

**Ubuntu/Debian:**
```bash
sudo apt install python3-venv python3-full
```

**Arch:**
```bash
sudo pacman -S python
```

### "uvicorn: command not found"

Ensure the virtual environment is activated:
```bash
source venv/bin/activate
```

Or reinstall dependencies:
```bash
source venv/bin/activate
pip install -r requirements.txt
```

### "Port already in use"

Kill the existing process:
```bash
lsof -ti:8000 | xargs kill -9  # For port 8000
lsof -ti:3000 | xargs kill -9  # For port 3000
```

### WSL-Specific Issues

If running in WSL and experiencing network issues:

1. Ensure WSL is updated:
   ```bash
   wsl --update
   ```

2. Access the app using the WSL IP address shown in the terminal output, or use `localhost` if WSL networking is properly configured.

---

## Development Notes

- Backend auto-reloads with `--reload` flag: `uvicorn api.main:app --reload --port 8000`
- Frontend auto-reloads by default with `npm run dev`
- Database file is stored at `xwa.db` in the project root
