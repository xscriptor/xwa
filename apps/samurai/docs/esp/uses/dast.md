# Motor de Pruebas Dinámicas de Seguridad en Aplicaciones (DAST)



Este documento detalla las capacidades y metodologías empleadas por el módulo Scanner DAST dentro de la plataforma Samurai.

El módulo DAST representa un enfoque de prueba activo de "caja negra". A diferencia del análisis estático, que examina el código fuente en reposo, DAST interactúa con la aplicación web en ejecución desde el exterior, exactamente como lo haría un atacante. Sonda la superficie de ataque de la aplicación en tiempo real, ejecutando exploits no destructivos y analizando las respuestas HTTP para identificar fallos de seguridad.

## Capacidades Principales

### 1. Validación Criptográfica (TLS/SSL)
Antes de interactuar con la capa HTTP, el motor establece una conexión de socket directa para evaluar la seguridad de la capa de transporte (TLS/SSL). Identifica la versión del protocolo negociado y el conjunto de cifrado, marcando explícitamente los servidores que aún soportan protocolos obsoletos y comprometidos como SSLv3, TLSv1.0 o TLSv1.1.

### 2. Auditoría de Cabeceras HTTP y Huellas Digitales
El motor inspecciona pasivamente las respuestas del servidor para identificar la pila tecnológica subyacente (por ejemplo, recuperando las cabeceras `Server` y `X-Powered-By`). También verifica la presencia y configuración correcta de cabeceras de seguridad críticas:
- `Strict-Transport-Security` (HSTS)
- `Content-Security-Policy` (CSP)
- `X-Content-Type-Options`
- Banderas `Set-Cookie` (`HttpOnly`, `Secure`, `SameSite`)

### 3. Mapeo de Superficie Expuesta (Enrutamiento Ciego)
Para descubrir archivos sensibles no protegidos que no están enlazados dentro del HTML de la aplicación, el motor realiza peticiones de fuerza bruta dirigidas contra rutas de configuración comunes. Estos incluyen metadatos de control de versiones (`/.git/config`), archivos de entorno (`/.env`) y listas de dependencias (`/requirements.txt`). Eludir la navegación estándar a menudo revela configuraciones incorrectas críticas de la infraestructura.

### 4. Fuzzing Activo de Parámetros
Durante el recorrido inicial del DOM, el motor intercepta todos los formularios HTML (`<form>`) y sus respectivos campos de entrada. Inyecta sistemáticamente cargas maliciosas en estos parámetros:
- **Inyección SQL (SQLi):** Se inyectan cargas como `' OR 1=1--` para provocar excepciones de base de datos no manejadas que resulten en errores HTTP 500 o filtraciones de sintaxis.
- **Cross-Site Scripting (XSS):** Se inyectan cargas como `<script>alert(1)</script>` para determinar si el servidor refleja la entrada sin modificar en la respuesta HTML, confirmando una vulnerabilidad XSS Reflejado.

## Sistema de Puntuación de Vulnerabilidades Comunes (CVSS)
Cada vulnerabilidad identificada se mapea a una puntuación CVSS estimada y un nivel de severidad basado en la naturaleza del fallo. Por ejemplo, un XSS Reflejado confirmado típicamente produce una puntuación de severidad Alta (ej. 6.1), mientras que una anomalía de Inyección SQL que rompe la sintaxis produce una puntuación Crítica (ej. 9.8).

## Generación de Prueba de Concepto
Para cada vulnerabilidad activa encontrada, el motor registra el método HTTP exacto, la URL y los datos de la carga necesarios para reproducir el exploit. Este rastro de Prueba de Concepto (PoC) permite a los auditores de seguridad verificar y remediar manualmente el problema sin ambigüedad.
