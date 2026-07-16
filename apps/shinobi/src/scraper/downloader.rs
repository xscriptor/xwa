use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use scraper::{Html, Selector};
use sha2::{Sha256, Digest};
use tokio::sync::mpsc;
use url::Url;

use crate::config::ScrapeConfig;
use crate::scraper::client::ScrapeClient;
use crate::scraper::renderer::Renderer;
use crate::scraper::robots::RobotsTxt;
use crate::scraper::sitemap;
use crate::scraper::extractor;
use crate::scraper::rewriter::{rewrite_html, generate_index};
use crate::scraper::warc::{WarcRecord, create_warc_file};
use crate::storage::manager::StorageManager;
use tracing::{error, info, warn};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeProgress {
    pub pages_scraped: usize,
    pub files_downloaded: usize,
    pub total_pages: usize,
    pub status: String,
    pub current_url: Option<String>,
    pub errors: Vec<String>,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
    pub deep_extracted: Option<serde_json::Value>,
}

pub struct Downloader {
    config: Arc<ScrapeConfig>,
    client: ScrapeClient,
    renderer: Option<Renderer>,
    storage: Arc<StorageManager>,
    base_url: Url,
    domain: String,
    visited: Arc<std::sync::Mutex<HashSet<String>>>,
    content_hashes: Arc<std::sync::Mutex<HashSet<String>>>,
    robots: Option<RobotsTxt>,
    page_count: Arc<AtomicUsize>,
    file_count: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
    errors: Arc<std::sync::Mutex<Vec<String>>>,
    emails: Arc<std::sync::Mutex<Vec<String>>>,
    phones: Arc<std::sync::Mutex<Vec<String>>>,
    saved_files: Arc<std::sync::Mutex<Vec<String>>>,
    warc_records: Arc<std::sync::Mutex<Vec<WarcRecord>>>,
    saved_paths: Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
}

impl Downloader {
    pub async fn new(config: Arc<ScrapeConfig>, storage: Arc<StorageManager>) -> Result<Self, String> {
        let base_url = Url::parse(&config.sanitize_url())
            .map_err(|e| format!("Invalid URL: {}", e))?;
        let domain = base_url.host_str().unwrap_or("unknown").to_string();
        let client = ScrapeClient::new(config.clone());

        let robots = if config.respect_robots_txt {
            let robots_url = format!("{}://{}/robots.txt", base_url.scheme(), base_url.host_str().unwrap_or(""));
            match client.get_with_retry(&robots_url).await {
                Ok(resp) => {
                    let body = resp.text().await.unwrap_or_default();
                    let r = RobotsTxt::parse(&body);
                    info!("Loaded robots.txt with {} rules", r.disallowed.len());
                    Some(r)
                }
                Err(e) => { warn!("robots.txt failed: {}", e); None }
            }
        } else { None };

        let renderer = if config.javascript_rendering {
            match Renderer::new().await {
                Ok(r) => { info!("Headless browser ready"); Some(r) }
                Err(e) => { warn!("Browser unavailable: {}", e); None }
            }
        } else { None };

        Ok(Self {
            config, client, renderer, storage, base_url, domain,
            visited: Arc::new(std::sync::Mutex::new(HashSet::new())),
            content_hashes: Arc::new(std::sync::Mutex::new(HashSet::new())),
            robots, page_count: Arc::new(AtomicUsize::new(0)),
            file_count: Arc::new(AtomicUsize::new(0)),
            cancelled: Arc::new(AtomicBool::new(false)),
            errors: Arc::new(std::sync::Mutex::new(Vec::new())),
            emails: Arc::new(std::sync::Mutex::new(Vec::new())),
            phones: Arc::new(std::sync::Mutex::new(Vec::new())),
            saved_files: Arc::new(std::sync::Mutex::new(Vec::new())),
            warc_records: Arc::new(std::sync::Mutex::new(Vec::new())),
            saved_paths: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        })
    }

    pub fn cancel(&self) { self.cancelled.store(true, Ordering::SeqCst); }

    fn is_valid_extension(&self, path: &str) -> bool {
        if self.config.file_types.is_empty() { return true; }
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        ext.is_empty() || self.config.file_types.contains(&ext)
    }

    fn should_download(&self, url: &Url) -> bool {
        if self.config.same_domain_only {
            if let Some(host) = url.host_str() {
                if !host.ends_with(&self.domain) && host != self.domain { return false; }
            } else { return false; }
        }
        if let Some(robots) = &self.robots {
            if !robots.is_allowed(url) { info!("Blocked by robots.txt: {}", url); return false; }
        }
        let ext = url.path().rsplit('.').next().unwrap_or("");
        let assets: HashSet<&str> = ["css","js","png","jpg","jpeg","gif","svg","webp","ico","woff","woff2","ttf","eot","mp4","webm","pdf","zip","gz","tar","mp3","wav"].into();
        if assets.contains(ext) { return self.config.download_assets && self.is_valid_extension(url.path()); }
        true
    }

    fn extract_links(&self, body: &str, base: &Url) -> Vec<Url> {
        let mut links = Vec::new();
        let doc = Html::parse_document(body);
        for sel_str in &["a[href]", "link[href]", "img[src]", "script[src]", "source[src]", "video[src]", "audio[src]"] {
            if let Ok(sel) = Selector::parse(sel_str) {
                for el in doc.select(&sel) {
                    let attr = if sel_str.contains("href") { "href" } else { "src" };
                    if let Some(val) = el.value().attr(attr) {
                        if let Ok(abs) = base.join(val) {
                            if abs.scheme() == "http" || abs.scheme() == "https" { links.push(abs); }
                        }
                    }
                }
            }
        }
        links
    }

    fn is_html_path(&self, path: &str) -> bool {
        let ext = path.rsplit('.').next().unwrap_or("");
        matches!(ext, "html" | "htm" | "php" | "asp" | "aspx" | "" | "/")
    }

    fn get_save_path(&self, url: &Url) -> String {
        let path = url.path();
        let host = url.host_str().unwrap_or("unknown");
        if path == "/" || path.is_empty() { format!("{}/index.html", host) }
        else {
            let clean = path.trim_start_matches('/');
            if clean.contains('.') { format!("{}/{}", host, clean) }
            else { format!("{}/{}/index.html", host, clean.trim_end_matches('/')) }
        }
    }

    fn canonicalize(&self, url: &str) -> String {
        if let Ok(mut p) = Url::parse(url) {
            p.set_fragment(None);
            let mut path = p.path().to_string();
            if path.len() > 1 && path.ends_with('/') { path = path.trim_end_matches('/').to_string(); p.set_path(&path); }
            p.as_str().trim_end_matches('/').to_string()
        } else { url.trim_end_matches('/').to_string() }
    }

    fn is_duplicate(&self, body: &str) -> bool {
        if !self.config.deduplicate { return false; }
        let mut h = Sha256::new(); h.update(body.as_bytes());
        !self.content_hashes.lock().unwrap().insert(format!("{:x}", h.finalize()))
    }

    async fn send_progress(&self, pages: usize, files: usize, total: usize, status: &str, url: Option<String>, tx: &mpsc::Sender<ScrapeProgress>) {
        let e = self.errors.lock().unwrap().clone();
        let em = self.emails.lock().unwrap().clone();
        let ph = self.phones.lock().unwrap().clone();
        let _ = tx.send(ScrapeProgress { pages_scraped: pages, files_downloaded: files, total_pages: total, status: status.to_string(), current_url: url, errors: e, emails: em, phones: ph, deep_extracted: None }).await;
    }

    async fn fire_webhook(&self, status: &str, pages: usize, files: usize) {
        if self.config.webhook_url.is_empty() { return; }
        let payload = serde_json::json!({"event":"scrape_complete","status":status,"url":self.config.url,"pages_scraped":pages,"files_downloaded":files,"domain":self.domain});
        if let Err(e) = self.client.client.post(&self.config.webhook_url).json(&payload).send().await {
            warn!("Webhook failed: {}", e);
        } else { info!("Webhook sent"); }
    }

    fn content_type_for_path(path: &str) -> &str {
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "json" => "application/json",
            "xml" => "application/xml",
            "pdf" => "application/pdf",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            _ => "application/octet-stream",
        }
    }

    pub async fn run(&self, tx: mpsc::Sender<ScrapeProgress>) {
        let pages_to_visit = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let total_pages = self.config.max_pages;
        { pages_to_visit.lock().await.push((self.base_url.clone(), 0u32)); }

        let sitemap_url = format!("{}://{}/sitemap.xml", self.base_url.scheme(), self.base_url.host_str().unwrap_or(""));
        if let Ok(resp) = self.client.get_with_retry(&sitemap_url).await {
            if let Ok(body) = resp.text().await {
                let urls = sitemap::parse_sitemap(&body, &self.base_url);
                info!("Found {} URLs in sitemap.xml", urls.len());
                let mut pages = pages_to_visit.lock().await;
                for u in urls {
                    let n = self.canonicalize(u.as_str());
                    if !self.visited.lock().unwrap().contains(&n) { pages.push((u, 0)); }
                }
            }
        }

        loop {
            if self.cancelled.load(Ordering::SeqCst) {
                self.send_progress(self.page_count.load(Ordering::SeqCst), self.file_count.load(Ordering::SeqCst), total_pages, "cancelled", None, &tx).await;
                self.fire_webhook("cancelled", self.page_count.load(Ordering::SeqCst), self.file_count.load(Ordering::SeqCst)).await;
                break;
            }

            let next = pages_to_visit.lock().await.pop();

            if let Some((url, depth)) = next {
                if depth > self.config.depth { continue; }
                let norm = self.canonicalize(url.as_str());
                { let mut v = self.visited.lock().unwrap(); if v.contains(&norm) { continue; } v.insert(norm); }

                let pc = self.page_count.fetch_add(1, Ordering::SeqCst) + 1;
                if pc > self.config.max_pages { continue; }

                info!("[{}/{}] {}", pc, total_pages, url);
                self.send_progress(pc, self.file_count.load(Ordering::SeqCst), total_pages, "scraping", Some(url.to_string()), &tx).await;
                if !self.should_download(&url) { continue; }

                let is_html = self.is_html_path(url.path());
                let should_render = self.renderer.is_some() && is_html;

                let (body, screenshot_data) = if should_render {
                    let r = self.renderer.as_ref().unwrap();
                    match r.fetch_page(url.as_str()).await {
                        Ok((html, ss)) => (html, ss),
                        Err(e) => { error!("JS render failed: {}", e); self.errors.lock().unwrap().push(format!("JS render {}: {}", url, e)); continue; }
                    }
                } else {
                    (match self.client.get_with_retry(url.as_str()).await {
                        Ok(resp) => resp.text().await.unwrap_or_default(),
                        Err(e) => { error!("Fetch failed: {}", e); self.errors.lock().unwrap().push(format!("{}: {}", url, e)); continue; }
                    }, None)
                };

                if self.is_duplicate(&body) { info!("Skipping duplicate: {}", url); continue; }

                if is_html {
                    let links = self.extract_links(&body, &url);
                    for link in links {
                        let n = self.canonicalize(link.as_str());
                        let mut pages = pages_to_visit.lock().await;
                        if !self.visited.lock().unwrap().contains(&n) && n.starts_with("http") {
                            pages.push((link, depth + 1));
                            if pages.len() > self.config.max_pages * 2 { break; }
                        }
                    }
                }

                let save_path = self.get_save_path(&url);
                let final_body;

                if self.config.rewrite_urls && is_html {
                    let rewritten = rewrite_html(&body, &url, &save_path, &self.domain);
                    final_body = rewritten;
                } else {
                    final_body = body.clone();
                }

                if let Err(e) = self.storage.save_file(&save_path, final_body.as_bytes()).await {
                    warn!("Save failed: {}", e);
                } else {
                    self.file_count.fetch_add(1, Ordering::SeqCst);
                    self.saved_files.lock().unwrap().push(save_path.clone());
                    self.saved_paths.lock().unwrap().insert(url.to_string(), save_path.clone());
                    if self.config.export_warc {
                        let content_type = Self::content_type_for_path(&save_path);
                        self.warc_records.lock().unwrap().push(WarcRecord::new(url.as_str(), content_type, final_body.as_bytes()));
                    }
                }

                if self.config.take_screenshots {
                    if let Some(ss) = &screenshot_data {
                        let ss_path = format!("{}/screenshots/{}.png", self.domain, url.path().trim_start_matches('/').replace('/', "_").trim_end_matches('_'));
                        let _ = self.storage.save_file(&ss_path, ss).await;
                    }
                }

                if self.config.extract_emails && is_html {
                    let data = extractor::extract_all(&body, url.as_str());
                    let mut emails = self.emails.lock().unwrap();
                    for e in &data.emails { if !emails.contains(e) { emails.push(e.clone()); } }
                    let mut phones = self.phones.lock().unwrap();
                    for p in &data.phones { if !phones.contains(p) { phones.push(p.clone()); } }
                }

                if self.config.deep_mode && is_html {
                    let endpoint = if self.config.extractor_endpoint.is_empty() {
                        "http://localhost:9090".to_string()
                    } else {
                        self.config.extractor_endpoint.clone()
                    };
                    let payload = serde_json::json!({
                        "url": url.to_string(),
                        "html": body,
                        "extract_structured": self.config.extract_structured,
                        "nlp_enabled": self.config.nlp_enabled,
                        "custom_selectors": self.config.custom_selectors,
                    });
                    match self.client.client.post(format!("{}/extract", endpoint)).json(&payload).send().await {
                        Ok(resp) => {
                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                let em = data.get("emails").and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                                    .unwrap_or_default();
                                let ph = data.get("phones").and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                                    .unwrap_or_default();
                                {
                                    let mut e = self.emails.lock().unwrap();
                                    for addr in &em { if !e.contains(addr) { e.push(addr.clone()); } }
                                    let mut p = self.phones.lock().unwrap();
                                    for phone in &ph { if !p.contains(phone) { p.push(phone.clone()); } }
                                }
                                let pc = self.page_count.load(Ordering::SeqCst);
                                let fc = self.file_count.load(Ordering::SeqCst);
                                let deep_url = url.to_string();
                                let deep_errors;
                                let deep_emails;
                                let deep_phones;
                                {
                                    deep_errors = self.errors.lock().unwrap().clone();
                                    deep_emails = self.emails.lock().unwrap().clone();
                                    deep_phones = self.phones.lock().unwrap().clone();
                                }
                                let _ = tx.send(ScrapeProgress {
                                    pages_scraped: pc,
                                    files_downloaded: fc,
                                    total_pages: self.config.max_pages,
                                    status: "deep".into(),
                                    current_url: Some(deep_url),
                                    errors: deep_errors,
                                    emails: deep_emails,
                                    phones: deep_phones,
                                    deep_extracted: Some(data),
                                }).await;
                            }
                        }
                        Err(e) => warn!("Deep extractor call failed for {}: {}", url, e),
                    }
                }
            } else {
                let pc = self.page_count.load(Ordering::SeqCst);
                let fc = self.file_count.load(Ordering::SeqCst);

                if self.config.generate_index {
                    let files = self.saved_files.lock().unwrap().clone();
                    let index = generate_index(&files, &self.domain);
                    let _ = self.storage.save_file(&format!("{}/index.html", &self.domain), index.as_bytes()).await;
                }

                if self.config.export_warc {
                    let records = self.warc_records.lock().unwrap().clone();
                    if !records.is_empty() {
                        let warc_data = create_warc_file(&records);
                        let warc_path = format!("{}/site.warc", &self.domain);
                        let _ = self.storage.save_file(&warc_path, &warc_data).await;
                        info!("WARC file saved: {}", warc_path);
                    }
                }

                self.send_progress(pc, fc, total_pages, "completed", None, &tx).await;
                self.fire_webhook("completed", pc, fc).await;
                break;
            }
        }
    }
}
