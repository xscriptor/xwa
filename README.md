
<h1 align="center">X Web Analyzer</h1>

<div align="center">
<p><em>Modular web analysis platform — under active development</em></p>
</div>

<p><em><a href="https://github.com/xscriptor/xwa">XWA</a></em> is the umbrella monorepo that aggregates specialized submodules into a cohesive web analysis ecosystem.</p>

<hr>

<h2>Submodules</h2>

<table>
  <tr>
    <th>Submodule</th>
    <th>Focus</th>
    <th>Stack</th>
    <th>Status</th>
  </tr>
  <tr>
    <td><strong><a href="https://github.com/xscriptor/samurai">samurai</a></strong></td>
    <td>Web cybersecurity analysis</td>
    <td>Angular 21 + FastAPI/Python + Rust TUI</td>
    <td>Released</td>
  </tr>
  <tr>
    <td><strong><a href="https://github.com/xscriptor/shinobi">shinobi</a></strong></td>
    <td>Stealth web scraping with anti-blocking</td>
    <td>Rust (Axum) + Angular 19 + Python extractor</td>
    <td>Released</td>
  </tr>
  <tr>
    <td><strong>kensei</strong></td>
    <td>Web technology stack profiler</td>
    <td>TBD</td>
    <td>Planned</td>
  </tr>
  <tr>
    <td><strong>tengu</strong></td>
    <td>Web quality auditor</td>
    <td>TBD</td>
    <td>Planned</td>
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

<h3>Clone with Submodules</h3>
<pre><code>git clone --recurse-submodules https://github.com/xscriptor/xwa.git</code></pre>

<h3>Or Initialize After Cloning</h3>
<pre><code>git clone https://github.com/xscriptor/xwa.git
cd xwa
git submodule update --init --recursive</code></pre>

<p>Each submodule contains its own Docker Compose configuration and standalone setup instructions. Refer to the individual README files for detailed guidance.</p>

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
