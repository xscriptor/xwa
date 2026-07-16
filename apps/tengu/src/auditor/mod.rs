use axum::extract::ws::{self, WebSocket};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::{AuditOptions, TenguConfig};

pub mod performance;
pub mod seo;
pub mod a11y;
pub mod best_practices;
pub mod html_pretty;

fn normalize_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    }
}

pub async fn run_audit(options: &AuditOptions, socket: &mut WebSocket) -> Result<Vec<Finding>, String> {
    let mut all_findings = Vec::new();
    let opts = AuditOptions {
        url: normalize_url(&options.url),
        ..options.clone()
    };

    let pages = if opts.is_batch() {
        let batch_source = if !opts.batch_url.is_empty() {
            opts.batch_url.clone()
        } else {
            opts.url.clone()
        };

        let _ = socket
            .send(ws::Message::Text(
                format!("[AUDIT] Fetching batch URLs from {}...", batch_source).into(),
            ))
            .await;

        let discovered = fetch_batch_urls(&batch_source, &opts.batch_format, socket).await?;
        let _ = socket
            .send(ws::Message::Text(
                format!("[AUDIT] Found {} URLs in batch", discovered.len()).into(),
            ))
            .await;
        discovered
    } else if opts.is_full_site() {
        let _ = socket
            .send(ws::Message::Text(
                format!("[AUDIT] Crawling {} for pages...", opts.url).into(),
            ))
            .await;
        let discovered = crawl_site(&opts.url, opts.subdomains, socket).await?;
        let _ = socket
            .send(ws::Message::Text(
                format!("[AUDIT] Found {} pages to analyze", discovered.len()).into(),
            ))
            .await;
        discovered
    } else {
        vec![opts.url.clone()]
    };

    for page_url in &pages {
        let _ = socket
            .send(ws::Message::Text(
                format!("[AUDIT] Analyzing {}", page_url).into(),
            ))
            .await;

        match analyze_page(page_url, &opts, socket).await {
            Ok(mut findings) => all_findings.append(&mut findings),
            Err(e) => {
                let _ = socket
                    .send(ws::Message::Text(format!("[!] Error on {}: {}", page_url, e).into()))
                    .await;
            }
        }
    }

    Ok(all_findings)
}

async fn crawl_site(
    entry_url: &str,
    include_subdomains: bool,
    socket: &mut WebSocket,
) -> Result<Vec<String>, String> {
    let cfg = TenguConfig::from_env();
    let client = cfg.http_client_builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let base_url = Url::parse(entry_url).map_err(|e| format!("Invalid URL: {}", e))?;
    let base_domain = base_url.host_str().unwrap_or("").to_string();

    let mut discovered = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(entry_url.to_string());

    while let Some(current) = queue.pop_front() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        let _ = socket
            .send(ws::Message::Text(format!("[PAGE] {}", current).into()))
            .await;

        discovered.push(current.clone());

        // Fetch and parse links
        if let Ok(resp) = client.get(&current).send().await {
            if let Ok(body) = resp.text().await {
                let doc = scraper::Html::parse_document(&body);
                let link_sel =
                    scraper::Selector::parse("a[href]").map_err(|_| "Invalid selector")?;

                for link in doc.select(&link_sel) {
                    if let Some(href) = link.value().attr("href") {
                        if let Ok(abs_url) = base_url.join(href) {
                            if abs_url.scheme() != "http" && abs_url.scheme() != "https" {
                                continue;
                            }
                            let host = abs_url.host_str().unwrap_or("");
                            if include_subdomains {
                                if !host.ends_with(&base_domain.trim_start_matches("www."))
                                    && host != &base_domain
                                {
                                    continue;
                                }
                            } else if host != &base_domain {
                                continue;
                            }
                            let url_str = abs_url.to_string();
                            if !visited.contains(&url_str) && queue.len() < 50 {
                                queue.push_back(url_str);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(discovered)
}

fn parse_sitemap_urls(xml: &str) -> Vec<String> {
    let doc = scraper::Html::parse_document(xml);
    let loc_sel = scraper::Selector::parse("loc").unwrap();
    let mut urls = Vec::new();
    for el in doc.select(&loc_sel) {
        let url = el.text().collect::<String>().trim().to_string();
        if !url.is_empty() && !urls.contains(&url) {
            urls.push(url);
        }
    }
    urls
}

fn parse_csv_urls(csv: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for line in csv.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let first_col = trimmed.split(',').next().unwrap_or("").trim().to_string();
        if !first_col.is_empty() && (first_col.starts_with("http://") || first_col.starts_with("https://")) {
            if !urls.contains(&first_col) {
                urls.push(first_col);
            }
        }
    }
    urls
}

async fn fetch_batch_urls(
    batch_url: &str,
    format: &str,
    socket: &mut WebSocket,
) -> Result<Vec<String>, String> {
    let cfg = TenguConfig::from_env();
    let client = cfg.http_client_builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(batch_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch batch source: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read batch response: {}", e))?;

    let urls = if format == "csv" || batch_url.ends_with(".csv") {
        parse_csv_urls(&body)
    } else {
        // Default to sitemap XML parsing
        let parsed = parse_sitemap_urls(&body);
        if parsed.is_empty() {
            // Try CSV as fallback
            let csv_urls = parse_csv_urls(&body);
            if !csv_urls.is_empty() {
                csv_urls
            } else {
                // Last resort: treat each non-empty line as a URL
                body.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect()
            }
        } else {
            parsed
        }
    };

    if urls.is_empty() {
        return Err("No URLs found in batch source".into());
    }

    Ok(urls)
}

async fn analyze_page(
    page_url: &str,
    options: &AuditOptions,
    socket: &mut WebSocket,
) -> Result<Vec<Finding>, String> {
    let cfg = TenguConfig::from_env();
    let client = cfg.http_client_builder().build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(page_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    let status = response.status();
    let headers = response.headers().clone();
    let html = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let _ = socket
        .send(ws::Message::Text(
            format!("[AUDIT] {} (HTTP {})", page_url, status).into(),
        ))
        .await;

    let mut all_findings = Vec::new();

    let audit_html = html_pretty::pretty_print(&html);

    if options.has_check("performance") {
        let mut findings = performance::analyze(&audit_html, &headers).await;
        for f in &mut findings {
            f.page_url = Some(page_url.to_string());
            let _ = socket
                .send(ws::Message::Text(f.to_ws_message().into()))
                .await;
        }
        all_findings.append(&mut findings);
    }

    if options.has_check("seo") {
        let mut findings = seo::analyze(&audit_html, page_url).await;
        for f in &mut findings {
            f.page_url = Some(page_url.to_string());
            let _ = socket
                .send(ws::Message::Text(f.to_ws_message().into()))
                .await;
        }
        all_findings.append(&mut findings);

        // Network-based SEO checks (robots.txt, sitemap, broken links) — runs after HTML analysis
        seo::analyze_seo_network(page_url, &audit_html, &mut all_findings).await;
        for f in all_findings.iter_mut().filter(|f| f.page_url.is_none() && (f.check == "robots_txt" || f.check == "sitemap" || f.check == "broken_links" || f.check == "redirect_chain")) {
            f.page_url = Some(page_url.to_string());
            let _ = socket
                .send(ws::Message::Text(f.to_ws_message().into()))
                .await;
        }
    }

    if options.has_check("accessibility") {
        let mut findings = a11y::analyze(&audit_html).await;
        for f in &mut findings {
            f.page_url = Some(page_url.to_string());
            let _ = socket
                .send(ws::Message::Text(f.to_ws_message().into()))
                .await;
        }
        all_findings.append(&mut findings);
    }

    if options.has_check("best_practices") {
        let mut findings = best_practices::analyze(&audit_html, &headers, page_url).await;
        for f in &mut findings {
            f.page_url = Some(page_url.to_string());
            let _ = socket
                .send(ws::Message::Text(f.to_ws_message().into()))
                .await;
        }
        all_findings.append(&mut findings);
    }

    let _ = socket
        .send(ws::Message::Text(format!("[HTML]{}", audit_html).into()))
        .await;

    Ok(all_findings)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub check: String,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub snippet: Option<String>,
    pub page_url: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    Pass,
    Info,
    Warning,
    Error,
}

impl Finding {
    pub fn to_ws_message(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}