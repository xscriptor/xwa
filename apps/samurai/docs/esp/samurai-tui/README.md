# Samurai Terminal Edition (samurai-tui)


Interfaz de terminal para la plataforma de ciberseguridad Samurai. Escaneo de puertos, reconocimiento web, crawling de vulnerabilidades y exportación de base de datos. Funciona de forma independiente sin configuración — no requiere Docker ni PostgreSQL.

**Versión:** 2.5.0

---

## Inicio Rápido (Independiente)

```bash
cd samurai-tui
cargo build --release
cargo run --release
```

La aplicación crea automáticamente un archivo `samurai.db` SQLite y está lista para usar inmediatamente.

> **Requisitos:** Rust 1.80+ y `nmap` en PATH. Acceso a Internet para reconocimiento web y crawling.

---

## Uso con Docker

### Construir la Imagen

```bash
docker compose build --no-cache
```

### Sesión Persistente (los datos sobreviven a reinicios)

```bash
docker compose up --build
```

- Los datos se almacenan en un volumen Docker nombrado (`samurai-tui-data`)
- Sobrevive a reinicios y recreaciones de contenedores
- Usa `docker compose up -d` para ejecutar en segundo plano, luego `docker attach samurai-tui` para conectarte

### Sesión Efímera (sin dejar rastro)

```bash
docker compose run --rm -it samurai-tui
```

- **`--rm`** destruye el contenedor + los datos al salir
- **`-it`** es **obligatorio** — asigna la TTY interactiva que la TUI necesita
- Sin `-it`, la aplicación se inicia y sale inmediatamente (no hay terminal disponible)
- El contenedor es efímero: cada ejecución es un entorno nuevo

### Por Qué `-it` es Obligatorio

La aplicación usa el modo raw de `crossterm` + la pantalla alternativa de `ratatui`, que requieren una terminal real. Sin `-it`:

```
$ docker compose run --rm samurai-tui
Contenedor creado -> sale inmediatamente (código 0, sin salida)
```

Con `-it`:

```
$ docker compose run --rm -it samurai-tui
Contenedor creado -> la TUI se inicia -> interactúas -> q para salir -> contenedor eliminado
```

### Ejecutar en Docker Remoto

```bash
# Construir en la máquina local
docker build -t samurai-tui .
docker save samurai-tui | ssh user@remote 'docker load'

# En el remoto
ssh -t user@remote 'docker run -it --rm samurai-tui'
```

La bandera `-t` en `ssh` también es necesaria por la misma razón — acceso raw a la terminal.

---

## Proxy, VPN y Anti-Bloqueo

Para evitar que los servidores destino bloqueen el escaneo por abuso, samurai-tui soporta enrutar todo el tráfico de red a través de proxies o VPNs.

Todas las variables son **opcionales** — sin ellas el tráfico sale directo.

### Proxy HTTP/HTTPS (recon, crawler)

Útil para usar con **Burp Suite**, **mitmproxy**, **Charles**, o cualquier proxy HTTP:

```bash
docker compose run --rm -it \
  -e SAMURAI_PROXY=http://host.docker.internal:8080 \
  samurai-tui
```

En modo independiente:
```bash
SAMURAI_PROXY=http://127.0.0.1:8080 cargo run --release
```

### Proxy SOCKS5 + Tor (rotación de IP cada 10 s)

El método más práctico para IP rotativa. Tor cambia de circuito cada ~10 segundos.

```bash
# 1. Iniciar Tor
docker run -d --rm --name tor-proxy \
  -p 9050:9050 -p 9051:9051 \
  dperson/torproxy

# 2. Ejecutar samurai-tui con Tor como proxy
docker compose run --rm -it \
  -e SAMURAI_PROXY_DNS=socks5h://host.docker.internal:9050 \
  -e SAMURAI_NMAP_PROXY=socks4://host.docker.internal:9050 \
  samurai-tui
```

**Explicación de `socks5h://`**: la `h` hace que la resolución DNS ocurra del lado del proxy (Tor), no local. Así el objetivo nunca ve tu DNS resolver real.

### Proxy para nmap

Nmap soporta `--proxies` con SOCKS4. Usa la variable `SAMURAI_NMAP_PROXY`:

```bash
docker compose run --rm -it \
  -e SAMURAI_NMAP_PROXY=socks4://host.docker.internal:9050 \
  samurai-tui
```

### VPN Dedicada (contenedor separado)

Para aislamiento completo de red, usa un contenedor VPN como sidecar:

```yaml
# docker-compose.override.yml (crear junto al docker-compose.yml existente)
services:
  vpn:
    image: qmcgaw/gluetun  # WireGuard / OpenVPN
    cap_add:
      - NET_ADMIN
    environment:
      - VPN_SERVICE_PROVIDER=mullad
      - VPN_TYPE=wireguard
      - WIREGUARD_PRIVATE_KEY=...
      - WIREGUARD_ADDRESSES=...
    volumes:
      - gluetun:/gluetun

  samurai-tui:
    network_mode: "service:vpn"  # comparte la red de la VPN
    depends_on:
      vpn:
        condition: service_healthy
    environment:
      - SAMURAI_NMAP_PROXY=         # tráfico directo por la VPN
      - SAMURAI_PROXY=              # (opcional, proxy dentro de la VPN)

volumes:
  gluetun:
```

Luego levantas todo junto:
```bash
docker compose up --build
```

### Múltiples Contenedores con Diferentes IPs (Rotación Manual)

Puedes lanzar varias instancias con diferentes proxies:

```bash
# Terminal 1 — Tor circuito A
docker compose run --rm -it \
  -e SAMURAI_PROXY_DNS=socks5h://host.docker.internal:9050 \
  samurai-tui

# Terminal 2 — Tor circuito B (otro puerto)
# Necesitas otro contenedor Tor en otro puerto
SAMURAI_PROXY=socks5h://127.0.0.1:9052 cargo run --release
```

---

## Cómo Funciona

### Ciclo de Vida de la Sesión

```
Inicio
  │
  ├─ ¿PostgreSQL disponible? ──► usar pool de PostgreSQL
  └─ ¿No hay PostgreSQL? ───────► usar SQLite (crear archivo .db automáticamente)
        │
  ╔═══════════════════════════════════════╗
  ║    Inicializar tablas DB (idempotente)║
  ║    Crear clientes HTTP/DNS compartidos║
  ║    Cargar historial de escaneos desde DB║
  ║    Entrar en modo raw + pantalla alt. ║
  ║    ──────────────────────────────     ║
  ║    Bucle principal (sondeo 33 ms):    ║
  ║    • Entrada de teclado (escritura,   ║
  ║      Tab, Enter, flechas, q/Esc...)   ║
  ║    • Clics de ratón (selección pestaña)║
  ║    • Eventos de fondo (logs de        ║
  ║      escaneo, descubrimientos, fin)   ║
  ║    ──────────────────────────────     ║
  ║    q presionada ──► salir del bucle   ║
  ╚═══════════════════════════════════════╝
        │
  Restaurar terminal, limpiar, salir
```

### Arquitectura de Tareas Asíncronas

Todas las operaciones de escaneo se ejecutan en tareas Tokio en segundo plano. Los resultados fluyen de vuelta a la interfaz a través de un canal de eventos asíncrono:

```
Enter presionado
       │
       ▼
  Establecer banderas de ejecución (en App)
  Limpiar resultados anteriores
  Generar tarea en segundo plano
       │
       ▼
  ┌──────────────────┐        mpsc::channel        ┌──────────────────┐
  │  Tarea de Fondo   │ ──────── BgEvent ─────────► │  Bucle Principal │
  │  (run_scanner,    │     (ScannerLog,            │  (handle_bg_event)│
  │   run_recon,      │      ScannerPort,           │       │          │
  │   run_crawler,    │      ScannerDone,           │       ▼          │
  │   run_export)     │      ReconLog/ReconDone,    │  Actualizar App  │
  │                   │      CrawlerLog/CrawlerDone,│  (puertos, res.,  │
  │  Cada tarea:      │      ExportDone, Status)    │   logs, estado)  │
  │  • Ejecuta nmap / │                             │       │          │
  │    consultas DNS /│                             │       ▼          │
  │    peticiones HTTP│                             │  Terminal.draw() │
  │  • Guarda en DB   │                             │  (re-render UI)  │
  │  • Envía eventos  │                             └──────────────────┘
  └──────────────────┘
```

- Los eventos se drenan **antes y después** de cada sondeo de entrada de usuario
- La terminal se re-renderiza cada 33 ms
- Las operaciones en ejecución pueden cancelarse con `Esc` (establece `_running = false`, la tarea finaliza correctamente)

---

## Atajos de Teclado

| Tecla | Acción |
|---|---|
| `Tab` | Cambiar pestaña (Scanner → Recon → Crawler → History → Export) |
| `Enter` | Ejecutar escaneo / reconocimiento / crawling / exportación (o eliminar escaneo seleccionado en History) |
| `Left / Right` | Ajustar perfil (Scanner) o máximo de páginas (Crawler) |
| `Up / Down` | Navegar por la lista de historial |
| `Space` | Alternar modo de exportación (Raw / Encrypted) |
| `Esc` | Cancelar operación en ejecución |
| `q` | Salir (solo cuando está inactivo) |
| Escritura | Ingresar objetivo, contraseña o ruta de exportación |
| `Backspace` | Eliminar el último carácter |
| **🖱️ Clic izquierdo** | **Cambiar pestaña** (clic en el nombre de la pestaña en la parte superior) |

---

## Pestañas

| Pestaña | Descripción |
|---|---|
| **Scanner** | Escaneo de puertos Nmap con 4 perfiles. Transmisión de logs en tiempo real + panel de puertos abiertos. |
| **Recon** | Enumeración DNS, descubrimiento de subdominios (crt.sh + resolución), sondeo de API (32 rutas), verificación de cabeceras de seguridad (7), huellas digitales de tecnología. Panel de resumen de resultados. |
| **Crawler** | Descubrimiento de páginas mediante extracción de enlaces HTML. Máximo de páginas configurable (1–20). Lista de páginas con código de estado. |
| **History** | Navegar, inspeccionar y eliminar escaneos pasados. Barra de resumen de severidad, hallazgos con puntuaciones CVSS, enlaces descubiertos con códigos de estado. |
| **Export** | Exportar base de datos como JSON crudo o binario cifrado AES-256-GCM. |

---

## Perfiles de Escaneo

| Perfil | Argumentos de Nmap |
|---|---|
| `quick` | `-T4 --min-rate 1000 -sV -sC --top-ports 1000 --host-timeout 180s` |
| `balanced` | `-T4 --min-rate 1000 -sV -sC -p 1-10000 --host-timeout 300s` |
| `deep` | `-T4 --min-rate 1000 -sV -sC -p- --script vuln --host-timeout 600s` |
| `udp` | `-T4 --min-rate 1000 -sU -sV --top-ports 1000 --host-timeout 300s` |

---

## Almacenamiento

Por defecto, `samurai-tui` almacena datos en un archivo SQLite local (`samurai.db`). No necesita servidor de base de datos.

| Escenario | Backend | Archivo |
|---|---|---|
| PostgreSQL disponible | PostgreSQL | Configurable mediante variables de entorno |
| No se encuentra PostgreSQL | SQLite (auto-fallback) | `samurai.db` (local) o `/data/samurai.db` (Docker) |

Los escaneos realizados en la TUI y la versión web de Samurai son interoperables cuando ambas comparten la misma base de datos PostgreSQL.

---

## Variables de Entorno

| Variable | Por Defecto | Descripción |
|---|---|---|
| `SAMURAI_DB_BACKEND` | auto-detect | Forzar: `sqlite` o `postgres` |
| `SAMURAI_SQLITE_PATH` | `samurai.db` | Ruta del archivo SQLite |
| `RUST_LOG` | `info` | `debug`, `info`, `warn`, `error` |
| `DB_HOST` | `localhost` | Host de PostgreSQL |
| `DB_NAME` | `samurai` | Nombre de la base de datos PostgreSQL |
| `DB_USER` | `postgres` | Usuario de PostgreSQL |
| `DB_PASS` | `postgres` | Contraseña de PostgreSQL |
| `SAMURAI_PROXY` | *(ninguno)* | Proxy HTTP/HTTPS para todas las peticiones HTTP (ej. `http://127.0.0.1:8080`) |
| `SAMURAI_PROXY_DNS` | *(ninguno)* | Proxy SOCKS5 con resolución DNS remota para peticiones HTTP (ej. `socks5h://127.0.0.1:9050`) |
| `SAMURAI_NMAP_PROXY` | *(ninguno)* | Proxy para escaneos nmap (se pasa como `--proxies` a nmap, ej. `socks4://127.0.0.1:9050`) |

### Registro de Logs

```bash
RUST_LOG=debug cargo run --release    # verboso
RUST_LOG=info cargo run --release     # por defecto
RUST_LOG=warn cargo run --release     # silencioso
```

---

## Exportación de Base de Datos y Compatibilidad Cruzada

### Exportar desde Samurai Web a TUI

1. En la aplicación web, navega a **Export DB** (barra lateral `05 // EXPORT DB`)
2. Descarga como **RAW JSON** — un archivo llamado `samurai-database-export-YYYY-MM-DD.json`
3. En la TUI, ve a la pestaña **Export** para exportar los mismos datos a un archivo

Ambas interfaces comparten esquemas de tabla idénticos (`scans`, `findings`, `discovered_links`). Cuando apuntan a la misma instancia de PostgreSQL, los datos se comparten automáticamente — no se necesita exportación.

### Formato de Cifrado

```
SAMURAI_DB_EXPORT_V1 | 16-byte salt | 12-byte nonce | AES-256-GCM ciphertext
```

Clave derivada mediante PBKDF2-SHA256 con 600,000 iteraciones. Compatible con el backend web.

---

## Arquitectura

```
src/
├── main.rs             # Punto de entrada, bucle de eventos, TerminalGuard, despacho de teclado + ratón
├── app.rs              # Estado de la aplicación (struct App), estado de pestañas/escáner/recon/crawler/export
├── tasks.rs            # Funciones de tareas en segundo plano (run_scanner, run_recon, run_crawler, run_export)
│                       # + enum BgEvent para canal de eventos asíncrono
├── db/
│   ├── connection.rs   # Pool de doble backend (PostgreSQL + SQLite con auto-fallback)
│   ├── models.rs       # Tipos Scan, Finding, DiscoveredLink, ExportPayload
│   └── operations.rs   # CRUD + constructor de payload de exportación (ambos backends)
├── scanner/
│   └── engine.rs       # Subproceso Nmap: transmisión en vivo, timeout, manejo seguro de errores
├── recon/
│   ├── dns.rs          # A, AAAA, MX, NS, TXT, SOA, CNAME mediante hickory-resolver
│   ├── subdomains.rs   # Transparencia de certificados crt.sh + resolución DNS
│   ├── api.rs          # 32 rutas de API sondeadas concurrentemente
│   ├── headers.rs      # 7 cabeceras de seguridad + verificación de divulgación del servidor
│   └── tech_stack.rs   # Selectores precompilados para React, Vue, Angular, jQuery, etc.
├── crawler/
│   └── mod.rs          # Análisis HTML, extracción de enlaces, búsqueda concurrente de subpáginas
├── export/
│   ├── mod.rs          # Coordinador de exportación (JSON crudo + binario cifrado)
│   └── crypto.rs       # AES-256-GCM + PBKDF2-SHA256 (600k iteraciones)
└── tui/
    ├── theme.rs        # Paleta de colores oscuros Nothing Design
    └── ui.rs           # Renderizado de 5 pestañas con insignias de severidad, áreas de ratón, colores de log

Flujo de datos: Entrada de Usuario → Bucle de Eventos → Tarea de Fondo ──(canal BgEvent)──→ Estado de App → Terminal.draw()
```

---

## Sistema de Diseño

Estética de panel de instrumentos oscuro siguiendo la filosofía Nothing Design:

| Color | Hex | Uso |
|---|---|---|
| Fondo | `#000000` | Negro OLED |
| Superficie | `#111111` | Paneles, tarjetas |
| Superficie Elevada | `#1A1A1A` | Selección activa |
| Texto | `#E8E8E8` | Cuerpo principal |
| Dorado | `#FFD700` | Puertos abiertos |
| Rojo | `#D71921` | Crítico, errores |
| Verde | `#4A9E5C` | Éxito, completado |
| Ámbar | `#D4A843` | Advertencias, en ejecución |
| Azul | `#5B9BF6` | Interactivo |

---

## Dependencias

| Crate | Propósito |
|---|---|
| `ratatui` + `crossterm` | Interfaz de terminal + modo raw + eventos de ratón |
| `tokio` | Runtime asíncrono (tareas, canales, temporizadores) |
| `sqlx` (postgres + sqlite) | Almacenamiento de doble backend |
| `reqwest` | Cliente HTTP (compartido, agrupado, relajación de certificados) |
| `hickory-resolver` | Resolución DNS (compartida, en caché) |
| `scraper` | Análisis HTML para el crawler |
| `aes-gcm` + `pbkdf2` + `sha2` | Cifrado |
| `serde` + `serde_json` | Serialización |
| `futures` | Primitivas de concurrencia |
| `clap` | Análisis de argumentos CLI |
| `chrono` | Formateo de marcas de tiempo |
