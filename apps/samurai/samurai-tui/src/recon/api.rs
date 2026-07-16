use crate::recon::ApiEndpoint;
use futures::future::join_all;

const COMMON_API_PATHS: &[&str] = &[
    "/api", "/api/v1", "/api/v2", "/graphql", "/swagger", "/swagger.json",
    "/swagger-ui.html", "/openapi.json", "/docs", "/api/docs", "/rest",
    "/v1", "/v2", "/api/auth", "/api/users", "/api/admin",
    "/wp-json", "/wp-json/wp/v2", "/.well-known", "/actuator",
    "/actuator/health", "/actuator/info", "/status", "/health",
    "/api/health", "/ping", "/metrics", "/api/metrics",
];

pub async fn discover_apis(target: &str, client: &reqwest::Client) -> Vec<ApiEndpoint> {
    let base = if target.starts_with("http") {
        target.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", target)
    };

    let futures: Vec<_> = COMMON_API_PATHS.iter().map(|path| {
        let url = format!("{}{}", base, path);
        let client = client.clone();
        async move {
            match client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if status != 404 {
                        Some(ApiEndpoint {
                            path: path.to_string(),
                            status,
                            content_type: resp.headers().get("content-type")
                                .and_then(|v| v.to_str().ok())
                                .map(|s| s.to_string()),
                        })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
    }).collect();

    join_all(futures).await.into_iter().flatten().collect()
}
