
<h1 align="center">Shinobi</h1>

<div align="center">
<p><em>Web scraper silencioso — Sistema de descarga con anti-bloqueo</em></p>
</div>

> **Idioma:** Esta es la versión en español de la documentación. Para la versión en inglés, consulta [README.md](../../README.md).

<p><em><a href="https://github.com/xscriptor/xwa">XWA</a>  <strong>submódulo enfocado</strong> en scraping web sigiloso con anti-bloqueo — en desarrollo activo</em></p>

<hr>

<p>
<a href="#resumen">Resumen</a> ·
<a href="#capacidades">Capacidades</a> ·
<a href="#sistema-anti-bloqueo">Anti-Bloqueo</a> ·
<a href="#inicio-rapido">Inicio Rápido</a> ·
<a href="#con-docker">Docker</a> ·
<a href="#sin-docker-independiente">Standalone</a> ·
<a href="#variables-de-entorno">Variables</a> ·
<a href="#documentos-relacionados">Docs</a>
</p>

<hr>

<h2>Resumen</h2>

<p>Shinobi es un web scraper sigiloso con una interfaz web única. Descarga sitios completos — HTML, CSS, JS, imágenes, PDFs — mientras evade la detección mediante múltiples capas de anti-bloqueo.</p>

<table>
  <tr>
    <th>Interfaz</th>
    <th>Directorio</th>
    <th>Lenguaje</th>
    <th>Tipo</th>
  </tr>
  <tr>
    <td><strong>Shinobi Web</strong></td>
    <td><code>/</code> (raíz del monorepo)</td>
    <td>Rust (Axum) + HTML/CSS/JS vanilla</td>
    <td>Aplicación web (independiente o Docker)</td>
  </tr>
</table>

<h3>Capacidades</h3>
<ul>
  <li><strong>Crawling Recursivo</strong> — Descubrimiento BFS con profundidad y límite de páginas configurables</li>
  <li><strong>Descarga de Activos</strong> — HTML, CSS, JS, imágenes, PDFs, archivos, multimedia, fuentes</li>
  <li><strong>Filtro por Tipo de Archivo</strong> — Selecciona qué extensiones descargar</li>
  <li><strong>Restricción de Dominio</strong> — Limita el crawling al dominio objetivo o explora libremente</li>
  <li><strong>Progreso en Tiempo Real</strong> — Stream SSE con páginas escrapeadas, archivos descargados, URL actual</li>
  <li><strong>Explorador de Archivos</strong> — Navega y abre archivos descargados desde la interfaz web</li>
</ul>

<h3>Sistema Anti-Bloqueo</h3>
<ul>
  <li><strong>Rotación de User-Agent</strong> — 15 UAs reales (Chrome, Firefox, Safari, Edge, móvil)</li>
  <li><strong>Aleatorización de Cabeceras</strong> — Accept, Accept-Language, Sec-CH-UA, Sec-Fetch-* por petición</li>
  <li><strong>Delay + Jitter</strong> — Retardo base configurable con jitter aleatorio</li>
  <li><strong>Backoff Exponencial</strong> — Reintento con jitter en fallos (intentos configurables)</li>
  <li><strong>Manejo de Rate Limit</strong> — Detecta HTTP 429/503, espera y reintenta con backoff más largo</li>
  <li><strong>Soporte de Proxies</strong> — Rotación de proxies HTTP/HTTPS/SOCKS5</li>
</ul>

<hr>

<h2>Inicio Rápido</h2>

<h3>Con Docker</h3>
<pre><code>docker compose up -d --build</code></pre>
<ul>
  <li>Interfaz web: <code>http://localhost:8080</code></li>
</ul>

<h3>Sin Docker (Independiente)</h3>
<pre><code>cargo run --release</code></pre>
<p>Sin configuración. Escucha en <code>http://localhost:8080</code>. Archivos descargados en <code>./downloads/</code>.</p>

<h3>Variables de Entorno</h3>
<table>
  <tr><th>Variable</th><th>Por Defecto</th><th>Descripción</th></tr>
  <tr><td><code>PORT</code></td><td><code>8080</code></td><td>Puerto HTTP</td></tr>
  <tr><td><code>DATA_DIR</code></td><td><code>downloads</code></td><td>Directorio de descargas</td></tr>
  <tr><td><code>RUST_LOG</code></td><td><code>shinobi=info,tower_http=info</code></td><td>Verbosidad de logs</td></tr>
</table>

<hr>

<h2>Documentos Relacionados</h2>

<table>
  <tr><th>Documento</th><th>Descripción</th></tr>
  <tr><td><a href="../manual.md">manual.md</a></td><td>Guía de despliegue de desarrollo y producción</td></tr>
  <tr><td><a href="../ui-architecture.md">ui-architecture.md</a></td><td>Especificación de arquitectura frontend</td></tr>
  <tr><td><a href="../../ROADMAP.md">ROADMAP.md</a></td><td>Fases de desarrollo e hitos</td></tr>
  <tr><td><a href="project-structure.md">project-structure.md</a></td><td>Estructura detallada del código con referencia de API</td></tr>
</table>

<hr>

<div id="x" align="center">
<h2>X</h2>

<a href="https://dev.xscriptor.com">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/verified-filled.svg" width="24" alt="X Web" />
</a>
 & 
<a href="https://github.com/xscriptor">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/github.svg" width="24" alt="Perfil de X en Github" />
</a>
 & 
<a href="https://www.xscriptor.com">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/quotes.svg" width="24" alt="Sitio web de Xscriptor" />
</a>

</div>
