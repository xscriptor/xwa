use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeConfig {
    pub url: String,
    #[serde(default = "default_depth")]
    pub depth: u32,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,
    #[serde(default = "default_max_pages")]
    pub max_pages: usize,
    #[serde(default = "default_same_domain")]
    pub same_domain_only: bool,
    #[serde(default = "default_respect_robots")]
    pub respect_robots_txt: bool,
    #[serde(default = "default_file_types")]
    pub file_types: Vec<String>,
    #[serde(default = "default_download_assets")]
    pub download_assets: bool,
    #[serde(default = "default_user_agent_rotation")]
    pub user_agent_rotation: bool,
    #[serde(default)]
    pub proxy_list: Vec<String>,
    #[serde(default = "default_use_proxies")]
    pub use_proxies: bool,
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    #[serde(default = "default_js_rendering")]
    pub javascript_rendering: bool,
    #[serde(default)]
    pub take_screenshots: bool,
    #[serde(default)]
    pub extract_emails: bool,
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default = "default_dedup")]
    pub deduplicate: bool,
    #[serde(default = "default_rewrite")]
    pub rewrite_urls: bool,
    #[serde(default)]
    pub generate_index: bool,
    #[serde(default)]
    pub export_warc: bool,
    #[serde(default)]
    pub auth_username: String,
    #[serde(default)]
    pub auth_password: String,
    #[serde(default)]
    pub auth_mode: String,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u64,
    #[serde(default)]
    pub deep_mode: bool,
    #[serde(default)]
    pub extract_structured: bool,
    #[serde(default)]
    pub nlp_enabled: bool,
    #[serde(default)]
    pub custom_selectors: Vec<String>,
    #[serde(default = "default_export_format")]
    pub export_format: String,
    #[serde(default)]
    pub extractor_endpoint: String,
}

fn default_depth() -> u32 { 2 }
fn default_concurrency() -> usize { 3 }
fn default_delay_ms() -> u64 { 1000 }
fn default_max_pages() -> usize { 100 }
fn default_same_domain() -> bool { true }
fn default_respect_robots() -> bool { true }
fn default_download_assets() -> bool { true }
fn default_user_agent_rotation() -> bool { true }
fn default_use_proxies() -> bool { false }
fn default_retry_count() -> u32 { 3 }
fn default_js_rendering() -> bool { false }
fn default_dedup() -> bool { true }
fn default_rewrite() -> bool { true }
fn default_rate_limit() -> u64 { 0 }
fn default_export_format() -> String { "json".into() }

fn default_file_types() -> Vec<String> {
    vec![
        "html".into(), "css".into(), "js".into(),
        "png".into(), "jpg".into(), "jpeg".into(), "gif".into(), "svg".into(), "webp".into(),
        "pdf".into(), "doc".into(), "docx".into(),
        "json".into(), "xml".into(), "csv".into(),
        "zip".into(), "tar".into(), "gz".into(),
        "mp4".into(), "mp3".into(), "woff2".into(),
    ]
}

impl Default for ScrapeConfig {
    fn default() -> Self {
        Self {
            url: String::new(), depth: default_depth(), concurrency: default_concurrency(),
            delay_ms: default_delay_ms(), max_pages: default_max_pages(),
            same_domain_only: default_same_domain(), respect_robots_txt: default_respect_robots(),
            file_types: default_file_types(), download_assets: default_download_assets(),
            user_agent_rotation: default_user_agent_rotation(), proxy_list: Vec::new(),
            use_proxies: default_use_proxies(), retry_count: default_retry_count(),
            javascript_rendering: default_js_rendering(), take_screenshots: false,
            extract_emails: false, webhook_url: String::new(), deduplicate: default_dedup(),
            rewrite_urls: default_rewrite(), generate_index: false, export_warc: false,
            auth_username: String::new(), auth_password: String::new(), auth_mode: String::new(),
            rate_limit: default_rate_limit(),
            deep_mode: false, extract_structured: false, nlp_enabled: false,
            custom_selectors: Vec::new(), export_format: default_export_format(),
            extractor_endpoint: String::new(),
        }
    }
}

impl ScrapeConfig {
    pub fn sanitize_url(&self) -> String {
        let u = self.url.trim();
        if !u.starts_with("http://") && !u.starts_with("https://") {
            format!("https://{}", u)
        } else { u.to_string() }
    }
}
