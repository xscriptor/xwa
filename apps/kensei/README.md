
<h1 align="center">Kensei</h1>

<div align="center">
<img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/kensei/kensei-colors.svg" width="120"/> 
</div>

**Language / Idioma**  
[English](#) | [Español](./docs/esp/README.md)

<p><em><a href="https://github.com/xscriptor/kensei">Kensei</a></em> : <em><a href="https://github.com/xscriptor/xwa">XWA</a>  <strong>submodule focused</strong> on web technology stack profiling — under active development</em></p>

<hr>

<h2>Overview</h2>

<p>Kensei is a web technology stack profiler with two interfaces sharing the same database:</p>

<table>
  <tr>
    <th>Interface</th>
    <th>Directory</th>
    <th>Language</th>
    <th>Type</th>
  </tr>
  <tr>
    <td><strong>Kensei Web</strong></td>
    <td><code>/frontend</code> + <code>/backend</code></td>
    <td>Angular 21 + FastAPI/Python</td>
    <td>Web application (Docker)</td>
  </tr>
</table>

<h3>Capabilities</h3>
<ul>
  <li><strong>Server Fingerprinting</strong> — HTTP header analysis, SSL/TLS handshake profiling, server banner detection</li>
  <li><strong>JS Bundle Analysis</strong> — Dependency extraction, version detection, library fingerprinting from JavaScript bundles</li>
  <li><strong>SPA Route Discovery</strong> — Angular/React/Vue route enumeration, lazy-loaded module detection, client-side path mapping</li>
  <li><strong>Technology Inventory</strong> — Full stack composition report (frontend frameworks, backend servers, CDN, analytics, third-party services)</li>
  <li><strong>History & Archive</strong> — Persistent profile storage with change detection over time</li>
</ul>

<hr>

<h2>Quick Start</h2>

<h3>Web Version (Docker Compose)</h3>
<pre><code>docker compose up -d --build</code></pre>
<ul>
  <li>Frontend: <code>http://localhost:4200</code></li>
  <li>API docs: <code>http://localhost:8000/docs</code></li>
</ul>

<h3>Launch Script (kensei.sh)</h3>
<p>The <code>kensei.sh</code> script handles all launch modes, dependency checks, and infra setup:</p>

<table>
  <tr><th>Command</th><th>Mode</th><th>Infrastructure</th><th>Use case</th></tr>
  <tr>
    <td><code>./kensei.sh</code></td>
    <td>Docker Compose</td>
    <td>Automatic (containers)</td>
    <td>Production-like, fully isolated</td>
  </tr>
  <tr>
    <td><code>./kensei.sh --sqlite</code></td>
    <td>Native (SQLite)</td>
    <td>None — zero infra</td>
    <td>Fast dev, no Docker needed</td>
  </tr>
  <tr>
    <td><code>./kensei.sh --native</code></td>
    <td>Native (venv + node)</td>
    <td>Auto-starts PG via Docker</td>
    <td>Development with PG, hot-reload</td>
  </tr>
  <tr>
    <td><code>./kensei.sh --native-no-infra</code></td>
    <td>Native (venv + node)</td>
    <td>You manage PG</td>
    <td>Custom infra setup</td>
  </tr>
</table>

<p>Press <kbd>Ctrl+C</kbd> to stop all services cleanly.</p>

<h3>Cleanup (clean.sh)</h3>
<pre><code>./clean.sh</code></pre>
<p>Kills leftover processes, removes Docker containers/volumes/images, deletes <code>node_modules/</code>, <code>.venv</code>, <code>dist/</code>, Python cache, and <code>.angular/</code> cache.</p>

<hr>

<h2>Related Documents</h2>

<table>
  <tr><th>Document</th><th>Description</th></tr>
  <tr><td><a href="ROADMAP.md">ROADMAP.md</a></td><td>Development phases and milestones</td></tr>
</table>

<hr>

<h2>Project Structure</h2>

<pre><code>kensei/
├── frontend/              # Angular 21 SPA (standalone components)
├── backend/               # FastAPI Python (REST + WebSocket)
│   └── app/
│       ├── main.py        # API routes and WebSocket endpoints
│       ├── database.py    # SQLAlchemy engine and session
│       ├── models.py      # Scan, Finding, Technology models
│       ├── profiler.py    # Main profiling orchestrator
│       └── modules/       # Profiling modules
│           ├── server_fingerprint.py  # HTTP/SSL server detection
│           ├── js_analyzer.py         # JS bundle analysis
│           └── spa_discovery.py       # SPA route enumeration
├── docs/                  # Technical documentation
├── kensei.sh              # Launch script (Docker / native)
├── clean.sh               # Cleanup script
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
