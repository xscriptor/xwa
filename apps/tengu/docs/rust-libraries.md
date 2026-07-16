# Rust Libraries (Backend Dependencies)

Tengu's backend is a Rust application using the Axum web framework. Below is the complete dependency inventory.

## Web Framework & Server

| Crate | Version | Purpose |
|---|---|---|
| `axum` | 0.7 | HTTP framework with WebSocket support |
| `tower-http` | 0.5 | CORS middleware |
| `tokio` | 1 | Async runtime (full features) |
| `tokio-stream` | 0.1 | Async stream utilities |
| `tower` | 0.5 | Async service layers |

## HTTP Client

| Crate | Version | Purpose |
|---|---|---|
| `reqwest` | 0.12 | HTTP client with rustls-tls, gzip, brotli, cookie support |

## HTML Parsing & Selection

| Crate | Version | Purpose |
|---|---|---|
| `scraper` | 0.21 | HTML parser and CSS selector engine (wraps html5ever and selectors) |
| `selectors` | 0.26 | CSS selector matching |
| `ego-tree` | 0.9 | DOM tree data structure (used by scraper) |
| `cssparser` | 0.34 | CSS tokenizer/parser |

## Serialization

| Crate | Version | Purpose |
|---|---|---|
| `serde` | 1 | Serialization framework (with derive) |
| `serde_json` | 1 | JSON serialization/deserialization |
| `url` | 2 | URL parsing and normalization |

## Utilities

| Crate | Version | Purpose |
|---|---|---|
| `uuid` | 1 | Audit record IDs (v4) |
| `chrono` | 0.4 | Timestamps for audit records |
| `dashmap` | 6 | Concurrent in-memory audit store |
| `regex` | 1 | Pattern matching for HTML analysis |
| `sha2` | 0.10 | Hashing (future use) |
| `base64` | 0.22 | Base64 encoding (future use) |
| `thiserror` | 2 | Error type derivation |

## Logging & Observability

| Crate | Version | Purpose |
|---|---|---|
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log output formatting with env-filter |
| `futures` | 0.3 | Async combinators |
