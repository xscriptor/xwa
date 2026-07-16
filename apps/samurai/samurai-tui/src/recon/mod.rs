use serde::{Deserialize, Serialize};

pub mod api;
pub mod dns;
pub mod headers;
pub mod subdomains;
pub mod tech_stack;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdomainInfo {
    pub name: String,
    pub resolved_ips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub status: u16,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeader {
    pub name: String,
    pub present: bool,
    pub value: Option<String>,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechInfo {
    pub category: String,
    pub name: String,
    pub version: Option<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconResults {
    pub target: String,
    pub dns_records: Vec<DnsRecord>,
    pub subdomains: Vec<SubdomainInfo>,
    pub api_endpoints: Vec<ApiEndpoint>,
    pub security_headers: Vec<SecurityHeader>,
    pub technologies: Vec<TechInfo>,
}

impl ReconResults {
    #[allow(dead_code)]
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            dns_records: Vec::new(),
            subdomains: Vec::new(),
            api_endpoints: Vec::new(),
            security_headers: Vec::new(),
            technologies: Vec::new(),
        }
    }

    pub fn to_finding_description(&self) -> String {
        format!(
            "Web reconnaissance for {}: {} DNS records, {} subdomains, {} API endpoints, {} headers checked, {} technologies",
            self.target,
            self.dns_records.len(),
            self.subdomains.len(),
            self.api_endpoints.len(),
            self.security_headers.len(),
            self.technologies.len(),
        )
    }
}
