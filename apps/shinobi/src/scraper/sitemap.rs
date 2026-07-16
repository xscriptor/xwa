use scraper::{Html, Selector};
use url::Url;

pub fn parse_sitemap(body: &str, base: &Url) -> Vec<Url> {
    let mut urls = Vec::new();
    let doc = Html::parse_document(body);

    if let Ok(sel) = Selector::parse("loc") {
        for el in doc.select(&sel) {
            let text = el.text().collect::<String>().trim().to_string();
            if let Ok(url) = base.join(&text) {
                if url.scheme() == "http" || url.scheme() == "https" {
                    urls.push(url);
                }
            }
        }
    }

    urls
}
