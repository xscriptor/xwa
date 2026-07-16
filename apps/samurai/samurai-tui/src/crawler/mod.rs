use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPage {
    pub url: String,
    pub status_code: u16,
    pub content_type: Option<String>,
    pub content_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub finding_type: String,
    pub severity: String,
    pub description: String,
    pub evidence: String,
    pub cvss_score: Option<String>,
}

pub async fn discover_pages(
    target: &str,
    max_pages: usize,
    client: &reqwest::Client,
) -> Vec<DiscoveredPage> {
    let mut pages = Vec::new();
    let mut seen = HashSet::new();

    let base = if target.starts_with("http") {
        target.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", target)
    };

    let base_url = match url::Url::parse(&base) {
        Ok(u) => u,
        Err(_) => return pages,
    };

    if let Ok(resp) = client.get(&base).send().await {
        let url = resp.url().to_string();
        seen.insert(url.clone());
        let status = resp.status().as_u16();
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let body = resp.text().await.unwrap_or_default();
        let body_len = body.len();
        pages.push(DiscoveredPage {
            url: url.clone(),
            status_code: status,
            content_type: ct,
            content_length: body_len,
        });

        if pages.len() >= max_pages {
            return pages;
        }

        let mut hrefs: Vec<String> = Vec::new();
        {
            let doc = scraper::Html::parse_document(&body);
            if let Ok(sel) = scraper::Selector::parse("a[href]") {
                for el in doc.select(&sel) {
                    if hrefs.len() + pages.len() >= max_pages {
                        break;
                    }
                    if let Some(href) = el.value().attr("href") {
                        if let Ok(abs) = base_url.join(href) {
                            let abs_str = abs.to_string();
                            if abs_str.starts_with("http") && seen.insert(abs_str.clone()) {
                                hrefs.push(abs_str);
                            }
                        }
                    }
                }
            }
        } // doc dropped here — no more non-Send values across awaits

        let futures: Vec<_> = hrefs.into_iter().map(|abs_str| {
            let client = client.clone();
            async move {
                if let Ok(resp) = client.get(&abs_str).send().await {
                    Some(DiscoveredPage {
                        url: abs_str,
                        status_code: resp.status().as_u16(),
                        content_type: resp.headers().get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string()),
                        content_length: 0,
                    })
                } else {
                    None
                }
            }
        }).collect();

        let results = futures::future::join_all(futures).await;
        for page in results.into_iter().flatten() {
            if pages.len() < max_pages {
                pages.push(page);
            }
        }
    }

    pages
}
