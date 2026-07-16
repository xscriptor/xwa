use crate::auditor::{Finding, Severity};
use reqwest::header::HeaderMap;
use scraper::{ElementRef, Html, Selector};

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 3).collect();
        format!("{}...", truncated)
    }
}

fn el_snippet(el: &ElementRef) -> Option<String> {
    let html = el.html();
    if html.is_empty() {
        return None;
    }
    if html.len() > 300 {
        Some(format!("{}…", &html[..297]))
    } else {
        Some(html)
    }
}

pub async fn analyze(html: &str, headers: &HeaderMap) -> Vec<Finding> {
    let mut findings = Vec::new();
    let document = Html::parse_document(html);

    page_weight_audit(html, &mut findings);
    resource_analysis(&document, &mut findings);
    image_audit(&document, &mut findings);
    font_audit(&document, &mut findings);
    cache_header_audit(headers, &mut findings);
    compression_audit(headers, &mut findings);
    render_blocking_audit(&document, &mut findings);
    third_party_script_audit(&document, &mut findings);
    web_vitals_note(&mut findings);

    findings
}

fn page_weight_audit(html: &str, findings: &mut Vec<Finding>) {
    let size_bytes = html.len();
    let size_kb = size_bytes as f64 / 1024.0;

    let (severity, threshold_note) = if size_kb > 2000.0 {
        (
            Severity::Error,
            format!(
                "exceeds the 2 MB recommended maximum by {:.0}%",
                (size_kb - 2000.0) / 20.0
            ),
        )
    } else if size_kb > 500.0 {
        (
            Severity::Warning,
            format!(
                "is between 500 KB and 2 MB — large but not critical",
            ),
        )
    } else if size_kb > 100.0 {
        (
            Severity::Info,
            format!("is between 100 KB and 500 KB — acceptable"),
        )
    } else {
        (
            Severity::Pass,
            format!("is under 100 KB — ideal"),
        )
    };

    let snippet = if size_bytes > 300 {
        Some(truncate(html, 300))
    } else {
        Some(html.to_string())
    };

    findings.push(Finding {
        category: "performance".to_string(),
        check: "page_weight".to_string(),
        severity,
        title: format!(
            "HTML document size: {:.1} KB ({:.2} MB)",
            size_kb,
            size_kb / 1024.0
        ),
        description: format!(
            "The raw HTML document is {:.1} KB ({:.2} MB, {} bytes). This {}.\n\n\
             Recommendations:\n  \
             · Target HTML size under 100 KB for optimal Time to First Byte (TTFB) and \
             First Contentful Paint (FCP). Every kilobyte adds latency, especially on \
             slow 3G connections where throughput is ~50 KB/s.\n  \
             · Server-side rendering: generate minimal HTML and hydrate on the client.\n  \
             · Remove unnecessary whitespace, comments, and unused markup.\n  \
             · Stream HTML using chunked transfer encoding so the browser can start \
             parsing before the full document arrives.\n  \
             · Use a CDN with edge caching to serve HTML from locations closer to the user.",
            size_kb,
            size_kb / 1024.0,
            size_bytes,
            threshold_note,
        ),
        snippet,
        page_url: None,
    });
}

fn resource_analysis(document: &Html, findings: &mut Vec<Finding>) {
    let css_sel = Selector::parse("link[rel=stylesheet]").unwrap();
    let js_sel = Selector::parse("script[src]").unwrap();
    let img_sel = Selector::parse("img[src], source[src]").unwrap();
    let iframe_sel = Selector::parse("iframe[src]").unwrap();
    let fetch_sel = Selector::parse("link[rel=preload], link[rel=prefetch]").unwrap();

    let css_count = document.select(&css_sel).count();
    let js_count = document.select(&js_sel).count();
    let img_count = document.select(&img_sel).count();
    let iframe_count = document.select(&iframe_sel).count();
    let fetch_count = document.select(&fetch_sel).count();
    let total = css_count + js_count + img_count + iframe_count + fetch_count;

    if total < 10 {
        return;
    }

    let mut examples: Vec<String> = Vec::new();
    let max_examples = 5;

    for el in document.select(&css_sel) {
        if examples.len() >= max_examples {
            break;
        }
        if let Some(snippet) = el_snippet(&el) {
            examples.push(snippet);
        }
    }
    for el in document.select(&js_sel) {
        if examples.len() >= max_examples {
            break;
        }
        if let Some(snippet) = el_snippet(&el) {
            examples.push(snippet);
        }
    }

    let (severity, threshold_note) = if total > 100 {
        (
            Severity::Error,
            format!(
                "The page loads {} external resources. This is extremely high and will \
                 significantly degrade load time, especially on mobile networks with high \
                 latency and limited parallel connections.",
                total
            ),
        )
    } else if total > 50 {
        (
            Severity::Warning,
            format!(
                "The page loads {} external resources. While not critical, reducing this \
                 count will improve load performance.",
                total
            ),
        )
    } else {
        (
            Severity::Info,
            format!(
                "The page loads {} external resources.",
                total
            ),
        )
    };

    let snippet = if examples.is_empty() {
        None
    } else {
        Some(examples.join("\n"))
    };

    findings.push(Finding {
        category: "performance".to_string(),
        check: "resource_waterfall".to_string(),
        severity,
        title: format!(
            "{} external resource(s) — {} CSS, {} JS, {} images, {} iframes, {} prefetch/preload",
            total, css_count, js_count, img_count, iframe_count, fetch_count,
        ),
        description: format!(
            "{}\n\n\
             Breakdown:\n  \
             · Stylesheets (CSS):   {}\n  \
             · Scripts (JS):        {}\n  \
             · Images/sources:      {}\n  \
             · Iframes:             {}\n  \
             · Preload/prefetch:    {}\n\n\
             HTTP/1.1 browsers typically open 6–8 concurrent connections per origin, \
             so resources are queued and downloaded sequentially in batches. \
             HTTP/2 multiplexing reduces this overhead, but each resource still \
             requires a request-response round trip.\n\n\
             Recommendations:\n  \
             · Bundle and minify CSS/JS files to reduce request count.\n  \
             · Use HTTP/2 or HTTP/3 for multiplexed resource delivery.\n  \
             · Lazy-load below-fold images and iframes with loading=\"lazy\".\n  \
             · Replace icon fonts with inline SVG sprites.\n  \
             · Remove unused CSS/JS — use coverage tools in DevTools to identify them.\n  \
             · Use code splitting for JavaScript to deliver only what the current route needs.",
            threshold_note,
            css_count,
            js_count,
            img_count,
            iframe_count,
            fetch_count,
        ),
        snippet,
        page_url: None,
    });
}

fn image_audit(document: &Html, findings: &mut Vec<Finding>) {
    let img_sel = Selector::parse("img").unwrap();
    let mut no_dim_count = 0u32;
    let mut no_lazy_count = 0u32;
    let mut no_dim_examples: Vec<ElementRef> = Vec::new();
    let mut no_lazy_examples: Vec<ElementRef> = Vec::new();
    let total = document.select(&img_sel).count();

    for img in document.select(&img_sel) {
        let src = img.value().attr("src").unwrap_or("");
        if src.starts_with("data:") {
            continue;
        }

        if img.value().attr("width").is_none() && img.value().attr("height").is_none() {
            no_dim_count += 1;
            if no_dim_examples.len() < 3 {
                no_dim_examples.push(img.clone());
            }
        }

        if img.value().attr("loading").is_none() && !src.starts_with("data:") {
            no_lazy_count += 1;
            if no_lazy_examples.len() < 3 {
                no_lazy_examples.push(img.clone());
            }
        }
    }

    if total == 0 {
        return;
    }

    if no_dim_count > 0 {
        let snippet = no_dim_examples.first().and_then(el_snippet);
        let lines: Vec<String> = no_dim_examples
            .iter()
            .map(|el| {
                let s = el.value().attr("src").unwrap_or("");
                format!("  · <img src=\"{}\">", truncate(s, 100))
            })
            .collect();

        findings.push(Finding {
            category: "performance".to_string(),
            check: "image_optimization".to_string(),
            severity: if no_dim_count as usize == total {
                Severity::Error
            } else {
                Severity::Warning
            },
            title: format!(
                "{}/{} image(s) missing explicit width and height",
                no_dim_count, total,
            ),
            description: format!(
                "Found {} out of {} images without width and/or height attributes. \
                 Without explicit dimensions, the browser cannot allocate the correct \
                 amount of space until the image loads, causing Cumulative Layout Shift \
                 (CLS) — a Core Web Vital metric that measures visual stability.\n\n\
                 Examples:\n{}\n\n\
                 Impact:\n  \
                 · CLS degrades user experience: content jumps as images load\n  \
                 · Google uses CLS as a ranking signal in search results\n  \
                 · Poor CLS scores (above 0.1) trigger the \"needs improvement\" or \
                 \"poor\" rating in Lighthouse\n\n\
                 Recommendation: Add width and height attributes to every <img> tag, \
                 matching the intrinsic dimensions of the source image:\n  \
                 <img src=\"example.jpg\" width=\"800\" height=\"600\" alt=\"...\">\n\n\
                 Alternatively, use CSS aspect-ratio combined with responsive sizing:\n  \
                 img {{ aspect-ratio: 800 / 600; width: 100%; height: auto; }}",
                no_dim_count, total,
                lines.join("\n"),
            ),
            snippet,
            page_url: None,
        });
    }

    if no_lazy_count > 0 {
        let snippet = no_lazy_examples.first().and_then(el_snippet);
        let lines: Vec<String> = no_lazy_examples
            .iter()
            .map(|el| {
                let s = el.value().attr("src").unwrap_or("");
                format!("  · <img src=\"{}\">", truncate(s, 100))
            })
            .collect();

        findings.push(Finding {
            category: "performance".to_string(),
            check: "image_optimization".to_string(),
            severity: Severity::Info,
            title: format!(
                "{}/{} image(s) without loading=\"lazy\"",
                no_lazy_count, total,
            ),
            description: format!(
                "Found {} images that are not using native lazy loading. Adding \
                 loading=\"lazy\" defers the download of below-the-fold images until \
                 the user scrolls near them, reducing initial page weight and \
                 improving LCP (Largest Contentful Paint).\n\n\
                 Examples:\n{}\n\n\
                 Recommendation: Add loading=\"lazy\" to all below-fold images:\n  \
                 <img src=\"photo.jpg\" loading=\"lazy\" alt=\"...\">\n\n\
                 Do NOT add loading=\"lazy\" to the Largest Contentful Paint (LCP) \
                 image — that image should load eagerly (the default) to ensure it \
                 renders as quickly as possible.\n\n\
                 Browser support: Chrome 76+, Firefox 75+, Safari 15.4+, Edge 79+.",
                no_lazy_count,
                lines.join("\n"),
            ),
            snippet,
            page_url: None,
        });
    }
}

fn font_audit(document: &Html, findings: &mut Vec<Finding>) {
    let font_sel = Selector::parse("link[rel=preload][as=font]").unwrap();
    let google_font_sel = Selector::parse(
        "link[href*=\"fonts.googleapis.com\"]",
    )
    .unwrap();
    let style_sel = Selector::parse("style").unwrap();
    let preconnect_google = Selector::parse(
        "link[rel=preconnect][href*=\"fonts.googleapis.com\"], \
         link[rel=preconnect][href*=\"fonts.gstatic.com\"]",
    )
    .unwrap();
    let preconnect_any = Selector::parse("link[rel=preconnect]").unwrap();

    let has_font_preload = document.select(&font_sel).next().is_some();
    let has_google_fonts = document.select(&google_font_sel).next().is_some();
    let has_font_face = document.select(&style_sel).any(|el| el.inner_html().contains("@font-face"));
    let has_preconnect_google = document.select(&preconnect_google).next().is_some();
    let preconnect_count = document.select(&preconnect_any).count();

    if has_google_fonts {
        if !has_preconnect_google {
            let el = document.select(&google_font_sel).next().unwrap();
            let href = el.value().attr("href").unwrap_or("");
            let snippet = el_snippet(&el);

            findings.push(Finding {
                category: "performance".to_string(),
                check: "font_loading".to_string(),
                severity: Severity::Warning,
                title: "Google Fonts loaded without preconnect hints".into(),
                description: format!(
                    "The page loads Google Fonts via `{}`, but does not include \
                     `<link rel=\"preconnect\">` hints for \"fonts.googleapis.com\" \
                     or \"fonts.gstatic.com\". Without preconnect, the browser must \
                     perform a DNS lookup, TCP handshake, and TLS negotiation for the \
                     font origins after discovering them in the stylesheet, adding \
                     ~200–400 ms of latency.\n\n\
                     Recommendations:\n  \
                     · Add to <head> before the font stylesheet:\n    \
                     <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n    \
                     <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" \
                     crossorigin>\n  \
                     · Combine preconnect with preload for the critical font files:\n    \
                     <link rel=\"preload\" href=\"/font.woff2\" as=\"font\" \
                     crossorigin>\n\n\
                     Impact: Preconnecting shaves ~200–400 ms off the font load time, \
                     directly improving LCP and reducing Flash of Invisible Text (FOIT).",
                    truncate(href, 150),
                ),
                snippet,
                page_url: None,
            });
        }

        if !has_font_preload {
            let el = document.select(&google_font_sel).next().unwrap();
            let href = el.value().attr("href").unwrap_or("");
            let snippet = el_snippet(&el);

            findings.push(Finding {
                category: "performance".to_string(),
                check: "font_loading".to_string(),
                severity: Severity::Info,
                title: "Google Fonts without font file preload".into(),
                description: format!(
                    "Google Fonts are loaded via stylesheet at `{}`, but none of the \
                     individual font files are preloaded with `<link rel=\"preload\" \
                     as=\"font\">`. The browser must download and parse the CSS \
                     stylesheet before it discovers the @font-face declarations and \
                     starts fetching the actual font files — this delays font rendering.\n\n\
                     Recommendation: Add preload hints for the most critical font files:\n  \
                     <link rel=\"preload\" href=\"https://fonts.gstatic.com/s/roboto/v27/...\" \
                     as=\"font\" crossorigin=\"anonymous\">\n\n\
                     Note: Only preload the font file(s) used by above-fold content. \
                     Preloading all font variants negates the benefit. Use Chrome DevTools \
                     → Network → \"Initiator\" column to identify the critical font files.",
                    truncate(href, 150),
                ),
                snippet,
                page_url: None,
            });
        }
    }

    if has_font_face && preconnect_count == 0 {
        let el = document.select(&style_sel).find(|el| el.inner_html().contains("@font-face")).unwrap();
        let snippet = el_snippet(&el);

        findings.push(Finding {
            category: "performance".to_string(),
            check: "font_loading".to_string(),
            severity: Severity::Info,
            title: "@font-face used without preconnect to font origins".into(),
            description: format!(
                "The page includes @font-face declarations but has no \
                 `<link rel=\"preconnect\">` hints for the font file origins. \
                 If fonts are hosted on a different origin (CDN, Google Fonts, \
                 Typekit), the browser must go through the full connection setup \
                 before it can start downloading the font files.\n\n\
                 Recommendation: Add preconnect hints for each font origin in <head>:\n  \
                 <link rel=\"preconnect\" href=\"https://fonts.example.com\" crossorigin>\n\n\
                 For self-hosted fonts (same origin), this is not needed.",
            ),
            snippet,
            page_url: None,
        });
    }

    if !has_google_fonts && !has_font_face {
        let total_sel = Selector::parse(
            "*[style*=\"font-family\"]:not(html):not(body)",
        )
        .unwrap();
        let styled_count = document.select(&total_sel).count();

        if styled_count > 0 {
            let snippet = None;

            findings.push(Finding {
                category: "performance".to_string(),
                check: "font_loading".to_string(),
                severity: Severity::Pass,
                title: "No web fonts detected — system fonts in use".into(),
                description: format!(
                    "The page does not load any web fonts via @font-face or Google Fonts. \
                     It relies on system fonts (e.g., Arial, Helvetica, system-ui). \
                     This is optimal for performance because:\n\n  \
                     · Zero font files to download (saves ~100–400 KB)\n  \
                     · No FOIT (Flash of Invisible Text) — text is immediately visible\n  \
                     · No FOUT (Flash of Unstyled Text) — no font swap delay\n  \
                     · Faster LCP and FCP by eliminating font-critical request chains\n\n\
                     If custom branding is required, consider:\n  \
                     · Using variable fonts (one file, multiple weights/styles)\n  \
                     · Subsetting fonts to include only the characters you need\n  \
                     · Using woff2 format for best compression (~30% smaller than woff)",
                ),
                snippet,
                page_url: None,
            });
        }
    }
}

fn cache_header_audit(headers: &HeaderMap, findings: &mut Vec<Finding>) {
    let cache_control = headers
        .get("cache-control")
        .and_then(|v| v.to_str().ok());
    let etag = headers
        .get("etag")
        .and_then(|v| v.to_str().ok());
    let last_modified = headers
        .get("last-modified")
        .and_then(|v| v.to_str().ok());
    let expires = headers
        .get("expires")
        .and_then(|v| v.to_str().ok());
    let pragma = headers
        .get("pragma")
        .and_then(|v| v.to_str().ok());

    if let Some(cc) = cache_control {
        let lower = cc.to_lowercase();
        if !lower.contains("max-age") && !lower.contains("s-maxage") && !lower.contains("no-cache") && !lower.contains("no-store") {
            findings.push(Finding {
                category: "performance".to_string(),
                check: "cache_policy".to_string(),
                severity: Severity::Info,
                title: "Cache-Control header present but no caching directive".into(),
                description: format!(
                    "The Cache-Control header is \"{}\", but it does not specify max-age, \
                     s-maxage, no-cache, or no-store. Without an explicit caching directive, \
                     the browser must apply heuristic freshness — typically 10%% of the \
                     Last-Modified delta or 24 hours for 200 responses. This is unreliable \
                     and may lead to unexpected cache behaviour.\n\n\
                     Recommendations:\n  \
                     · For static assets with fingerprinting (e.g., style.a1b2c3.css):\n    \
                     Cache-Control: public, max-age=31536000, immutable\n  \
                     · For dynamic HTML that changes often:\n    \
                     Cache-Control: no-cache (forces revalidation via ETag/Last-Modified)\n  \
                     · For API responses that should never be cached:\n    \
                     Cache-Control: no-store\n\n\
                     Current value: {}",
                    cc,
                    cc,
                ),
                snippet: Some(format!("Cache-Control: {}", cc)),
                page_url: None,
            });
        }
    } else {
        let snippet = if let Some(p) = pragma {
            Some(format!("Pragma: {}", p))
        } else {
            Some("Cache-Control: <missing>".to_string())
        };

        findings.push(Finding {
            category: "performance".to_string(),
            check: "cache_policy".to_string(),
            severity: Severity::Warning,
            title: "Missing Cache-Control header".into(),
            description: format!(
                "The HTTP response does not include a Cache-Control header. \
                 Browsers and intermediary caches apply default heuristics:\n\n  \
                 · 200 OK responses: fresh for 10%% of the time since Last-Modified, \
                 or up to 24 hours if no Last-Modified is present\n  \
                 · 301/302 redirects: not cached by default\n  \
                 · 404 responses: not cached by default\n\n\
                 These heuristics vary across browsers and are unreliable. \
                 An explicit Cache-Control policy is strongly recommended.\n\n\
                 Recommendations:\n  \
                 · For the HTML document (non-fingerprinted):\n    \
                 Cache-Control: public, max-age=0, must-revalidate\n  \
                 · For API responses:\n    \
                 Cache-Control: no-cache, no-store, must-revalidate\n  \
                 · For static assets with content hashes:\n    \
                 Cache-Control: public, max-age=31536000, immutable{}",
                if let Some(p) = pragma {
                    format!("\n\nNote: A Pragma: {} header was found. Pragma is a legacy HTTP/1.0 header and is largely ignored by modern browsers in favour of Cache-Control.", p)
                } else {
                    String::new()
                },
            ),
            snippet,
            page_url: None,
        });
    }

    if etag.is_none() && last_modified.is_none() {
        let snippet = Some(
            "ETag: \"<file-hash>\"\nLast-Modified: <HTTP-date>".to_string(),
        );
        findings.push(Finding {
            category: "performance".to_string(),
            check: "cache_policy".to_string(),
            severity: Severity::Info,
            title: "No conditional request headers (ETag or Last-Modified)".into(),
            description: format!(
                "The response has neither an ETag nor a Last-Modified header. \
                 Without these, conditional requests (If-None-Match and \
                 If-Modified-Since) cannot be used. Every request results in a full \
                 200 response with the complete body, even when the resource has not \
                 changed — wasting bandwidth and increasing latency.\n\n\
                 How conditional requests work:\n  \
                 1. Browser sends request with If-None-Match: \"<etag>\" or \
                 If-Modified-Since: <date>\n  \
                 2. Server compares with the current resource\n  \
                 3. If unchanged: server returns 304 Not Modified (empty body, ~200 bytes)\n  \
                 4. If changed: server returns 200 with the new content\n\n\
                 Recommendations:\n  \
                 · Add ETag: \"<hash-of-content>\" for content-based validation\n    \
                 Example: ETag: \"33a64df551425fcc55e4d42a148795d9f25f89d4\"\n  \
                 · Add Last-Modified: <HTTP-date> for timestamp-based validation\n    \
                 Example: Last-Modified: Wed, 21 Oct 2025 07:28:00 GMT\n\n\
                 Note: If both are present, ETag takes precedence. \
                 Weak ETags (W/\"...\") allow semantically equivalent responses \
                 to be considered unchanged even if bytes differ.",
            ),
            snippet,
            page_url: None,
        });
    }

    if let Some(exp) = expires {
        let snippet = Some(format!("Expires: {}", exp));

        findings.push(Finding {
            category: "performance".to_string(),
            check: "cache_policy".to_string(),
            severity: Severity::Info,
            title: "Legacy Expires header should be replaced by Cache-Control max-age".into(),
            description: format!(
                "The response includes an Expires header (\"{}\") alongside or instead \
                 of Cache-Control max-age. Expires is an HTTP/1.0 header that specifies \
                 an absolute expiration date. Cache-Control max-age (relative seconds) \
                 is preferred because:\n\n  \
                 · Expires depends on accurate clock synchronization between server and client\n  \
                 · max-age is relative and immune to clock skew\n  \
                 · Cache-Control overrides Expires when both are present in HTTP/1.1\n\n\
                 Recommendation: Replace Expires with Cache-Control max-age:\n  \
                 Cache-Control: public, max-age=31536000\n\n\
                 Current value: {}",
                exp,
                exp,
            ),
            snippet,
            page_url: None,
        });
    }
}

fn compression_audit(headers: &HeaderMap, findings: &mut Vec<Finding>) {
    let content_encoding = headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok());
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let vary = headers
        .get("vary")
        .and_then(|v| v.to_str().ok());
    let content_length = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok());

    match content_encoding {
        Some("br") => {
            let snippet = Some("Content-Encoding: br".to_string());

            findings.push(Finding {
                category: "performance".to_string(),
                check: "compression".to_string(),
                severity: Severity::Pass,
                title: "Content compressed with Brotli (br)".into(),
                description: format!(
                    "The response is compressed with Brotli (Content-Encoding: br), \
                     the most efficient compression algorithm for web content. \
                     Brotli typically achieves 20–30%% smaller files than gzip for \
                     HTML, CSS, and JavaScript.\n\n\
                     Compression ratios by algorithm (typical):\n  \
                     · Brotli (level 5–11): 70–80%% reduction\n  \
                     · Gzip (level 6–9):   60–70%% reduction\n  \
                     · Deflate:            55–65%% reduction\n\n\
                     Current response type: {}\n\
                     Vary header{}: {}\n\n\
                     Ensure the Vary header includes Accept-Encoding so CDNs and \
                     caches properly distinguish compressed vs uncompressed variants.",
                    content_type,
                    if vary.is_some() { "" } else { " missing" },
                    vary.unwrap_or("(none)"),
                ),
                snippet,
                page_url: None,
            });
        }
        Some("gzip") => {
            let snippet = Some("Content-Encoding: gzip".to_string());

            findings.push(Finding {
                category: "performance".to_string(),
                check: "compression".to_string(),
                severity: Severity::Pass,
                title: "Content compressed with Gzip".into(),
                description: format!(
                    "The response is compressed with Gzip (Content-Encoding: gzip). \
                     Gzip is widely supported and provides good compression, but \
                     consider upgrading to Brotli which offers 20–30%% better \
                     compression ratios for text-based content.\n\n\
                     Recommendation: Configure your server to prefer Brotli when the \
                     client advertises support:\n  \
                     · Nginx: add Brotli module and set brotli on\n  \
                     · Apache: mod_brotli\n  \
                     · Cloudflare: enable Brotli in Speed → Optimization\n  \
                     · CDN: most CDNs support Brotli with automatic content negotiation\n\n\
                     Current response type: {}\n\
                     Content-Length{}: {}",
                    content_type,
                    if content_length.is_some() { "" } else { " missing" },
                    content_length.unwrap_or("(unknown)"),
                ),
                snippet,
                page_url: None,
            });
        }
        Some(other) => {
            let snippet = Some(format!("Content-Encoding: {}", other));

            findings.push(Finding {
                category: "performance".to_string(),
                check: "compression".to_string(),
                severity: Severity::Warning,
                title: "Unknown or uncommon content encoding".into(),
                description: format!(
                    "The response uses Content-Encoding: \"{}\", which is not one of \
                     the standard web compression methods (gzip, br, deflate). \
                     Most browsers support:\n\n  \
                     · br (Brotli) — best compression, supported in Chrome 49+, \
                     Firefox 44+, Safari 15.4+, Edge 15+\n  \
                     · gzip — universal support since HTTP/1.1\n  \
                     · deflate — supported but rarely used; Brotli or gzip are preferred\n\n\
                     The client's Accept-Encoding header typically looks like:\n  \
                     Accept-Encoding: gzip, deflate, br\n\n\
                     Recommendation: Ensure the server is configured to return \
                     Content-Encoding: br (preferred) or gzip for text-based resources. \
                     Uncommon encodings (e.g., compress, identity, x-gzip) may not be \
                     supported by all clients and can cause download failures.",
                    other,
                ),
                snippet,
                page_url: None,
            });
        }
        None => {
            let snippet = Some("Content-Encoding: br (recommended)".to_string());

            findings.push(Finding {
                category: "performance".to_string(),
                check: "compression".to_string(),
                severity: Severity::Error,
                title: "No content compression — response is uncompressed".into(),
                description: format!(
                    "The HTTP response body is not compressed (no Content-Encoding \
                     header). The client likely sent an Accept-Encoding header \
                     advertising support for gzip, deflate, and br, but the server \
                     returned the raw uncompressed content. This dramatically increases \
                     transfer size and load time.\n\n\
                     Estimated impact:\n  \
                     · HTML typically compresses by 70–80%% with Brotli\n  \
                     · A 100 KB HTML page → ~20–30 KB with compression\n  \
                     · On slow 3G (~50 KB/s), this saves 1.4–1.6 seconds of transfer time\n\n\
                     Recommendations by server type:\n  \
                     · Nginx: gzip on; gzip_types text/html text/css application/javascript;\n  \
                     · Apache: AddOutputFilterByType DEFLATE text/html text/css text/javascript\n  \
                     · Node/Express: use compression middleware (compression npm package)\n  \
                     · Cloudflare/AWS CloudFront: enable automatic compression in settings\n  \
                     · CDN: most CDNs compress at the edge — verify compression is enabled\n\n\
                     Current response type: {}\n\
                     Content-Length{}: {}",
                    content_type,
                    if content_length.is_some() { "" } else { " missing" },
                    content_length.unwrap_or("(unknown)"),
                ),
                snippet,
                page_url: None,
            });
        }
    }
}

fn render_blocking_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sync_css_sel = Selector::parse(
        "link[rel=stylesheet]:not([media=print]):not([disabled])",
    )
    .unwrap();
    let sync_js_sel = Selector::parse(
        "script[src]:not([defer]):not([async]):not([type=module])",
    )
    .unwrap();
    let inline_style_sel = Selector::parse("style:not([media=print])").unwrap();
    let import_style_sel = Selector::parse("style").unwrap();
    let import_link_sel = Selector::parse(
        "link[href*=\".css\"]:not([rel=preload])",
    )
    .unwrap();

    let blocking_css: Vec<ElementRef> = document.select(&sync_css_sel).collect();
    let blocking_js: Vec<ElementRef> = document.select(&sync_js_sel).collect();
    let inline_styles: Vec<ElementRef> = document.select(&inline_style_sel).collect();
    let css_imports: Vec<ElementRef> = document
        .select(&import_style_sel)
        .filter(|el| el.inner_html().contains("@import"))
        .chain(document.select(&import_link_sel))
        .collect();

    let css_count = blocking_css.len();
    let js_count = blocking_js.len();
    let inline_count = inline_styles.len();
    let import_count = css_imports.len();
    let total_blocking = css_count + js_count;

    if total_blocking == 0 {
        return;
    }

    let mut examples: Vec<String> = Vec::new();
    let max_examples = 5;

    for el in &blocking_css {
        if examples.len() >= max_examples {
            break;
        }
        if let Some(s) = el_snippet(el) {
            examples.push(s);
        }
    }
    for el in &blocking_js {
        if examples.len() >= max_examples {
            break;
        }
        if let Some(s) = el_snippet(el) {
            examples.push(s);
        }
    }

    let (severity, severity_note) = if total_blocking > 10 {
        (
            Severity::Error,
            format!(
                "{} render-blocking resources is extremely high and will severely \
                 delay First Contentful Paint (FCP).",
                total_blocking,
            ),
        )
    } else if total_blocking > 3 {
        (
            Severity::Warning,
            format!(
                "{} render-blocking resources — reducing these will improve FCP and LCP.",
                total_blocking,
            ),
        )
    } else {
        (
            Severity::Info,
            format!(
                "{} render-blocking resource(s).",
                total_blocking,
            ),
        )
    };

    let snippet = if examples.is_empty() {
        None
    } else {
        Some(examples.join("\n"))
    };

    findings.push(Finding {
        category: "performance".to_string(),
        check: "render_blocking".to_string(),
        severity,
        title: format!(
            "{} render-blocking resource(s) — {} CSS, {} JS{}",
            total_blocking,
            css_count,
            js_count,
            if import_count > 0 {
                format!(", {} CSS @import(s)", import_count)
            } else {
                String::new()
            },
        ),
        description: format!(
            "{}\n\n\
             Breakdown:\n  \
             · Synchronous CSS stylesheets: {}\n  \
             · Synchronous JS scripts:      {}\n  \
             · Inline <style> blocks:       {}\n  \
             · CSS @import statements:      {}\n\n\
             How render blocking works:\n  \
             · The browser must download and parse all blocking CSS before rendering \
             any content (CSS is considered a render-blocking resource by default)\n  \
             · Synchronous scripts block HTML parsing — the browser stops parsing \
             until the script is downloaded and executed\n  \
             · Each blocking resource adds at least one RTT (Round Trip Time) to the \
             critical rendering path\n\n\
             Recommendations:\n  \
             1. Inline critical CSS: Extract above-fold styles into a <style> tag \
             in <head> (typically 10–15 KB)\n  \
             2. Defer non-critical CSS: Add media=\"print\" onload=\"this.media='all'\" \
             to non-critical stylesheets:\n    \
             <link rel=\"stylesheet\" href=\"non-critical.css\" media=\"print\" \
             onload=\"this.media='all'\">\n  \
             3. Add defer to scripts: <script defer src=\"app.js\"> — deferred scripts \
             download in parallel but execute after HTML parsing\n  \
             4. Add async to independent scripts: <script async src=\"analytics.js\"> \
             — async scripts execute as soon as downloaded, blocking parsing briefly\n  \
             5. Remove CSS @import: @import is serial and blocks rendering — use \
             multiple <link> tags instead\n  \
             6. Use HTTP/2 Push or 103 Early Hints to deliver critical resources \
             before the browser requests them",
            severity_note,
            css_count,
            js_count,
            inline_count,
            import_count,
        ),
        snippet,
        page_url: None,
    });
}

fn web_vitals_note(findings: &mut Vec<Finding>) {
    findings.push(Finding {
        category: "performance".to_string(),
        check: "web_vitals".to_string(),
        severity: Severity::Info,
        title: "LCP, CLS, and INP require a browser-based audit".into(),
        description: format!(
            "Tengu's server-side HTML fetch cannot measure Core Web Vitals because they require \
             a real browser to render, paint, and interact with the page.\n\n\
             What each metric measures:\n  \
             · LCP (Largest Contentful Paint) — When the largest content element becomes visible. \
             Target: ≤2.5s (Good), ≤4.0s (Needs Improvement).\n  \
             · CLS (Cumulative Layout Shift) — Visual stability; measures unexpected layout shifts. \
             Target: ≤0.1 (Good), ≤0.25 (Needs Improvement).\n  \
             · INP (Interaction to Next Paint) — Responsiveness; measures the time from user \
             interaction to the next paint. Replaces FID in March 2024. \
             Target: ≤200ms (Good), ≤500ms (Needs Improvement).\n\n\
             To audit Web Vitals:\n  \
             · Use Google Chrome DevTools → Lighthouse tab\n  \
             · Use PageSpeed Insights (pagespeed.web.dev)\n  \
             · Use Chrome User Experience Report (CrUX) for real-user data\n  \
             · Use Web Vitals library (npm: web-vitals) for RUM data\n  \
             · Run `tengu --browser` for a headless Chromium audit (planned)\n\n\
             Related Tengu checks that help improve Web Vitals:\n  \
             · Page weight audit — reducing bytes improves LCP\n  \
             · Render-blocking audit — eliminating blocking resources improves LCP\n  \
             · Image audit — setting dimensions prevents CLS\n  \
             · Font audit — font-display: swap prevents invisible text\n  \
             · Cache audit — caching improves repeat-visit LCP\n\n\
             Reference: web.dev/vitals, Chrome UX Report, W3C Web Performance WG"
        ),
        snippet: None,
        page_url: None,
    });
}

fn third_party_script_audit(document: &Html, findings: &mut Vec<Finding>) {
    let script_sel = Selector::parse("script[src]").unwrap();
    let mut third_party: Vec<(String, String)> = Vec::new();
    let mut first_party: Vec<String> = Vec::new();

    let known_third_party: &[(&str, &str)] = &[
        // Analytics
        ("google-analytics.com", "Google Analytics"),
        ("googletagmanager.com", "Google Tag Manager"),
        ("googlesyndication.com", "Google AdSense"),
        ("googleadservices.com", "Google Ads"),
        ("doubleclick.net", "DoubleClick"),
        ("facebook.net", "Facebook Pixel"),
        ("facebook.com", "Facebook SDK"),
        ("connect.facebook.net", "Facebook SDK"),
        ("twitter.com", "Twitter/X Widget"),
        ("twimg.com", "Twitter/X Assets"),
        ("linkedin.com", "LinkedIn"),
        ("hotjar.com", "Hotjar"),
        ("mouseflow.com", "Mouseflow"),
        ("fullstory.com", "FullStory"),
        ("clarity.ms", "Microsoft Clarity"),
        ("crazyegg.com", "CrazyEgg"),
        ("mixpanel.com", "Mixpanel"),
        ("amplitude.com", "Amplitude"),
        ("segment.com", "Segment"),
        ("segment.io", "Segment"),
        ("heap.io", "Heap"),
        ("optimizely.com", "Optimizely"),
        ("vwo.com", "VWO"),
        ("hubspot.com", "HubSpot"),
        ("hsforms.net", "HubSpot Forms"),
        ("salesforce.com", "Salesforce"),
        ("pardot.com", "Pardot"),
        ("marketo.com", "Marketo"),
        ("munchkin.marketo.com", "Marketo Munchkin"),
        ("adobe.com", "Adobe"),
        ("demdex.net", "Adobe Audience Manager"),
        ("everesttech.net", "Adobe"),
        ("krxd.net", "Adobe"),
        ("2o7.net", "Adobe Analytics"),
        ("omniture.com", "Adobe Analytics"),
        ("sc.omtrdc.net", "Adobe Analytics"),

        // Tag managers
        ("tagmanager.google.com", "Google Tag Manager"),
        ("cdn.optimizely.com", "Optimizely"),
        ("cdn.segment.com", "Segment"),
        ("cdn.tealium.com", "Tealium"),
        ("tealium.com", "Tealium"),
        ("tiqcdn.com", "Tealium"),
        ("ensighten.com", "Ensighten"),
        ("cdn.ensighten.com", "Ensighten"),

        // Fonts & CDNs
        ("fonts.googleapis.com", "Google Fonts"),
        ("fonts.gstatic.com", "Google Fonts (static)"),
        ("use.typekit.net", "Adobe Typekit"),
        ("use.fontawesome.com", "Font Awesome"),
        ("kit.fontawesome.com", "Font Awesome"),
        ("cdn.jsdelivr.net", "jsDelivr CDN"),
        ("cdnjs.cloudflare.com", "Cloudflare CDNjs"),
        ("unpkg.com", "unpkg CDN"),
        ("cdn.jsdelivr.net", "jsDelivr"),
        ("stackpath.bootstrapcdn.com", "Bootstrap CDN"),
        ("maxcdn.bootstrapcdn.com", "Bootstrap CDN"),
        ("code.jquery.com", "jQuery CDN"),
        ("ajax.googleapis.com", "Google AJAX Libraries"),
        ("ajax.aspnetcdn.com", "Microsoft Ajax CDN"),
        ("cdn.socket.io", "Socket.io CDN"),

        // Recaptcha & anti-bot
        ("google.com/recaptcha", "reCAPTCHA"),
        ("recaptcha.net", "reCAPTCHA"),
        ("hcaptcha.com", "hCaptcha"),
        ("js.hcaptcha.com", "hCaptcha"),
        ("api.turn.com", "Turnstile"),
        ("challenges.cloudflare.com", "Cloudflare Turnstile"),

        // Payments
        ("stripe.com", "Stripe"),
        ("js.stripe.com", "Stripe.js"),
        ("checkout.stripe.com", "Stripe Checkout"),
        ("paypal.com", "PayPal"),
        ("paypalobjects.com", "PayPal Assets"),
        ("square.com", "Square"),
        ("js.squareup.com", "Square SDK"),
        ("braintreegateway.com", "Braintree"),
        ("js.braintreegateway.com", "Braintree JS"),

        // Maps
        ("maps.googleapis.com", "Google Maps"),
        ("maps.google.com", "Google Maps"),
        ("maps.gstatic.com", "Google Maps (static)"),
        ("mapbox.com", "Mapbox"),
        ("api.mapbox.com", "Mapbox API"),
        ("openstreetmap.org", "OpenStreetMap"),

        // Social
        ("platform.twitter.com", "Twitter Platform"),
        ("platform.linkedin.com", "LinkedIn Platform"),
        ("platform.instagram.com", "Instagram Platform"),
        ("platform.youtube.com", "YouTube Platform"),
        ("www.youtube.com", "YouTube"),
        ("www.youtube-nocookie.com", "YouTube (privacy)"),
        ("player.vimeo.com", "Vimeo"),
        ("i.ytimg.com", "YouTube Thumbnails"),

        // Chat & support
        ("intercom.io", "Intercom"),
        ("widget.intercom.io", "Intercom Widget"),
        ("app.intercom.io", "Intercom App"),
        ("crisp.chat", "Crisp Chat"),
        ("app.crisp.chat", "Crisp Chat"),
        ("zendesk.com", "Zendesk"),
        ("eucrm.zendesk.com", "Zendesk"),
        ("zen desk.com", "Zendesk"),
        ("tawk.to", "Tawk.to"),
        ("embed.tawk.to", "Tawk.to"),
        ("livechat.com", "LiveChat"),
        ("livechatinc.com", "LiveChat"),
        ("drift.com", "Drift"),
        ("js.driftt.com", "Drift"),
        ("snapengage.com", "SnapEngage"),
        ("olark.com", "Olark"),
        ("static.olark.com", "Olark"),
        ("freshdesk.com", "Freshdesk"),
        ("freshworks.com", "Freshworks"),

        // A/B testing
        ("app.launchdarkly.com", "LaunchDarkly"),
        ("client.launchdarkly.com", "LaunchDarkly Client"),
        ("cdn.launchdarkly.com", "LaunchDarkly CDN"),
        ("split.io", "Split.io"),
        ("cdn.split.io", "Split.io"),

        // Video
        ("cdn.embedly.com", "Embedly"),
        ("cdn.video", "Video CDN"),
        ("vimeo.com", "Vimeo"),
        ("player.vimeo.com", "Vimeo Player"),
        ("wistia.com", "Wistia"),
        ("fast.wistia.com", "Wistia"),
        ("cdn.wistia.com", "Wistia"),

        // Security
        ("cdn.sitespect.com", "SiteSpect"),
        ("cdn.sucuri.net", "Sucuri"),
        ("cdn.akamai.net", "Akamai"),
        ("cloudflare.com", "Cloudflare"),
        ("cdn.cloudflare.com", "Cloudflare CDN"),
        ("cdnjs.cloudflare.com", "Cloudflare CDNjs"),

        // Other common
        ("gstatic.com", "Google Static"),
        ("www.gstatic.com", "Google Static"),
        ("ytimg.com", "YouTube Assets"),
        ("googlevideo.com", "YouTube Video"),
        ("googleusercontent.com", "Google User Content"),
        ("ggpht.com", "Google Photos"),
        ("gravatar.com", "Gravatar"),
        ("s3.amazonaws.com", "AWS S3"),
        ("s3.us-east-1.amazonaws.com", "AWS S3"),
        ("cloudfront.net", "AWS CloudFront"),
        ("netdna-ssl.com", "StackPath CDN"),
        ("netdna.bootstrapcdn.com", "Bootstrap CDN"),
        ("bootstrapcdn.com", "Bootstrap CDN"),
    ];

    for el in document.select(&script_sel) {
        if let Some(src) = el.value().attr("src") {
            let lower = src.to_lowercase();
            let mut matched = false;
            for &(domain, name) in known_third_party {
                if lower.contains(domain) {
                    third_party.push((name.to_string(), truncate(src, 80)));
                    matched = true;
                    break;
                }
            }
            if !matched {
                first_party.push(truncate(src, 80));
            }
        }
    }

    if third_party.is_empty() {
        return;
    }

    let impact = match third_party.len() {
        0..=3 => "low",
        4..=8 => "medium",
        _ => "high",
    };

    let severity = match impact {
        "high" => Severity::Warning,
        "medium" => Severity::Info,
        _ => Severity::Info,
    };

    let grouped: Vec<String> = {
        let mut seen: Vec<&str> = Vec::new();
        let mut lines = Vec::new();
        for (name, src) in &third_party {
            if !seen.contains(&name.as_str()) {
                seen.push(name.as_str());
                let count = third_party.iter().filter(|(n, _)| n == name).count();
                lines.push(format!("  · {} ({} script{})", name, count, if count == 1 { "" } else { "s" }));
            }
        }
        lines.truncate(10);
        lines
    };

    findings.push(Finding {
        category: "performance".to_string(),
        check: "third_party_scripts".to_string(),
        severity,
        title: format!("{} third-party script(s) detected — impact: {}", third_party.len(), impact),
        description: format!(
            "The page loads {} third-party script(s) from known external services. Each \
             third-party script adds DNS resolution, TCP/TLS handshake, download time, and \
             potentially render-blocking execution. Impact score: {} ({} total third-party \
             scripts).\n\n\
             Third-party services detected:\n{}\n\n\
             Performance impact of third-party scripts:\n  \
             · Each script = ~50-300ms additional load time\n  \
             · Render-blocking third-party scripts delay LCP by 1-3 seconds\n  \
             · Third-party scripts often set additional cookies and make further requests\n  \
             · Script failure (CDN down) can break site functionality\n\n\
             Recommendations to reduce third-party impact:\n  \
             · Load scripts with async or defer when possible\n  \
             · Self-host critical third-party scripts (fonts, analytics)\n  \
             · Use resource hints: <link rel=\"preconnect\" href=\"...\"> for critical origins\n  \
             · Audit regularly — remove unused or duplicate third-party services\n  \
             · Consider using a single tag manager to consolidate multiple scripts\n  \
             · Use Subresource Integrity (SRI) for all third-party scripts\n\n\
             Reference: Web Almanac Third-Party Chapter, CSS Wizardry — Managing Third-Party \
             Scripts, Request Map",
            third_party.len(),
            impact,
            third_party.len(),
            grouped.join("\n"),
        ),
        snippet: None,
        page_url: None,
    });
}
