<h1 align="center">Samurai</h1>

<div align="center">
<img src="https://raw.githubusercontent.com/xscriptor/samurai/main/frontend/src/app/icon.svg" width="43"/> 
</div>

> **Idioma:** Esta es la versión en español de la documentación. Para la versión en inglés, consulta [README.md](../../README.md).

<p><em><a href="https://github.com/xscriptor/samurai">Samurai</a></em> : <em><a href="https://github.com/xscriptor/xwa">XWA</a>  <strong>submódulo enfocado</strong> en ciberseguridad web — en desarrollo activo</em></p>

<img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-01.png" alt="Samurai XWA Captura 01" width="100%">

<details>
  <summary>Más capturas de pantalla...</summary>
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-02.png" alt="Samurai XWA Captura 02" width="100%">
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-03.png" alt="Samurai XWA Captura 03" width="100%">
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-04.png" alt="Samurai XWA Captura 04" width="100%">
  <br>
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/xwa/samurai/samurai-xwa-screenshot-05.png" alt="Samurai XWA Captura 05" width="100%">
</details>

<hr>

<h2>Resumen</h2>

<p>Samurai es una plataforma de análisis de ciberseguridad con dos interfaces que comparten la misma base de datos:</p>

<table>
  <tr>
    <th>Interfaz</th>
    <th>Directorio</th>
    <th>Lenguaje</th>
    <th>Tipo</th>
  </tr>
  <tr>
    <td><strong>Samurai Web</strong></td>
    <td><code>/frontend</code> + <code>/backend</code></td>
    <td>Angular 21 + FastAPI/Python</td>
    <td>Aplicación web (Docker)</td>
  </tr>
  <tr>
    <td><strong>Samurai TUI</strong></td>
    <td><code>/samurai-tui</code></td>
    <td>Rust</td>
    <td>Aplicación de terminal (independiente o Docker)</td>
  </tr>
</table>

<h3>Capacidades</h3>
<ul>
  <li><strong>Escaneo de Puertos</strong> — Nmap con perfiles configurables (rápido, equilibrado, profundo, UDP)</li>
  <li><strong>Reconocimiento Web</strong> — Enumeración DNS, descubrimiento de subdominios, sondeo de API, auditoría de cabeceras de seguridad, huellas digitales de tecnología</li>
  <li><strong>Crawling de Vulnerabilidades (DAST)</strong> — Descubrimiento de páginas, análisis de cabeceras HTTP, extracción de enlaces</li>
  <li><strong>Exportación de Base de Datos</strong> — Volcado completo de análisis como JSON (crudo) o binario cifrado AES-256-GCM</li>
  <li><strong>Historial y Archivo</strong> — Almacenamiento persistente de escaneos con hallazgos y topología descubierta</li>
</ul>

<h3>Exportación de Base de Datos y Compatibilidad Cruzada</h3>
<p>El backend web exporta la base de datos mediante <code>GET /api/database/export/raw</code> (JSON) y <code>POST /api/database/export/encrypted</code> (binario AES-256-GCM). La TUI exporta mediante la pestaña <strong>Export</strong> usando el mismo formato de cifrado (<code>SAMURAI_DB_EXPORT_V1</code>). Ambas interfaces comparten esquemas de tabla idénticos (<code>scans</code>, <code>findings</code>, <code>discovered_links</code>).</p>
<p>Cuando ambas apuntan a la misma base de datos PostgreSQL, las exportaciones son innecesarias: los escaneos aparecen automáticamente en ambas interfaces. La TUI también puede funcionar de forma independiente con SQLite, almacenando datos localmente en <code>samurai.db</code>.</p>

<hr>

<h2>Inicio Rápido</h2>

<h3>Versión Web (Docker Compose)</h3>
<pre><code>docker compose up -d --build</code></pre>
<ul>
  <li>Frontend: <code>http://localhost:4200</code></li>
  <li>Documentación API: <code>http://localhost:8000/docs</code></li>
</ul>

<h3>Versión de Terminal (Independiente)</h3>
<pre><code>cd samurai-tui
cargo build --release
cargo run --release</code></pre>
<p>Sin configuración. Crea automáticamente una base de datos SQLite local (<code>samurai.db</code>). No requiere Docker.</p>

<h3>Versión de Terminal (Docker — Sesión Efímera)</h3>
<pre><code>cd samurai-tui
docker compose up --build
# La sesión se destruye al salir — no se persisten datos</code></pre>
<p>Consulta <a href="samurai-tui/README.md">samurai-tui/README.md</a> para la configuración completa de Docker y opciones de volúmenes persistentes.</p>

<hr>

<h2>Documentos Relacionados</h2>

<table>
  <tr><th>Documento</th><th>Descripción</th></tr>
  <tr><td><a href="manual.md">manual.md</a></td><td>Guía de despliegue de desarrollo y producción</td></tr>
  <tr><td><a href="ui-architecture.md">ui-architecture.md</a></td><td>Especificación de arquitectura frontend basada en características</td></tr>
  <tr><td><a href="python-libraries.md">python-libraries.md</a></td><td>Inventario de dependencias Python del backend</td></tr>
  <tr><td><a href="uses/dast.md">uses/dast.md</a></td><td>Uso del escaneo de vulnerabilidades DAST</td></tr>
  <tr><td><a href="samurai-tui/README.md">samurai-tui/README.md</a></td><td>Aplicación de terminal: instalación, configuración, Docker, uso</td></tr>
  <tr><td><a href="../../ROADMAP.md">ROADMAP.md</a></td><td>Fases de desarrollo e hitos</td></tr>
</table>

<hr>

<h2>Estructura del Proyecto</h2>

<pre><code>samurai/
├── frontend/              # Angular 21 SPA (componentes independientes)
├── backend/               # FastAPI Python (REST + WebSocket)
│   └── app/
│       ├── main.py        # Rutas API y endpoints WebSocket
│       ├── scanner.py     # Motor de escaneo de puertos Nmap
│       ├── crawler.py     # Crawler de vulnerabilidades DAST
│       ├── db_exporter.py # Exportación de base de datos (crudo + cifrado)
│       └── recon/         # Módulos de reconocimiento web
├── samurai-tui/           # Aplicación de terminal en Rust
│   ├── Dockerfile         # Construcción de contenedor (Rust + nmap)
│   ├── docker-compose.yml # Ejecutor de sesión efímera
│   └── src/
│       ├── main.rs        # Bucle de eventos TUI y atajos de teclado
│       ├── scanner/       # Motor Nmap con transmisión en vivo
│       ├── recon/         # DNS, subdominios, APIs, cabeceras, tecnología
│       ├── crawler/       # Descubrimiento de páginas HTTP
│       ├── export/        # Exportación cruda + cifrada AES-256-GCM
│       ├── db/            # SQLx de doble backend (Postgres + SQLite)
│       └── tui/           # Interfaz de terminal Nothing Design
├── docs/                  # Documentación técnica
└── docker-compose.yml     # 4 servicios: frontend, backend, redis, postgres
</code></pre>

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
