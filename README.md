
<h1 align="center">X Web Analysis</h1>

<div align="center">
<p><em>Modular web analysis platform — under active development</em></p>
</div>

<p><em>XWA</em> is a monorepo housing a suite of web analysis and security tools. Each tool lives under <code>apps/</code> as a self-contained project with its own Docker Compose setup and documentation.</p>

<hr>

<h2>Apps</h2>

<table>
  <tr>
    <th>App</th>
    <th>Focus</th>
    <th>Stack</th>
    <th>Status</th>
  </tr>
  <tr>
    <td><strong><a href="apps/samurai/">samurai</a></strong></td>
    <td>Web cybersecurity analysis</td>
    <td>Angular + FastAPI/Python + Rust TUI</td>
    <td>Released</td>
  </tr>
  <tr>
    <td><strong><a href="apps/shinobi/">shinobi</a></strong></td>
    <td>Stealth web scraping with anti-blocking</td>
    <td>Rust (Axum) + Angular + Python extractor</td>
    <td>Released</td>
  </tr>
  <tr>
    <td><strong><a href="apps/kensei/">kensei</a></strong></td>
    <td>Web technology stack profiler</td>
    <td>Angular + FastAPI/Python</td>
    <td>Planned</td>
  </tr>
  <tr>
    <td><strong><a href="apps/tengu/">tengu</a></strong></td>
    <td>Web quality auditor</td>
    <td>Rust (Axum) + Angular</td>
    <td>Released</td>
  </tr>
  <tr>
    <td><strong>kabuki</strong></td>
    <td>WAF and CDN analysis</td>
    <td>TBD</td>
    <td>Planned</td>
  </tr>
  <tr>
    <td><strong>yari</strong></td>
    <td>API security testing</td>
    <td>TBD</td>
    <td>Planned</td>
  </tr>
  <tr>
    <td><strong>musha</strong></td>
    <td>Web content and DOM analysis</td>
    <td>TBD</td>
    <td>Planned</td>
  </tr>
  <tr>
    <td><strong>azuma</strong></td>
    <td>Web form and authentication flow analysis</td>
    <td>TBD</td>
    <td>Planned</td>
  </tr>
</table>

<hr>

<h2>Getting Started</h2>

<h3>Clone</h3>
<pre><code>git clone https://github.com/xscriptor/xwa.git
cd xwa/xwa</code></pre>

<h3>Run an App</h3>
<p>Each app is self-contained. For example, to start <strong>samurai</strong>:</p>
<pre><code>cd apps/samurai
docker compose up</code></pre>

<p>Refer to each app's <code>README.md</code> for detailed setup instructions.</p>

<hr>

<h2>Related Documents</h2>

<table>
  <tr><th>Document</th><th>Description</th></tr>
  <tr><td><a href="ROADMAP.md">ROADMAP.md</a></td><td>Development phases and milestones</td></tr>
  <tr><td><a href="CHANGELOG.md">CHANGELOG.md</a></td><td>Release history and version log</td></tr>
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
