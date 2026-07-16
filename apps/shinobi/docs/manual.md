<h1>Shinobi Application Manual</h1>

<p>This document covers local development, production configuration, and the anti-blocking architecture.</p>

<hr>

<h2>1. Local Development Execution</h2>

<h3>1.1 Prerequisites</h3>
<ul>
    <li>Rust 1.75+ with <code>cargo</code></li>
    <li>Docker Engine + Docker Compose (optional, for containerised runs)</li>
</ul>

<h3>1.2 Standalone (No Docker)</h3>
<pre><code>cargo run
# Listens on http://localhost:8080
# Data dir: ./downloads/</code></pre>
<p>Hot-reload with <code>cargo-watch</code>:</p>
<pre><code>cargo install cargo-watch
cargo watch -x run</code></pre>

<h3>1.3 Frontend Architecture</h3>
<p>The frontend is an <strong>Angular 19</strong> SPA located in <code>frontend/</code>. At build time, <code>build.rs</code> automatically runs <code>ng build</code> and outputs to <code>static/browser/</code>. The Rust binary serves these files at runtime. If Angular dependencies are unavailable, the old vanilla static files in <code>static/</code> serve as fallback.</p>

<h3>1.4 Manual Frontend Build</h3>
<pre><code>cd frontend
npm install --legacy-peer-deps
npx ng build</code></pre>

<h3>1.3 Docker (Development)</h3>
<pre><code>docker compose up --build
# Listens on http://localhost:8080</code></pre>

<h3>1.4 Verifying the Setup</h3>
<p>Open <code>http://localhost:8080</code> in a browser. Enter a target URL and click <strong>Start Scrape</strong>. The progress card should update in real time via SSE.</p>

<hr>

<h2>2. Production Configuration</h2>

<h3>2.1 Frontend</h3>
<p>The frontend is vanilla HTML/CSS/JS served as static files by the Rust binary. No build step required — the files in <code>static/</code> are embedded at compile time or served at runtime from disk. For production:</p>
<ul>
    <li>Set <code>RUST_LOG=warn</code> to reduce verbosity.</li>
    <li>Set <code>PORT=443</code> behind a reverse proxy (nginx/caddy) for HTTPS termination.</li>
    <li>The <code>DATA_DIR</code> environment variable controls where scraped files are stored.</li>
</ul>

<h3>2.2 Backend (Rust)</h3>
<ul>
    <li>Build with <code>cargo build --release</code> for optimised binary.</li>
    <li>Run behind a reverse proxy or expose port 8080 directly (not recommended without TLS).</li>
    <li>For high-traffic deployments, increase <code>concurrency</code> and <code>max_pages</code> via the UI or API.</li>
</ul>

<h3>2.3 Docker Production Build</h3>
<p>The provided <code>Dockerfile</code> uses a multi-stage build:</p>
<ul>
    <li><strong>Stage 1 (builder):</strong> Compiles the Rust binary with <code>--release</code>.</li>
    <li><strong>Stage 2 (runtime):</strong> Copies the binary + <code>static/</code> into a slim Debian image. Runs as non-root user <code>shinobi</code>.</li>
</ul>
<p>To build for production:</p>
<pre><code>docker build -t shinobi:latest .
docker run -d -p 8080:8080 -v downloads:/data/downloads shinobi:latest</code></pre>

<h3>2.4 Security Considerations</h3>
<ul>
    <li>The <code>storage/manager.rs</code> implements path traversal protection — all file reads are validated against the base data directory.</li>
    <li>The API has no authentication by design (local/internal tool). Add a reverse proxy with auth for external exposure.</li>
    <li>CORS is permissive (<code>CorsLayer::permissive()</code>) — restrict in production to specific origins.</li>
</ul>

<hr>

<h2>3. Anti-Blocking Architecture</h2>

<p>The anti-blocking system (<code>src/scraper/anti_block.rs</code>) applies evasion techniques before each HTTP request:</p>

<h3>3.1 User-Agent Rotation</h3>
<p>A pool of 15 real browser user-agent strings is maintained. A random UA is selected per request when <code>user_agent_rotation</code> is enabled.</p>

<h3>3.2 Header Randomisation</h3>
<p>Each request generates randomised but realistic headers:</p>
<ul>
    <li><code>Accept</code> — 4 variants of HTML/image/webp preferences</li>
    <li><code>Accept-Language</code> — 8 locale variants (en-US, es-ES, de-DE, fr-FR, pt-BR, etc.)</li>
    <li><code>Sec-CH-UA</code> — Random Chrome version between 120–126</li>
    <li><code>Sec-CH-UA-Platform</code> — Random: Windows, macOS, or Linux</li>
    <li><code>Sec-Fetch-*</code> — Navigate-mode headers with randomised site origin</li>
</ul>

<h3>3.3 Request Timing</h3>
<ul>
    <li><strong>Base delay:</strong> Configurable (default 1000 ms) between requests.</li>
    <li><strong>Jitter:</strong> Random 0–100% of base delay added to each request.</li>
    <li><strong>Backoff:</strong> On failure, delay = <code>base_ms * 2^attempt + random(0..1000)</code>.</li>
</ul>

<h3>3.4 Rate-Limit Handling</h3>
<p>HTTP 429 (Too Many Requests) and 503 (Service Unavailable) are detected. When hit, the scraper waits <code>5000 * 2^attempt + jitter</code> ms before retrying.</p>

<h3>3.5 Proxy Support</h3>
<p>HTTP/HTTPS/SOCKS5 proxies are supported via <code>reqwest::Proxy</code>. The proxy list is configured in the scrape request. When multiple proxies are provided, the client can rotate through them (extensible — currently uses the first proxy).</p>

<hr>

<h2>4. Additional Features</h2>

<h3>4.1 robots.txt & Sitemap</h3>
<p>When <code>respect_robots_txt</code> is enabled (default), Shinobi fetches <code>/robots.txt</code> from the target domain before crawling and skips disallowed paths. It also attempts to load <code>/sitemap.xml</code> and adds discovered URLs to the crawl queue — useful for finding pages not linked from the homepage.</p>

<h3>4.2 Content Deduplication</h3>
<p>Each page body is hashed with SHA-256. When <code>deduplicate</code> is enabled (default), pages with identical content are skipped. This avoids saving duplicate pages caused by URL parameters, session IDs, or mirror paths.</p>

<h3>4.3 Email & Phone Extraction</h3>
<p>When <code>extract_emails</code> is enabled, Shinobi scans every scraped HTML page with regex patterns and collects email addresses and phone numbers. Results appear in the progress card and are included in JSON exports.</p>

<h3>4.4 Webhooks</h3>
<p>If a <code>webhook_url</code> is configured, Shinobi sends a POST request with a JSON payload when the scrape completes or is cancelled:</p>
<pre><code>{
  "event": "scrape_complete",
  "status": "completed",
  "url": "https://example.com",
  "pages_scraped": 42,
  "files_downloaded": 128,
  "domain": "example.com"
}</code></pre>

<h3>4.5 JSON Export</h3>
<p>Each completed job has an <strong>Export</strong> button in the Jobs tab. Clicking it downloads a JSON file containing the job metadata, all scraped file paths, and any extracted emails/phones.</p>

<h3>4.6 Screenshots</h3>
<p>When <code>take_screenshots</code> is enabled alongside JavaScript rendering, Shinobi captures a full-page screenshot of each scraped page. Screenshots are saved as PNG files under <code>{domain}/screenshots/</code>.</p>

<hr>

<h2>5. JavaScript Rendering Engine</h2>

<p>Shinobi includes an optional headless browser engine for rendering JavaScript-heavy sites (SPA, React, Vue, Angular, etc.).</p>

<h3>5.1 How It Works</h3>
<p>When <code>javascript_rendering</code> is enabled, Shinobi launches a headless Chromium instance via the Chrome DevTools Protocol (<code>chromiumoxide</code> crate). For each HTML page, instead of fetching the raw source via HTTP, it loads the page in the browser, waits 3 seconds for JS execution, then extracts the fully rendered DOM.</p>

<h3>5.2 Requirements</h3>
<ul>
    <li><strong>Standalone:</strong> <code>chromium</code> or <code>google-chrome</code> must be installed and available in <code>PATH</code>.</li>
    <li><strong>Docker:</strong> Chromium is pre-installed in the Docker image.</li>
</ul>

<h3>5.3 Limitations</h3>
<ul>
    <li>JS rendering is slower than plain HTTP fetching (browser launch + navigation + render wait).</li>
    <li>Each page takes ~3-5 seconds regardless of the configured delay.</li>
    <li>Only HTML pages go through the renderer — assets (CSS, JS, images) are fetched via the regular HTTP client.</li>
    <li>If Chromium is not installed, Shinobi logs a warning and falls back to the regular HTTP client.</li>
</ul>

<h3>5.4 Browser Configuration</h3>
<p>The renderer runs in headless mode with these flags:</p>
<pre><code>--no-sandbox
--disable-gpu
--disable-dev-shm-usage
--disable-setuid-sandbox
--disable-software-rasterizer</code></pre>

<hr>

<p><i>End of Manual.</i></p>
