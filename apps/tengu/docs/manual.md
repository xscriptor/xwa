# Tengu Manual

## Deployment

### Docker (recommended)

```bash
docker compose up -d --build
```

Web UI at `http://localhost:8080`.

### Standalone

```bash
cargo run --release
```

Requires Rust toolchain (1.81+). Frontend is built automatically at compile time via `build.rs`. Web UI at `http://localhost:8080`.

### Launch Script

```bash
./tengu.sh                       # Standalone ephemeral (no disk state)
./tengu.sh --docker               # via docker-compose
./tengu.sh --docker -e            # Docker ephemeral (--rm, auto-cleanup)
./tengu.sh --export file.json     # Save audit export to ./exports/
./tengu.sh --import file.json     # Import audit JSON into running instance
./tengu.sh --build                # Build only (no run)
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8080` | HTTP listen port |
| `RUST_LOG` | `tengu=info,tower_http=info` | Logging verbosity (uses env-filter format) |

## Usage

### Single URL Audit

1. Enter a URL in the input field
2. Select the audit categories (Performance, SEO, Accessibility, Best Practices)
3. Click START AUDIT
4. Results stream in real-time via WebSocket

### Full Site Audit

1. Toggle FULL SITE mode
2. Optionally enable INCLUDE SUBDOMAINS
3. Enter the starting URL
4. Tengu crawls discovered pages (up to 50) and audits each one

### Understanding Results

Each finding displays:
- **Severity**: Error, Warning, Info, or Pass
- **Check**: The specific audit check name
- **Title**: Summary of the issue
- **Description**: Detailed explanation with recommendations
- **Snippet**: The relevant HTML element (if applicable)
- **Line**: Line number in the pretty-printed HTML source

### HTML Source Viewer

After an audit completes, click VIEW HTML to see the pretty-printed source. Lines with findings are highlighted with a red left border.

### Export

Results can be exported in five formats:

| Format | Extension | Content |
|---|---|---|
| CSV | `.csv` | Tabular data with all fields |
| JSON | `.json` | Full payload with metadata |
| PDF | `.pdf` | Landscape report with findings table |
| HTML | `.html` | Self-contained styled report |
| Markdown | `.md` | Lightweight text report |

## API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/health` | Health check |
| `GET` | `/api/audit/live` | WebSocket for real-time audit |
| `GET` | `/api/audits` | List past audits |
| `GET` | `/api/audits/:id` | Get audit details |
| `DELETE` | `/api/audits/:id` | Delete audit |
| `GET` | `/api/audits/export` | Export all audits as JSON |
| `POST` | `/api/audits/import` | Import audits from JSON |

## Troubleshooting

### WebSocket connection fails
Ensure the port is reachable and no firewall blocks WebSocket upgrades. Check `RUST_LOG=debug` for request details.

### Audit returns no findings
The audit parses the HTML after pretty-printing. If the page is empty, behind a login wall, or blocks bots, results may be empty. Try a publicly accessible page.

### Build fails
- Rust: ensure 1.81+ with `rustup update`
- Frontend: `cd frontend && npm ci` to install dependencies
- Clean build: `./clean.sh` then retry
