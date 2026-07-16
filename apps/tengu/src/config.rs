use serde::Deserialize;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Per-request audit options (from WebSocket query params)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct AuditOptions {
    pub url: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub subdomains: bool,
    #[serde(default = "default_checks")]
    pub checks: String,
    #[serde(default)]
    pub batch_url: String,
    #[serde(default)]
    pub batch_format: String,
}

fn default_checks() -> String {
    "performance,seo,accessibility,best_practices".to_string()
}

impl AuditOptions {
    pub fn has_check(&self, name: &str) -> bool {
        self.checks.split(',').any(|c| c.trim() == name)
    }

    pub fn is_full_site(&self) -> bool {
        self.mode == "fullsite"
    }

    pub fn is_batch(&self) -> bool {
        self.mode == "batch"
    }
}

// ---------------------------------------------------------------------------
// Global server configuration (from environment variables)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TenguConfig {
    pub port: u16,
    pub static_dir: String,
    pub max_history: usize,
    pub api_key: Option<String>,

    pub http_timeout_secs: u64,
    pub http_max_redirects: usize,
    pub http_retry_count: u32,
    pub http_user_agent: String,

    pub rate_limit_burst: u32,
    pub rate_limit_per_second: f64,

    pub max_concurrent_audits: usize,

    pub database_url: Option<String>,
}

impl TenguConfig {
    pub fn from_env() -> Self {
        Self {
            port: env_var("PORT").and_then(|v| v.parse().ok()).unwrap_or(8080),
            static_dir: env_var("STATIC_DIR").unwrap_or_else(|| "static/browser".into()),
            max_history: env_var("TENGU_MAX_HISTORY").and_then(|v| v.parse().ok()).unwrap_or(100),
            api_key: env_var("TENGU_API_KEY").filter(|s| !s.is_empty()),

            http_timeout_secs: env_var("TENGU_HTTP_TIMEOUT").and_then(|v| v.parse().ok()).unwrap_or(30),
            http_max_redirects: env_var("TENGU_HTTP_MAX_REDIRECTS").and_then(|v| v.parse().ok()).unwrap_or(10),
            http_retry_count: env_var("TENGU_HTTP_RETRY").and_then(|v| v.parse().ok()).unwrap_or(2),
            http_user_agent: env_var("TENGU_USER_AGENT").unwrap_or_else(|| "Tengu/0.2.0".into()),

            rate_limit_burst: env_var("TENGU_RATE_LIMIT_BURST").and_then(|v| v.parse().ok()).unwrap_or(10),
            rate_limit_per_second: env_var("TENGU_RATE_LIMIT_PER_SECOND").and_then(|v| v.parse().ok()).unwrap_or(2.0),

            max_concurrent_audits: env_var("TENGU_MAX_CONCURRENT").and_then(|v| v.parse().ok()).unwrap_or(5),

            database_url: env_var("DATABASE_URL").filter(|s| !s.is_empty()),
        }
    }

    pub fn http_timeout(&self) -> Duration {
        Duration::from_secs(self.http_timeout_secs)
    }

    pub fn http_client_builder(&self) -> reqwest::ClientBuilder {
        let mut builder = reqwest::Client::builder()
            .user_agent(&self.http_user_agent)
            .timeout(self.http_timeout())
            .redirect(reqwest::redirect::Policy::limited(self.http_max_redirects));

        if self.http_retry_count > 0 {
            // reqwest doesn't have built-in retry, but we set a reasonable timeout
        }

        builder
    }
}

fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}
