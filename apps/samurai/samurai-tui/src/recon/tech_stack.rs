use crate::recon::TechInfo;
use scraper::{Html, Selector};
use std::sync::LazyLock;

static TECH_CHECKS: LazyLock<Vec<(&str, &str, Selector)>> = LazyLock::new(|| {
    let raw: Vec<(&str, &str, &str)> = vec![
        ("Frontend", "React", "script[src*=\"react\"]"),
        ("Frontend", "Vue.js", "script[src*=\"vue\"]"),
        ("Frontend", "Angular", "[ng-version], script[src*=\"angular\"]"),
        ("Frontend", "jQuery", "script[src*=\"jquery\"]"),
        ("Frontend", "Bootstrap", "link[href*=\"bootstrap\"]"),
        ("CSS", "Tailwind CSS", "[class*=\"tailwind\"], script[src*=\"tailwind\"]"),
        ("CSS", "Bulma", "link[href*=\"bulma\"]"),
    ];
    raw.into_iter()
        .filter_map(|(cat, name, sel)| {
            Selector::parse(sel).ok().map(|s| (cat, name, s))
        })
        .collect()
});

pub async fn fingerprint_technology(
    target: &str,
    client: &reqwest::Client,
) -> Vec<TechInfo> {
    let mut techs = Vec::new();

    let url = if target.starts_with("http") {
        target.to_string()
    } else {
        format!("https://{}", target)
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return techs,
    };

    let server_header = response
        .headers()
        .get("server")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body = match response.text().await {
        Ok(b) => b,
        Err(_) => return techs,
    };

    let document = Html::parse_document(&body);

    for (category, name, selector) in TECH_CHECKS.iter() {
        if document.select(selector).next().is_some() {
            techs.push(TechInfo {
                category: category.to_string(),
                name: name.to_string(),
                version: None,
                evidence: "Found via CSS selector match".to_string(),
            });
        }
    }

    if let Some(server) = server_header {
        let server_lower = server.to_lowercase();
        if server_lower.contains("nginx") {
            techs.push(TechInfo { category: "Backend".into(), name: "nginx".into(), version: None, evidence: format!("Server header: {}", server) });
        } else if server_lower.contains("apache") {
            techs.push(TechInfo { category: "Backend".into(), name: "Apache".into(), version: None, evidence: format!("Server header: {}", server) });
        } else if server_lower.contains("cloudflare") {
            techs.push(TechInfo { category: "CDN".into(), name: "Cloudflare".into(), version: None, evidence: format!("Server header: {}", server) });
        }
    }

    techs
}
