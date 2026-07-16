# Tengu

*Auditor de calidad web -- Rendimiento, SEO, Accesibilidad, Buenas Practicas*

[Tengu](https://github.com/xscriptor/tengu) : [XWA](https://github.com/xscriptor/xwa) submodule focused on web quality auditing -- under active development

[English](../../README.md) | [Espanol](README.md)

---

## Descripcion General

Tengu es un auditor de calidad web que evalua aplicaciones en cuatro dimensiones:

| Categoria | Alcance |
|---|---|
| **Rendimiento** | Peso de pagina, Core Web Vitals (LCP, CLS, INP), cascada de recursos, optimizacion de imagenes, carga de fuentes, politica de cache, compresion, impacto de terceros |
| **SEO** | Etiquetas title, meta descriptions, jerarquia de encabezados, URLs canonicas, Open Graph, Twitter Cards, JSON-LD, sitemaps, robots.txt, hreflang, cadenas de redireccion, enlaces rotos |
| **Accesibilidad** | Texto alternativo, estructura de encabezados, atributos ARIA, elementos landmark, contraste de color (WCAG AA/AAA), navegacion por teclado, etiquetas de formularios, texto de enlaces, tablas, iframes, configuracion de viewport |
| **Buenas Practicas** | HTTPS obligatorio, headers de seguridad (HSTS, CSP, XFO, etc.), auditoria de cookies, deteccion de banner GDPR, validacion de doctype, HTML obsoleto, contenido mixto, SRI, errores de consola |

| Interfaz | Directorio | Lenguaje | Tipo |
|---|---|---|---|
| **Tengu Web** | `/` (raiz del monorepo) | Rust (Axum) + Angular 19 | Aplicacion web (standalone o Docker) |

## Inicio Rapido

### Version Web (Docker Compose)

```
docker compose up -d --build
```

Web UI en `http://localhost:8080`.

### Standalone (Sin Docker)

```
cargo run --release
```

Web UI en `http://localhost:8080`.

### Variables de Entorno

| Variable | Por Defecto | Descripcion |
|---|---|---|
| `PORT` | `8080` | Puerto de escucha HTTP |
| `RUST_LOG` | `tengu=info,tower_http=info` | Verbosidad de logging |

## Documentos Relacionados

| Documento | Descripcion |
|---|---|
| [ROADMAP.md](../../ROADMAP.md) | Fases de desarrollo y hitos |
| [CHANGELOG.md](../../CHANGELOG.md) | Historial de versiones |
| [manual.md](manual.md) | Guia de despliegue y uso |
| [ui-architecture.md](ui-architecture.md) | Arquitectura del frontend |
| [rust-libraries.md](rust-libraries.md) | Dependencias Rust del backend |
| [uses/audit.md](uses/audit.md) | Guia de uso del auditor |

## Estructura del Proyecto

```
tengu/
├── src/
│   ├── main.rs              # Punto de entrada del servidor Axum
│   ├── config.rs            # Configuracion de auditoria
│   ├── api/
│   │   └── routes.rs        # API REST + WebSocket
│   ├── auditor/
│   │   ├── mod.rs           # Orquestador + modelo Finding
│   │   ├── performance.rs   # Peso, recursos, imagenes, fuentes
│   │   ├── seo.rs           # Meta tags, datos estructurados
│   │   ├── a11y.rs          # Cumplimiento WCAG, ARIA
│   │   ├── best_practices.rs# Headers de seguridad, cookies
│   │   └── html_pretty.rs   # Pretty-printing de HTML
│   └── storage/
│       └── mod.rs           # Almacenamiento en memoria
├── frontend/                # Angular 19
│   ├── package.json
│   ├── angular.json
│   └── src/app/
│       ├── features/audit/  # Configuracion y resultados
│       └── features/history/# Historial de auditorias
├── docs/                    # Documentacion tecnica
│   ├── manual.md
│   ├── ui-architecture.md
│   ├── rust-libraries.md
│   ├── uses/audit.md
│   └── esp/                 # Documentacion en espanol
├── Dockerfile
├── docker-compose.yml
├── Cargo.toml
├── ROADMAP.md
└── CHANGELOG.md
```

## Script de Inicio

```
./tengu.sh                       # Standalone efimero
./tengu.sh --docker               # via docker-compose
./tengu.sh --docker -e            # Docker efimero (--rm)
./tengu.sh --export archivo.json  # Exportar a ./exports/
./tengu.sh --import archivo.json  # Importar JSON
```
