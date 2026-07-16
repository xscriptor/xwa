
<h1 align="center">Shinobi</h1>

<div align="center">
<img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/shinobi/shinobi-colors.svg" width="150"/> 
<p><em>Silent Web Scraper — Anti-Blocking Download System</em></p>
</div>




**Language / Idioma**  
[English](#) | [Español](./docs/esp/README.md)

<p><em><a href="https://github.com/xscriptor/xwa">XWA</a>  <strong>submodule focused</strong> on silent web scraping with anti-blocking — under active development</em></p>

<hr>

<p>
<a href="#overview">Overview</a> ·
<a href="#capabilities">Capabilities</a> ·
<a href="#anti-blocking-system">Anti-Blocking</a> ·
<a href="#quick-start">Quick Start</a> ·
<a href="#launch-script-recommended">Launch Script</a> ·
<a href="#web-version-docker-compose">Docker Compose</a> ·
<a href="#standalone-no-docker">Standalone</a> ·
<a href="#execution-modes">Modes</a> ·
<a href="#environment-variables">Env Variables</a> ·
<a href="#related-documents">Docs</a>
</p>

<hr>

<h2>Overview</h2>

<p>Shinobi is a stealth web scraper with a single web interface. It downloads entire sites — HTML, CSS, JS, images, PDFs — while evading detection through multiple anti-blocking layers.</p>

<table>
  <tr>
    <th>Interface</th>
    <th>Directory</th>
    <th>Language</th>
    <th>Type</th>
  </tr>
  <tr>
    <td><strong>Shinobi Web</strong></td>
    <td><code>/</code> (monorepo root)</td>
    <td>Rust (Axum) + Angular 19</td>
    <td>Web application (standalone or Docker)</td>
  </tr>
</table>

<h3>Capabilities</h3>
<ul>
  <li><strong>Recursive Crawling</strong> — BFS page discovery with configurable depth and max pages</li>
  <li><strong>JavaScript Rendering</strong> — Headless Chromium engine for SPA/React/Vue/Angular sites</li>
  <li><strong>Asset Download</strong> — HTML, CSS, JS, images, PDFs, archives, media, fonts</li>
  <li><strong>File Type Filtering</strong> — Select which file extensions to download</li>
  <li><strong>Same-Domain Scoping</strong> — Stay within target domain or crawl freely</li>
  <li><strong>Real-Time Progress</strong> — SSE live stream with pages scraped, files downloaded, current URL</li>
  <li><strong>Downloaded File Browser</strong> — Browse and open downloaded files from the web UI</li>
  <li><strong>Two Operation Modes:</strong>
    <ul>
      <li><em>Fast Test</em> — Pure Rust, zero external deps, crawl + download + anti-blocking</li>
      <li><em>Deep Research</em> — Python sidecar adds structured data extraction (JSON-LD, microdata, Open Graph, RDFa), NLP analysis (summary, entities, keywords), custom CSS selectors, and enriched email/phone extraction</li>
    </ul>
  </li>
</ul>

<h3>Anti-Blocking System</h3>
<ul>
  <li><strong>User-Agent Rotation</strong> — 15 real browser UAs (Chrome, Firefox, Safari, Edge, mobile)</li>
  <li><strong>Header Randomization</strong> — Accept, Accept-Language, Sec-CH-UA, Sec-Fetch-* per request</li>
  <li><strong>Request Delay + Jitter</strong> — Configurable base delay with random jitter</li>
  <li><strong>Exponential Backoff</strong> — Retry with jitter on failures (configurable attempts)</li>
  <li><strong>Rate Limit Handling</strong> — Detects HTTP 429/503, waits, retries with longer backoff</li>
  <li><strong>Proxy Support</strong> — HTTP/HTTPS/SOCKS5 proxy rotation</li>
</ul>

<hr>

<h2>Quick Start</h2>

<h3>Launch Script (Recommended)</h3>
<pre><code>./shinobi.sh --fast       # Rust backend + frontend (Fast Test mode)
./shinobi.sh --deep        # Rust + Python extractor (Fast Test + Deep Research)
./shinobi.sh --docker      # Everything via docker-compose
./shinobi.sh --help        # Full usage help</code></pre>

<h3>Web Version (Docker Compose)</h3>
<pre><code>docker compose up -d --build</code></pre>
<ul>
  <li>Web UI: <code>http://localhost:8080</code></li>
  <li>Extractor API: <code>http://localhost:9090</code> (Deep Research only)</li>
</ul>

<h3>Standalone (No Docker)</h3>
<pre><code>Fast Test mode (Rust only)
cargo run --release

Full stack (Rust + Python extractor)
./shinobi.sh --deep</code></pre>
<p>Web UI at <code>http://localhost:8080</code>. Downloaded files stored in <code>./downloads/</code>.</p>

<h3>Execution Modes</h3>
<table>
  <tr><th>Mode</th><th>Command</th><th>Description</th></tr>
  <tr><td><strong>Fast Test</strong></td><td><code>./shinobi.sh --fast</code></td><td>Crawling + asset download + anti-blocking. No external dependencies.</td></tr>
  <tr><td><strong>Deep Research</strong></td><td><code>./shinobi.sh --deep</code></td><td>Adds Python sidecar for structured data extraction (JSON-LD, microdata, OG), NLP analysis (summary, entities, keywords), and custom CSS selectors.</td></tr>
  <tr><td><strong>Docker</strong></td><td><code>./shinobi.sh --docker</code></td><td>Both services via docker-compose.</td></tr>
  <tr><td><strong>Python Only</strong></td><td><code>./shinobi.sh --python-only</code></td><td>Extractor standalone for development.</td></tr>
</table>

<h3>Environment Variables</h3>
<table>
  <tr><th>Variable</th><th>Default</th><th>Description</th></tr>
  <tr><td><code>PORT</code></td><td><code>8080</code></td><td>HTTP listen port (Rust)</td></tr>
  <tr><td><code>DATA_DIR</code></td><td><code>downloads</code></td><td>Downloaded files directory</td></tr>
  <tr><td><code>RUST_LOG</code></td><td><code>shinobi=info,tower_http=info</code></td><td>Logging verbosity</td></tr>
  <tr><td><code>EXTRACTOR_URL</code></td><td><code>http://localhost:9090</code></td><td>Python extractor endpoint (Deep Research)</td></tr>
</table>

<hr>

<h2>Related Documents</h2>

<table>
  <tr><th>Document</th><th>Description</th></tr>
  <tr><td><a href="docs/manual.md">docs/manual.md</a></td><td>Development and production deployment guide</td></tr>
  <tr><td><a href="docs/ui-architecture.md">docs/ui-architecture.md</a></td><td>Frontend architecture specification</td></tr>
  <tr><td><a href="ROADMAP.md">ROADMAP.md</a></td><td>Development phases and milestones</td></tr>
  <tr><td><a href="docs/project-structure.md">docs/project-structure.md</a></td><td>Detailed codebase structure with file-by-file breakdown and API reference</td></tr>
</table>

<hr>

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
