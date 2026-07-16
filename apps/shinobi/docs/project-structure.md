<h1 align="center">Project Structure</h1>

<p>Detailed breakdown of the Shinobi codebase.</p>

<hr>

<h2>Root Directory</h2>

<table>
  <tr><th>File</th><th>Purpose</th></tr>
  <tr>
    <td><code>Cargo.toml</code></td>
    <td>Rust package manifest. Dependencies: <code>axum</code>, <code>reqwest</code> (with socks/cookies/gzip/brotli), <code>scraper</code>, <code>chromiumoxide</code> (headless Chrome), <code>rusqlite</code> (SQLite), <code>tokio</code>, <code>serde</code>, <code>tower-http</code>, <code>zip</code>, <code>flate2</code>, <code>sha2</code>, <code>regex</code>, <code>uuid</code>, <code>chrono</code>, <code>rand</code>, <code>base64</code>.</td>
  </tr>
  <tr>
    <td><code>build.rs</code></td>
    <td>Build script that auto-compiles the Angular frontend during <code>cargo build</code>. Runs <code>npm install && npx ng build</code> in <code>frontend/</code>, output to <code>static/browser/</code>. Falls back gracefully if Node.js is unavailable.</td>
  </tr>
  <tr>
    <td><code>Dockerfile</code></td>
    <td>Multi-stage build: stage 1 compiles Rust + Angular, stage 2 copies binary + Chromium for JS rendering. Runs as non-root <code>shinobi</code> user.</td>
  </tr>
  <tr>
    <td><code>docker-compose.yml</code></td>
    <td>Two-service deployment: <code>shinobi</code> (Rust, port 8080) and <code>extractor</code> (Python, port 9090) with shared <code>downloads</code> volume.</td>
  </tr>
  <tr>
    <td><code>shinobi.sh</code></td>
    <td>Convenience launcher. Modes: <code>--fast</code> (Rust only), <code>--deep</code> (Rust + Python), <code>--python-only</code>, <code>--docker</code>. Handles npm install, ng build, Python venv, and graceful shutdown.</td>
  </tr>
  <tr>
    <td><code>shinobi.db</code></td>
    <td>SQLite database (runtime). Stores jobs, deep results, and schedules.</td>
  </tr>
</table>

<hr>

<h2><code>src/</code> — Rust Backend</h2>

<h3><code>main.rs</code></h3>
<p>Application entry point. Initializes tracing, reads env vars (<code>PORT</code>, <code>DATA_DIR</code>, <code>SHINOBI_DB_PATH</code>), sets up <code>StorageManager</code> + <code>DbStore</code>, configures Axum router with CORS, and spawns the scheduler worker (runs every 60s to trigger scheduled scrapes).</p>

<h3><code>config.rs</code></h3>
<p>Defines <code>ScrapeConfig</code> — the full scraping parameter model with sensible defaults:</p>
<ul>
  <li>URL, depth (2), concurrency (3), delay (1000ms), max pages (100)</li>
  <li>Same-domain enforcement, robots.txt respect, asset download toggles</li>
  <li>Anti-blocking: UA rotation, proxy list, retry count (3)</li>
  <li>JS rendering, screenshot capture, email extraction</li>
  <li>Deep mode: structured data, NLP, custom CSS selectors</li>
  <li>Auth: Basic username/password with configurable mode</li>
  <li>Export: WARC, ZIP, index.html generation</li>
</ul>

<h3><code>api/</code> — REST API Layer</h3>

<h4><code>routes.rs</code></h4>
<p>~980 lines — all HTTP endpoints in a single file. Key groups:</p>

<p><strong>Scrape Jobs</strong></p>
<ul>
  <li><code>POST /api/scrape</code> — start scrape, returns job ID immediately, runs BFS in background tokio task</li>
  <li><code>GET /api/jobs</code> — paginated job listing</li>
  <li><code>GET /api/jobs/:id</code> — job detail</li>
  <li><code>GET /api/jobs/:id/stream</code> — SSE real-time progress</li>
  <li><code>POST /api/jobs/:id/cancel</code> — cancel running job</li>
  <li><code>DELETE /api/jobs/:id</code> — delete job + optional file cleanup</li>
  <li><code>POST /api/jobs/:id/export</code> — JSON export of job metadata</li>
  <li><code>GET /api/jobs/:id/download</code> — ZIP archive of downloaded files</li>
</ul>

<p><strong>Files</strong></p>
<ul>
  <li><code>GET /api/files</code> — paginated file listing</li>
  <li><code>GET /api/files/*path</code> — serve downloaded file</li>
  <li><code>GET /api/search</code> — file search by name</li>
</ul>

<p><strong>System</strong></p>
<ul>
  <li><code>GET /api/stats</code> — stats (job count, active scrapes, file count, disk usage)</li>
  <li><code>GET /api/health</code> — health check</li>
  <li><code>POST /api/database/export</code> / <code>import</code> / <code>clear</code> — DB management</li>
</ul>

<p><strong>Deep Research</strong></p>
<ul>
  <li><code>POST /api/deep/scrape</code> — single-URL extract</li>
  <li><code>POST /api/deep/batch</code> — batch URL extraction</li>
  <li><code>POST /api/deep/crawl</code> — Python httrack crawl</li>
  <li><code>GET /api/deep/crawl/:id/status</code> / <code>results</code> / <code>cancel</code> — crawl lifecycle</li>
  <li><code>GET /api/deep/results</code> / <code>:id</code> — list/read results</li>
  <li><code>DELETE /api/deep/results</code> / <code>:id</code> — delete results</li>
  <li><code>GET /api/deep/results.csv</code> — CSV export</li>
</ul>

<p><strong>Schedules</strong></p>
<ul>
  <li><code>GET /api/schedules</code> — list schedules</li>
  <li><code>POST /api/schedules</code> — create schedule (min interval: 5 min)</li>
  <li><code>DELETE /api/schedules/:id</code> — delete schedule</li>
</ul>

<p>Complete endpoint reference:</p>
<table>
  <tr><th>Method</th><th>Path</th><th>Description</th></tr>
  <tr><td><code>POST</code></td><td><code>/api/scrape</code></td><td>Start a new scrape job</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/jobs</code></td><td>List all jobs</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/jobs/:id</code></td><td>Get job status</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/jobs/:id/stream</code></td><td>SSE live progress stream</td></tr>
  <tr><td><code>POST</code></td><td><code>/api/jobs/:id/cancel</code></td><td>Cancel a running job</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/files</code></td><td>List downloaded files</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/files/*path</code></td><td>Download a scraped file</td></tr>
  <tr><td><code>POST</code></td><td><code>/api/deep/scrape</code></td><td>Deep Research extract (requires Python sidecar)</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/deep/results</code></td><td>List all deep research results</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/deep/results/:id</code></td><td>Get a specific deep result</td></tr>
</table>

<h3><code>scraper/</code> — Crawling Engine</h3>

<h4><code>anti_block.rs</code></h4>
<p>Anti-blocking evasion system:</p>
<ul>
  <li>15 real browser User-Agent strings (Chrome, Firefox, Safari, Edge, Opera, Vivaldi, mobile)</li>
  <li>Header randomization: Accept variants, Accept-Language (en/es/de/fr/pt-BR), Sec-CH-UA Chrome version (120–126), Sec-CH-UA-Platform, Sec-Fetch-* headers</li>
  <li><code>random_user_agent()</code> + <code>random_headers()</code> — per-request randomization</li>
  <li><code>backoff_ms(attempt, base_ms)</code> — exponential backoff: <code>base × 2^attempt + random(0..1000)</code></li>
</ul>

<h4><code>client.rs</code></h4>
<p>HTTP client wrapper built on <code>reqwest</code>:</p>
<ul>
  <li>Configurable timeout (30s), gzip + brotli decompression, cookie store</li>
  <li>Basic Auth support (base64 credentials)</li>
  <li>HTTP/HTTPS/SOCKS5 proxy via <code>reqwest::Proxy</code></li>
  <li>Per-domain rate limiting with configurable delay</li>
  <li><code>get_with_retry()</code> — retry with exponential backoff, special 429/503 rate-limit handling</li>
</ul>

<h4><code>downloader.rs</code></h4>
<p>BFS crawling engine (~392 lines):</p>
<ul>
  <li>Parses target URL, loads <code>robots.txt</code>, optionally launches headless Chromium</li>
  <li>BFS loop: pop URL → fetch (optionally via JS renderer) → extract links → save files</li>
  <li>URL canonicalization: strip fragments, normalize slashes</li>
  <li>Content deduplication via SHA-256 hashing</li>
  <li>Link extraction from <code>a[href]</code>, <code>link[href]</code>, <code>img[src]</code>, <code>script[src]</code>, <code>source[src]</code>, <code>video[src]</code>, <code>audio[src]</code></li>
  <li>Sitemap.xml parsing for additional URL discovery</li>
  <li>Asset download filtering by extension whitelist</li>
  <li>Offline HTML URL rewriting (same-domain only)</li>
  <li>Screenshot capture (PNG)</li>
  <li>Email/phone extraction from page content</li>
  <li>Deep mode: sends HTML to Python extractor</li>
  <li>SSE progress reporting via <code>mpsc::channel</code></li>
  <li>Webhook notification on completion</li>
  <li>WARC export + index.html generation</li>
</ul>

<h4><code>renderer.rs</code></h4>
<p>Headless Chromium JS rendering via <code>chromiumoxide</code> (Chrome DevTools Protocol):</p>
<ul>
  <li>Launches with flags: <code>--no-sandbox</code>, <code>--disable-gpu</code>, <code>--disable-dev-shm-usage</code></li>
  <li><code>fetch_page()</code> — navigates to URL, waits 3s for JS execution, returns rendered HTML + optional screenshot</li>
</ul>

<h4><code>extractor.rs</code></h4>
<p>Regex-based email and phone extraction:</p>
<ul>
  <li>Email pattern: <code>[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}</code></li>
  <li>Phone pattern: international format</li>
  <li>Returns deduplicated sorted lists</li>
</ul>

<h4><code>rewriter.rs</code></h4>
<p>URL rewriting for offline browsing:</p>
<ul>
  <li>Rewrites <code>href</code> and <code>src</code> attributes to relative paths</li>
  <li>Handles absolute and protocol-relative URLs (<code>//</code>)</li>
  <li>Same-domain only</li>
  <li><code>generate_index()</code> — creates dark-themed index.html listing all downloaded files</li>
</ul>

<h4><code>robots.rs</code></h4>
<p>Minimal robots.txt parser:</p>
<ul>
  <li>Parses <code>User-agent: *</code> and <code>Disallow:</code> directives</li>
  <li><code>is_allowed(path)</code> — checks URL against disallowed paths</li>
</ul>

<h4><code>sitemap.rs</code></h4>
<p>Sitemap.xml parser:</p>
<ul>
  <li>Extracts <code>&lt;loc&gt;</code> elements</li>
  <li>Returns discovered URLs for seeding the crawl queue</li>
</ul>

<h4><code>warc.rs</code></h4>
<p>WARC archive format support:</p>
<ul>
  <li><code>WarcRecord</code> struct: target URI, date, content type, body</li>
  <li>Generates WARC/1.0 format records</li>
  <li><code>create_warc_file()</code> — concatenates records into a single archive</li>
</ul>

<h3><code>storage/</code> — Persistence Layer</h3>

<h4><code>manager.rs</code></h4>
<p>File storage manager:</p>
<ul>
  <li><code>save_file()</code> — creates parent dirs, writes to <code>{DATA_DIR}/{path}</code></li>
  <li><code>read_file()</code> — reads with path traversal protection (<code>join_safe()</code> validates resolved path is under base dir)</li>
  <li><code>list_files()</code> — recursive dir listing returning <code>FileInfo</code> (name, path, is_dir, size, modified)</li>
</ul>

<h4><code>db.rs</code></h4>
<p>SQLite persistence via <code>rusqlite</code> (bundled):</p>
<ul>
  <li>Auto-creates tables: <code>jobs</code>, <code>deep_results</code>, <code>schedules</code></li>
  <li>CRUD for jobs, deep results, and schedules</li>
  <li>Bulk export/import of jobs</li>
  <li>Optional: app continues without DB if init fails</li>
</ul>

<hr>

<h2><code>frontend/</code> — Angular 19 SPA</h2>

<table>
  <tr><th>File</th><th>Purpose</th></tr>
  <tr>
    <td><code>package.json</code></td>
    <td>NPM config. Dependencies: Angular 19 core/forms/router, RxJS, zone.js, tslib. DevDeps: Angular CLI 19, TypeScript 5.6.</td>
  </tr>
  <tr>
    <td><code>angular.json</code></td>
    <td>CLI config. Build output: <code>../static</code>. Builder: <code>@angular-devkit/build-angular:application</code>.</td>
  </tr>
  <tr>
    <td><code>tsconfig.json</code></td>
    <td>TypeScript config. Target: ES2022, strict mode, Angular strict templates.</td>
  </tr>
</table>

<h3><code>src/</code></h3>

<h4><code>index.html</code></h4>
<p>Shell HTML. Loads Space Grotesk + Space Mono from Google Fonts. Lang: Spanish. Title: "Shinobi — Web Scraper".</p>

<h4><code>styles.scss</code></h4>
<p>~614 lines — full design system with CSS custom properties:</p>
<ul>
  <li>Dark theme (default), light theme variant (<code>.theme-light</code>)</li>
  <li>Monospace typography (Space Mono for data, Space Grotesk for UI)</li>
  <li>Component styles: cards, form grids, tabs, mode-tabs, progress bars, stat cards, pagination, file preview modals, terminal output, keyboard shortcuts hint</li>
  <li>Color tokens: <code>--interactive</code> (#5B9BF6), <code>--accent</code> (#D71921), <code>--success</code> (#4A9E5C), <code>--warning</code> (#D4A843), <code>--gold</code> (#FFD700)</li>
  <li>Status badges: queued (blue), running (warning), completed (success), failed (error), cancelled (gold)</li>
  <li>Responsive single-column layout on mobile</li>
</ul>

<h4><code>app/</code></h4>

<p><strong><code>app.component.ts</code></strong> — Root component. Shows "// shinobi." branding, Rust/Python health indicators (green/red/gray dots), theme toggle. Polls <code>/api/health</code> and <code>:9090/health</code> on init. Persists theme in localStorage.</p>

<p><strong><code>app.config.ts</code></strong> — Angular Router configuration.</p>

<p><strong><code>app.routes.ts</code></strong> — Single route: <code>""</code> → <code>DashboardComponent</code>.</p>

<p><strong><code>models/models.ts</code></strong> — TypeScript interfaces matching Rust API types: <code>ScrapeConfig</code>, <code>JobInfo</code>, <code>FileInfo</code>, <code>DeepConfig</code>, <code>DeepResult</code>, <code>PaginatedResponse</code>.</p>

<p><strong><code>services/api.service.ts</code></strong> — ~146 lines. Full API client using native <code>fetch</code> (no HttpClient). Methods for all endpoints including SSE streams via <code>EventSource</code>.</p>

<p><strong><code>services/confirm.service.ts</code></strong> — Promise-based confirmation dialog.</p>

<p><strong><code>services/toast.service.ts</code></strong> — Toast notifications with auto-dismiss (ok/error/warn types).</p>

<p><strong><code>pages/dashboard.component.ts</code></strong> — ~368 lines. Main dashboard logic: mode selection (fast/deep), SSE streaming, paginated lists, file preview, Python crawl polling, keyboard shortcuts (<code>Ctrl+Enter</code>, <code>?</code>, <code>Escape</code>), JSON/CSV exports, DB import/export.</p>

<p><strong><code>pages/dashboard.component.html</code></strong> — ~366 lines. Template with: stat cards, mode tabs, Fast Test form (URL, depth, delay, file types, anti-blocking toggles, auth config), Deep Research sub-modes (Single/Batch/Crawl), progress card, search bar, tabbed panels (Jobs, Files, Deep Results, Schedules), file preview modal, DB actions, keyboard shortcuts overlay, footer.</p>

<hr>

<h2><code>extractor/</code> — Python Sidecar</h2>

<h3><code>main.py</code> — FastAPI Server</h3>
<p>~274 lines, runs on port 9090:</p>
<ul>
  <li><code>GET /health</code> — health check</li>
  <li><code>POST /extract</code> — single-URL extraction pipeline (structured, NLP, metadata, headings, links, tables, images, custom selectors, emails, phones)</li>
  <li><code>POST /crawl</code> — start httrack-based crawl (background thread)</li>
  <li><code>GET /crawl/{id}</code> — crawl details</li>
  <li><code>GET /crawl/{id}/status</code> — progress poll (pages, files, %, current URL, errors, log)</li>
  <li><code>GET /crawl/{id}/results</code> — extracted data + ZIP path</li>
  <li><code>POST /crawl/{id}/cancel</code> — cancel crawl</li>
</ul>

<h3><code>extractors/</code></h3>

<h4><code>structured.py</code></h4>
<p>~166 lines — structured data extraction:</p>
<ul>
  <li><code>extruct</code> for JSON-LD, microdata, Open Graph, RDFa</li>
  <li>Manual OG fallback from <code>&lt;meta&gt;</code> tags</li>
  <li>Custom CSS selector extraction via BeautifulSoup</li>
  <li>Metadata: title, description, keywords, canonical URL</li>
  <li>Headings (h1–h6 outline), internal/external links, tables, images</li>
</ul>

<h4><code>nlp.py</code></h4>
<p>~285 lines — natural language processing:</p>
<ul>
  <li>Text extraction (strips <code>&lt;script&gt;</code>, <code>&lt;style&gt;</code>, nav, footer, header)</li>
  <li>Summarization: TF-based with position scoring, top 5 sentences</li>
  <li>Entity extraction: pattern-based capitalized entities + emails</li>
  <li>Keywords: TF-IDF style frequency/density + bigram extraction</li>
  <li>Sentiment: dictionary-based (positive/negative word lists), score + label</li>
  <li>Readability: Flesch Reading Ease</li>
  <li>spaCy NER integration (PERSON, ORG, GPE, DATE, MONEY)</li>
</ul>

<h4><code>crawler.py</code></h4>
<p>~285 lines — Python crawling via httrack:</p>
<ul>
  <li><code>CrawlJob</code> class: manages httrack subprocess in background thread</li>
  <li>Configurable depth, max pages, same-domain</li>
  <li>Progress parsing from httrack stdout</li>
  <li>Result extraction from downloaded HTML files</li>
  <li>ZIP creation of all files</li>
  <li>Email/phone collection across all pages</li>
  <li><code>CrawlManager</code> singleton: queue + single-worker execution</li>
</ul>

<h3><code>requirements.txt</code></h3>
<ul>
  <li><code>fastapi>=0.115.0</code>, <code>uvicorn[standard]</code>, <code>httpx</code></li>
  <li><code>extruct>=0.16.0</code> (structured data)</li>
  <li><code>spacy>=3.8.0</code> + <code>en_core_web_sm</code> (NLP)</li>
  <li><code>beautifulsoup4>=4.12.0</code>, <code>lxml>=5.3.0</code>, <code>cssselect>=1.2.0</code> (HTML parsing)</li>
</ul>

<hr>

<h2><code>static/</code> — Built Frontend</h2>

<table>
  <tr><th>File</th><th>Purpose</th></tr>
  <tr><td><code>browser/index.html</code></td><td>Compiled Angular SPA shell</td></tr>
  <tr><td><code>browser/main.js</code></td><td>Compiled Angular bundle</td></tr>
  <tr><td><code>browser/polyfills.js</code></td><td>Zone.js polyfills</td></tr>
  <tr><td><code>browser/styles.css</code></td><td>Compiled styles from styles.scss</td></tr>
</table>

<hr>

<h2><code>downloads/</code> — Scraped Output</h2>

<table>
  <tr><th>File</th><th>Purpose</th></tr>
  <tr><td><code>.gitkeep</code></td><td>Placeholder to keep directory in git</td></tr>
  <tr><td><code>*.json</code></td><td>Crawl metadata JSON exports</td></tr>
  <tr><td><code>*.zip</code></td><td>Crawl result ZIP archives</td></tr>
  <tr><td><code>{domain}/</code></td><td>Mirrored site directories</td></tr>
</table>

<hr>

<h2><code>docs/</code> — Documentation</h2>

<table>
  <tr><th>File</th><th>Purpose</th></tr>
  <tr><td><code>manual.md</code></td><td>Full development/production manual</td></tr>
  <tr><td><code>ui-architecture.md</code></td><td>Frontend architecture specification</td></tr>
  <tr><td><code>project-structure.md</code></td><td>This file — detailed codebase breakdown</td></tr>
  <tr><td><code>esp/README.md</code></td><td>Spanish translation of main README</td></tr>
</table>

<hr>

<h2>Architecture Overview</h2>

<pre><code>                          ┌─────────────────────────────┐
                          │      Browser (Angular 19)    │
                          │   localhost:8080             │
                          └──────────┬──────────────────┘
                                     │ HTTP / SSE
                          ┌──────────▼──────────────────┐
                          │     Rust Backend (Axum)      │
                          │     localhost:8080            │
                          │                              │
                          │  api/routes.rs               │
                          │  scraper/ (anti_block,       │
                          │    client, downloader,       │
                          │    renderer, extractor,      │
                          │    rewriter, robots,         │
                          │    sitemap, warc)            │
                          │  storage/ (manager, db)      │
                          └──────────┬──────────────────┘
                                     │ HTTP (JSON)
                          ┌──────────▼──────────────────┐
                          │  Python Extractor (FastAPI)  │
                          │  localhost:9090              │
                          │                              │
                          │  extractors/structured.py    │
                          │  extractors/nlp.py           │
                          │  extractors/crawler.py       │
                          └─────────────────────────────┘</code></pre>
