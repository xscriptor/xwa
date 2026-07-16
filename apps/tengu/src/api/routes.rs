use axum::{
    extract::{Path, Query, State, ws::{self, WebSocketUpgrade, WebSocket}},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auditor::{self, Finding};
use crate::config::AuditOptions;
use crate::storage::{self, request_watermark};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditRecord {
    pub id: Uuid,
    pub url: String,
    pub status: String,
    pub findings: Vec<Finding>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportPayload {
    pub records: Vec<AuditRecord>,
}

pub async fn list_audits(State(state): State<AppState>) -> Json<Vec<AuditRecord>> {
    Json(state.store.list())
}

pub async fn get_audit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditRecord>, StatusCode> {
    state.store.get(&id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete_audit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    if state.store.delete(&id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub records: Vec<AuditRecord>,
    pub exported_at: String,
}

pub async fn export_audits(State(state): State<AppState>) -> Json<ExportResponse> {
    Json(ExportResponse {
        records: state.store.list(),
        exported_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn clear_all_audits(State(state): State<AppState>) -> impl IntoResponse {
    state.store.clear_all();
    tracing::info!("All audit records cleared");
    (StatusCode::OK, Json(serde_json::json!({"status": "cleared"})))
}

pub async fn import_audits(
    State(state): State<AppState>,
    Json(payload): Json<ImportPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let count = payload.records.len();
    for record in payload.records {
        state.store.insert(record);
    }
    Ok(Json(serde_json::json!({
        "status": "imported",
        "count": count,
        "message": format!("{} audit records imported", count)
    })))
}

pub async fn audit_live(
    State(state): State<AppState>,
    Query(params): Query<AuditOptions>,
    ws_upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    let store = state.store.clone();
    let rate = state.rate_limiter.clone();
    let sema = state.audit_semaphore.clone();
    let req_counter = state.request_count.clone();

    ws_upgrade.on_upgrade(move |mut socket| async move {
        req_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if !rate.check() {
            let _ = socket
                .send(ws::Message::Text("[!] Rate limit exceeded. Try again later.".into()))
                .await;
            return;
        }

        let _permit = match sema.acquire().await {
            Ok(p) => p,
            Err(_) => {
                let _ = socket
                    .send(ws::Message::Text("[!] Too many concurrent audits. Try again later.".into()))
                    .await;
                return;
            }
        };

        handle_ws(socket, params, store).await;
    })
}

async fn handle_ws(mut socket: WebSocket, options: AuditOptions, store: storage::AuditStore) {
    let (audit_id, audit_ts) = request_watermark();
    tracing::info!(audit_id = %audit_id, url = %options.url, mode = %options.mode, "starting audit");

    if socket
        .send(ws::Message::Text(format!("[AUDIT_META] audit_id={}", audit_id).into()))
        .await
        .is_err()
    {
        return;
    }

    match auditor::run_audit(&options, &mut socket).await {
        Ok(findings) => {
            let record = AuditRecord {
                id: Uuid::parse_str(&audit_id).unwrap_or_else(|_| Uuid::new_v4()),
                url: options.url.clone(),
                status: "COMPLETED".to_string(),
                findings,
                created_at: audit_ts,
            };
            store.insert(record);
            let _ = socket
                .send(ws::Message::Text("[done] audit completed".into()))
                .await;
            tracing::info!(audit_id = %audit_id, "audit completed");
        }
        Err(e) => {
            let _ = socket
                .send(ws::Message::Text(format!("[!] ERROR: {}", e).into()))
                .await;
            tracing::error!(audit_id = %audit_id, error = %e, "audit failed");
        }
    }
}
