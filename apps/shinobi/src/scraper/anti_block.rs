use rand::Rng;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (iPad; CPU OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Linux; Android 14; Pixel 8 Pro) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 OPR/109.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Vivaldi/6.7",
];

fn random_accept() -> &'static str {
    let accepts = [
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
    ];
    accepts[rand::thread_rng().gen_range(0..accepts.len())]
}

fn random_accept_language() -> &'static str {
    let langs = [
        "en-US,en;q=0.9,es;q=0.8",
        "en-GB,en;q=0.9,es;q=0.8",
        "en-US,en;q=0.9",
        "en-CA,en;q=0.9,fr;q=0.8",
        "es-ES,es;q=0.9,en;q=0.8",
        "de-DE,de;q=0.9,en;q=0.8",
        "fr-FR,fr;q=0.9,en;q=0.8",
        "pt-BR,pt;q=0.9,en;q=0.8",
    ];
    langs[rand::thread_rng().gen_range(0..langs.len())]
}

pub fn random_user_agent() -> String {
    USER_AGENTS[rand::thread_rng().gen_range(0..USER_AGENTS.len())].to_string()
}

pub fn random_headers() -> Vec<(String, String)> {
    let mut rng = rand::thread_rng();
    vec![
        ("Accept".into(), random_accept().into()),
        ("Accept-Language".into(), random_accept_language().into()),
        ("Accept-Encoding".into(), "gzip, deflate, br".into()),
        ("Sec-Fetch-Dest".into(), "document".into()),
        ("Sec-Fetch-Mode".into(), "navigate".into()),
        ("Sec-Fetch-Site".into(), if rng.gen_bool(0.7) { "none".into() } else { "cross-site".into() }),
        ("Sec-Fetch-User".into(), "?1".into()),
        ("Upgrade-Insecure-Requests".into(), "1".into()),
        ("Sec-Ch-Ua".into(), format!(
            "\"Not)A;Brand\";v=\"99\", \"Google Chrome\";v=\"{}\", \"Chromium\";v=\"{}\"",
            rng.gen_range(120..=126),
            rng.gen_range(120..=126),
        )),
        ("Sec-Ch-Ua-Mobile".into(), "?0".into()),
        ("Sec-Ch-Ua-Platform".into(), {
            let platforms = ["\"Windows\"", "\"macOS\"", "\"Linux\""];
            platforms[rng.gen_range(0..3)].into()
        }),
    ]
}

pub fn backoff_ms(attempt: u32, base_ms: u64) -> u64 {
    let jitter = rand::thread_rng().gen_range(0..1000);
    base_ms * 2u64.pow(attempt) + jitter
}
