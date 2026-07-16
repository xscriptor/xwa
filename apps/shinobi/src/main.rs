mod api;
mod config;
mod scraper;
mod storage;

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use dashmap::DashMap;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use api::routes::{AppState, self, Schedule};
use storage::db::DbStore;
use storage::manager::StorageManager;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("shinobi=info,tower_http=info")),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let data_dir = std::env::var("DATA_DIR")
        .unwrap_or_else(|_| "downloads".into());

    let db_path = std::env::var("SHINOBI_DB_PATH")
        .unwrap_or_else(|_| "shinobi.db".into());

    let storage = Arc::new(StorageManager::new(&data_dir));

    let db = match DbStore::new(&db_path) {
        Ok(d) => {
            let store = Arc::new(d);
            tracing::info!("Database persistence enabled at {}", db_path);
            Some(store)
        }
        Err(e) => {
            tracing::warn!("Database persistence disabled: {}", e);
            None
        }
    };

    let jobs: Arc<DashMap<String, api::routes::JobInfo>> = Arc::new(DashMap::new());

    if let Some(ref db) = db {
        match db.load_jobs() {
            Ok(saved) => {
                for job in &saved {
                    jobs.insert(job.id.clone(), job.clone());
                }
                tracing::info!("Loaded {} jobs from database", saved.len());
            }
            Err(e) => tracing::error!("Failed to load jobs from database: {}", e),
        }
    }

    let state = AppState::new(storage, jobs, Arc::new(DashMap::new()), db);

    let scheduler_state = state.clone();
    let api_routes = routes::create_router(state);

    let static_dir = if Path::new("static/browser").exists() {
        "static/browser"
    } else {
        "static"
    };
    tracing::info!("Serving static files from: {}", static_dir);

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new(static_dir))
        .layer(CorsLayer::permissive());

    tokio::spawn(async move {
        scheduler_worker(scheduler_state).await;
    });

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Shinobi running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn scheduler_worker(state: AppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        let db = match &state.db {
            Some(d) => d.clone(),
            None => continue,
        };
        let schedules = match db.load_schedules() {
            Ok(s) => s,
            Err(_) => continue,
        };
        let now = chrono::Utc::now();
        for sched in &schedules {
            if !sched.enabled { continue; }
            let next = match chrono::DateTime::parse_from_rfc3339(&sched.next_run) {
                Ok(t) => t.with_timezone(&chrono::Utc),
                Err(_) => continue,
            };
            if next <= now {
                tracing::info!("Scheduler triggering scrape for {}", sched.url);
                let _cfg = serde_json::from_value::<api::routes::ScrapeQuery>(sched.config.clone()).unwrap_or(api::routes::ScrapeQuery {
                    url: sched.url.clone(),
                    depth: Some(2),
                    max_pages: Some(100),
                    ..Default::default()
                });
                let _ = db.save_schedule(&Schedule {
                    next_run: (now + chrono::Duration::minutes(sched.interval_min as i64)).to_rfc3339(),
                    last_run: Some(now.to_rfc3339()),
                    ..sched.clone()
                });
            }
        }
    }
}
