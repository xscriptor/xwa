use regex::Regex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedData {
    pub emails: Vec<String>,
    pub phones: Vec<String>,
}

pub fn extract_all(body: &str, _base_url: &str) -> ExtractedData {
    ExtractedData {
        emails: extract_emails(body),
        phones: extract_phones(body),
    }
}

fn extract_emails(body: &str) -> Vec<String> {
    let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    let mut results: Vec<String> = re.find_iter(body)
        .map(|m| m.as_str().to_lowercase())
        .collect();
    results.sort();
    results.dedup();
    results
}

fn extract_phones(body: &str) -> Vec<String> {
    let re = Regex::new(r"(\+?\d[\s.-]?){1,3}\(\d{2,4}\)[\s.-]?\d{3,4}[\s.-]?\d{3,4}").unwrap();
    let mut results: Vec<String> = re.find_iter(body)
        .map(|m| m.as_str().to_string())
        .collect();
    results.sort();
    results.dedup();
    results
}


