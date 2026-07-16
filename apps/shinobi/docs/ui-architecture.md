<h1>Shinobi: UI Architecture & Structure</h1>

<p>The frontend is a single-page dashboard built with vanilla HTML5, CSS3, and JavaScript — intentionally framework-free to keep the tool lightweight, zero-dependency, and instantly loadable.</p>

<hr>

<h2>1. Design Philosophy</h2>

<p>The UI follows a dark instrument-panel aesthetic: high information density, monospace typography, and minimal visual noise. It prioritises <em>glanceability</em> — status, progress, and errors are visible without scrolling or clicking.</p>

<hr>

<h2>2. File Layout</h2>

<pre><code>static/
├── index.html     # Main SPA shell — form, progress card, tabs, lists
├── styles.css     # Design tokens, layout grid, component styles
└── app.js         # Application logic — API client, SSE stream, DOM rendering
</code></pre>

<p>There is no build step. Files are served as static assets by the Rust backend from the <code>static/</code> directory.</p>

<hr>

<h2>3. Component Map</h2>

<h3>3.1 Scrape Configuration Form</h3>
<p>Located at the top of <code>index.html</code>. Collects:</p>
<ul>
    <li><strong>Target URL</strong> — Full URL input with Enter-to-submit</li>
    <li><strong>Depth</strong> — Recursion depth (0–10)</li>
    <li><strong>Max Pages</strong> — Page limit (1–10000)</li>
    <li><strong>Delay (ms)</strong> — Base delay between requests</li>
    <li><strong>Concurrency</strong> — Parallel request count (1–20)</li>
    <li><strong>File Types</strong> — Comma-separated extension whitelist</li>
    <li><strong>Same Domain</strong> — Toggle to restrict crawling to the target domain</li>
    <li><strong>Download Assets</strong> — Toggle to include CSS/JS/images/etc.</li>
    <li><strong>Rotate User-Agent</strong> — Toggle for UA rotation per request</li>
</ul>

<h3>3.2 Progress Card</h3>
<p>Appears after a scrape starts. Shows:</p>
<ul>
    <li>Progress bar (percentage fill)</li>
    <li>Status badge (queued / running / completed / failed / cancelled)</li>
    <li>Pages scraped (current / total)</li>
    <li>Files downloaded count</li>
    <li>Current URL being scraped</li>
    <li>Error count</li>
    <li>Cancel button</li>
</ul>

<h3>3.3 Tabbed Panels</h3>
<p>Two tabs below the progress card:</p>

<h4>A. Jobs Tab</h4>
<p>Lists all scrape jobs with:</p>
<ul>
    <li>Target URL</li>
    <li>Status badge (colour-coded)</li>
    <li>Page progress</li>
    <li>File count</li>
    <li>Creation time</li>
    <li>Cancel button for running jobs</li>
</ul>

<h4>B. Downloaded Files Tab</h4>
<p>Explorer-style file browser:</p>
<ul>
    <li>File icon by extension type</li>
    <li>Relative path</li>
    <li>Human-readable file size</li>
    <li>Click to open/download in new tab</li>
    <li>Refresh button</li>
</ul>

<hr>

<h2>4. Data Flow</h2>

<pre><code>User submits form
       │
       ▼
  fetch POST /api/scrape  ───► Backend spawns scrape task
       │                           │
       ▼                           ▼
  Job ID returned          Background BFS crawler
       │                    (Tokio task, concurrent)
       ▼                           │
  EventSource connects            │
  to /api/jobs/:id/stream         ▼
       │                    Sends ScrapeProgress
       ▼                    via mpsc::channel
  onmessage callback               │
       │                           ▼
       ▼                    Updates DashMap<JobInfo>
  DOM update:                      │
  • progress bar                   ▼
  • status badge           SSE endpoint polls
  • pages/files counts     DashMap every 1 s
  • current URL             and pushes to client
       │
       ▼
  Job completes → SSE closes → refresh jobs + files
</code></pre>

<ul>
    <li><strong>Scrape request:</strong> <code>POST /api/scrape</code> returns immediately with a job ID. The backend spawns two Tokio tasks: one runs the crawler, the other forwards channel messages to the shared job state.</li>
    <li><strong>SSE streaming:</strong> The client opens an <code>EventSource</code> to <code>/api/jobs/:id/stream</code>. The server pushes a JSON-serialised <code>JobInfo</code> every second.</li>
    <li><strong>No state library:</strong> The frontend is stateless — it re-renders from the received SSE data. The jobs list polls <code>GET /api/jobs</code> every 3 seconds as fallback.</li>
</ul>

<hr>

<h2>5. State Management</h2>

<p>There is no client-side state library. The JavaScript module uses simple closures and direct DOM manipulation:</p>

<ul>
    <li><code>activeJobId</code> — Tracks which job's stream is active</li>
    <li><code>activeStream</code> — Holds the current EventSource reference</li>
    <li>Progress updates are applied directly to <code>textContent</code> and <code>style.width</code></li>
    <li>Job list and file list are re-rendered from fresh API responses each time</li>
</ul>

<p>This zero-framework approach is intentional — the dashboard has a single responsibility (display scrape progress) and does not benefit from the overhead of a reactive framework.</p>

<hr>

<p><i>Base architecture designed for the Shinobi web scraping tool within the XWA suite.</i></p>
