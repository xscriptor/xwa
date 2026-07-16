<h1>Samurai: Arquitectura de UI y Estructura de Carpetas</h1>


<p>Para escalar Samurai a una plataforma integral de ciberseguridad (capaz de manejar descubrimiento nativo de enlaces, grafos topológicos, conteo de vulnerabilidades e inyecciones automatizadas), el frontend Angular debe migrar inmediatamente de un esquema plano a una <strong>Arquitectura Basada en Características</strong>. Esto garantiza escalabilidad infinita, separación de responsabilidades y la capacidad de implementar <em>carga diferida</em> para no saturar el navegador web bajo el esquema <em>Nothing Design</em>.</p>

<hr>

<h2>1. Vista General de Alto Nivel del Directorio</h2>

<pre><code>frontend/src/app/
├── core/         # Servicios Singleton, Interceptores y Guards.
├── shared/       # Componentes de UI "tontos" (Nothing Design), Pipes, Directivas.
├── features/     # Dominios de negocio aislados (Los pilares de la ciberseguridad).
├── layouts/      # Vistas envolventes globales (Barras laterales, Menús, Barras de navegación).
└── app.routes.ts # Reglas de enrutamiento asíncrono (Carga diferida).
</code></pre>

<hr>

<h2>2. Anatomía de Carpetas</h2>

<h3>2.1 El Núcleo: <code>core/</code></h3>
<p>Contiene lógica pesada que se instancia una sola vez. Ningún componente visual debe existir aquí bajo ninguna circunstancia.</p>
<ul>
    <li><code>core/services/websocket.service.ts</code>: Mantiene y monitorea conexiones bidireccionales para comandos stdout entrantes desde FastAPI.</li>
    <li><code>core/services/api.service.ts</code>: Encapsula el cliente HTTP principal.</li>
    <li><code>core/interceptors/auth.interceptor.ts</code>: Inyecta tokens JWT para asegurar rutas API cerradas para usuarios autorizados.</li>
    <li><code>core/guards/auth.guard.ts</code>: Protege rutas del navegador (por ejemplo, redirigiendo si no es administrador).</li>
</ul>

<h3>2.2 Filosofía Visual: <code>shared/</code></h3>
<p>El hogar del ecosistema del <strong>Sistema de Diseño Nothing</strong>. Contiene todo lo reutilizable, "tonto" (componentes sin inyección de dependencias a servicios HTTP). Están dedicados estrictamente a <em>renderizar datos inyectados mediante Inputs</em>.</p>
<ul>
    <li><code>shared/components/nothing-terminal/</code>: La UI final de la consola estilo terminal UNIX en texto puro phosphor.</li>
    <li><code>shared/components/metric-card/</code>: Cajas asimétricas que muestran contadores OLED numéricos.</li>
    <li><code>shared/components/status-badge/</code>: Indicadores luminosos (Rojo, Ámbar o Verde).</li>
    <li><code>shared/styles/</code>: Variables SCSS globales, tokens de tipografía y <em>mixins</em>.</li>
</ul>

<h3>2.3 El Sistema Operativo: <code>features/</code></h3>
<p>Donde residen la lógica y la magia de la ciberseguridad. Cada "característica" dentro de esta carpeta debe comportarse como una micro-aplicación independiente, siendo autónoma y vinculada solo por su ruta principal.</p>

<h4>A. <code>features/scanner/</code></h4>
<p>Responsable de gestionar el escaneo activo de infraestructura y puertos (Nmap, Ping).</p>
<ul>
    <li><code>pages/active-scan-dashboard/</code>: Panel que agrega la terminal y las entradas de configuración de IP.</li>
    <li><code>scanner.service.ts</code>: Servicio aislado que gestiona el estado activo del websocket.</li>
</ul>

<h4>B. <code>features/recon/</code></h4>
<p>Para lógica agresiva que involucra descubrimiento de dominios, fuzzing y navegadores headless.</p>
<ul>
    <li><code>components/network-graph/</code>: Herramienta de visualización topológica (estructura de árbol) que enlaza enlaces descubiertos y URIs de la fase de Spidering.</li>
    <li><code>components/headless-gallery/</code>: Módulo visual encargado de paginar y mostrar <em>Capturas de Pantalla</em> de objetivos recuperados usando Puppeteer o Playwright.</li>
    <li><code>pages/recon-dashboard/</code></li>
</ul>

<h4>C. <code>features/vulnerabilities/</code></h4>
<p>Encargado de listar y consolidar todos los hallazgos críticos (CVEs, fallos XSS, Inyecciones SQL dictadas por SQLMap).</p>
<ul>
    <li><code>components/finding-data-grid/</code>: Una cuadrícula de datos de densidad visual ultra alta (fuente Monospace obligatoria).</li>
    <li><code>components/severity-chart/</code>: Métricas para aislar severidades críticas en tarjetas informativas.</li>
    <li><code>pages/findings-report/</code></li>
</ul>

<h4>D. <code>features/automation/</code></h4>
<p>Planificador de tareas en segundo plano y gestor de horarios (Workers / Celery Beat).</p>
<ul>
    <li><code>components/cron-builder/</code></li>
    <li><code>pages/scheduler/</code></li>
</ul>

<hr>

<h2>3. Gestión de Estado en Alta Demanda</h2>
<p>En el análisis profundo de seguridad donde cientos de URLs o endpoints se descubren por segundo, el "prop-drilling" destruiría el rendimiento de renderizado de la aplicación. Para mitigar visualizaciones estancadas, el árbol de componentes debe emplear una estrategia de <strong>Gestión de Estado Basada en Señales</strong> utilizando las Signals nativas de Angular 17. Las rutas hijas se suscribirán a estas <em>señales</em>, emitiendo reactividad precisa (por ejemplo, un contador de puertos que se incrementa rápidamente) sin re-renderizar todo el contenedor padre (como un grafo topológico a medio construir).</p>

<hr>
<p><i>Arquitectura base diseñada para estructurar el proyecto de ciberseguridad Samurai a nivel empresarial.</i></p>
