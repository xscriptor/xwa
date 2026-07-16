use crate::db::connection::DbPool;
use sqlx::{PgPool, SqlitePool};
use crate::db::models::*;

// --- Public API dispatching to backend ---

pub async fn get_all_scans(pool: &DbPool) -> Result<Vec<Scan>, sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_all_scans(p).await, DbPool::Sqlite(p) => sqlite_all_scans(p).await }
}

pub async fn get_findings_for_scan(pool: &DbPool, scan_id: i32) -> Result<Vec<Finding>, sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_findings_for_scan(p, scan_id).await, DbPool::Sqlite(p) => sqlite_findings_for_scan(p, scan_id).await }
}

pub async fn get_links_for_scan(pool: &DbPool, scan_id: i32) -> Result<Vec<DiscoveredLink>, sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_links_for_scan(p, scan_id).await, DbPool::Sqlite(p) => sqlite_links_for_scan(p, scan_id).await }
}

pub async fn get_findings_for_link(pool: &DbPool, link_id: i32) -> Result<Vec<Finding>, sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_findings_for_link(p, link_id).await, DbPool::Sqlite(p) => sqlite_findings_for_link(p, link_id).await }
}

pub async fn insert_scan(pool: &DbPool, target: &str, scan_type: &str) -> Result<Scan, sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_insert_scan(p, target, scan_type).await, DbPool::Sqlite(p) => sqlite_insert_scan(p, target, scan_type).await }
}

pub async fn update_scan_status(pool: &DbPool, id: i32, status: &str) -> Result<(), sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_update_scan(p, id, status).await, DbPool::Sqlite(p) => sqlite_update_scan(p, id, status).await }
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_finding(pool: &DbPool, scan_id: i32, link_id: Option<i32>, severity: &str, finding_type: &str, description: &str, poc_payload: Option<&str>, cvss_score: Option<&str>) -> Result<Finding, sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_insert_finding(p, scan_id, link_id, severity, finding_type, description, poc_payload, cvss_score).await, DbPool::Sqlite(p) => sqlite_insert_finding(p, scan_id, link_id, severity, finding_type, description, poc_payload, cvss_score).await }
}

pub async fn insert_discovered_link(pool: &DbPool, scan_id: i32, url: &str, status_code: Option<i32>, content_type: Option<&str>) -> Result<DiscoveredLink, sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_insert_link(p, scan_id, url, status_code, content_type).await, DbPool::Sqlite(p) => sqlite_insert_link(p, scan_id, url, status_code, content_type).await }
}

pub async fn delete_scan(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    match pool { DbPool::Postgres(p) => pg_delete_scan(p, id).await, DbPool::Sqlite(p) => sqlite_delete_scan(p, id).await }
}

pub async fn build_export_payload(pool: &DbPool) -> Result<ExportPayload, sqlx::Error> {
    let scans = get_all_scans(pool).await?;
    let mut scan_exports = Vec::new();
    let mut total_findings = 0usize;
    let mut total_links = 0usize;

    for scan in &scans {
        let findings = get_findings_for_scan(pool, scan.id).await?;
        let links = get_links_for_scan(pool, scan.id).await?;

        let mut link_exports = Vec::new();
        for link in &links {
            let link_findings = get_findings_for_link(pool, link.id).await?;
            total_findings += link_findings.len();
            link_exports.push(LinkExport {
                id: link.id, url: link.url.clone(), status_code: link.status_code,
                content_type: link.content_type.clone(),
                findings: to_finding_exports(&link_findings),
            });
        }

        total_findings += findings.len();
        total_links += links.len();

        scan_exports.push(ScanExport {
            id: scan.id, domain_target: scan.domain_target.clone(),
            status: scan.status.clone(), scan_type: scan.scan_type.clone(),
            created_at: scan.created_at.clone(),
            findings: to_finding_exports(&findings),
            discovered_links: link_exports,
        });
    }

    Ok(ExportPayload {
        export_metadata: ExportMetadata {
            exported_at: chrono::Utc::now().to_rfc3339(),
            samurai_version: "2.5.0".into(),
            scan_count: scans.len(), finding_count: total_findings, link_count: total_links,
        },
        scans: scan_exports,
    })
}

fn to_finding_exports(findings: &[Finding]) -> Vec<FindingExport> {
    findings.iter().map(|f| FindingExport {
        id: f.id, severity: f.severity.clone(), finding_type: f.finding_type.clone(),
        description: f.description.clone(), poc_payload: f.poc_payload.clone(),
        cvss_score: f.cvss_score.clone(), link_id: f.link_id,
    }).collect()
}

// --- PostgreSQL implementations ---

async fn pg_all_scans(p: &PgPool) -> Result<Vec<Scan>, sqlx::Error> {
    sqlx::query_as("SELECT id, domain_target, status, created_at, scan_type FROM scans ORDER BY id DESC").fetch_all(p).await
}
async fn pg_findings_for_scan(p: &PgPool, id: i32) -> Result<Vec<Finding>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM findings WHERE scan_id = $1 AND link_id IS NULL ORDER BY id").bind(id).fetch_all(p).await
}
async fn pg_links_for_scan(p: &PgPool, id: i32) -> Result<Vec<DiscoveredLink>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM discovered_links WHERE scan_id = $1 ORDER BY id").bind(id).fetch_all(p).await
}
async fn pg_findings_for_link(p: &PgPool, id: i32) -> Result<Vec<Finding>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM findings WHERE link_id = $1 ORDER BY id").bind(id).fetch_all(p).await
}
async fn pg_insert_scan(p: &PgPool, target: &str, scan_type: &str) -> Result<Scan, sqlx::Error> {
    sqlx::query_as("INSERT INTO scans (domain_target, status, scan_type) VALUES ($1, 'RUNNING', $2) RETURNING id, domain_target, status, created_at, scan_type").bind(target).bind(scan_type).fetch_one(p).await
}
async fn pg_update_scan(p: &PgPool, id: i32, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE scans SET status = $1 WHERE id = $2").bind(status).bind(id).execute(p).await.map(|_| ())
}
#[allow(clippy::too_many_arguments)]
async fn pg_insert_finding(p: &PgPool, scan_id: i32, link_id: Option<i32>, severity: &str, finding_type: &str, description: &str, poc_payload: Option<&str>, cvss_score: Option<&str>) -> Result<Finding, sqlx::Error> {
    sqlx::query_as("INSERT INTO findings (scan_id, link_id, severity, finding_type, description, poc_payload, cvss_score) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, scan_id, link_id, severity, finding_type, description, poc_payload, cvss_score").bind(scan_id).bind(link_id).bind(severity).bind(finding_type).bind(description).bind(poc_payload).bind(cvss_score).fetch_one(p).await
}
async fn pg_insert_link(p: &PgPool, scan_id: i32, url: &str, status_code: Option<i32>, content_type: Option<&str>) -> Result<DiscoveredLink, sqlx::Error> {
    sqlx::query_as("INSERT INTO discovered_links (scan_id, url, status_code, content_type) VALUES ($1, $2, $3, $4) RETURNING id, scan_id, url, status_code, content_type").bind(scan_id).bind(url).bind(status_code).bind(content_type).fetch_one(p).await
}
async fn pg_delete_scan(p: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM scans WHERE id = $1").bind(id).execute(p).await.map(|_| ())
}

// --- SQLite implementations ---

async fn sqlite_all_scans(p: &SqlitePool) -> Result<Vec<Scan>, sqlx::Error> {
    sqlx::query_as("SELECT id, domain_target, status, created_at, scan_type FROM scans ORDER BY id DESC").fetch_all(p).await
}
async fn sqlite_findings_for_scan(p: &SqlitePool, id: i32) -> Result<Vec<Finding>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM findings WHERE scan_id = ?1 AND link_id IS NULL ORDER BY id").bind(id).fetch_all(p).await
}
async fn sqlite_links_for_scan(p: &SqlitePool, id: i32) -> Result<Vec<DiscoveredLink>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM discovered_links WHERE scan_id = ?1 ORDER BY id").bind(id).fetch_all(p).await
}
async fn sqlite_findings_for_link(p: &SqlitePool, id: i32) -> Result<Vec<Finding>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM findings WHERE link_id = ?1 ORDER BY id").bind(id).fetch_all(p).await
}
async fn sqlite_insert_scan(p: &SqlitePool, target: &str, scan_type: &str) -> Result<Scan, sqlx::Error> {
    sqlx::query("INSERT INTO scans (domain_target, status, scan_type) VALUES (?1, 'RUNNING', ?2)").bind(target).bind(scan_type).execute(p).await?;
    let id = sqlx::query_scalar("SELECT last_insert_rowid()").fetch_one(p).await?;
    Ok(Scan { id, domain_target: target.into(), status: "RUNNING".into(), created_at: None, scan_type: scan_type.into() })
}
async fn sqlite_update_scan(p: &SqlitePool, id: i32, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE scans SET status = ?1 WHERE id = ?2").bind(status).bind(id).execute(p).await.map(|_| ())
}
#[allow(clippy::too_many_arguments)]
async fn sqlite_insert_finding(p: &SqlitePool, scan_id: i32, link_id: Option<i32>, severity: &str, finding_type: &str, description: &str, poc_payload: Option<&str>, cvss_score: Option<&str>) -> Result<Finding, sqlx::Error> {
    sqlx::query("INSERT INTO findings (scan_id, link_id, severity, finding_type, description, poc_payload, cvss_score) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)").bind(scan_id).bind(link_id).bind(severity).bind(finding_type).bind(description).bind(poc_payload).bind(cvss_score).execute(p).await?;
    let id = sqlx::query_scalar("SELECT last_insert_rowid()").fetch_one(p).await?;
    Ok(Finding { id, scan_id, link_id, severity: severity.into(), finding_type: finding_type.into(), description: description.into(), poc_payload: poc_payload.map(|s| s.into()), cvss_score: cvss_score.map(|s| s.into()) })
}
async fn sqlite_insert_link(p: &SqlitePool, scan_id: i32, url: &str, status_code: Option<i32>, content_type: Option<&str>) -> Result<DiscoveredLink, sqlx::Error> {
    sqlx::query("INSERT INTO discovered_links (scan_id, url, status_code, content_type) VALUES (?1, ?2, ?3, ?4)").bind(scan_id).bind(url).bind(status_code).bind(content_type).execute(p).await?;
    let id = sqlx::query_scalar("SELECT last_insert_rowid()").fetch_one(p).await?;
    Ok(DiscoveredLink { id, scan_id, url: url.into(), status_code, content_type: content_type.map(|s| s.into()) })
}
async fn sqlite_delete_scan(p: &SqlitePool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM scans WHERE id = ?1").bind(id).execute(p).await.map(|_| ())
}
