
<h1 align="center">Samurai</h1>

<div align="center">
<img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-colors.svg" width="120"/> 
</div>

**Language / Idioma**  
[English](#) | [Español](./docs/esp/README.md)

<p><em><a href="https://github.com/xscriptor/samurai">Samurai</a></em> : <em><a href="https://github.com/xscriptor/xwa">XWA</a>  <strong>submodule focused</strong> on web cybersecurity — under active development</em></p>

<img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-01.png" alt="Samurai XWA Screenshot 01" width="100%">

<details>
  <summary>More screenshots...</summary>
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-02.png" alt="Samurai XWA Screenshot 02" width="100%">
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-03.png" alt="Samurai XWA Screenshot 03" width="100%">
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-04.png" alt="Samurai XWA Screenshot 04" width="100%">
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-05.png" alt="Samurai XWA Screenshot 05" width="100%">
</details>

<hr>

<h2>Overview</h2>

<p>Samurai is a cybersecurity analysis platform with two interfaces sharing the same database:</p>

<table>
  <tr>
    <th>Interface</th>
    <th>Directory</th>
    <th>Language</th>
    <th>Type</th>
  </tr>
  <tr>
    <td><strong>Samurai Web</strong></td>
    <td><code>/frontend</code> + <code>/backend</code></td>
    <td>Angular 21 + FastAPI/Python</td>
    <td>Web application (Docker)</td>
  </tr>
  <tr>
    <td><strong>Samurai TUI</strong></td>
    <td><code>/samurai-tui</code></td>
    <td>Rust</td>
    <td>Terminal application (standalone or Docker)</td>
  </tr>
</table>

<h3>Capabilities</h3>
<ul>
  <li><strong>Port Scanning</strong> — Nmap with configurable profiles (quick, balanced, deep, UDP)</li>
  <li><strong>Web Reconnaissance</strong> — DNS enumeration, subdomain discovery, API probing, security headers audit, technology fingerprinting</li>
  <li><strong>Vulnerability Crawling (DAST)</strong> — Page discovery, HTTP header analysis, link extraction</li>
  <li><strong>Database Export</strong> — Full analytics dump as JSON (raw) or AES-256-GCM encrypted binary</li>
  <li><strong>History & Archive</strong> — Persistent scan storage with findings and discovered topology</li>
</ul>

<h3>Database Export & Cross-Compatibility</h3>
<p>The web backend exports the database via <code>GET /api/database/export/raw</code> (JSON) and <code>POST /api/database/export/encrypted</code> (AES-256-GCM binary). The TUI exports via the <strong>Export</strong> tab using the same encryption format (<code>SAMURAI_DB_EXPORT_V1</code>). Both interfaces share identical table schemas (<code>scans</code>, <code>findings</code>, <code>discovered_links</code>).</p>
<p>When both point to the same PostgreSQL database, exports are unnecessary — scans appear in both interfaces automatically. The TUI can also run standalone with SQLite, storing data locally in <code>samurai.db</code>.</p>

<hr>

<h2>Quick Start</h2>

<h3>Web Version (Docker Compose)</h3>
<pre><code>docker compose up -d --build</code></pre>
<ul>
  <li>Frontend: <code>http://localhost:4200</code></li>
  <li>API docs: <code>http://localhost:8000/docs</code></li>
</ul>

<h3>Terminal Version (Standalone)</h3>
<pre><code>cd samurai-tui
cargo build --release
cargo run --release</code></pre>
<p>Zero setup. Auto-creates a local SQLite database (<code>samurai.db</code>). No Docker required.</p>

<h3>Terminal Version (Docker — Ephemeral Session)</h3>
<pre><code>cd samurai-tui
docker compose up --build
# Session is destroyed on exit — no data persisted</code></pre>
<p>See <a href="samurai-tui/README.md">samurai-tui/README.md</a> for full Docker configuration and persistent volume options.</p>

<h3>Launch Script (samurai.sh)</h3>
<p>The <code>samurai.sh</code> script handles all launch modes, dependency checks, and infra setup:</p>

<table>
  <tr><th>Command</th><th>Mode</th><th>Infrastructure</th><th>Use case</th></tr>
  <tr>
    <td><code>./samurai.sh</code></td>
    <td>Docker Compose</td>
    <td>Automatic (containers)</td>
    <td>Production-like, fully isolated</td>
  </tr>
  <tr>
    <td><code>./samurai.sh --native</code></td>
    <td>Native (venv + node)</td>
    <td>Auto-starts PG &amp; Redis via Docker or native</td>
    <td>Development, hot-reload</td>
  </tr>
  <tr>
    <td><code>./samurai.sh --native-no-infra</code></td>
    <td>Native (venv + node)</td>
    <td>You manage PG &amp; Redis</td>
    <td>Custom infra setup</td>
  </tr>
</table>

<p>Respects existing <code>DB_HOST</code>, <code>REDIS_URL</code>, <code>REDIS_HOST</code> when set externally. Press <kbd>Ctrl+C</kbd> to stop all services cleanly. Non-Docker infra installs were removed — if Docker is unavailable use <code>--native-no-infra</code> with your own PG/Redis.</p>

<h3>Cleanup (clean.sh)</h3>
<pre><code>./clean.sh</code></pre>
<p>Kills leftover processes, removes Docker containers/volumes/images, deletes <code>node_modules/</code>, <code>.venv</code>, <code>dist/</code>, Python cache, and <code>.angular/</code> cache.</p>

<hr>

<h2>Related Documents</h2>

<table>
  <tr><th>Document</th><th>Description</th></tr>
  <tr><td><a href="docs/manual.md">docs/manual.md</a></td><td>Development and production deployment guide</td></tr>
  <tr><td><a href="docs/ui-architecture.md">docs/ui-architecture.md</a></td><td>Frontend feature-driven architecture specification</td></tr>
  <tr><td><a href="docs/python-libraries.md">docs/python-libraries.md</a></td><td>Backend Python dependency inventory</td></tr>
  <tr><td><a href="docs/uses/dast.md">docs/uses/dast.md</a></td><td>DAST vulnerability scanning usage</td></tr>
  <tr><td><a href="samurai-tui/README.md">samurai-tui/README.md</a></td><td>Terminal application: installation, configuration, Docker, usage</td></tr>
  <tr><td><a href="ROADMAP.md">ROADMAP.md</a></td><td>Development phases and milestones</td></tr>
</table>

<hr>

<h2>Project Structure</h2>

<pre><code>samurai/
├── frontend/              # Angular 21 SPA (standalone components)
├── backend/               # FastAPI Python (REST + WebSocket)
│   └── app/
│       ├── main.py        # API routes and WebSocket endpoints
│       ├── scanner.py     # Nmap port scanning engine
│       ├── crawler.py     # DAST vulnerability crawler
│       ├── db_exporter.py # Database export (raw + encrypted)
│       └── recon/         # Web reconnaissance modules
├── samurai-tui/           # Rust terminal application
│   ├── Dockerfile         # Container build (Rust + nmap)
│   ├── docker-compose.yml # Ephemeral session runner
│   └── src/
│       ├── main.rs        # TUI event loop and keybindings
│       ├── scanner/       # Nmap engine with streaming
│       ├── recon/         # DNS, subdomains, APIs, headers, tech
│       ├── crawler/       # HTTP page discovery
│       ├── export/        # Raw + AES-256-GCM encrypted export
│       ├── db/            # SQLx dual-backend (Postgres + SQLite)
│       └── tui/           # Nothing Design terminal UI
├── docs/                  # Technical documentation
├── samurai.sh             # Launch script (Docker / native / full-native)
├── clean.sh               # Cleanup script (containers, caches, artifacts)
└── docker-compose.yml     # 4 services: frontend, backend, redis, postgres
</code></pre>

<div id="x" align="center">
<h2>X</h2>

<a href="https://dev.xscriptor.com">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/verified-filled.svg" width="24" alt="X Web" />
</a>
 & 
<a href="https://github.com/xscriptor">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/github.svg" width="24" alt="X Github Profile" />
</a>
 & 
<a href="https://www.xscriptor.com">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/quotes.svg" width="24" alt="Xscriptor web" />
</a>

</div>
