use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, delete},
    Router,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::CorsLayer;

mod api;
mod auditor;
mod config;
mod storage;

#[derive(Clone)]
pub struct AppState {
    pub store: storage::AuditStore,
    pub config: config::TenguConfig,
    pub rate_limiter: Arc<RateLimiter>,
    pub request_count: Arc<AtomicU32>,
    pub audit_semaphore: Arc<tokio::sync::Semaphore>,
}

pub struct RateLimiter {
    max_burst: u32,
    tokens_per_sec: f64,
    state: Mutex<RateState>,
}

struct RateState {
    tokens: f64,
    last_time: Instant,
}

impl RateLimiter {
    pub fn new(burst: u32, per_sec: f64) -> Self {
        Self {
            max_burst: burst,
            tokens_per_sec: per_sec,
            state: Mutex::new(RateState {
                tokens: burst as f64,
                last_time: Instant::now(),
            }),
        }
    }

    pub fn check(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_time).as_secs_f64();
        state.tokens = (state.tokens + elapsed * self.tokens_per_sec).min(self.max_burst as f64);
        state.last_time = now;
        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tengu=info,tower_http=info".into()),
        )
        .init();

    let cfg = config::TenguConfig::from_env();

    // Initialize store (PostgreSQL or in-memory)
    let store = if let Some(db_url) = &cfg.database_url {
        match storage::create_pg_store(db_url).await {
            Ok(s) => {
                tracing::info!("Using PostgreSQL store");
                s
            }
            Err(e) => {
                tracing::warn!("Failed to connect to PostgreSQL ({}), falling back to in-memory", e);
                storage::AuditStore::new()
            }
        }
    } else {
        tracing::info!("Using in-memory store (set DATABASE_URL for PostgreSQL)");
        storage::AuditStore::new()
    };

    let state = AppState {
        store,
        config: cfg.clone(),
        rate_limiter: Arc::new(RateLimiter::new(cfg.rate_limit_burst, cfg.rate_limit_per_second)),
        request_count: Arc::new(AtomicU32::new(0)),
        audit_semaphore: Arc::new(tokio::sync::Semaphore::new(cfg.max_concurrent_audits)),
    };

    let api_routes = Router::new()
        .route("/api/health", get(health))
        .route("/api/metrics", get(metrics_handler))
        .route("/api/audits", get(api::routes::list_audits))
        .route("/api/audits/clear", delete(api::routes::clear_all_audits))
        .route("/api/audits/export", get(api::routes::export_audits))
        .route("/api/audits/import", post(api::routes::import_audits))
        .route("/api/audits/:id", get(api::routes::get_audit))
        .route("/api/audits/:id", delete(api::routes::delete_audit))
        .route("/api/audit/live", get(api::routes::audit_live))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = api_routes
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback(fallback);

    let addr = format!("0.0.0.0:{}", cfg.port);
    tracing::info!("Tengu listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Only check /api/* routes
    let path = req.uri().path();
    if !path.starts_with("/api/") || path == "/api/health" || path == "/api/metrics" {
        return next.run(req).await;
    }

    if let Some(ref expected_key) = state.config.api_key {
        let header_key = req
            .headers()
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let query_key = req.uri().query().and_then(|q| {
            for pair in q.split('&') {
                if let Some(val) = pair.strip_prefix("api_key=") {
                    return Some(val.to_string());
                }
            }
            None
        });

        let provided = header_key.or(query_key);

        match provided {
            Some(ref key) if key == expected_key => next.run(req).await,
            _ => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", "Bearer")
                .body(Body::from("Unauthorized. Provide X-API-Key header or api_key query parameter."))
                .unwrap(),
        }
    } else {
        next.run(req).await
    }
}

async fn health() -> &'static str {
    "OK"
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.request_count.load(Ordering::Relaxed);
    let store_count = state.store.count();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; charset=utf-8")],
        format!(
            "# HELP tengu_requests_total Total HTTP requests\n\
             # TYPE tengu_requests_total counter\n\
             tengu_requests_total {}\n\
             \n\
             # HELP tengu_audits_total Total audits in store\n\
             # TYPE tengu_audits_total gauge\n\
             tengu_audits_total {}\n",
            count, store_count,
        ),
    )
}

async fn fallback(req: Request<Body>) -> impl IntoResponse {
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static/browser".to_string());
    let path = req.uri().path();

    let file_path = if path == "/" {
        format!("{}/index.html", static_dir)
    } else {
        let candidate = format!("{}{}", static_dir, path);
        if tokio::fs::metadata(&candidate).await.is_ok() {
            candidate
        } else {
            format!("{}/index.html", static_dir)
        }
    };

    let ext = file_path.rsplit('.').next().unwrap_or("");
    let content_type = match ext {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    };

    let is_text = matches!(ext, "html" | "js" | "css" | "json" | "svg" | "txt");

    if is_text {
        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .body(Body::from(content))
                .unwrap(),
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap(),
        }
    } else {
        match tokio::fs::read(&file_path).await {
            Ok(content) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .body(Body::from(content))
                .unwrap(),
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap(),
        }
    }
}
