# Samurai Terminal Edition (samurai-tui)

Terminal UI for the Samurai cybersecurity platform. Port scanning, web reconnaissance, vulnerability crawling, and database export. Works standalone with zero setup — no Docker, no PostgreSQL required.

**Version:** 2.5.0

---

## Quick Start (Standalone)

```bash
cd samurai-tui
cargo build --release
cargo run --release
```

The application auto-creates a `samurai.db` SQLite file and is immediately ready.

> **Requirements:** Rust 1.80+ and `nmap` in PATH. Internet access for web recon/crawling.

---

## Docker Usage

### Build the Image

```bash
docker compose build --no-cache
```

### Persistent Session (data survives restarts)

```bash
docker compose up --build
```

- Data stored in a named Docker volume (`samurai-tui-data`)
- Survives container restarts and re-creates
- Use `docker compose up -d` to run in background, then `docker attach samurai-tui` to connect

### Ephemeral Session (no traces left)

```bash
docker compose run --rm -it samurai-tui
```

- **`--rm`** destroys container + data on exit
- **`-it`** is **required** — allocates the interactive TTY that the TUI needs
- Without `-it`, the app starts and exits immediately (no terminal available)
- The container is ephemeral: each run is a fresh environment

### Why `-it` is Required

The app uses `crossterm` raw mode + `ratatui` alternate screen, which demand a real terminal. Without `-it`:

```
$ docker compose run --rm samurai-tui
Container created -> exits immediately  (exit 0, no output)
```

With `-it`:

```
$ docker compose run --rm -it samurai-tui
Container created -> TUI starts -> you interact -> q to quit -> container removed
```

### Run on Remote Docker

```bash
# Build on the machine
docker build -t samurai-tui .
docker save samurai-tui | ssh user@remote 'docker load'

# On the remote
ssh -t user@remote 'docker run -it --rm samurai-tui'
```

The `-t` flag on `ssh` is also required for the same reason — raw terminal access.

---

## Proxy, VPN & Anti-Blocking

To prevent target servers from blocking scans due to abuse, samurai-tui supports routing all network traffic through proxies or VPNs.

All variables are **optional** — without them, traffic goes out direct.

### HTTP/HTTPS Proxy (recon, crawler)

Useful with **Burp Suite**, **mitmproxy**, **Charles**, or any HTTP proxy:

```bash
docker compose run --rm -it \
  -e SAMURAI_PROXY=http://host.docker.internal:8080 \
  samurai-tui
```

In standalone mode:
```bash
SAMURAI_PROXY=http://127.0.0.1:8080 cargo run --release
```

### SOCKS5 + Tor Proxy (IP rotation every 10 s)

The most practical method for rotating IP address. Tor changes its circuit approximately every 10 seconds.

```bash
# 1. Start Tor
docker run -d --rm --name tor-proxy \
  -p 9050:9050 -p 9051:9051 \
  dperson/torproxy

# 2. Run samurai-tui with Tor as proxy
docker compose run --rm -it \
  -e SAMURAI_PROXY_DNS=socks5h://host.docker.internal:9050 \
  -e SAMURAI_NMAP_PROXY=socks4://host.docker.internal:9050 \
  samurai-tui
```

**Explanation of `socks5h://`**: the `h` causes DNS resolution to happen on the proxy side (Tor), not locally. This way, the target never sees your real DNS resolver.

### Proxy for nmap

Nmap supports `--proxies` with SOCKS4. Use the `SAMURAI_NMAP_PROXY` variable:

```bash
docker compose run --rm -it \
  -e SAMURAI_NMAP_PROXY=socks4://host.docker.internal:9050 \
  samurai-tui
```

### Dedicated VPN (separate container)

For complete network isolation, use a VPN container as a sidecar:

```yaml
# docker-compose.override.yml (create alongside the existing docker-compose.yml)
services:
  vpn:
    image: qmcgaw/gluetun  # WireGuard / OpenVPN
    cap_add:
      - NET_ADMIN
    environment:
      - VPN_SERVICE_PROVIDER=mullad
      - VPN_TYPE=wireguard
      - WIREGUARD_PRIVATE_KEY=...
      - WIREGUARD_ADDRESSES=...
    volumes:
      - gluetun:/gluetun

  samurai-tui:
    network_mode: "service:vpn"  # shares the VPN network
    depends_on:
      vpn:
        condition: service_healthy
    environment:
      - SAMURAI_NMAP_PROXY=         # direct traffic through VPN
      - SAMURAI_PROXY=              # (optional, proxy inside the VPN)

volumes:
  gluetun:
```

Then bring everything up together:
```bash
docker compose up --build
```

### Multiple Containers with Different IPs (Manual Rotation)

You can launch several instances with different proxies:

```bash
# Terminal 1 — Tor circuit A
docker compose run --rm -it \
  -e SAMURAI_PROXY_DNS=socks5h://host.docker.internal:9050 \
  samurai-tui

# Terminal 2 — Tor circuit B (different port)
# You need another Tor container on a different port
SAMURAI_PROXY=socks5h://127.0.0.1:9052 cargo run --release
```

---

## How It Works

### Session Lifecycle

```
Start
  │
  ├─ PostgreSQL available? ──► use PostgreSQL pool
  └─ No PostgreSQL? ──────────► fall back to SQLite (auto-create .db file)
        │
  ╔═══════════════════════════════════════╗
  ║    Initialise DB tables (idempotent)  ║
  ║    Create shared HTTP/DNS clients     ║
  ║    Load scan history from DB          ║
  ║    Enter raw mode + alternate screen  ║
  ║    ──────────────────────────────     ║
  ║    Main event loop (33 ms polling):   ║
  ║    • Keyboard input (typing, Tab,     ║
  ║      Enter, arrows, q/Esc...)         ║
  ║    • Mouse clicks (tab selection)     ║
  ║    • Background events (scan logs,    ║
  ║      port discoveries, completion)    ║
  ║    ──────────────────────────────     ║
  ║    q pressed ──► exit loop            ║
  ╚═══════════════════════════════════════╝
        │
  Restore terminal, clean up, exit
```

### Async Task Architecture

All scanning operations run in background Tokio tasks. Results flow back to the UI through an asynchronous event channel:

```
Enter pressed
       │
       ▼
  Set running flags (in App)
  Clear previous results
  Spawn background task
       │
       ▼
  ┌──────────────────┐        mpsc::channel        ┌──────────────────┐
  │  Background Task  │ ──────── BgEvent ─────────► │  Main Event Loop │
  │  (run_scanner,    │     (ScannerLog,            │  (handle_bg_event)│
  │   run_recon,      │      ScannerPort,           │       │          │
  │   run_crawler,    │      ScannerDone,           │       ▼          │
  │   run_export)     │      ReconLog/ReconDone,    │  Update App      │
  │                   │      CrawlerLog/CrawlerDone,│  (ports, results, │
  │  Each task:       │      ExportDone, Status)    │   logs, status)  │
  │  • Runs nmap /    │                             │       │          │
  │    DNS queries /  │                             │       ▼          │
  │    HTTP requests  │                             │  Terminal.draw() │
  │  • Saves to DB    │                             │  (UI re-render)  │
  │  • Sends events   │                             └──────────────────┘
  └──────────────────┘
```

- Events are drained **before and after** each user input poll
- The terminal re-renders every 33 ms
- Running operations can be cancelled with `Esc` (sets `_running = false`, task completes peacefully)

---

## Key Bindings

| Key | Action |
|---|---|
| `Tab` | Switch tab (Scanner → Recon → Crawler → History → Export) |
| `Enter` | Execute scan / recon / crawl / export (or delete selected scan in History) |
| `Left / Right` | Adjust profile (Scanner) or max pages (Crawler) |
| `Up / Down` | Navigate history list |
| `Space` | Toggle export mode (Raw / Encrypted) |
| `Esc` | Cancel running operation |
| `q` | Quit (only when idle) |
| Typing | Enter target, password, or export path |
| `Backspace` | Delete last character |
| **🖱️ Left click** | **Switch tab** (click on tab name at top) |

---

## Tabs

| Tab | Description |
|---|---|
| **Scanner** | Nmap port scanning with 4 profiles. Real-time log streaming + open ports panel. |
| **Recon** | DNS enumeration, subdomain discovery (crt.sh + resolution), API probing (32 paths), security headers check (7), technology fingerprinting. Results summary panel. |
| **Crawler** | Page discovery via HTML link extraction. Configurable max pages (1–20). Status-coded page list. |
| **History** | Browse, inspect, and delete past scans. Severity summary bar, findings with CVSS scores, discovered links with status codes. |
| **Export** | Export database as RAW JSON or AES-256-GCM encrypted binary. |

---

## Scan Profiles

| Profile | Nmap arguments |
|---|---|
| `quick` | `-T4 --min-rate 1000 -sV -sC --top-ports 1000 --host-timeout 180s` |
| `balanced` | `-T4 --min-rate 1000 -sV -sC -p 1-10000 --host-timeout 300s` |
| `deep` | `-T4 --min-rate 1000 -sV -sC -p- --script vuln --host-timeout 600s` |
| `udp` | `-T4 --min-rate 1000 -sU -sV --top-ports 1000 --host-timeout 300s` |

---

## Storage

By default, `samurai-tui` stores data in a local SQLite file (`samurai.db`). No database server needed.

| Scenario | Backend | File |
|---|---|---|
| PostgreSQL available | PostgreSQL | Configurable via env vars |
| No PostgreSQL found | SQLite (auto-fallback) | `samurai.db` (local) or `/data/samurai.db` (Docker) |

Scans performed in the TUI and the Samurai Web version are interoperable when both share the same PostgreSQL database.

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `SAMURAI_DB_BACKEND` | auto-detect | Force: `sqlite` or `postgres` |
| `SAMURAI_SQLITE_PATH` | `samurai.db` | SQLite file path |
| `RUST_LOG` | `info` | `debug`, `info`, `warn`, `error` |
| `DB_HOST` | `localhost` | PostgreSQL host |
| `DB_NAME` | `samurai` | PostgreSQL database name |
| `DB_USER` | `postgres` | PostgreSQL user |
| `DB_PASS` | `postgres` | PostgreSQL password |
| `SAMURAI_PROXY` | *(none)* | HTTP/HTTPS proxy for all HTTP requests (e.g. `http://127.0.0.1:8080`) |
| `SAMURAI_PROXY_DNS` | *(none)* | SOCKS5 proxy with remote DNS resolution for HTTP requests (e.g. `socks5h://127.0.0.1:9050`) |
| `SAMURAI_NMAP_PROXY` | *(none)* | Proxy for nmap scans (passed as `--proxies` to nmap, e.g. `socks4://127.0.0.1:9050`) |

### Logging

```bash
RUST_LOG=debug cargo run --release    # verbose
RUST_LOG=info cargo run --release     # default
RUST_LOG=warn cargo run --release     # quiet
```

---

## Database Export & Cross-Compatibility

### Exporting from Samurai Web to TUI

1. In the web app, navigate to **Export DB** (sidebar `05 // EXPORT DB`)
2. Download as **RAW JSON** — a file named `samurai-database-export-YYYY-MM-DD.json`
3. In the TUI, go to the **Export** tab to export the same data to a file

Both interfaces share identical table schemas (`scans`, `findings`, `discovered_links`). When pointing to the same PostgreSQL instance, data is shared automatically — no export needed.

### Encryption Format

```
SAMURAI_DB_EXPORT_V1 | 16-byte salt | 12-byte nonce | AES-256-GCM ciphertext
```

Key derived via PBKDF2-SHA256 with 600,000 iterations. Cross-compatible with the web backend.

---

## Architecture

```
src/
├── main.rs             # Entry point, event loop, TerminalGuard, keyboard + mouse dispatch
├── app.rs              # Application state (App struct), tab/scanner/recon/crawler/export state
├── tasks.rs            # Background task functions (run_scanner, run_recon, run_crawler, run_export)
│                       # + BgEvent enum for async event channel
├── db/
│   ├── connection.rs   # Dual-backend pool (PostgreSQL + SQLite auto-fallback)
│   ├── models.rs       # Scan, Finding, DiscoveredLink, ExportPayload types
│   └── operations.rs   # CRUD + export payload builder (both backends)
├── scanner/
│   └── engine.rs       # Nmap subprocess: streaming, timeout, safe error handling
├── recon/
│   ├── dns.rs          # A, AAAA, MX, NS, TXT, SOA, CNAME via hickory-resolver
│   ├── subdomains.rs   # crt.sh certificate transparency + DNS resolution
│   ├── api.rs          # 32 API paths probed concurrently
│   ├── headers.rs      # 7 security headers + server disclosure check
│   └── tech_stack.rs   # Precompiled selectors for React, Vue, Angular, jQuery, etc.
├── crawler/
│   └── mod.rs          # HTML parsing, link extraction, concurrent sub-page fetching
├── export/
│   ├── mod.rs          # Export coordinator (raw JSON + encrypted binary)
│   └── crypto.rs       # AES-256-GCM + PBKDF2-SHA256 (600k iterations)
└── tui/
    ├── theme.rs        # Nothing Design dark color palette
    └── ui.rs           # 5-tab rendering with severity badges, mouse areas, log coloring

Data flow:  User Input → Event Loop → Background Task ──(BgEvent channel)──→ App state → Terminal.draw()
```

---

## Design System

Dark instrument-panel aesthetic following the Nothing Design philosophy:

| Color | Hex | Usage |
|---|---|---|
| Background | `#000000` | OLED black |
| Surface | `#111111` | Panels, cards |
| Surface Raised | `#1A1A1A` | Active selection |
| Text | `#E8E8E8` | Primary body |
| Gold | `#FFD700` | Open ports |
| Red | `#D71921` | Critical, errors |
| Green | `#4A9E5C` | Success, completed |
| Amber | `#D4A843` | Warnings, running |
| Blue | `#5B9BF6` | Interactive |

---

## Dependencies

| Crate | Purpose |
|---|---|
| `ratatui` + `crossterm` | Terminal UI + raw mode + mouse events |
| `tokio` | Async runtime (tasks, channels, timers) |
| `sqlx` (postgres + sqlite) | Dual-backend storage |
| `reqwest` | HTTP client (shared, pooled, certificate relaxation) |
| `hickory-resolver` | DNS resolution (shared, cached) |
| `scraper` | HTML parsing for crawler |
| `aes-gcm` + `pbkdf2` + `sha2` | Encryption |
| `serde` + `serde_json` | Serialization |
| `futures` | Concurrency primitives |
| `clap` | CLI argument parsing |
| `chrono` | Timestamp formatting |
