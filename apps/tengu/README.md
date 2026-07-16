
<h1 align="center">Tengu</h1>

<div align="center">
<p><em>Web quality auditor -- Performance, SEO, Accessibility, Best Practices</em></p>
</div>

<div align="center">
<a href="README.md">English</a> | <a href="docs/esp/README.md">Espanol</a>
</div>

<p><em><a href="https://github.com/xscriptor/tengu">Tengu</a></em> : <em><a href="https://github.com/xscriptor/xwa">XWA</a> <strong>submodule focused</strong> on web quality auditing -- under active development</em></p>

<hr>

<h2>Overview</h2>

<p>Tengu is a web quality auditor that evaluates web applications across four dimensions:</p>

<table>
  <tr>
    <th>Category</th>
    <th>Scope</th>
  </tr>
  <tr>
    <td><strong>Performance</strong></td>
    <td>Page weight, Core Web Vitals (LCP, CLS, INP), resource waterfall, image optimization, font loading, cache policy, compression, third-party impact</td>
  </tr>
  <tr>
    <td><strong>SEO</strong></td>
    <td>Title tags, meta descriptions, heading hierarchy, canonical URLs, Open Graph, Twitter Cards, JSON-LD, sitemaps, robots.txt, hreflang, redirect chains, broken links</td>
  </tr>
  <tr>
    <td><strong>Accessibility</strong></td>
    <td>Alt text, heading structure, ARIA attributes, landmark elements, color contrast (WCAG AA/AAA), keyboard navigation, form labels, link text, tables, iframes, viewport configuration</td>
  </tr>
  <tr>
    <td><strong>Best Practices</strong></td>
    <td>HTTPS enforcement, security headers (HSTS, CSP, XFO, etc.), cookie audit, GDPR banner detection, doctype validation, deprecated HTML, mixed content, SRI, console errors</td>
  </tr>
</table>

<table>
  <tr>
    <th>Interface</th>
    <th>Directory</th>
    <th>Language</th>
    <th>Type</th>
  </tr>
  <tr>
    <td><strong>Tengu Web</strong></td>
    <td><code>/</code> (monorepo root)</td>
    <td>Rust (Axum) + Angular 19</td>
    <td>Web application (standalone or Docker)</td>
  </tr>
</table>

<hr>

<h2>Quick Start</h2>

<h3>Web Version (Docker Compose)</h3>
<pre><code>docker compose up -d --build</code></pre>
<ul>
  <li>Web UI: <code>http://localhost:8080</code></li>
</ul>

<h3>Standalone (No Docker)</h3>
<pre><code>cargo run --release</code></pre>
<p>Web UI at <code>http://localhost:8080</code>.</p>

<h3>Environment Variables</h3>
<table>
  <tr><th>Variable</th><th>Default</th><th>Description</th></tr>
  <tr><td><code>PORT</code></td><td><code>8080</code></td><td>HTTP listen port</td></tr>
  <tr><td><code>RUST_LOG</code></td><td><code>tengu=info,tower_http=info</code></td><td>Logging verbosity</td></tr>
</table>

<hr>

<h2>Related Documents</h2>

<table>
  <tr><th>Document</th><th>Description</th></tr>
  <tr><td><a href="docs/manual.md">docs/manual.md</a></td><td>Deployment and usage manual</td></tr>
  <tr><td><a href="docs/ui-architecture.md">docs/ui-architecture.md</a></td><td>Frontend architecture overview</td></tr>
  <tr><td><a href="docs/rust-libraries.md">docs/rust-libraries.md</a></td><td>Rust backend dependency inventory</td></tr>
  <tr><td><a href="docs/uses/audit.md">docs/uses/audit.md</a></td><td>Audit usage guide</td></tr>
  <tr><td><a href="ROADMAP.md">ROADMAP.md</a></td><td>Development phases and milestones</td></tr>
  <tr><td><a href="CHANGELOG.md">CHANGELOG.md</a></td><td>Release history and version log</td></tr>
</table>

<hr>

<h2>Project Structure</h2>

<pre><code>tengu/
├── src/
│   ├── main.rs              # Axum server entry point
│   ├── config.rs            # Audit configuration, category filters
│   ├── api/
│   │   └── routes.rs        # REST API + WebSocket audit endpoint
│   ├── auditor/
│   │   ├── mod.rs           # Audit orchestrator + Finding model
│   │   ├── performance.rs   # Page weight, resources, images, fonts
│   │   ├── seo.rs           # Meta tags, structured data, sitemaps
│   │   ├── a11y.rs          # WCAG compliance, ARIA, contrast
│   │   └── best_practices.rs # Security headers, cookies, HTML validation
│   ├── extractor/
│   │   └── html.rs          # HTML metadata extraction
│   └── storage/
│       └── mod.rs           # In-memory audit record store
├── frontend/                # Angular 19 SPA source
│   ├── package.json
│   ├── angular.json
│   └── src/app/
│       ├── components/      # App shell, sidebar, theme toggle
│       └── features/
│           ├── audit/       # Audit configuration and results
│           └── history/     # Past audit records
├── docs/                    # Technical documentation
├── Dockerfile               # Multi-stage Rust build
├── docker-compose.yml       # Single service Docker deployment
├── Cargo.toml               # Rust dependencies
├── ROADMAP.md               # Development roadmap
└── CHANGELOG.md             # Release history
</code></pre>

<hr>

<h2>API Endpoints</h2>

<table>
  <tr><th>Method</th><th>Path</th><th>Description</th></tr>
  <tr><td><code>GET</code></td><td><code>/api/health</code></td><td>Health check</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/audit/live</code></td><td>WebSocket endpoint for real-time audit</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/audits</code></td><td>List all past audits</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/audits/:id</code></td><td>Get audit details</td></tr>
  <tr><td><code>DELETE</code></td><td><code>/api/audits/:id</code></td><td>Delete an audit record</td></tr>
  <tr><td><code>GET</code></td><td><code>/api/audits/export</code></td><td>Export all audit records as JSON</td></tr>
  <tr><td><code>POST</code></td><td><code>/api/audits/import</code></td><td>Import audit records from JSON</td></tr>
</table>

<h2>Launch Script</h2>

<pre><code>./tengu.sh                    # Standalone ephemeral (privacy-first)
./tengu.sh --docker            # via docker-compose
./tengu.sh --docker -e         # Docker ephemeral (--rm)
./tengu.sh --export file.json  # Save export to ./exports/
./tengu.sh --import file.json  # Import audit JSON into running instance</code></pre>

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
