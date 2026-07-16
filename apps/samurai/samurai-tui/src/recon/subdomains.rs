use crate::recon::SubdomainInfo;
use hickory_resolver::TokioAsyncResolver;

pub async fn enumerate_subdomains(
    target: &str,
    client: &reqwest::Client,
    resolver: &TokioAsyncResolver,
) -> Vec<SubdomainInfo> {
    let mut results = Vec::new();

    let cert_url = format!("https://crt.sh/?q=%25.{}&output=json", target);

    if let Ok(resp) = client.get(&cert_url).send().await {
        if let Ok(json) = resp.json::<Vec<serde_json::Value>>().await {
            let mut seen = std::collections::HashSet::new();
            for entry in json {
                if let Some(name) = entry.get("name_value").and_then(|v| v.as_str()) {
                    for sub in name.split('\n') {
                        let sub = sub.trim().trim_start_matches("*.");
                        if sub.ends_with(target) && sub.len() > target.len() && seen.insert(sub.to_string()) {
                            let mut ips = Vec::new();
                            if let Ok(lookup) = resolver.lookup_ip(sub).await {
                                for ip in lookup.iter() {
                                    ips.push(ip.to_string());
                                }
                            }
                            results.push(SubdomainInfo { name: sub.to_string(), resolved_ips: ips });
                        }
                    }
                }
            }
        }
    }

    results
}
