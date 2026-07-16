use crate::app::ExportMode;
use crate::crawler::{self, DiscoveredPage, Vulnerability};
use crate::db::connection::DbPool;
use crate::recon::{self, ReconResults};
use crate::scanner::engine::{self as scanner_engine, PortInfo, ScanEvent};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Events that background tasks send to the main loop
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum BgEvent {
    // Scanner
    ScannerLog(String),
    ScannerPort(PortInfo),
    ScannerDone(Vec<PortInfo>),
    // Recon
    ReconLog(String),
    ReconDone(ReconResults),
    // Crawler
    CrawlerLog(String),
    CrawlerDone(Vec<DiscoveredPage>, Vec<Vulnerability>),
    // Export
    ExportDone,
    ExportSummary(usize, usize, usize),
    // Generic status
    Status(String),
}

// ---------------------------------------------------------------------------
// Scanner
// ---------------------------------------------------------------------------

pub async fn run_scanner(
    target: String,
    profile: String,
    pool: DbPool,
    event_tx: tokio::sync::mpsc::UnboundedSender<BgEvent>,
) {
    let _ = event_tx.send(BgEvent::ScannerLog(format!("[CMD] nmap {} -{}", target, profile)));

    // Proxy para nmap desde SAMURAI_NMAP_PROXY
    let nmap_proxy = std::env::var("SAMURAI_NMAP_PROXY").ok()
        .filter(|s| !s.is_empty());
    if let Some(ref proxy) = nmap_proxy {
        let _ = event_tx.send(BgEvent::ScannerLog(format!("[PROXY] nmap proxy: {}", proxy)));
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let nmap_target = target.clone();
    let nmap_profile = profile.clone();
    let nmap_proxy_clone = nmap_proxy.clone();
    let nmap_handle = tokio::spawn(async move {
        scanner_engine::run_nmap_streaming(
            &nmap_target,
            &nmap_profile,
            nmap_proxy_clone.as_deref(),
            tx,
        ).await;
    });

    let mut open_ports: Vec<PortInfo> = Vec::new();

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Some(ScanEvent::Log(line)) => {
                        let _ = event_tx.send(BgEvent::ScannerLog(line.clone()));
                        if line.contains("/tcp") && line.contains("open") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 3 {
                                let port = parts[0].to_string();
                                let service = parts[2..].join(" ");
                                let port_info = PortInfo {
                                    port: port.clone(),
                                    service: service.clone(),
                                    raw_line: line.clone(),
                                };
                                open_ports.push(port_info.clone());
                                let _ = event_tx.send(BgEvent::ScannerPort(port_info));
                            }
                        }
                    }
                    Some(ScanEvent::Error(e)) => {
                        let _ = event_tx.send(BgEvent::ScannerLog(format!("[ERROR] {}", e)));
                    }
                    Some(ScanEvent::Done { open_ports: _ }) => break,
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    let _ = nmap_handle.await;

    // Save to DB
    if let Ok(scan) = crate::db::operations::insert_scan(&pool, &target, &format!("port_scan:{}", profile)).await {
        for port_info in &open_ports {
            let _ = crate::db::operations::insert_finding(
                &pool,
                scan.id,
                None,
                "info",
                "OPEN_PORT",
                &format!("{} — {}", port_info.port, port_info.service),
                Some(&port_info.raw_line),
                None,
            )
            .await;
        }
        let _ = crate::db::operations::update_scan_status(&pool, scan.id, "COMPLETED").await;
    }

    let _ = event_tx.send(BgEvent::ScannerDone(open_ports));
}

// ---------------------------------------------------------------------------
// Reconnaissance
// ---------------------------------------------------------------------------

pub async fn run_recon(
    target: String,
    pool: DbPool,
    http_client: Arc<reqwest::Client>,
    dns_resolver: Arc<hickory_resolver::TokioAsyncResolver>,
    event_tx: tokio::sync::mpsc::UnboundedSender<BgEvent>,
) {
    let _ = event_tx.send(BgEvent::ReconLog("[DNS] Enumerating records...".into()));

    let dns_records = recon::dns::enumerate_dns(&target, &dns_resolver).await;
    let _ = event_tx.send(BgEvent::ReconLog(format!("[DNS] {} records found", dns_records.len())));

    let _ = event_tx.send(BgEvent::ReconLog("[SUB] Discovering subdomains...".into()));
    let subdomains =
        recon::subdomains::enumerate_subdomains(&target, &http_client, &dns_resolver).await;
    let _ = event_tx.send(BgEvent::ReconLog(format!("[SUB] {} subdomains found", subdomains.len())));

    let _ = event_tx.send(BgEvent::ReconLog("[API] Probing endpoints...".into()));
    let api_endpoints = recon::api::discover_apis(&target, &http_client).await;
    let _ = event_tx.send(BgEvent::ReconLog(format!("[API] {} endpoints found", api_endpoints.len())));

    let _ = event_tx.send(BgEvent::ReconLog("[HDR] Checking security headers...".into()));
    let headers = recon::headers::check_security_headers(&target, &http_client).await;
    let missing = headers.iter().filter(|h| !h.present).count();
    let _ = event_tx.send(BgEvent::ReconLog(format!(
        "[HDR] {} headers checked, {} missing",
        headers.len(),
        missing
    )));

    let _ = event_tx.send(BgEvent::ReconLog("[TECH] Fingerprinting...".into()));
    let technologies = recon::tech_stack::fingerprint_technology(&target, &http_client).await;
    let _ = event_tx
        .send(BgEvent::ReconLog(format!("[TECH] {} technologies detected", technologies.len())));

    let results = ReconResults {
        target: target.clone(),
        dns_records,
        subdomains,
        api_endpoints,
        security_headers: headers,
        technologies,
    };

    // Save to DB
    let json = serde_json::to_string_pretty(&results).unwrap_or_default();
    if let Ok(scan) = crate::db::operations::insert_scan(&pool, &target, "web_recon").await {
        let _ = crate::db::operations::insert_finding(
            &pool,
            scan.id,
            None,
            "info",
            "web_recon_results",
            &results.to_finding_description(),
            Some(&json),
            None,
        )
        .await;
        let _ = crate::db::operations::update_scan_status(&pool, scan.id, "COMPLETED").await;
    }

    let _ = event_tx.send(BgEvent::ReconDone(results));
}

// ---------------------------------------------------------------------------
// Crawler
// ---------------------------------------------------------------------------

pub async fn run_crawler(
    target: String,
    max_pages: usize,
    pool: DbPool,
    http_client: Arc<reqwest::Client>,
    event_tx: tokio::sync::mpsc::UnboundedSender<BgEvent>,
) {
    let _ = event_tx.send(BgEvent::CrawlerLog(format!(
        "[CRAWL] Discovering pages (max {})...",
        max_pages
    )));

    let pages = crawler::discover_pages(&target, max_pages, &http_client).await;
    let _ = event_tx.send(BgEvent::CrawlerLog(format!(
        "[CRAWL] {} pages discovered",
        pages.len()
    )));

    for page in &pages {
        let _ = event_tx
            .send(BgEvent::CrawlerLog(format!("[PAGE] {} HTTP {}", page.url, page.status_code)));
    }

    // Save to DB
    if let Ok(scan) = crate::db::operations::insert_scan(&pool, &target, "crawler").await {
        for page in &pages {
            let _ = crate::db::operations::insert_discovered_link(
                &pool,
                scan.id,
                &page.url,
                Some(page.status_code as i32),
                page.content_type.as_deref(),
            )
            .await;
        }
        let _ = crate::db::operations::update_scan_status(&pool, scan.id, "COMPLETED").await;
    }

    let _ = event_tx.send(BgEvent::CrawlerDone(pages, Vec::new()));
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

pub async fn run_export(
    mode: ExportMode,
    password: String,
    path_str: String,
    pool: DbPool,
    event_tx: tokio::sync::mpsc::UnboundedSender<BgEvent>,
) {
    let path = std::path::PathBuf::from(path_str.clone());

    let result = match mode {
        ExportMode::Raw => crate::export::export_raw_to_file(&pool, &path).await,
        ExportMode::Encrypted => crate::export::export_encrypted_to_file(&pool, &path, &password).await,
    };

    match result {
        Ok(()) => {
            if let Ok(payload) = crate::db::operations::build_export_payload(&pool).await {
                let _ = event_tx.send(BgEvent::ExportSummary(
                    payload.export_metadata.scan_count,
                    payload.export_metadata.finding_count,
                    payload.export_metadata.link_count,
                ));
            }
            let _ = event_tx.send(BgEvent::Status(format!("[EXPORTED] {}", path_str)));
        }
        Err(e) => {
            let _ = event_tx.send(BgEvent::Status(format!("[ERROR] {}", e)));
        }
    }

    let _ = event_tx.send(BgEvent::ExportDone);
}
