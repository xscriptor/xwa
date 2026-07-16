use std::sync::Arc;
use std::time::Duration;

use crate::config::ScrapeConfig;
use crate::scraper::anti_block;

use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Client;
use tokio::sync::Mutex;
use tracing::info;

pub struct ScrapeClient {
    pub client: Client,
    pub config: Arc<ScrapeConfig>,
    domain_timers: Mutex<std::collections::HashMap<String, tokio::time::Instant>>,
}

impl ScrapeClient {
    pub fn new(config: Arc<ScrapeConfig>) -> Self {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .gzip(true)
            .brotli(true)
            .pool_max_idle_per_host(config.concurrency.max(5))
            .tcp_keepalive(Duration::from_secs(30))
            .cookie_store(true);

        if !config.auth_username.is_empty() && !config.auth_password.is_empty() {
            if config.auth_mode == "basic" {
                let encoded = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    format!("{}:{}", config.auth_username, config.auth_password),
                );
                if let Ok(auth_val) = reqwest::header::HeaderValue::from_str(&format!("Basic {}", encoded)) {
                    let mut h = reqwest::header::HeaderMap::new();
                    h.insert(reqwest::header::AUTHORIZATION, auth_val);
                    builder = builder.default_headers(h);
                }
            }
        }

        if config.use_proxies && !config.proxy_list.is_empty() {
            if let Some(proxy_url) = config.proxy_list.first() {
                match reqwest::Proxy::all(proxy_url) {
                    Ok(p) => { builder = builder.proxy(p); info!("Using proxy: {}", proxy_url); }
                    Err(e) => { info!("Invalid proxy {}: {}", proxy_url, e); }
                }
            }
        }

        if !config.user_agent_rotation {
            builder = builder.user_agent(anti_block::random_user_agent());
        }

        let client = builder.build().expect("Failed to build HTTP client");

        Self {
            client,
            config,
            domain_timers: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub async fn get(&self, url: &str, attempt: u32) -> Result<reqwest::Response, String> {
        let mut req = self.client.get(url);

        if self.config.user_agent_rotation {
            req = req.header("User-Agent", anti_block::random_user_agent());
        }

        for (k, v) in anti_block::random_headers() {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(&v),
            ) {
                req = req.header(name, val);
            }
        }

        if let Some(host) = url.split('/').nth(2) {
            let mut timers = self.domain_timers.lock().await;
            let domain = host.to_string();
            let rate_limit_ms = if self.config.rate_limit > 0 {
                self.config.rate_limit
            } else {
                self.config.delay_ms
            };

            if let Some(last) = timers.get(&domain) {
                let elapsed = last.elapsed().as_millis() as u64;
                if elapsed < rate_limit_ms {
                    let wait = rate_limit_ms - elapsed;
                    tokio::time::sleep(Duration::from_millis(wait)).await;
                }
            }
            timers.insert(domain, tokio::time::Instant::now());
        }

        let response = req.send().await.map_err(|e| {
            if attempt < self.config.retry_count {
                format!("Request failed (attempt {}): {}", attempt + 1, e)
            } else {
                format!("Request failed after {} attempts: {}", attempt + 1, e)
            }
        })?;

        let status = response.status();
        if status.is_success() {
            Ok(response)
        } else if status.as_u16() == 429 || status.as_u16() == 503 {
            Err(format!("Rate limited (HTTP {})", status))
        } else if status.is_server_error() {
            Err(format!("Server error (HTTP {})", status))
        } else if status.is_client_error() {
            Err(format!("Client error (HTTP {})", status))
        } else {
            Ok(response)
        }
    }

    pub async fn get_with_retry(&self, url: &str) -> Result<reqwest::Response, String> {
        let mut last_err = String::new();
        for attempt in 0..self.config.retry_count {
            match self.get(url, attempt).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    last_err = e.clone();
                    if e.contains("Rate limited") || e.contains("429") || e.contains("503") {
                        let wait = anti_block::backoff_ms(attempt, 5000);
                        info!("Rate limited (attempt {}), waiting {}ms", attempt + 1, wait);
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                    } else if attempt < self.config.retry_count - 1 {
                        let wait = anti_block::backoff_ms(attempt, 1000);
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                    }
                }
            }
        }
        Err(last_err)
    }

}
