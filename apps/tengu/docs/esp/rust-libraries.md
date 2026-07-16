# Librerias Rust (Dependencias del Backend)

El backend de Tengu es una aplicacion Rust que utiliza el framework web Axum. A continuacion se presenta el inventario completo de dependencias.

## Framework Web y Servidor

| Crate | Version | Proposito |
|---|---|---|
| `axum` | 0.7 | Framework HTTP con soporte WebSocket |
| `tower-http` | 0.5 | Middleware CORS |
| `tokio` | 1 | Runtime asincrono (features completas) |
| `tokio-stream` | 0.1 | Utilidades de streams asincronos |
| `tower` | 0.5 | Capas de servicio asincrono |

## Cliente HTTP

| Crate | Version | Proposito |
|---|---|---|
| `reqwest` | 0.12 | Cliente HTTP con rustls-tls, gzip, brotli, soporte de cookies |

## Parseo y Seleccion HTML

| Crate | Version | Proposito |
|---|---|---|
| `scraper` | 0.21 | Parser HTML y motor de seleccion CSS (envuelve html5ever y selectors) |
| `selectors` | 0.26 | Coincidencia de selectores CSS |
| `ego-tree` | 0.9 | Estructura de datos de arbol DOM (usado por scraper) |
| `cssparser` | 0.34 | Tokenizador/parser CSS |

## Serializacion

| Crate | Version | Proposito |
|---|---|---|
| `serde` | 1 | Framework de serializacion (con derive) |
| `serde_json` | 1 | Serializacion/deserializacion JSON |
| `url` | 2 | Parseo y normalizacion de URLs |

## Utilidades

| Crate | Version | Proposito |
|---|---|---|
| `uuid` | 1 | IDs de registros de auditoria (v4) |
| `chrono` | 0.4 | Marcas de tiempo para registros de auditoria |
| `dashmap` | 6 | Almacenamiento concurrente en memoria |
| `regex` | 1 | Coincidencia de patrones para analisis HTML |
| `sha2` | 0.10 | Hashing (uso futuro) |
| `base64` | 0.22 | Codificacion Base64 (uso futuro) |
| `thiserror` | 2 | Derivacion de tipos de error |

## Logging y Observabilidad

| Crate | Version | Proposito |
|---|---|---|
| `tracing` | 0.1 | Logging estructurado |
| `tracing-subscriber` | 0.3 | Formateo de salida de log con env-filter |
| `futures` | 0.3 | Combinadores asincronos |
