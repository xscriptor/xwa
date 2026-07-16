use scraper::{Html, Selector};
use url::Url;

pub fn rewrite_html(html: &str, _source_url: &Url, save_path: &str, domain: &str) -> String {
    let save_dir = save_path.rsplit('/').skip(1).next().unwrap_or("");
    let depth = save_dir.matches('/').count() + 1;

    let doc = Html::parse_document(html);
    let mut result = html.to_string();

    let attrs = [
        ("a[href]", "href"),
        ("link[href]", "href"),
        ("img[src]", "src"),
        ("script[src]", "src"),
        ("source[src]", "src"),
        ("video[src]", "src"),
        ("audio[src]", "src"),
        ("form[action]", "action"),
        ("iframe[src]", "src"),
        ("object[data]", "data"),
    ];

    for (sel_str, attr) in &attrs {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in doc.select(&sel) {
                if let Some(val) = el.value().attr(attr) {
                    if val.starts_with("http://") || val.starts_with("https://") {
                        if let Ok(parsed) = Url::parse(val) {
                            if let Some(host) = parsed.host_str() {
                                if host.contains(domain) || domain.contains(host) {
                                    let rel = to_relative_path(parsed.path(), depth);
                                    let old = format!("{}=\"{}\"", attr, val);
                                    let new = format!("{}=\"{}\"", attr, rel);
                                    result = result.replace(&old, &new);
                                }
                            }
                        }
                    } else if val.starts_with("//") {
                        if let Ok(parsed) = Url::parse(&format!("https:{}", val)) {
                            if let Some(host) = parsed.host_str() {
                                if host.contains(domain) || domain.contains(host) {
                                    let rel = to_relative_path(parsed.path(), depth);
                                    let old = format!("{}=\"{}\"", attr, val);
                                    let new = format!("{}=\"{}\"", attr, rel);
                                    result = result.replace(&old, &new);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

fn to_relative_path(path: &str, depth: usize) -> String {
    if path == "/" || path.is_empty() {
        let mut rel = String::new();
        for _ in 0..depth {
            rel.push_str("../");
        }
        rel.push_str("index.html");
        return rel;
    }
    let clean = path.trim_start_matches('/');
    let mut rel = String::new();
    for _ in 0..depth {
        rel.push_str("../");
    }
    rel.push_str(clean);
    rel
}

pub fn generate_index(files: &[String], domain: &str) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{} — cloned site</title>
<style>
body{{font-family:monospace;background:#000;color:#e8e8e8;padding:20px;max-width:800px;margin:0 auto}}
h1{{color:#5b9bf6;font-size:14px;text-transform:uppercase;letter-spacing:1px}}
a{{color:#5b9bf6;text-decoration:none;display:block;padding:4px 8px;font-size:12px}}
a:hover{{background:#1a1a1a}}
</style></head><body>
<h1>// {} — cloned site</h1>
<hr style="border-color:#2a2a2a">
"#, domain, domain);

    for file in files {
        if file.contains("screenshots") { continue; }
        html.push_str(&format!("<a href=\"{}\">{}</a>\n", file, file));
    }

    html.push_str("</body></html>");
    html
}
