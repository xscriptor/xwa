# xwa - Setup & Installation Guide

This guide explains how to run the **xwa** Web Analysis Dashboard locally. The architecture is divided into two separate services that need to run concurrently:
1. **Python FastAPI Backend** (Core Engine + API)
2. **Next.js Web Frontend** (User Interface)

---

## Prerequisites

Ensure you have the following installed on your system:
- **Python 3.10+** (for the Core Engine and FastAPI)
- **Node.js 18+** & **npm** (for the Next.js frontend)

---

## 1. Starting the Backend (FastAPI + SQLModel)

**Note:** ./start.sh starts all for you. 

The backend handles all the heavy lifting: SEO analysis, concurrent sitemap crawling, and passive security audits.

Open a terminal at the root of the project (`xwa/`):

```bash
# 1. Activate the Python virtual environment
source venv/bin/activate

# 2. Start the FastAPI server using Uvicorn
# The --reload flag automatically restarts the server if you modify python files.
uvicorn api.main:app --reload --port 8000
```

> **Note:** The backend will be available at `http://localhost:8000`. You can test if it's running by visiting `http://localhost:8000/docs` in your browser to see the automatic Swagger API documentation.

---

## 2. Starting the Frontend (Next.js)

The frontend provides the premium Glassmorphism Dashboard to submit URLs and view real-time Server-Sent Events (SSE) progress.

Open a **second terminal window**, navigate to the `web/` directory:

```bash
# 1. Go into the web directory
cd web/

# 2. Start the Next.js development server
npm run dev
```

> **Note:** The frontend will be available at `http://localhost:3000`.

---

## Ready to Scan!

Once both servers are running:
1. Open your browser and go to: **[http://localhost:3000](http://localhost:3000)**
2. Enter a URL (e.g., `https://duckduckgo.com` or `https://example.com`) in the sleek input box.
3. Click **Initialize Scan**.
4. You will be redirected to the Report Dashboard where you can watch the engine run its checks in real-time. Once finished, it will display the metrics cards and give you options to export the results as JSONC or Markdown.
