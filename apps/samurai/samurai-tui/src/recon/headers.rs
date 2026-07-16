use crate::recon::SecurityHeader;

pub async fn check_security_headers(
    target: &str,
    client: &reqwest::Client,
) -> Vec<SecurityHeader> {
    let url = if target.starts_with("http") {
        target.to_string()
    } else {
        format!("https://{}", target)
    };

    let checks: [(&str, &str); 7] = [
        ("Content-Security-Policy", "Missing CSP allows XSS and data injection"),
        ("X-Content-Type-Options", "Missing X-Content-Type-Options allows MIME sniffing"),
        ("X-Frame-Options", "Missing X-Frame-Options allows clickjacking"),
        ("Strict-Transport-Security", "Missing HSTS allows SSL stripping"),
        ("Referrer-Policy", "Missing Referrer-Policy may leak URL data"),
        ("Permissions-Policy", "Missing Permissions-Policy allows feature abuse"),
        ("X-XSS-Protection", "Missing X-XSS-Protection (legacy)"),
    ];

    let mut headers = Vec::new();

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => {
            return checks.iter().map(|(name, risk)| SecurityHeader {
                name: name.to_string(),
                present: false,
                value: None,
                risk: risk.to_string(),
            }).collect();
        }
    };

    let resp_headers = response.headers();

    for (name, risk) in &checks {
        let value = resp_headers
            .get(*name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let present = value.is_some();
        headers.push(SecurityHeader {
            name: name.to_string(),
            present,
            value,
            risk: if present { "OK".into() } else { risk.to_string() },
        });
    }

    if let Some(server) = resp_headers.get("server").and_then(|v| v.to_str().ok()) {
        headers.push(SecurityHeader {
            name: "Server".into(),
            present: true,
            value: Some(server.to_string()),
            risk: format!("Server header disclosure: {}", server),
        });
    }

    headers
}
