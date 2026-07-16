use std::sync::Arc;
use std::convert::Infallible;
use std::sync::atomic::AtomicUsize;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Sse},
    routing::{get, post, delete},
};
use axum::response::sse::Event;
use chrono::Utc;
use dashmap::DashMap;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::config::ScrapeConfig;
use crate::scraper::downloader::{Downloader, ScrapeProgress};
use crate::storage::db::DbStore;
use crate::storage::manager::StorageManager;
use serde_json::Value as JsonValue;
use std::io::Write;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageManager>,
    pub jobs: Arc<DashMap<String, JobInfo>>,
    pub downloaders: Arc<DashMap<String, Arc<Downloader>>>,
    pub db: Option<Arc<DbStore>>,
    pub scrape_semaphore: Arc<Semaphore>,
    pub active_scrapes: Arc<AtomicUsize>,
}

impl AppState {
    pub fn new(storage: Arc<StorageManager>, jobs: Arc<DashMap<String, JobInfo>>, downloaders: Arc<DashMap<String, Arc<Downloader>>>, db: Option<Arc<DbStore>>) -> Self {
        Self { storage, jobs, downloaders, db, scrape_semaphore: Arc::new(Semaphore::new(3)), active_scrapes: Arc::new(AtomicUsize::new(0)) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub url: String,
    pub status: String,
    pub created_at: String,
    pub pages_scraped: usize,
    pub files_downloaded: usize,
    pub total_pages: usize,
    pub current_url: Option<String>,
    pub errors: Vec<String>,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ScrapeQuery {
    pub url: String,
    #[serde(default)]
    pub depth: Option<u32>,
    #[serde(default)]
    pub concurrency: Option<usize>,
    #[serde(default = "default_delay")]
    pub delay_ms: Option<u64>,
    #[serde(default)]
    pub max_pages: Option<usize>,
    #[serde(default = "default_true")]
    pub same_domain_only: Option<bool>,
    #[serde(default)]
    pub download_assets: Option<bool>,
    #[serde(default = "default_true")]
    pub user_agent_rotation: Option<bool>,
    #[serde(default)]
    pub file_types: Option<Vec<String>>,
    #[serde(default)]
    pub javascript_rendering: Option<bool>,
    #[serde(default)]
    pub take_screenshots: Option<bool>,
    #[serde(default)]
    pub extract_emails: Option<bool>,
    #[serde(default)]
    pub webhook_url: Option<String>,
    #[serde(default = "default_true")]
    pub deduplicate: Option<bool>,
    #[serde(default = "default_true")]
    pub respect_robots_txt: Option<bool>,
    #[serde(default = "default_true")]
    pub rewrite_urls: Option<bool>,
    #[serde(default)]
    pub generate_index: Option<bool>,
    #[serde(default)]
    pub export_warc: Option<bool>,
    #[serde(default)]
    pub auth_username: Option<String>,
    #[serde(default)]
    pub auth_password: Option<String>,
    #[serde(default)]
    pub auth_mode: Option<String>,
    #[serde(default)]
    pub rate_limit: Option<u64>,
    #[serde(default)]
    pub deep_mode: Option<bool>,
    #[serde(default)]
    pub extract_structured: Option<bool>,
    #[serde(default)]
    pub nlp_enabled: Option<bool>,
    #[serde(default)]
    pub custom_selectors: Option<Vec<String>>,
    #[serde(default = "default_export_format")]
    pub export_format: Option<String>,
    #[serde(default)]
    pub extractor_endpoint: Option<String>,
}

fn default_delay() -> Option<u64> { Some(1000) }
fn default_true() -> Option<bool> { Some(true) }
fn default_export_format() -> Option<String> { Some("json".into()) }

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/scrape", post(start_scrape))
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/jobs/:id/stream", get(stream_job))
        .route("/jobs/:id/cancel", post(cancel_job))
        .route("/jobs/:id", delete(delete_job))
        .route("/jobs/:id/export", post(export_job))
        .route("/jobs/:id/download", get(download_job_zip))
        .route("/database/export", get(export_database))
        .route("/database/import", post(import_database))
        .route("/health", get(health_check))
        .route("/files", get(list_files))
        .route("/files/*path", get(get_file))
        .route("/search", get(search_files))
        .route("/stats", get(get_stats))
        .route("/python/docs", get(python_docs_proxy))
        .route("/deep/scrape", post(start_deep_scrape))
        .route("/deep/batch", post(start_deep_batch))
        .route("/deep/crawl", post(start_deep_crawl))
        .route("/deep/crawl/:id/status", get(get_deep_crawl_status))
        .route("/deep/crawl/:id/results", get(get_deep_crawl_results))
        .route("/deep/crawl/:id/cancel", post(cancel_deep_crawl))
        .route("/deep/results", get(list_deep_results))
        .route("/deep/results", delete(clear_deep_results))
        .route("/deep/results/:id", get(get_deep_result))
        .route("/deep/results/:id", delete(delete_deep_result))
        .route("/deep/results.csv", get(export_deep_csv))
        .route("/database/clear", post(clear_database))
        .route("/schedules", get(list_schedules).post(create_schedule))
        .route("/schedules/:id", delete(delete_schedule))
        .with_state(state)
}

async fn start_scrape(
    State(state): State<AppState>,
    Json(query): Json<ScrapeQuery>,
) -> Result<Json<JobInfo>, (StatusCode, Json<serde_json::Value>)> {
    if query.url.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "URL is required"}))));
    }

    let config = Arc::new(ScrapeConfig {
        url: query.url.clone(),
        depth: query.depth.unwrap_or(2),
        concurrency: query.concurrency.unwrap_or(3),
        delay_ms: query.delay_ms.unwrap_or(1000),
        max_pages: query.max_pages.unwrap_or(100),
        same_domain_only: query.same_domain_only.unwrap_or(true),
        download_assets: query.download_assets.unwrap_or(true),
        user_agent_rotation: query.user_agent_rotation.unwrap_or(true),
        file_types: query.file_types.unwrap_or_default(),
        javascript_rendering: query.javascript_rendering.unwrap_or(false),
        take_screenshots: query.take_screenshots.unwrap_or(false),
        extract_emails: query.extract_emails.unwrap_or(false),
        webhook_url: query.webhook_url.unwrap_or_default(),
        deduplicate: query.deduplicate.unwrap_or(true),
        respect_robots_txt: query.respect_robots_txt.unwrap_or(true),
        rewrite_urls: query.rewrite_urls.unwrap_or(true),
        generate_index: query.generate_index.unwrap_or(false),
        export_warc: query.export_warc.unwrap_or(false),
        auth_username: query.auth_username.unwrap_or_default(),
        auth_password: query.auth_password.unwrap_or_default(),
        auth_mode: query.auth_mode.unwrap_or_default(),
        rate_limit: query.rate_limit.unwrap_or(0),
        deep_mode: query.deep_mode.unwrap_or(false),
        extract_structured: query.extract_structured.unwrap_or(false),
        nlp_enabled: query.nlp_enabled.unwrap_or(false),
        custom_selectors: query.custom_selectors.unwrap_or_default(),
        export_format: query.export_format.unwrap_or_else(|| "json".into()),
        extractor_endpoint: query.extractor_endpoint.unwrap_or_default(),
        ..Default::default()
    });

    let job_id = Uuid::new_v4().to_string();
    let job_info = JobInfo {
        id: job_id.clone(), url: query.url.clone(), status: "queued".into(),
        created_at: Utc::now().to_rfc3339(), pages_scraped: 0, files_downloaded: 0,
        total_pages: config.max_pages, current_url: None, errors: Vec::new(),
        emails: Vec::new(), phones: Vec::new(),
    };

    let storage = state.storage.clone();
    let jobs = state.jobs.clone();
    let downloaders = state.downloaders.clone();
    let db = state.db.clone();
    let sem = state.scrape_semaphore.clone();
    let active = state.active_scrapes.clone();

    jobs.insert(job_id.clone(), job_info.clone());

    tokio::spawn(async move {
        let _permit = sem.acquire().await.ok();
        active.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, mut rx) = mpsc::channel::<ScrapeProgress>(100);

        let downloader = match Downloader::new(config, storage).await {
            Ok(d) => Arc::new(d),
            Err(e) => {
                if let Some(mut job) = jobs.get_mut(&job_id) {
                    job.status = "failed".into();
                    job.errors.push(e);
                    if let Some(ref db) = db { let _ = db.save_job(&job); }
                }
                return;
            }
        };

        downloaders.insert(job_id.clone(), downloader.clone());
        if let Some(mut job) = jobs.get_mut(&job_id) {
            job.status = "running".into();
            if let Some(ref db) = db { let _ = db.save_job(&job); }
        }

        let dl = downloader.clone();
        let jid = job_id.clone();
        let j = jobs.clone();
        let dls = downloaders.clone();
        let db2 = db.clone();
        tokio::spawn(async move {
            dl.run(tx).await;
            if let Some(mut job) = j.get_mut(&jid) {
                if job.status != "cancelled" { job.status = "completed".into(); }
                if let Some(ref db) = db2 { let _ = db.save_job(&job); }
            }
            dls.remove(&jid);
        });

        while let Some(progress) = rx.recv().await {
            if let Some(mut job) = jobs.get_mut(&job_id) {
                job.pages_scraped = progress.pages_scraped;
                job.files_downloaded = progress.files_downloaded;
                job.current_url = progress.current_url.clone();
                job.status = progress.status.clone();
                job.errors = progress.errors;
                job.emails = progress.emails;
                job.phones = progress.phones;
                if let Some(ref db) = db { let _ = db.save_job(&job); }
            }
            if let Some(deep) = &progress.deep_extracted {
                let deep_result = DeepResult {
                    id: Uuid::new_v4().to_string(),
                    job_id: job_id.clone(),
                    url: progress.current_url.clone().unwrap_or_default(),
                    structured_data: deep.get("structured").cloned(),
                    nlp_data: deep.get("nlp").cloned(),
                    extracted: crate::scraper::extractor::ExtractedData {
                        emails: deep.get("emails").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
                        phones: deep.get("phones").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
                    },
                    created_at: Utc::now().to_rfc3339(),
                };
                if let Some(ref db) = db { let _ = db.save_deep_result(&deep_result); }
            }
        }
        active.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    });

    Ok(Json(job_info))
}

async fn download_job_zip(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    if !state.jobs.contains_key(&id) {
        return Err(StatusCode::NOT_FOUND);
    }

    let job = state.jobs.get(&id).unwrap();
    let domain = url::Url::parse(&job.url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or("site".to_string());

    let all_files = state.storage.list_files(&domain).await.unwrap_or_default();
    let job_files: Vec<_> = all_files.iter().filter(|f| !f.is_dir).collect();

    let mut buf = Vec::new();
    let mut zip_writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
    let options = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for f in &job_files {
        let full_path = format!("{}/{}", domain, f.path);
        if let Ok(data) = state.storage.read_file(&full_path).await {
            let _ = zip_writer.start_file(&f.path, options.clone());
            let _ = zip_writer.write_all(&data);
        }
    }
    if let Err(_) = zip_writer.finish() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let filename = format!("shinobi-{}-{}.zip", domain, id.split('-').next().unwrap_or(&id));
    let disposition = format!("attachment; filename=\"{}\"", filename);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(axum::http::header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/zip"));
    if let Ok(h) = axum::http::HeaderValue::from_str(&disposition) {
        headers.insert(axum::http::header::CONTENT_DISPOSITION, h);
    }
    Ok((headers, buf))
}

async fn list_jobs(State(state): State<AppState>, Query(query): Query<FilesQuery>) -> Json<serde_json::Value> {
    let mut all: Vec<JobInfo> = state.jobs.iter().map(|e| e.value().clone()).collect();
    all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let total = all.len();
    let items: Vec<JobInfo> = all.into_iter().skip(query.offset).take(query.limit).collect();
    Json(serde_json::json!({"items": items, "total": total, "offset": query.offset, "limit": query.limit}))
}

async fn get_job(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<JobInfo>, StatusCode> {
    state.jobs.get(&id).map(|j| Json(j.value().clone())).ok_or(StatusCode::NOT_FOUND)
}

async fn export_job(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let job = state.jobs.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    let job = job.value().clone();

    let files = state.storage.list_files("").await.unwrap_or_default();
    let file_list: Vec<String> = files.iter().filter(|f| !f.is_dir).map(|f| f.path.clone()).collect();

    let export = serde_json::json!({
        "job": {
            "id": job.id,
            "url": job.url,
            "status": job.status,
            "created_at": job.created_at,
            "pages_scraped": job.pages_scraped,
            "files_downloaded": job.files_downloaded,
            "total_pages": job.total_pages,
        },
        "files": file_list,
        "emails": job.emails,
        "phones": job.phones,
        "exported_at": Utc::now().to_rfc3339(),
    });

    Ok(Json(export))
}

fn event_stream(state: AppState, id: String) -> impl Stream<Item = Result<Event, Infallible>> {
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);
    let jobs = state.jobs.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            if let Some(job) = jobs.get(&id) {
                let data = serde_json::to_string(&job.value()).unwrap_or_default();
                let _ = tx.send(Ok(Event::default().data(data))).await;
                if matches!(job.status.as_str(), "completed" | "failed" | "cancelled") { break; }
            } else {
                let _ = tx.send(Ok(Event::default().data(r#"{"status":"completed"}"#))).await;
                break;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

async fn stream_job(
    State(state): State<AppState>, Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    if state.jobs.contains_key(&id) { Ok(Sse::new(event_stream(state, id))) }
    else { Err(StatusCode::NOT_FOUND) }
}

async fn cancel_job(
    State(state): State<AppState>, Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(downloader) = state.downloaders.get(&id) {
        downloader.cancel();
        if let Some(mut job) = state.jobs.get_mut(&id) { job.status = "cancelled".into(); }
        Ok(Json(serde_json::json!({"status": "cancelled"})))
    } else { Err(StatusCode::NOT_FOUND) }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "shinobi",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[derive(Debug, Deserialize)]
pub struct FilesQuery {
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize { 50 }

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub domain: Option<String>,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

async fn list_files(State(state): State<AppState>, Query(query): Query<FilesQuery>) -> Json<serde_json::Value> {
    let mut all = state.storage.list_files(&query.prefix).await.unwrap_or_default();
    all.sort_by(|a, b| b.modified.cmp(&a.modified));
    let total = all.len();
    let items: Vec<_> = all.into_iter().skip(query.offset).take(query.limit).collect();
    Json(serde_json::json!({"items": items, "total": total, "offset": query.offset, "limit": query.limit}))
}

async fn get_file(State(state): State<AppState>, Path(path): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    match state.storage.read_file(&path).await {
        Ok(data) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Ok(([(axum::http::header::CONTENT_TYPE, mime.to_string())], data))
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn export_database(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match &state.db {
        Some(db) => db.export_all().map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ImportPayload {
    pub jobs: Vec<JobInfo>,
}

async fn import_database(
    State(state): State<AppState>,
    Json(payload): Json<ImportPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match &state.db {
        Some(db) => {
            match db.import_jobs(&payload.jobs) {
                Ok(count) => {
                    for job in &payload.jobs {
                        state.jobs.insert(job.id.clone(), job.clone());
                    }
                    Ok(Json(serde_json::json!({"imported": count})))
                }
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResult {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub structured_data: Option<JsonValue>,
    pub nlp_data: Option<JsonValue>,
    pub extracted: crate::scraper::extractor::ExtractedData,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct DeepQuery {
    pub url: String,
    #[serde(default)]
    pub extract_structured: Option<bool>,
    #[serde(default)]
    pub nlp_enabled: Option<bool>,
    #[serde(default)]
    pub custom_selectors: Option<Vec<String>>,
}

async fn start_deep_scrape(
    State(state): State<AppState>,
    Json(query): Json<DeepQuery>,
) -> Result<Json<DeepResult>, (StatusCode, Json<serde_json::Value>)> {
    if query.url.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "URL is required"}))));
    }

    let extractor_endpoint = std::env::var("EXTRACTOR_URL").unwrap_or_else(|_| "http://localhost:9090".into());
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "url": query.url,
        "extract_structured": query.extract_structured.unwrap_or(true),
        "nlp_enabled": query.nlp_enabled.unwrap_or(false),
        "custom_selectors": query.custom_selectors.unwrap_or_default(),
    });

    let resp = match client.post(format!("{}/extract", extractor_endpoint))
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Err((StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Extractor unavailable: {}", e)})))),
    };

    let result: JsonValue = resp.json().await.map_err(|e| {
        (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Invalid response: {}", e)})))
    })?;

    let deep_result = DeepResult {
        id: Uuid::new_v4().to_string(),
        job_id: String::new(),
        url: query.url.clone(),
        structured_data: result.get("structured").cloned(),
        nlp_data: result.get("nlp").cloned(),
        extracted: crate::scraper::extractor::ExtractedData {
            emails: result.get("emails").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
            phones: result.get("phones").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
        },
        created_at: Utc::now().to_rfc3339(),
    };

    let db_result = deep_result.clone();
    if let Some(ref db) = state.db {
        let _ = db.save_deep_result(&db_result);
    }

    Ok(Json(deep_result))
}

async fn list_deep_results(State(state): State<AppState>, Query(query): Query<FilesQuery>) -> Json<serde_json::Value> {
    let all = match &state.db {
        Some(db) => db.load_deep_results().unwrap_or_default(),
        None => Vec::new(),
    };
    let total = all.len();
    let items: Vec<DeepResult> = all.into_iter().skip(query.offset).take(query.limit).collect();
    Json(serde_json::json!({"items": items, "total": total, "offset": query.offset, "limit": query.limit}))
}

async fn get_deep_result(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeepResult>, StatusCode> {
    match &state.db {
        Some(db) => db.get_deep_result(&id)
            .map(Json)
            .map_err(|_| StatusCode::NOT_FOUND),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeepBatchQuery {
    pub urls: Vec<String>,
    #[serde(default)]
    pub extract_structured: Option<bool>,
    #[serde(default)]
    pub nlp_enabled: Option<bool>,
    #[serde(default)]
    pub custom_selectors: Option<Vec<String>>,
}

async fn start_deep_batch(
    State(state): State<AppState>,
    Json(query): Json<DeepBatchQuery>,
) -> Result<Json<Vec<DeepResult>>, (StatusCode, Json<serde_json::Value>)> {
    if query.urls.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "At least one URL is required"}))));
    }

    let extractor_endpoint = std::env::var("EXTRACTOR_URL").unwrap_or_else(|_| "http://localhost:9090".into());
    let client = reqwest::Client::new();
    let mut results = Vec::new();

    let cs = query.custom_selectors.clone().unwrap_or_default();
    for url in &query.urls {
        let payload = serde_json::json!({
            "url": url,
            "extract_structured": query.extract_structured.unwrap_or(true),
            "nlp_enabled": query.nlp_enabled.unwrap_or(false),
            "custom_selectors": cs,
        });

        if let Ok(resp) = client.post(format!("{}/extract", extractor_endpoint))
            .json(&payload)
            .send()
            .await
        {
            if let Ok(result) = resp.json::<JsonValue>().await {
                let deep_result = DeepResult {
                    id: Uuid::new_v4().to_string(),
                    job_id: String::new(),
                    url: url.clone(),
                    structured_data: result.get("structured").cloned(),
                    nlp_data: result.get("nlp").cloned(),
                    extracted: crate::scraper::extractor::ExtractedData {
                        emails: result.get("emails").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
                        phones: result.get("phones").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
                    },
                    created_at: Utc::now().to_rfc3339(),
                };
                if let Some(ref db) = state.db {
                    let _ = db.save_deep_result(&deep_result);
                }
                results.push(deep_result);
            }
        }
    }

    Ok(Json(results))
}

async fn export_deep_csv(State(state): State<AppState>) -> Result<String, StatusCode> {
    let results = match &state.db {
        Some(db) => db.load_deep_results().unwrap_or_default(),
        None => return Err(StatusCode::NOT_FOUND),
    };

    let mut csv = String::from("id,url,created_at,emails,phones,has_structured,has_nlp\n");
    for r in &results {
        let emails = r.extracted.emails.join("; ");
        let phones = r.extracted.phones.join("; ");
        let has_structured = if r.structured_data.is_some() { "yes" } else { "no" };
        let has_nlp = if r.nlp_data.is_some() { "yes" } else { "no" };
        csv.push_str(&format!("{},{},{},{},{},{},{}\n",
            r.id, r.url, r.created_at, emails, phones, has_structured, has_nlp));
    }

    Ok(csv)
}

#[derive(Debug, Deserialize)]
pub struct DeepCrawlQuery {
    pub url: String,
    #[serde(default)]
    pub depth: Option<u32>,
    #[serde(default)]
    pub max_pages: Option<usize>,
    #[serde(default)]
    pub extract_structured: Option<bool>,
    #[serde(default)]
    pub nlp_enabled: Option<bool>,
    #[serde(default)]
    pub custom_selectors: Option<Vec<String>>,
}

async fn start_deep_crawl(
    State(_state): State<AppState>,
    Json(query): Json<DeepCrawlQuery>,
) -> Result<Json<JsonValue>, (StatusCode, Json<serde_json::Value>)> {
    if query.url.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "URL is required"}))));
    }

    let extractor_endpoint = std::env::var("EXTRACTOR_URL").unwrap_or_else(|_| "http://localhost:9090".into());
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "url": query.url,
        "depth": query.depth.unwrap_or(3),
        "max_pages": query.max_pages.unwrap_or(100),
        "same_domain": true,
        "download_assets": true,
        "file_types": [],
        "extract_structured": query.extract_structured.unwrap_or(true),
        "nlp_enabled": query.nlp_enabled.unwrap_or(false),
        "custom_selectors": query.custom_selectors.unwrap_or_default(),
    });

    match client.post(format!("{}/crawl", extractor_endpoint))
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => {
            let result: JsonValue = r.json().await.unwrap_or(serde_json::json!({"error": "invalid response"}));
            Ok(Json(result))
        }
        Err(e) => Err((StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Crawler unavailable: {}", e)})))),
    }
}

async fn get_deep_crawl_status(
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let extractor_endpoint = std::env::var("EXTRACTOR_URL").unwrap_or_else(|_| "http://localhost:9090".into());
    let client = reqwest::Client::new();
    match client.get(format!("{}/crawl/{}/status", extractor_endpoint, id)).send().await {
        Ok(r) if r.status().is_success() => {
            let result: JsonValue = r.json().await.unwrap_or_default();
            Ok(Json(result))
        }
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_deep_crawl_results(
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let extractor_endpoint = std::env::var("EXTRACTOR_URL").unwrap_or_else(|_| "http://localhost:9090".into());
    let client = reqwest::Client::new();
    match client.get(format!("{}/crawl/{}/results", extractor_endpoint, id)).send().await {
        Ok(r) if r.status().is_success() => {
            let result: JsonValue = r.json().await.unwrap_or_default();
            Ok(Json(result))
        }
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn cancel_deep_crawl(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let extractor_endpoint = std::env::var("EXTRACTOR_URL").unwrap_or_else(|_| "http://localhost:9090".into());
    let client = reqwest::Client::new();
    match client.post(format!("{}/crawl/{}/cancel", extractor_endpoint, id)).send().await {
        Ok(r) if r.status().is_success() => {
            let result: JsonValue = r.json().await.unwrap_or_default();
            Ok(Json(result))
        }
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let domain_opt = state.jobs.get(&id).map(|j| {
        url::Url::parse(&j.url).ok().and_then(|u| u.host_str().map(|h| h.to_string()))
    }).flatten();
    state.jobs.remove(&id);
    state.downloaders.remove(&id);
    if let Some(ref db) = state.db {
        let _ = db.delete_job(&id);
    }
    if let Some(domain) = domain_opt {
        let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "downloads".into());
        let dir = std::path::Path::new(&data_dir).join(&domain);
        let zip = std::path::Path::new(&data_dir).join(format!("{}.zip", domain));
        let _ = tokio::fs::remove_dir_all(dir).await;
        let _ = tokio::fs::remove_file(zip).await;
    }
    Ok(Json(serde_json::json!({"status": "deleted"})))
}

async fn delete_deep_result(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match &state.db {
        Some(db) => {
            db.delete_deep_result(&id).map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(Json(serde_json::json!({"status": "deleted"})))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn clear_deep_results(State(state): State<AppState>) -> Json<serde_json::Value> {
    if let Some(ref db) = state.db {
        let _ = db.clear_deep_results();
    }
    Json(serde_json::json!({"status": "cleared"}))
}

async fn clear_database(State(state): State<AppState>) -> Json<serde_json::Value> {
    state.jobs.clear();
    state.downloaders.clear();
    if let Some(ref db) = state.db {
        let _ = db.clear_all();
    }
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "downloads".into());
    let dir = std::path::Path::new(&data_dir);
    if dir.exists() {
        let _ = tokio::fs::remove_dir_all(dir).await;
        let _ = tokio::fs::create_dir_all(dir).await;

    }
    Json(serde_json::json!({"status": "cleared"}))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub url: String,
    pub interval_min: u64,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSchedule {
    pub url: String,
    pub interval_min: u64,
    pub config: Option<serde_json::Value>,
}

async fn list_schedules(State(state): State<AppState>) -> Json<Vec<Schedule>> {
    match &state.db {
        Some(db) => Json(db.load_schedules().unwrap_or_default()),
        None => Json(Vec::new()),
    }
}

async fn create_schedule(
    State(state): State<AppState>,
    Json(query): Json<CreateSchedule>,
) -> Result<Json<Schedule>, (StatusCode, Json<serde_json::Value>)> {
    if query.url.trim().is_empty() || query.interval_min < 5 {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Valid URL and interval >= 5 min required"}))));
    }

    let next_run = (chrono::Utc::now() + chrono::Duration::minutes(query.interval_min as i64)).to_rfc3339();
    let schedule = Schedule {
        id: Uuid::new_v4().to_string(),
        url: query.url,
        interval_min: query.interval_min,
        config: query.config.unwrap_or(serde_json::json!({})),
        enabled: true,
        last_run: None,
        next_run,
        created_at: Utc::now().to_rfc3339(),
    };

    if let Some(ref db) = state.db {
        let _ = db.save_schedule(&schedule);
    }

    Ok(Json(schedule))
}

async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match &state.db {
        Some(db) => {
            db.delete_schedule(&id).map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(Json(serde_json::json!({"status": "deleted"})))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn search_files(
    State(_state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "downloads".into());
    let dir = std::path::Path::new(&data_dir);
    let mut results = Vec::new();

    if dir.exists() {
        let q = query.q.to_lowercase();
        let mut entries = Vec::new();
        if let Ok(mut read_dir) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                let path = entry.path();
                if path.is_dir() || path.extension().map(|e| e == "zip").unwrap_or(false) { continue; }
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&q) {
                    if let Ok(meta) = entry.metadata().await {
                        entries.push(serde_json::json!({
                            "path": path.strip_prefix(dir).map(|p| p.to_string_lossy()).unwrap_or_default().to_string(),
                            "size": meta.len(),
                            "modified": meta.modified().ok().and_then(|t| t.elapsed().ok()).map(|d| d.as_secs()).unwrap_or(0),
                        }));
                    }
                }
            }
        }
        entries.sort_by(|a, b| b.get("modified").and_then(|v| v.as_u64()).unwrap_or(0).cmp(&a.get("modified").and_then(|v| v.as_u64()).unwrap_or(0)));
        let items: Vec<_> = entries.into_iter().skip(query.offset).take(query.limit).collect();
        results = items;
    }

    Json(serde_json::json!({"items": results, "total": results.len(), "query": query.q}))
}

async fn get_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "downloads".into());
    let dir = std::path::Path::new(&data_dir);
    let mut disk_size: u64 = 0;
    let mut file_count: u64 = 0;

    if dir.exists() {
        if let Ok(mut read_dir) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                let path = entry.path();
                if path.extension().map(|e| e == "zip").unwrap_or(false) { continue; }
                if let Ok(meta) = entry.metadata().await {
                    if meta.is_file() { file_count += 1; disk_size += meta.len(); }
                    else if meta.is_dir() {
                        disk_size += dir_size(&path).await;
                    }
                }
            }
        }
    }

    Json(serde_json::json!({
        "jobs": state.jobs.len(),
        "active_scrapes": state.active_scrapes.load(std::sync::atomic::Ordering::Relaxed),
        "files": file_count,
        "disk_size": disk_size,
        "disk_size_human": format_size_human(disk_size),
    }))
}

fn dir_size_sync(path: &std::path::Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(read_dir) = std::fs::read_dir(path) {
        for entry in read_dir.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    total += dir_size_sync(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

async fn dir_size(path: &std::path::Path) -> u64 {
    tokio::task::spawn_blocking({
        let path = path.to_owned();
        move || dir_size_sync(&path)
    }).await.unwrap_or(0)
}

fn format_size_human(bytes: u64) -> String {
    if bytes < 1024 { return format!("{}B", bytes); }
    if bytes < 1024 * 1024 { return format!("{:.1}KB", bytes as f64 / 1024.0); }
    if bytes < 1024 * 1024 * 1024 { return format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0)); }
    format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

async fn python_docs_proxy() -> Result<impl IntoResponse, StatusCode> {
    let extractor_endpoint = std::env::var("EXTRACTOR_URL").unwrap_or_else(|_| "http://localhost:9090".into());
    match reqwest::get(format!("{}/docs", extractor_endpoint)).await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            Ok(([("Content-Type", "text/html; charset=utf-8")], body))
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}
