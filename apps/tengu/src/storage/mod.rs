use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;

use crate::api::routes::AuditRecord;

// ---------------------------------------------------------------------------
// In-memory store (default runtime)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AuditStore {
    pub audits: Arc<DashMap<Uuid, AuditRecord>>,
    max_records: usize,
}

impl AuditStore {
    pub fn new() -> Self {
        let max = std::env::var("TENGU_MAX_HISTORY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(100);

        let store = Self {
            audits: Arc::new(DashMap::new()),
            max_records: max,
        };

        if max < 1000 {
            tracing::info!("Audit retention: max {} records (set TENGU_MAX_HISTORY)", max);
        }

        store
    }

    pub fn list(&self) -> Vec<AuditRecord> {
        let mut all: Vec<AuditRecord> = self.audits.iter().map(|r| r.value().clone()).collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        all
    }

    pub fn get(&self, id: &Uuid) -> Option<AuditRecord> {
        self.audits.get(id).map(|r| r.clone())
    }

    pub fn insert(&self, record: AuditRecord) {
        self.audits.insert(record.id, record);
        self.enforce_retention();
    }

    pub fn delete(&self, id: &Uuid) -> bool {
        self.audits.remove(id).is_some()
    }

    pub fn clear_all(&self) {
        self.audits.clear();
    }

    pub fn count(&self) -> usize {
        self.audits.len()
    }

    fn enforce_retention(&self) {
        if self.audits.len() <= self.max_records {
            return;
        }
        let mut all: Vec<(Uuid, String)> = self
            .audits
            .iter()
            .map(|r| (r.key().clone(), r.value().created_at.clone()))
            .collect();
        all.sort_by(|a, b| b.1.cmp(&a.1));
        let to_remove: usize = all.len().saturating_sub(self.max_records);
        for (id, _) in all.iter().rev().take(to_remove) {
            self.audits.remove(id);
        }
        tracing::info!("Retention policy pruned {} old audit(s)", to_remove);
    }
}

// ---------------------------------------------------------------------------
// PostgreSQL persistence layer
// ---------------------------------------------------------------------------

#[cfg(feature = "pg")]
pub async fn create_pg_store(database_url: &str) -> Result<AuditStore, String> {
    use sqlx::postgres::PgPoolOptions;
    use sqlx::Row;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

    // Run migrations
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audits (
            id UUID PRIMARY KEY,
            url TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'COMPLETED',
            findings JSONB NOT NULL DEFAULT '[]'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Migration failed: {}", e))?;

    // Load existing records into memory
    let store = AuditStore::new();
    let rows = sqlx::query("SELECT id, url, status, findings, created_at FROM audits ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to load audits: {}", e))?;

    for row in &rows {
        let id: Uuid = row.get("id");
        let url: String = row.get("url");
        let status: String = row.get("status");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let findings_json: serde_json::Value = row.get("findings");
        let findings: Vec<crate::auditor::Finding> = serde_json::from_value(findings_json).unwrap_or_default();

        let record = AuditRecord {
            id,
            url,
            status,
            findings,
            created_at: created_at.to_rfc3339(),
        };
        store.audits.insert(id, record);
    }

    tracing::info!("Loaded {} audit(s) from PostgreSQL", rows.len());

    // Spawn background sync: every insert/delete persists to PG
    let pool_clone = pool.clone();
    let audits = store.audits.clone();
    tokio::spawn(async move {
        pg_sync_loop(pool_clone, audits).await;
    });

    Ok(store)
}

#[cfg(feature = "pg")]
async fn pg_sync_loop(
    pool: sqlx::PgPool,
    audits: Arc<DashMap<Uuid, AuditRecord>>,
) {
    use sqlx::Row;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Sync memory → PG
        for entry in audits.iter() {
            let record = entry.value();
            let findings_json = serde_json::to_value(&record.findings).unwrap_or_default();
            let ts = match chrono::DateTime::parse_from_rfc3339(&record.created_at) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(_) => chrono::Utc::now(),
            };

            let _ = sqlx::query(
                r#"
                INSERT INTO audits (id, url, status, findings, created_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO UPDATE SET
                    url = EXCLUDED.url,
                    status = EXCLUDED.status,
                    findings = EXCLUDED.findings
                "#,
            )
            .bind(record.id)
            .bind(&record.url)
            .bind(&record.status)
            .bind(&findings_json)
            .bind(ts)
            .execute(&pool)
            .await;
        }
    }
}

#[cfg(not(feature = "pg"))]
pub async fn create_pg_store(_database_url: &str) -> Result<AuditStore, String> {
    Err("PostgreSQL support not compiled. Build with --features pg".into())
}

/// Request watermarking
pub fn request_watermark() -> (String, String) {
    let id = Uuid::new_v4().to_string();
    let ts = chrono::Utc::now().to_rfc3339();
    (id, ts)
}
