use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Scan {
    pub id: i32,
    pub domain_target: String,
    pub status: String,
    pub created_at: Option<String>,
    pub scan_type: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DiscoveredLink {
    pub id: i32,
    pub scan_id: i32,
    pub url: String,
    pub status_code: Option<i32>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Finding {
    pub id: i32,
    pub scan_id: i32,
    pub link_id: Option<i32>,
    pub severity: String,
    pub finding_type: String,
    pub description: String,
    pub poc_payload: Option<String>,
    pub cvss_score: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanExport {
    pub id: i32,
    pub domain_target: String,
    pub status: String,
    pub scan_type: String,
    pub created_at: Option<String>,
    pub findings: Vec<FindingExport>,
    pub discovered_links: Vec<LinkExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingExport {
    pub id: i32,
    pub severity: String,
    pub finding_type: String,
    pub description: String,
    pub poc_payload: Option<String>,
    pub cvss_score: Option<String>,
    pub link_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkExport {
    pub id: i32,
    pub url: String,
    pub status_code: Option<i32>,
    pub content_type: Option<String>,
    pub findings: Vec<FindingExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: String,
    pub samurai_version: String,
    pub scan_count: usize,
    pub finding_count: usize,
    pub link_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    pub export_metadata: ExportMetadata,
    pub scans: Vec<ScanExport>,
}
