use std::collections::HashSet;
use url::Url;

pub struct RobotsTxt {
    pub disallowed: HashSet<String>,
}

impl RobotsTxt {
    pub fn parse(body: &str) -> Self {
        let mut disallowed = HashSet::new();
        let mut in_agent = false;

        for line in body.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let lower = line.to_lowercase();
            if lower.starts_with("user-agent:") {
                let agent = lower.trim_start_matches("user-agent:").trim();
                in_agent = agent == "*" || agent.is_empty();
            } else if in_agent && lower.starts_with("disallow:") {
                let path = lower.trim_start_matches("disallow:").trim().to_string();
                if !path.is_empty() {
                    disallowed.insert(path);
                }
            }
        }

        Self { disallowed }
    }

    pub fn is_allowed(&self, url: &Url) -> bool {
        let path = url.path();
        !self.disallowed.iter().any(|d| path.starts_with(d))
    }
}
