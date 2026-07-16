use reqwest::header::HeaderMap;
use scraper::{ElementRef, Html, Selector};

use crate::auditor::{Finding, Severity};

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

pub async fn analyze(html: &str, headers: &HeaderMap, url: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let document = Html::parse_document(html);

    https_audit(url, &mut findings);
    security_headers_audit(headers, &mut findings);
    csp_audit(headers, &mut findings);
    permissions_policy_audit(headers, &mut findings);
    cookie_audit(headers, &mut findings);
    doctype_audit(html, &mut findings);
    deprecated_html_audit(&document, &mut findings);
    mixed_content_audit(&document, &mut findings);
    sri_audit(&document, &mut findings);
    gdpr_cookie_consent_audit(&document, &mut findings);
    console_error_note(&mut findings);

    findings
}

fn https_audit(url: &str, findings: &mut Vec<Finding>) {
    if url.starts_with("http://") {
        findings.push(Finding {
            category: "best_practices".to_string(),
            check: "https".to_string(),
            severity: Severity::Error,
            title: "Page served over insecure HTTP".into(),
            description: format!(
                "The page is served over unencrypted HTTP at \"{}\". \
                 This is a critical security and SEO issue:\n\n\
                 Security: All data (including passwords, cookies, API responses) is transmitted \
                 in plaintext and can be intercepted by attackers on the same network (MITM). \
                 Modern browsers mark HTTP pages as \"Not Secure\" in the address bar, eroding \
                 user trust.\n\n\
                 SEO: Google uses HTTPS as a ranking signal and Chrome shows \"Not Secure\" \
                 warnings on HTTP pages, negatively impacting both ranking and click-through rates.\n\n\
                 Recommendation: Redirect all HTTP traffic to HTTPS via a 301 redirect. Obtain \
                 a free TLS certificate from Let's Encrypt or use your hosting provider's \
                 certificate service. Enforce HSTS (Strict-Transport-Security) once HTTPS is live.",
                url
            ),
            snippet: Some(url.to_string()),
            page_url: None,
        });
    }
}

fn security_headers_audit(headers: &HeaderMap, findings: &mut Vec<Finding>) {
    let checks: [(&str, Severity, &str, &str); 6] = [
        (
            "Strict-Transport-Security",
            Severity::Warning,
            "Strict-Transport-Security: max-age=63072000; includeSubDomains; preload",
            "HTTP Strict-Transport-Security (HSTS) instructs browsers to always connect via \
             HTTPS, even when the user types http:// or follows an HTTP link. It prevents \
             SSL-stripping MITM attacks.\n\n\
             Recommended value:\n  Strict-Transport-Security: max-age=63072000; includeSubDomains; preload\n\n\
             • max-age=63072000 (2 years) — how long the browser remembers to use HTTPS\n\
             • includeSubDomains — applies to all subdomains\n\
             • preload — allows inclusion in browser preload lists for first-visit protection",
        ),
        (
            "Content-Security-Policy",
            Severity::Warning,
            "Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'",
            "Content-Security-Policy (CSP) is a powerful defence-in-depth header that controls \
             which resources the browser is allowed to load, mitigating XSS, clickjacking, and \
             data injection attacks.\n\n\
             Example value:\n  Content-Security-Policy: default-src 'self'; script-src 'self'; \
             style-src 'self' 'unsafe-inline'\n\n\
             • default-src 'self' — only load resources from the same origin\n\
             • script-src — restricts JavaScript sources\n\
             • style-src — restricts CSS sources\n\n\
             Start with a restrictive policy and relax as needed using report-only mode first.",
        ),
        (
            "X-Frame-Options",
            Severity::Warning,
            "X-Frame-Options: DENY",
            "X-Frame-Options prevents clickjacking by controlling whether the page can be \
             embedded in <frame>, <iframe>, or <object> elements.\n\n\
             Recommended values:\n  • X-Frame-Options: DENY — blocks all embedding (strictest)\n  \
             • X-Frame-Options: SAMEORIGIN — allows embedding on the same origin\n\n\
             Note: CSP's frame-ancestors directive is a more modern and flexible replacement, \
             but X-Frame-Options is still widely supported and should be included for older \
             browsers.",
        ),
        (
            "X-Content-Type-Options",
            Severity::Warning,
            "X-Content-Type-Options: nosniff",
            "X-Content-Type-Options: nosniff tells the browser to trust the Content-Type header \
             and not perform MIME-type sniffing. Without it, browsers may interpret a \
             user-uploaded .jpg as text/html, enabling XSS attacks.\n\n\
             Required value:\n  X-Content-Type-Options: nosniff\n\n\
             This header has no other valid values — it either exists as \"nosniff\" or does not.",
        ),
        (
            "Referrer-Policy",
            Severity::Info,
            "Referrer-Policy: strict-origin-when-cross-origin",
            "Referrer-Policy controls how much referrer information (the Referer header) is sent \
             when navigating to other origins. Leaking the full URL in the Referer header can \
             expose sensitive data like session IDs in query parameters.\n\n\
             Recommended value:\n  Referrer-Policy: strict-origin-when-cross-origin\n\n\
             • same-origin requests send the full URL\n\
             • cross-origin requests send only the origin (no path/query)\n\
             • HTTPS→HTTP downgrades send no referrer at all (strict)",
        ),
        (
            "Permissions-Policy",
            Severity::Info,
            "Permissions-Policy: camera=(), microphone=(), geolocation=()",
            "Permissions-Policy (formerly Feature-Policy) restricts which browser APIs and \
             features the page and its embedded iframes can use. This prevents third-party \
             scripts from abusing device access (camera, microphone, geolocation, etc.).\n\n\
             Example value:\n  Permissions-Policy: camera=(), microphone=(), geolocation=()\n\n\
             • empty parentheses = fully blocked\n\
             • self = allowed on same origin\n\
             • specific origins = allowed on those origins only",
        ),
    ];

    for (name, severity, example_value, description) in &checks {
        if headers.get(*name).is_none() {
            findings.push(Finding {
                category: "best_practices".to_string(),
                check: "security_headers".to_string(),
                severity: *severity,
                title: format!("Missing {} header", name),
                description: description.to_string(),
                snippet: Some(example_value.to_string()),
                page_url: None,
            });
        }
    }
}

fn cookie_audit(headers: &HeaderMap, findings: &mut Vec<Finding>) {
    let cookies = headers.get_all("set-cookie");
    for cookie in cookies {
        if let Ok(val) = cookie.to_str() {
            let lower = val.to_lowercase();
            let cookie_name = val.split(';').next().unwrap_or("").to_string();

            if !lower.contains("secure") {
                findings.push(Finding {
                    category: "best_practices".to_string(),
                    check: "cookies".to_string(),
                    severity: Severity::Warning,
                    title: format!("Cookie \"{}\" missing Secure flag", cookie_name),
                    description: format!(
                        "The cookie \"{}\" is missing the Secure flag. Cookies without Secure \
                         are sent over both HTTP and HTTPS connections, exposing session tokens \
                         and other sensitive data to network interception on unencrypted \
                         connections.\n\n\
                         The Secure flag instructs the browser to only send the cookie over TLS \
                         (HTTPS) connections, never over plain HTTP.\n\n\
                         Recommendation: Append ; Secure to the Set-Cookie header:\n  \
                         Set-Cookie: {}; Secure",
                        cookie_name, val
                    ),
                    snippet: Some(truncate(val, 300)),
                    page_url: None,
                });
            }

            if !lower.contains("httponly") {
                findings.push(Finding {
                    category: "best_practices".to_string(),
                    check: "cookies".to_string(),
                    severity: Severity::Warning,
                    title: format!("Cookie \"{}\" missing HttpOnly flag", cookie_name),
                    description: format!(
                        "The cookie \"{}\" is missing the HttpOnly flag. Cookies without \
                         HttpOnly can be read and modified by client-side JavaScript via \
                         document.cookie, making them accessible to XSS attacks — even a single \
                         XSS vulnerability can exfiltrate all session cookies.\n\n\
                         The HttpOnly flag prevents JavaScript from accessing the cookie; it is \
                         still sent automatically with HTTP requests.\n\n\
                         Recommendation: Append ; HttpOnly to the Set-Cookie header:\n  \
                         Set-Cookie: {}; HttpOnly",
                        cookie_name, val
                    ),
                    snippet: Some(truncate(val, 300)),
                    page_url: None,
                });
            }

            if !lower.contains("samesite") {
                findings.push(Finding {
                    category: "best_practices".to_string(),
                    check: "cookies".to_string(),
                    severity: Severity::Info,
                    title: format!("Cookie \"{}\" missing SameSite attribute", cookie_name),
                    description: format!(
                        "The cookie \"{}\" has no SameSite attribute. SameSite controls whether \
                         the cookie is sent in cross-site requests, providing protection against \
                         CSRF (Cross-Site Request Forgery) attacks.\n\n\
                         • SameSite=Lax — (default in modern browsers) cookie is sent for \
                         top-level navigations (clicking a link) but not for subrequests \
                         (images, iframes, fetch/XHR from another site)\n\
                         • SameSite=Strict — cookie is never sent in cross-site requests at all \
                         (best security, but may break legitimate link-based flows)\n\
                         • SameSite=None — cookie is sent in all contexts; requires Secure flag \
                         (HTTPS only)\n\n\
                         Recommendation: Append ; SameSite=Lax or ; SameSite=Strict (for \
                         session cookies) to the Set-Cookie header:\n  \
                         Set-Cookie: {}; SameSite=Lax",
                        cookie_name, val
                    ),
                    snippet: Some(truncate(val, 300)),
                    page_url: None,
                });
            }
        }
    }
}

fn doctype_audit(html: &str, findings: &mut Vec<Finding>) {
    let trimmed = html.trim_start();
    if !trimmed.starts_with("<!DOCTYPE html") && !trimmed.starts_with("<!doctype html") {
        let snippet = Some(truncate(trimmed, 100));
        findings.push(Finding {
            category: "best_practices".to_string(),
            check: "doctype".to_string(),
            severity: Severity::Warning,
            title: "Missing or incorrect doctype declaration".into(),
            description: format!(
                "The page does not start with <!DOCTYPE html>. Without this declaration, \
                 browsers render the page in \"quirks mode\" — emulating old IE5-era box models, \
                 incorrect CSS layout, and non-standard behaviour. This leads to inconsistent \
                 rendering across browsers and layout bugs that are difficult to debug.\n\n\
                 Standards mode (with <!DOCTYPE html>) triggers modern browser rendering with \
                 consistent CSS box model, standard event handling, and proper element semantics.\n\n\
                 Recommendation: Add the HTML5 doctype as the very first line of the document:\n  \
                 <!DOCTYPE html>\n\n\
                 Note: The doctype is case-insensitive but <!DOCTYPE html> (uppercase DOCTYPE, \
                 lowercase html) is the conventional form."
            ),
            snippet,
            page_url: None,
        });
    }
}

fn deprecated_html_audit(document: &Html, findings: &mut Vec<Finding>) {
    let deprecated = [
        ("center", "Use CSS Flexbox or Grid for centering (e.g., display: flex; justify-content: center; align-items: center)"),
        ("font", "Use CSS font properties: font-family, font-size, color, font-weight"),
        ("marquee", "Use CSS animations (keyframes + animation property) or JavaScript for scrolling text effects"),
        ("blink", "Use CSS animation with visibility or opacity keyframes; be mindful of accessibility — WCAG 2.2 requires user control of moving content"),
        ("strike", "Use CSS text-decoration: line-through or the semantic <del> element for deleted content"),
        ("tt", "Use CSS font-family: monospace, or the semantic <code>, <kbd>, <samp>, or <var> elements depending on meaning"),
        ("big", "Use CSS font-size with relative units (em, rem) for scalable text sizing"),
        ("frame", "Use <iframe> for embedding content, or consider modern SPA component architecture"),
        ("frameset", "Use <iframe> for embedding content, or a server-side template approach — framesets are unsupported in HTML5"),
        ("noframes", "No longer needed since frameset is obsolete; just serve a no-JavaScript fallback if required"),
        ("applet", "Use <canvas>, WebGL, or modern JavaScript libraries instead of Java applets (Java plugin removed from all major browsers)"),
        ("basefont", "Use CSS on the <body> or :root selector to set base font styles"),
    ];

    for (tag, replacement) in &deprecated {
        if let Ok(sel) = Selector::parse(tag) {
            let matches: Vec<ElementRef> = document.select(&sel).collect();
            let count = matches.len();
            if count > 0 {
                let snippet = matches.first().and_then(el_snippet);
                findings.push(Finding {
                    category: "best_practices".to_string(),
                    check: "deprecated_html".to_string(),
                    severity: Severity::Warning,
                    title: format!("Deprecated <{}> element found ({})", tag, count),
                    description: format!(
                        "Found {} <{}> element(s) on the page. The <{}> element is deprecated \
                         in HTML5 and may not be supported in future browser versions. \
                         Validation tools (W3C HTML Validator) will flag this as an error, \
                         and browser support may eventually be dropped.\n\n\
                         Recommendation: {}\n\n\
                         Replace all <{}> instances with the recommended CSS or semantic HTML \
                         equivalent to ensure forward compatibility and standards compliance.",
                        count, tag, tag, replacement, tag
                    ),
                    snippet,
                    page_url: None,
                });
            }
        }
    }
}

fn mixed_content_audit(document: &Html, findings: &mut Vec<Finding>) {
    let checks: &[(&str, &str)] = &[
        ("script[src]", "src"),
        ("link[href]", "href"),
        ("img[src]", "src"),
        ("iframe[src]", "src"),
    ];

    let mut total = 0u32;
    let mut examples: Vec<String> = Vec::new();
    let max_examples = 5;

    for &(selector, attr) in checks {
        let sel = Selector::parse(selector).unwrap();
        for el in document.select(&sel) {
            if let Some(val) = el.value().attr(attr) {
                if val.starts_with("http://") {
                    total += 1;
                    if examples.len() < max_examples {
                        if let Some(snippet) = el_snippet(&el) {
                            examples.push(snippet);
                        }
                    }
                }
            }
        }
    }

    if total > 0 {
        let mut details = Vec::new();
        let checks_detail: &[(&str, &str, &str)] = &[
            ("script[src]", "src", "script"),
            ("link[href]", "href", "link"),
            ("img[src]", "src", "img"),
            ("iframe[src]", "src", "iframe"),
        ];
        for &(selector, attr, tag) in checks_detail {
            let sel = Selector::parse(selector).unwrap();
            for el in document.select(&sel) {
                if let Some(val) = el.value().attr(attr) {
                    if val.starts_with("http://") {
                        details.push(format!("  · <{} {}>", tag, val));
                        if details.len() >= max_examples {
                            break;
                        }
                    }
                }
            }
            if details.len() >= max_examples {
                break;
            }
        }

        let snippet = examples.first().cloned();

        findings.push(Finding {
            category: "best_practices".to_string(),
            check: "mixed_content".to_string(),
            severity: Severity::Error,
            title: format!("Mixed content detected — {} HTTP resource(s)", total),
            description: format!(
                "The page loads {} resource(s) over insecure HTTP on what appears to be an \
                 HTTPS page. Modern browsers block active mixed content (scripts, iframes) \
                 entirely and display a mixed content warning for passive content (images, \
                 audio). Blocked resources can break page functionality, while loaded HTTP \
                 resources compromise the page's security guarantee.\n\n\
                 Example HTTP resource URLs found:\n{}\n\n\
                 Recommendation: Replace all http:// URLs with https:// equivalents. If the \
                 resource does not support HTTPS, consider:\n  \
                 • Hosting a local copy over HTTPS\n  \
                 • Using a CDN or service that provides HTTPS\n  \
                 • Using protocol-relative URLs (//example.com/file.js) as a temporary measure\n\n\
                 Use your browser's DevTools → Network tab to identify all mixed content \
                 resources on the page.",
                total,
                details.join("\n"),
            ),
            snippet,
            page_url: None,
        });
    }
}

fn sri_audit(document: &Html, findings: &mut Vec<Finding>) {
    let ext_scripts = Selector::parse("script[src]:not([integrity])").unwrap();
    let ext_styles = Selector::parse("link[rel=stylesheet][href]:not([integrity])").unwrap();

    let script_count = document.select(&ext_scripts).count();
    let style_count = document.select(&ext_styles).count();
    let total = script_count + style_count;

    if total > 0 {
        let mut examples: Vec<String> = Vec::new();
        let max_examples = 3;

        for el in document.select(&ext_scripts) {
            if examples.len() >= max_examples {
                break;
            }
            if let Some(snippet) = el_snippet(&el) {
                examples.push(snippet);
            }
        }

        for el in document.select(&ext_styles) {
            if examples.len() >= max_examples {
                break;
            }
            if let Some(snippet) = el_snippet(&el) {
                examples.push(snippet);
            }
        }

        let examples_text = if examples.is_empty() {
            String::new()
        } else {
            let listed: Vec<String> = examples
                .iter()
                .map(|s| format!("  · {}", s))
                .collect();
            format!("\n\nResources missing integrity:\n{}", listed.join("\n"))
        };

        let snippet = examples.first().cloned();

        findings.push(Finding {
            category: "best_practices".to_string(),
            check: "sri".to_string(),
            severity: Severity::Info,
            title: format!("{} external resource(s) missing Subresource Integrity", total),
            description: format!(
                "Found {} external resource(s) ({} script(s), {} stylesheet(s)) loaded without \
                 integrity attributes. Subresource Integrity (SRI) allows the browser to verify \
                 that fetched resources have not been unexpectedly modified — for example, if a \
                 CDN is compromised, SRI prevents the tampered file from executing.\n\n\
                 How SRI works:\n  \
                 1. Generate a base64-encoded hash of the file:\n    \
                 openssl dgst -sha384 -binary file.js | openssl base64 -A\n  \
                 2. Add the hash to the integrity attribute:\n    \
                 integrity=\"sha384-<hash>\"\n  \
                 3. Add crossorigin=\"anonymous\" for CORS-enabled resources\n\n\
                 Example with hash applied:\n  \
                 <script src=\"https://cdn.example.com/lib.js\"\n  \
                   integrity=\"sha384-ABC123...\"\n  \
                   crossorigin=\"anonymous\"></script>\n\n\
                 Recommendation: Generate and add integrity hashes for all external script and \
                 stylesheet resources. Use the SRI Hash Generator \
                 (https://www.srihash.org/) or the openssl command shown above.{examples}",
                total,
                script_count,
                style_count,
                examples = examples_text,
            ),
            snippet,
            page_url: None,
        });
    }
}

fn csp_audit(headers: &HeaderMap, findings: &mut Vec<Finding>) {
    let csp = match headers.get("content-security-policy") {
        Some(v) => v.to_str().unwrap_or("").to_string(),
        None => return,
    };

    if csp.is_empty() {
        return;
    }

    let mut issues: Vec<String> = Vec::new();
    let mut directives: Vec<&str> = Vec::new();
    let lower = csp.to_lowercase();

    for part in csp.split(';') {
        let part = part.trim();
        if let Some(semi) = part.find(' ') {
            directives.push(&part[..semi]);
        } else if !part.is_empty() {
            directives.push(part);
        }
    }

    let has_default_src = directives.iter().any(|d| *d == "default-src");
    let has_script_src = directives.iter().any(|d| *d == "script-src");
    let has_style_src = directives.iter().any(|d| *d == "style-src");
    let has_img_src = directives.iter().any(|d| *d == "img-src");
    let has_connect_src = directives.iter().any(|d| *d == "connect-src");
    let has_frame_ancestors = directives.iter().any(|d| *d == "frame-ancestors");
    let has_report_uri = directives.iter().any(|d| *d == "report-uri" || *d == "report-to");
    let has_base_uri = directives.iter().any(|d| *d == "base-uri");
    let has_form_action = directives.iter().any(|d| *d == "form-action");

    if !has_default_src && !has_script_src && !has_style_src {
        issues.push("CSP is present but has no default-src, script-src, or style-src — it may be too permissive or use only uncommon directives".to_string());
    }

    if !has_default_src && (has_script_src || has_style_src) {
        issues.push("Missing default-src directive — fallback is 'none', which may unexpectedly block resources not explicitly listed".to_string());
    }

    if !has_frame_ancestors {
        issues.push("Missing frame-ancestors directive — page can be embedded in any third-party site (clickjacking risk)".to_string());
    }

    if !has_base_uri {
        issues.push("Missing base-uri directive — attackers can inject <base> tags to hijack relative URLs".to_string());
    }

    if !has_form_action {
        issues.push("Missing form-action directive — forms can submit to any destination, enabling phishing via injected forms".to_string());
    }

    if !has_report_uri {
        issues.push("Missing report-uri/report-to directive — CSP violations will not be reported back to the server".to_string());
    }

    if lower.contains("'unsafe-inline'") {
        issues.push("Uses 'unsafe-inline' in script-src or style-src — weakens XSS protection significantly".to_string());
    }

    if lower.contains("'unsafe-eval'") {
        issues.push("Uses 'unsafe-eval' — allows eval(), setTimeout(string), and similar dangerous patterns".to_string());
    }

    if lower.contains("https://") && !lower.contains("https://*.google-analytics.com") && !lower.contains("'self'") {
        issues.push("Uses https:// whitelisting — prefer 'self' and specific origins over broad https:// allowlists".to_string());
    }

    if !lower.contains("http://") && !lower.contains("https://") && !lower.contains("'self'") && !lower.contains("'none'") {
        issues.push("Directives may be overly restrictive — consider using 'self' for same-origin resources".to_string());
    }

    if issues.is_empty() {
        findings.push(Finding {
            category: "best_practices".to_string(),
            check: "csp".to_string(),
            severity: Severity::Pass,
            title: "Content-Security-Policy is well-configured".into(),
            description: format!(
                "The Content-Security-Policy header is present and no obvious security issues were \
                 found in its configuration.\n\nDirectives detected: {}\n\n\
                 Recommendation: Periodically review the CSP against your site's actual resource \
                 loading patterns. Use report-uri/report-to to collect violation reports.",
                directives.join(", ")
            ),
            snippet: Some(truncate(&csp, 300)),
            page_url: None,
        });
        return;
    }

    let detail: Vec<String> = issues.iter().enumerate().map(|(i, issue)| {
        format!("  {}. {}", i + 1, issue)
    }).collect();

    findings.push(Finding {
        category: "best_practices".to_string(),
        check: "csp".to_string(),
        severity: Severity::Warning,
        title: "Content-Security-Policy has configuration issues".into(),
        description: format!(
            "The Content-Security-Policy header is present but has {} issue(s) that may weaken \
             its effectiveness.\n\nCurrent CSP:\n  {}\n\nIssues found:\n{}\n\n\
             Recommendation:\n  · Start with a strict policy and relax as needed\n  \
             · Use report-uri / report-to to monitor violations before enforcing\n  \
             · Avoid 'unsafe-inline' and 'unsafe-eval' where possible\n  \
             · Always include frame-ancestors, base-uri, and form-action directives\n  \
             · Test your CSP with report-only mode (Content-Security-Policy-Report-Only) first\n\n\
             Reference: W3C CSP 3.0, OWASP CSP Cheat Sheet, MDN CSP Guide",
            issues.len(),
            truncate(&csp, 200),
            detail.join("\n"),
        ),
        snippet: Some(truncate(&csp, 300)),
        page_url: None,
    });
}

fn permissions_policy_audit(headers: &HeaderMap, findings: &mut Vec<Finding>) {
    let pp = match headers.get("permissions-policy") {
        Some(v) => v.to_str().unwrap_or("").to_string(),
        None => return,
    };

    if pp.is_empty() {
        return;
    }

    let mut allowed_features: Vec<String> = Vec::new();
    let mut blocked_features: Vec<String> = Vec::new();
    let mut unlisted_features: Vec<String> = Vec::new();

    for part in pp.split(',') {
        let part = part.trim();
        if let Some(eq) = part.find('=') {
            let feature = part[..eq].trim();
            let value = part[eq + 1..].trim();
            let feature_clean = feature.to_string();
            if value == "()" || value == "none" {
                blocked_features.push(feature_clean);
            } else {
                allowed_features.push(format!("{} ({})", feature_clean, value));
            }
        }
    }

    let sensitive = [
        "camera", "microphone", "geolocation", "gyroscope", "accelerometer",
        "magnetometer", "usb", "serial", "bluetooth", "nfc", "midi",
        "ambient-light-sensor", "display-capture", "fullscreen",
        "payment", "picture-in-picture", "screen-wake-lock",
    ];

    for feature in &sensitive {
        let lower_pp = pp.to_lowercase();
        if !lower_pp.contains(feature) {
            unlisted_features.push(feature.to_string());
        }
    }

    let mut notes = Vec::new();

    if !allowed_features.is_empty() {
        notes.push(format!("Allowed features: {}", allowed_features.join(", ")));
    }
    if !blocked_features.is_empty() {
        notes.push(format!("Blocked features: {}", blocked_features.join(", ")));
    }
    if !unlisted_features.is_empty() && unlisted_features.len() > 3 {
        notes.push(format!(
            "{} sensitive feature(s) not explicitly listed (allowed by default in top-level pages, blocked in cross-origin iframes)",
            unlisted_features.len()
        ));
    }

    if notes.is_empty() {
        return;
    }

    findings.push(Finding {
        category: "best_practices".to_string(),
        check: "permissions_policy".to_string(),
        severity: Severity::Info,
        title: "Permissions-Policy header analysis".into(),
        description: format!(
            "The Permissions-Policy header is present.\n\n{}\n\n\
             Best practices:\n  \
             · Explicitly block all sensitive features: camera=(), microphone=(), geolocation=()\n  \
             · Only grant access to features your application actually needs\n  \
             · Use 'self' to restrict features to same-origin content only\n  \
             · For cross-origin iframe delegation, specify allowed origins explicitly\n\n\
             Reference: W3C Permissions-Policy, OWASP Feature Policy, MDN Permissions-Policy Guide",
            notes.join("\n")
        ),
        snippet: Some(truncate(&pp, 300)),
        page_url: None,
    });
}

fn gdpr_cookie_consent_audit(document: &Html, findings: &mut Vec<Finding>) {
    let known_cmps: &[(&str, &str, &str)] = &[
        ("cookiebot.com", "Cookiebot", "https://www.cookiebot.com"),
        ("onetrust.com", "OneTrust", "https://www.onetrust.com"),
        ("cdn.cookielaw.org", "OneTrust", "https://www.onetrust.com"),
        ("quantcast.mgr.consensu.org", "Quantcast Choice", "https://quantcast.com"),
        ("osano.com", "Osano", "https://www.osano.com"),
        ("cookieyes.com", "CookieYes", "https://www.cookieyes.com"),
        ("usercentrics.eu", "Usercentrics", "https://usercentrics.com"),
        ("cdn.usercentrics.eu", "Usercentrics", "https://usercentrics.com"),
        ("didomi.io", "Didomi", "https://www.didomi.io"),
        ("sdk.didomi.io", "Didomi", "https://www.didomi.io"),
        ("complianz.io", "Complianz", "https://complianz.io"),
        ("borlabs.io", "Borlabs", "https://borlabs.io"),
        ("iubenda.com", "iubenda", "https://www.iubenda.com"),
        ("cdn.iubenda.com", "iubenda", "https://www.iubenda.com"),
        ("cookieinformation.com", "Cookie Information", "https://cookieinformation.com"),
        ("cookiescript.com", "CookieScript", "https://cookiescript.com"),
        ("termly.io", "Termly", "https://termly.io"),
        ("consentmanager.net", "ConsentManager", "https://www.consentmanager.net"),
        ("cookiefirst.com", "CookieFirst", "https://cookiefirst.com"),
        ("cookiepro.com", "CookiePro", "https://www.cookiepro.com"),
    ];

    let banner_keywords: &[&str] = &[
        "cookie-consent", "cookieconsent", "cookie-notice", "cookienotice",
        "cookie-banner", "cookiebanner", "cookie-bar", "cookiebar",
        "gdpr", "gdpr-banner", "gdprbanner", "consent-banner", "consentbanner",
        "cc-banner", "cc-banner", "cookie-overlay", "cookieoverlay",
        "cookie-popup", "cookiepopup", "cookie-dialog", "cookiedialog",
        "notice-bar", "noticebar", "cookie-law", "cookielaw",
        "cookie-compliance", "cookiecompliance", "eu-cookie", "eucookie",
    ];

    let mut detected_cmp: Option<&str> = None;
    let script_sel = Selector::parse("script[src]").unwrap();

    for el in document.select(&script_sel) {
        if let Some(src) = el.value().attr("src") {
            let lower = src.to_lowercase();
            for &(domain, name, _) in known_cmps {
                if lower.contains(domain) {
                    detected_cmp = Some(name);
                    break;
                }
            }
            if detected_cmp.is_some() {
                break;
            }
        }
    }

    if detected_cmp.is_none() {
        let inline_sel = Selector::parse("script:not([src])").unwrap();
        for el in document.select(&inline_sel) {
            let text = el.text().collect::<String>().to_lowercase();
            let indicators = ["cookieconsent", "onetrust", "cookiebot", "__tcfapi",
                "cmp.show", "gdpr", "consent", "cookie_notice", "cookieBanner",
                "data-cookieconsent", "cookiehub", "ccpa"];
            if indicators.iter().any(|i| text.contains(i)) {
                detected_cmp = Some("a CMP (detected via inline script)");
                break;
            }
        }
    }

    if detected_cmp.is_none() {
        let all_els = Selector::parse("div, section, aside, nav, header, footer, span, p").unwrap();
        'banner_search: for el in document.select(&all_els) {
            let id = el.value().attr("id").unwrap_or("");
            let cls = el.value().attr("class").unwrap_or("");
            let aria = el.value().attr("aria-label").unwrap_or("");
            let combined = format!("{} {} {}", id, cls, aria).to_lowercase();
            for kw in banner_keywords {
                if combined.contains(kw) {
                    detected_cmp = Some("a banner element (detected via HTML class/id pattern)");
                    break 'banner_search;
                }
            }
        }
    }

    match detected_cmp {
        Some(cmp) => {
            let is_user_managed = cmp.starts_with("a banner") || cmp.starts_with("a CMP");
            let severity = if is_user_managed { Severity::Info } else { Severity::Pass };
            let sev_label = if is_user_managed { "user-managed" } else { "trusted" };

            findings.push(Finding {
                category: "best_practices".to_string(),
                check: "gdpr_consent".to_string(),
                severity,
                title: format!("GDPR cookie consent mechanism detected ({})", sev_label),
                description: format!(
                    "A cookie consent mechanism was detected on the page: {}.\n\n\
                     GDPR (and similar regulations like ePrivacy Directive, LGPD, CCPA/CPRA) \
                     require websites to obtain informed user consent before placing non-essential \
                     cookies (analytics, advertising, social media).\n\n\
                     {}\n\n\
                     Best practices:\n  \
                     · Ensure the CMP respects user choices across all scripts\n  \
                     · Provide clear Accept All / Reject All options with equal prominence\n  \
                     · Store consent proof (timestamp + user choice) for auditability\n  \
                     · Respect Do Not Track (DNT) and Global Privacy Control (GPC) signals\n  \
                     · Test that no non-essential cookies fire before consent is given\n\n\
                     Reference: EU GDPR Art. 7, ePrivacy Directive Art. 5(3), \
                     IAB Europe TCF v2.2",
                    cmp,
                    if is_user_managed {
                        "No known Consent Management Platform (CMP) was identified, but an \
                         element matching common cookie banner patterns was found. Verify that \
                         this banner correctly implements consent management (blocking scripts \
                         until consent, storing preferences, etc.)."
                    } else {
                        "A recognized Consent Management Platform (CMP) was identified, which \
                         suggests the site has a GDPR compliance mechanism in place. Verify \
                         that it is properly configured for all regions served."
                    }
                ),
                snippet: None,
                page_url: None,
            });
        }
        None => {
            findings.push(Finding {
                category: "best_practices".to_string(),
                check: "gdpr_consent".to_string(),
                severity: Severity::Warning,
                title: "No GDPR cookie consent mechanism detected".into(),
                description: format!(
                    "No cookie consent banner, CMP script, or cookie notice was detected on the \
                     page. If this site serves users in the EU/EEA, UK, Brazil (LGPD), California \
                     (CCPA/CPRA), or other regions with privacy regulations, this may be a legal \
                     compliance issue.\n\n\
                     GDPR and similar regulations require:\n  \
                     · Informed consent before setting non-essential cookies\n  \
                     · A clear, easily accessible cookie preferences interface\n  \
                     · The ability to withdraw consent as easily as it was given\n  \
                     · Proof of consent (timestamped records)\n\n\
                     Recommended Consent Management Platforms (CMPs):\n  \
                     · Cookiebot (cookiebot.com)\n  \
                     · OneTrust (onetrust.com)\n  \
                     · Quantcast Choice (quantcast.com)\n  \
                     · Osano (osano.com)\n  \
                     · CookieYes (cookieyes.com)\n  \
                     · Usercentrics (usercentrics.eu)\n\n\
                     Note: This audit checks for known CMP scripts, common banner patterns, and \
                     inline consent variables. A custom or lightweight implementation may not be \
                     detected by pattern matching.\n\n\
                     Reference: EU GDPR Art. 7, ePrivacy Directive Art. 5(3), \
                     IAB Europe TCF v2.2"
                ),
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn console_error_note(findings: &mut Vec<Finding>) {
    findings.push(Finding {
        category: "best_practices".to_string(),
        check: "console_errors".to_string(),
        severity: Severity::Info,
        title: "JavaScript console error detection requires browser runtime".into(),
        description: format!(
            "Tengu's server-side HTTP fetch cannot capture JavaScript runtime errors because JS \
             does not execute during a plain HTTP request.\n\n\
             To audit console errors:\n  \
             • Run tengu with the --browser flag to use a headless browser (Playwright/Puppeteer) \
             that executes JavaScript and captures console.error(), uncaught exceptions, and \
             network errors.\n\n\
             Common console issues that --browser mode detects:\n  \
             • Uncaught TypeError / ReferenceError\n  \
             • Failed to load resource (404, 500, CORS errors)\n  \
             • Content Security Policy violations\n  \
             • Deprecated API warnings\n  \
              • Unhandled Promise rejections"
        ),
        snippet: None,
        page_url: None,
    });
}