<h1>Manual de la Aplicación Samurai</h1>



<p>Este documento detalla los pasos necesarios para ejecutar la aplicación en un entorno de desarrollo local y proporciona la configuración arquitectónica requerida para un despliegue en producción.</p>

<hr>

<h2>1. Ejecución en Desarrollo Local</h2>

<p>El entorno local está configurado con Reemplazo de Módulos en Caliente (HMR) tanto para el frontend Angular como para el backend FastAPI. Los cambios realizados en los archivos fuente se reflejarán inmediatamente sin necesidad de reconstruir los contenedores.</p>

<h3>1.1 Prerrequisitos</h3>
<ul>
    <li>Docker Engine instalado y ejecutándose.</li>
    <li>Docker Compose instalado.</li>
</ul>

<h3>1.2 Instrucciones de Inicio</h3>
<ol>
    <li>Abre una terminal y navega al directorio raíz del proyecto: <code>/samurai</code></li>
    <li>Ejecuta el siguiente comando para construir e iniciar todos los contenedores orquestados en modo desacoplado:</li>
</ol>

<pre><code>docker compose up -d --build</code></pre>

<h3>1.3 Ejecución Local (Sin Docker) / Configuración de IDE</h3>
<p>Si deseas ejecutar la aplicación Angular manualmente o simplemente quieres que tu IDE (como VSCode) deje de resaltar errores de TypeScript, debes instalar las dependencias Node localmente en tu máquina anfitriona:</p>
<ol>
    <li>Navega a la carpeta frontend: <code>cd frontend/</code></li>
    <li>Instala los paquetes: <code>npm install</code></li>
    <li>Para iniciar el servidor web manualmente sin Docker: <code>npm run start</code> (El frontend arrancará en <code>localhost:4200</code>)</li>
</ol>

<h3>1.4 Acceso a los Servicios</h3>
<ul>
    <li><strong>Frontend (Interfaz Angular):</strong> Accesible en <code>http://localhost:4200</code></li>
    <li><strong>Backend API (FastAPI):</strong> Accesible en <code>http://localhost:8000/docs</code> (Swagger UI)</li>
</ul>

<hr>

<h2>2. Configuración de Producción</h2>

<p>Para un entorno de producción, la configuración de desarrollo debe modificarse para garantizar seguridad, rendimiento y estabilidad. Los siguientes pasos describen los cambios necesarios para realizar la transición de la arquitectura.</p>

<h3>2.1 Optimización del Frontend (Nginx)</h3>
<p>En producción, el servidor de desarrollo de Angular debe ser reemplazado por una compilación estática servida a través de un servidor web de alto rendimiento como Nginx.</p>
<ul>
    <li>Crea un <code>Dockerfile.prod</code> dentro del directorio frontend que implemente una compilación multi-etapa.</li>
    <li>Etapa 1: Compila el proyecto Angular usando <code>npx @angular/cli build --configuration production</code>.</li>
    <li>Etapa 2: Copia los archivos compilados de <code>/dist/samurai-web/browser</code> al directorio de servicio estático de Nginx <code>/usr/share/nginx/html</code>.</li>
    <li>Actualiza el servicio frontend en <code>docker-compose.yml</code> para que use <code>Dockerfile.prod</code> y exponga el puerto <code>80</code> (o <code>443</code> para HTTPS) en lugar de <code>4200</code>.</li>
    <li>Elimina los volúmenes locales del servicio frontend en <code>docker-compose.yml</code>.</li>
</ul>

<h3>2.2 Seguridad y Rendimiento del Backend API</h3>
<p>La aplicación FastAPI debe optimizarse para cargas de trabajo de producción.</p>
<ul>
    <li>Actualiza el comando del <code>Dockerfile</code> del backend para usar Gunicorn con workers Uvicorn en lugar de ejecutar el script shell de Uvicorn directamente. Ejemplo: <code>CMD ["gunicorn", "app.main:app", "--workers", "4", "--worker-class", "uvicorn.workers.UvicornWorker", "--bind", "0.0.0.0:8000"]</code></li>
    <li>Elimina la bandera <code>--reload</code>.</li>
    <li>Elimina los volúmenes de desarrollo del servicio backend en <code>docker-compose.yml</code> para evitar la manipulación del código fuente desde el anfitrión.</li>
    <li>Configura el middleware CORS en <code>main.py</code>. Reemplaza <code>allow_origins=["*"]</code> con el dominio de producción específico.</li>
</ul>

<h3>2.3 Gestión de Base de Datos y Secretos</h3>
<p>La seguridad es primordial para las capas persistentes.</p>
<ul>
    <li>No expongas los puertos de Redis y PostgreSQL a la red pública. Elimina el enlace <code>ports:</code> de ambos servicios en <code>docker-compose.yml</code> para que permanezcan aislados dentro de la red interna de Docker.</li>
    <li>Migra las credenciales codificadas (como <code>DB_USER</code> y <code>DB_PASS</code>) a Docker Secrets o un gestor de secretos externo (por ejemplo, AWS Secrets Manager, HashiCorp Vault). Usa un archivo <code>.env</code> inyectado como solución intermedia.</li>
    <li>Asegúrate de que el volumen de la base de datos PostgreSQL tenga copias de seguridad regulares mediante tareas automatizadas de cron adjuntas a la capa de persistencia.</li>
</ul>

<hr>

<p><i>Fin del Manual.</i></p>
