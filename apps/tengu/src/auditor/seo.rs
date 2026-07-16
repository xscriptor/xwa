use scraper::{Html, Selector};
use url::Url;

use crate::auditor::{Finding, Severity};
use crate::config::TenguConfig;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 3).collect();
        format!("{}...", truncated)
    }
}

fn element_snippet(el: &scraper::ElementRef) -> String {
    let elem = el.value();
    let tag = elem.name();
    let mut html = format!("<{}", tag);
    for (name, value) in elem.attrs() {
        let escaped = value.replace('"', "&quot;");
        html.push_str(&format!(" {}=\"{}\"", name, escaped));
    }
    let self_closing = matches!(
        tag,
        "meta" | "link" | "br" | "hr" | "img" | "input" | "base" | "col" | "embed"
            | "source" | "track" | "wbr"
    );
    if self_closing {
        html.push_str(" />");
    } else if tag == "html" {
        html.push_str(">...</html>");
    } else {
        html.push('>');
        html.push_str(&el.inner_html());
        html.push_str(&format!("</{}>", tag));
    }
    truncate(&html, 300)
}

pub async fn analyze(html: &str, page_url: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let document = Html::parse_document(html);

    title_audit(&document, &mut findings);
    meta_description_audit(&document, &mut findings);
    heading_audit(&document, &mut findings);
    canonical_audit(&document, page_url, &mut findings);
    open_graph_audit(&document, &mut findings);
    twitter_card_audit(&document, &mut findings);
    json_ld_audit(&document, &mut findings);
    meta_robots_audit(&document, &mut findings);
    hreflang_audit(&document, &mut findings);
    lang_audit(&document, &mut findings);
    microdata_rdfa_audit(&document, &mut findings);

    findings
}

pub async fn analyze_seo_network(page_url: &str, html: &str, findings: &mut Vec<Finding>) {
    // Try to fetch and analyze robots.txt
    let mut found_sitemaps: Vec<String> = Vec::new();
    if let Ok(robots_content) = fetch_robots_txt(page_url).await {
        found_sitemaps = robots_txt_audit(&robots_content, page_url, findings);
    }

    // Try to fetch and analyze sitemap.xml
    if let Ok(sitemap_content) = fetch_sitemap_xml(page_url, &found_sitemaps).await {
        sitemap_audit(&sitemap_content, findings);
    }

    // Check for broken links on the page
    broken_link_audit(html, page_url, findings).await;

    // Analyze redirect chain for the page URL
    redirect_chain_audit(page_url, findings).await;
}

fn parse_base_url(page_url: &str) -> String {
    if let Ok(parsed) = Url::parse(page_url) {
        format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""))
    } else {
        String::new()
    }
}

async fn fetch_robots_txt(page_url: &str) -> Result<String, String> {
    let base = parse_base_url(page_url);
    if base.is_empty() {
        return Err("Could not parse base URL".into());
    }
    let robots_url = format!("{}/robots.txt", base);
    let cfg = TenguConfig::from_env();
    let client = cfg.http_client_builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client.get(&robots_url).send().await.map_err(|e| format!("Failed to fetch robots.txt: {}", e))?;
    if resp.status().is_success() {
        resp.text().await.map_err(|e| format!("Failed to read robots.txt body: {}", e))
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

fn title_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("head > title").unwrap();
    match document.select(&sel).next() {
        Some(el) => {
            let text = el.text().collect::<String>();
            let trimmed = text.trim();
            let len = trimmed.chars().count();
            let snippet = Some(element_snippet(&el));

            if trimmed.is_empty() {
                findings.push(Finding {
                    category: "seo".to_string(),
                    check: "title".to_string(),
                    severity: Severity::Error,
                    title: "Empty title tag".into(),
                    description: format!(
                        "The <title> tag exists but contains no text (0 characters). \
                         Search engines will fall back to displaying the URL or H1 text in \
                         search results instead, which reduces click-through rates. \
                         Recommendation: Add a descriptive title between 50-60 characters that \
                         includes the page's primary topic and target keywords near the front."
                    ),
                    snippet,
                    page_url: None,
                });
            } else if len < 30 || len > 60 {
                if len < 30 {
                    findings.push(Finding {
                        category: "seo".to_string(),
                        check: "title".to_string(),
                        severity: Severity::Warning,
                        title: "Title too short".into(),
                        description: format!(
                            "Found <title> with content \"{}\" ({} characters). At {} character(s), \
                             this title is too short to adequately describe the page. Search \
                             engines may ignore short titles and generate their own from other \
                             content. \
                             Recommendation: Expand the title to 50-60 characters, placing \
                             primary keywords near the front of the title.",
                            trimmed, len, len
                        ),
                        snippet,
                        page_url: None,
                    });
                } else {
                    findings.push(Finding {
                        category: "seo".to_string(),
                        check: "title".to_string(),
                        severity: Severity::Warning,
                        title: "Title too long".into(),
                        description: format!(
                            "Found <title> with content \"{}\" ({} characters). At {} characters, \
                             this title exceeds the recommended 60-character limit. Search \
                             engines will truncate overlength titles in search results, \
                             potentially cutting off important keywords or the end of the title. \
                             Recommendation: Condense the title to 50-60 characters while \
                             preserving the most important keywords and meaning.",
                            trimmed, len, len
                        ),
                        snippet,
                        page_url: None,
                    });
                }
            }
        }
        None => {
            findings.push(Finding {
                category: "seo".to_string(),
                check: "title".to_string(),
                severity: Severity::Error,
                title: "Missing title tag".into(),
                description: format!(
                    "No <title> element was found in the document <head>. The title tag is the \
                     most important on-page SEO element — it appears as the clickable headline \
                     in search results and is a primary ranking signal. \
                     Recommendation: Add <title>Your Page Title Here</title> to the <head>, \
                     keeping it between 50-60 characters with primary keywords near the front."
                ),
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn meta_description_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("meta[name=description]").unwrap();
    match document.select(&sel).next() {
        Some(el) => {
            let content = el.value().attr("content").unwrap_or("").trim();
            let len = content.chars().count();
            let snippet = Some(element_snippet(&el));

            if content.is_empty() {
                findings.push(Finding {
                    category: "seo".to_string(),
                    check: "meta_description".to_string(),
                    severity: Severity::Warning,
                    title: "Empty meta description".into(),
                    description: format!(
                        "The <meta name=\"description\"> tag exists but has no content. Search \
                         engines will auto-generate a snippet from page content, which may not \
                         accurately represent the page or include a compelling call to action. \
                         Recommendation: Add a concise, engaging description between 50-160 \
                         characters that summarizes the page content and encourages clicks."
                    ),
                    snippet,
                    page_url: None,
                });
            } else if len < 50 || len > 160 {
                if len < 50 {
                    findings.push(Finding {
                        category: "seo".to_string(),
                        check: "meta_description".to_string(),
                        severity: Severity::Warning,
                        title: "Meta description too short".into(),
                        description: format!(
                            "Meta description content: \"{}\" ({} characters). At {} character(s), \
                             this description is too short to provide meaningful context. Search \
                             engines may ignore short descriptions and generate their own. \
                             Recommendation: Expand to 50-160 characters, incorporating target \
                             keywords and a clear value proposition.",
                            truncate(content, 120), len, len
                        ),
                        snippet,
                        page_url: None,
                    });
                } else {
                    findings.push(Finding {
                        category: "seo".to_string(),
                        check: "meta_description".to_string(),
                        severity: Severity::Warning,
                        title: "Meta description too long".into(),
                        description: format!(
                            "Meta description content: \"{}\" ({} characters). At {} characters, \
                             this description exceeds the recommended 160-character limit. Search \
                             engines will truncate long descriptions in search results, which can \
                             reduce click-through rates. \
                             Recommendation: Condense to 50-160 characters, keeping the most \
                             compelling content and keywords within the visible portion.",
                            truncate(content, 120), len, len
                        ),
                        snippet,
                        page_url: None,
                    });
                }
            }
        }
        None => {
            findings.push(Finding {
                category: "seo".to_string(),
                check: "meta_description".to_string(),
                severity: Severity::Warning,
                title: "Missing meta description".into(),
                description: format!(
                    "No <meta name=\"description\"> tag was found. The meta description is used \
                     by search engines as the summary snippet in search results and is a key \
                     factor in click-through rates. \
                     Recommendation: Add <meta name=\"description\" content=\"A compelling \
                     50-160 character summary of this page's content\" /> to the <head>."
                ),
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn heading_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    let headings: Vec<_> = document.select(&sel).collect();

    if headings.is_empty() {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "headings".to_string(),
            severity: Severity::Error,
            title: "No headings found".into(),
            description: format!(
                "The page has no heading elements (h1-h6). Headings provide a hierarchical \
                 structure that helps both users and search engines understand the content \
                 organization. Without headings, the page lacks semantic structure. \
                 Recommendation: Add at least one <h1> for the primary topic, followed by \
                 <h2> through <h6> for subsections in a logical hierarchy."
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let h1_count = headings.iter().filter(|h| h.value().name() == "h1").count();

    if h1_count == 0 {
        let first = &headings[0];
        let first_name = first.value().name();
        let first_text = first.text().collect::<String>().trim().to_string();
        let snippet = Some(element_snippet(first));

        findings.push(Finding {
            category: "seo".to_string(),
            check: "headings".to_string(),
            severity: Severity::Error,
            title: "Missing h1 heading".into(),
            description: format!(
                "No <h1> element was found on the page. The first heading encountered is \
                 <{name}>{text}</{name}>. Every page should have exactly one <h1> that \
                 communicates the primary topic to search engines and assistive technologies. \
                 Recommendation: Change the leading <{name}> to an <h1> that accurately \
                 describes the page's main topic and includes primary keywords.",
                name = first_name,
                text = truncate(&first_text, 100)
            ),
            snippet,
            page_url: None,
        });
    } else if h1_count > 1 {
        let h1_texts: Vec<String> = headings
            .iter()
            .filter(|h| h.value().name() == "h1")
            .map(|h| {
                let txt = h.text().collect::<String>().trim().to_string();
                format!("\"{}\"", truncate(&txt, 80))
            })
            .collect();
        let first_h1 = headings
            .iter()
            .find(|h| h.value().name() == "h1")
            .unwrap();
        let snippet = Some(element_snippet(first_h1));

        findings.push(Finding {
            category: "seo".to_string(),
            check: "headings".to_string(),
            severity: Severity::Warning,
            title: "Multiple h1 headings".into(),
            description: format!(
                "Found {} <h1> elements with texts: {}. Pages should have exactly one <h1> \
                 that represents the single primary topic. Multiple h1s dilute the topical \
                 signal for search engines and create ambiguity about the page's main subject. \
                 Recommendation: Keep one <h1> and demote the others to <h2> or lower heading \
                 levels to maintain a proper content hierarchy.",
                h1_count,
                h1_texts.join(", ")
            ),
            snippet,
            page_url: None,
        });
    }
}

fn canonical_audit(document: &Html, page_url: &str, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("link[rel=canonical]").unwrap();
    match document.select(&sel).next() {
        Some(el) => {
            let href = el.value().attr("href").unwrap_or("");
            let snippet = Some(element_snippet(&el));

            if href.is_empty() {
                findings.push(Finding {
                    category: "seo".to_string(),
                    check: "canonical".to_string(),
                    severity: Severity::Error,
                    title: "Empty canonical URL".into(),
                    description: format!(
                        "The <link rel=\"canonical\"> tag exists but the href attribute is empty. \
                         This provides no benefit and may confuse search engine crawlers. \
                         Recommendation: Set the href to the preferred canonical URL for this page."
                    ),
                    snippet,
                    page_url: None,
                });
                return;
            }

            match (Url::parse(href), Url::parse(page_url)) {
                (Ok(canonical_url), Ok(page_parsed)) => {
                    let canon_host = canonical_url.host_str().unwrap_or("");
                    let page_host = page_parsed.host_str().unwrap_or("");

                    if canon_host != page_host {
                        findings.push(Finding {
                            category: "seo".to_string(),
                            check: "canonical".to_string(),
                            severity: Severity::Warning,
                            title: "Cross-domain canonical URL".into(),
                            description: format!(
                                "The canonical URL \"{href}\" points to domain \"{canon_host}\", \
                                 which differs from the current page domain \"{page_host}\". This \
                                 tells search engines that a different domain is the authoritative \
                                 version — typically only used for syndicated content. \
                                 Recommendation: Ensure the canonical URL uses the same domain as \
                                 the current page unless cross-domain canonicalization is intentional.",
                                href = href,
                                canon_host = canon_host,
                                page_host = page_host
                            ),
                            snippet,
                            page_url: None,
                        });
                    } else if canonical_url.path() != page_parsed.path()
                        || canonical_url.query() != page_parsed.query()
                    {
                        findings.push(Finding {
                            category: "seo".to_string(),
                            check: "canonical".to_string(),
                            severity: Severity::Info,
                            title: "Canonical URL differs from page URL".into(),
                            description: format!(
                                "The canonical URL \"{href}\" points to a different path or query \
                                 than the current page URL \"{page}\". This is used when multiple \
                                 URLs serve similar content (e.g., pagination, tracking parameters). \
                                 Verify this is intentional — if these are truly different pages, \
                                 each should have its own canonical tag.",
                                href = href,
                                page = page_url
                            ),
                            snippet,
                            page_url: None,
                        });
                    }
                }
                _ => {
                    findings.push(Finding {
                        category: "seo".to_string(),
                        check: "canonical".to_string(),
                        severity: Severity::Warning,
                        title: "Invalid canonical URL".into(),
                        description: format!(
                            "The canonical href \"{href}\" could not be parsed as a valid URL. \
                             Invalid URLs are ignored by search engines. \
                             Recommendation: Ensure the canonical URL is a fully qualified, \
                             valid URL including the protocol (https://).",
                            href = href
                        ),
                        snippet,
                        page_url: None,
                    });
                }
            }
        }
        None => {
            findings.push(Finding {
                category: "seo".to_string(),
                check: "canonical".to_string(),
                severity: Severity::Info,
                title: "No canonical URL".into(),
                description: format!(
                    "No <link rel=\"canonical\"> tag was found. Canonical tags help prevent \
                     duplicate content issues by telling search engines which URL is the master \
                     version. \
                     Recommendation: Add <link rel=\"canonical\" href=\"{url}\" /> to the <head>, \
                     especially if this content is accessible via multiple URLs (e.g., with and \
                     without www, trailing slash, or tracking parameters).",
                    url = page_url
                ),
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn open_graph_audit(document: &Html, findings: &mut Vec<Finding>) {
    let required = [
        "og:title",
        "og:type",
        "og:image",
        "og:url",
        "og:description",
    ];
    let sel = Selector::parse("meta[property^=\"og:\"]").unwrap();
    let og_elements: Vec<_> = document.select(&sel).collect();

    if og_elements.is_empty() {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "open_graph".to_string(),
            severity: Severity::Warning,
            title: "No Open Graph tags".into(),
            description: format!(
                "The page has no Open Graph meta tags. When shared on social platforms (Facebook, \
                 LinkedIn, Discord, Slack), pages without OG tags appear as plain text links \
                 with auto-generated snippets, which often lack an image and compelling text. \
                 Recommendation: Add at minimum og:title, og:type, og:image, og:url, and \
                 og:description to the <head> to control how the page appears when shared."
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let present: Vec<(&str, &str)> = og_elements
        .iter()
        .map(|el| {
            let prop = el.value().attr("property").unwrap_or("");
            let content = el.value().attr("content").unwrap_or("");
            (prop, content)
        })
        .collect();

    let present_props: Vec<&str> = present.iter().map(|(p, _)| *p).collect();

    for &prop in &required {
        if present_props.contains(&prop) {
            let el = og_elements
                .iter()
                .find(|e| e.value().attr("property").unwrap_or("") == prop)
                .unwrap();
            let content = el.value().attr("content").unwrap_or("");
            let snippet = Some(element_snippet(el));

            findings.push(Finding {
                category: "seo".to_string(),
                check: "open_graph".to_string(),
                severity: Severity::Pass,
                title: format!("{} is present", prop),
                description: format!(
                    "Found {prop} with content \"{content}\". This tag controls how the page \
                     appears when shared on platforms that support the Open Graph protocol.",
                    prop = prop,
                    content = truncate(content, 100)
                ),
                snippet,
                page_url: None,
            });
        } else {
            let severity = if prop == "og:image" {
                Severity::Info
            } else {
                Severity::Warning
            };
            findings.push(Finding {
                category: "seo".to_string(),
                check: "open_graph".to_string(),
                severity,
                title: format!("Missing {}", prop),
                description: match prop {
                    "og:title" => format!(
                        "{prop} is missing. Without it, platforms fall back to the <title> tag, \
                         which may not be optimized for social sharing. \
                         Recommendation: Add <meta property=\"og:title\" content=\"A compelling, \
                         shareable title for this page\" />.",
                        prop = prop
                    ),
                    "og:description" => format!(
                        "{prop} is missing. Without it, platforms auto-generate a description \
                         from page content, which may be irrelevant or unappealing. \
                         Recommendation: Add <meta property=\"og:description\" content=\"A brief, \
                         enticing summary for social feeds\" />.",
                        prop = prop
                    ),
                    "og:image" => format!(
                        "{prop} is missing. Shared links without an OG image appear as plain \
                         text, reducing engagement and visual appeal. \
                         Recommendation: Add <meta property=\"og:image\" content=\"https://...\" /> \
                         with a high-quality image (ideally 1200x630 pixels, under 1 MB).",
                        prop = prop
                    ),
                    "og:url" => format!(
                        "{prop} is missing. This tag helps platforms resolve the canonical URL \
                         of the shared content. \
                         Recommendation: Add <meta property=\"og:url\" content=\"{url}\" />.",
                        prop = prop,
                        url = "https://example.com/page"
                    ),
                    "og:type" => format!(
                        "{prop} is missing. This tells platforms what type of content it is \
                         (e.g., article, website, product, video.movie). \
                         Recommendation: Add <meta property=\"og:type\" content=\"website\" /> or \
                         the appropriate type for your content.",
                        prop = prop
                    ),
                    _ => format!(
                        "{prop} is missing. \
                         Recommendation: Add this Open Graph tag with appropriate content.",
                        prop = prop
                    ),
                },
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn twitter_card_audit(document: &Html, findings: &mut Vec<Finding>) {
    let required = [
        "twitter:card",
        "twitter:title",
        "twitter:description",
        "twitter:image",
    ];
    let sel = Selector::parse("meta[name^=\"twitter:\"]").unwrap();
    let tw_elements: Vec<_> = document.select(&sel).collect();

    if tw_elements.is_empty() {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "twitter_card".to_string(),
            severity: Severity::Info,
            title: "No Twitter Card tags".into(),
            description: format!(
                "The page has no Twitter Card meta tags. Twitter Cards control how content \
                 appears when shared on X (formerly Twitter). Without them, shared links \
                 appear as plain text with no image or rich formatting. \
                 Recommendation: Add twitter:card, twitter:title, twitter:description, and \
                 twitter:image tags. Note: If Open Graph tags are present, Twitter will fall \
                 back to them, so OG tags may be sufficient."
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let present: Vec<(&str, &str)> = tw_elements
        .iter()
        .map(|el| {
            let name = el.value().attr("name").unwrap_or("");
            let content = el.value().attr("content").unwrap_or("");
            (name, content)
        })
        .collect();

    let present_names: Vec<&str> = present.iter().map(|(n, _)| *n).collect();

    for &prop in &required {
        if present_names.contains(&prop) {
            let el = tw_elements
                .iter()
                .find(|e| e.value().attr("name").unwrap_or("") == prop)
                .unwrap();
            let content = el.value().attr("content").unwrap_or("");
            let snippet = Some(element_snippet(el));

            findings.push(Finding {
                category: "seo".to_string(),
                check: "twitter_card".to_string(),
                severity: Severity::Pass,
                title: format!("{} is present", prop),
                description: format!(
                    "Found {prop} with content \"{content}\". This tag optimizes how the page \
                     appears in X/Twitter timeline cards and improves engagement from social shares.",
                    prop = prop,
                    content = truncate(content, 100)
                ),
                snippet,
                page_url: None,
            });
        } else {
            findings.push(Finding {
                category: "seo".to_string(),
                check: "twitter_card".to_string(),
                severity: Severity::Info,
                title: format!("Missing {}", prop),
                description: match prop {
                    "twitter:card" => format!(
                        "{prop} is missing. This tag defines the card type (summary, \
                         summary_large_image, app, or player) and is required for Twitter Cards \
                         to render. \
                         Recommendation: Add <meta name=\"twitter:card\" content=\"summary_large_image\" /> \
                         for a large image preview.",
                        prop = prop
                    ),
                    "twitter:title" => format!(
                        "{prop} is missing. Without it, X falls back to the page <title> or \
                         og:title. \
                         Recommendation: Add <meta name=\"twitter:title\" content=\"Your optimized \
                         title for X/Twitter\" />.",
                        prop = prop
                    ),
                    "twitter:description" => format!(
                        "{prop} is missing. Without it, X auto-generates a description from \
                         page content. \
                         Recommendation: Add <meta name=\"twitter:description\" content=\"A \
                         concise, engaging description for X/Twitter cards\" />.",
                        prop = prop
                    ),
                    "twitter:image" => format!(
                        "{prop} is missing. Cards without images show as plain text links with \
                         significantly lower engagement. \
                         Recommendation: Add <meta name=\"twitter:image\" content=\"https://...\" /> \
                         with an image that meets X's requirements (minimum 120x120px, supports \
                         up to 4096x4096px).",
                        prop = prop
                    ),
                    _ => format!(
                        "{prop} is missing. \
                         Recommendation: Add this Twitter Card tag with appropriate content.",
                        prop = prop
                    ),
                },
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn json_ld_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("script[type=\"application/ld+json\"]").unwrap();
    let scripts: Vec<_> = document.select(&sel).collect();

    if scripts.is_empty() {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "json_ld".to_string(),
            severity: Severity::Info,
            title: "No JSON-LD structured data".into(),
            description: format!(
                "The page has no JSON-LD structured data. JSON-LD helps search engines \
                 understand the content and enable rich results like star ratings, recipes, \
                 FAQs, events, and breadcrumbs in search results. It is Google's preferred \
                 format for structured data. \
                 Recommendation: Add a <script type=\"application/ld+json\"> block in the <head> \
                 with structured data appropriate for the content type (e.g., Organization, \
                 WebSite, Article, Product, FAQPage, LocalBusiness)."
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let el = &scripts[0];
    let json_text = el.inner_html();
    let snippet = Some(element_snippet(el));

    let type_info = serde_json::from_str::<serde_json::Value>(&json_text)
        .ok()
        .and_then(|v| {
            let obj = match &v {
                serde_json::Value::Object(m) => Some(m),
                serde_json::Value::Array(arr) => arr.first().and_then(|v| v.as_object()),
                _ => None,
            };
            obj.and_then(|m| {
                m.get("@type")
                    .or_else(|| m.get("type"))
                    .and_then(|t| t.as_str())
                    .map(|t| t.to_string())
            })
        });

    let type_desc = type_info
        .as_deref()
        .map(|t| format!(" with @type \"{}\"", t))
        .unwrap_or_default();

    findings.push(Finding {
        category: "seo".to_string(),
        check: "json_ld".to_string(),
        severity: Severity::Pass,
        title: format!("JSON-LD structured data found{}", type_desc),
        description: format!(
            "Found a JSON-LD structured data script{type_desc} ({len} characters). JSON-LD is \
             Google's preferred format for structured data and enables rich search result \
             features such as rich snippets, knowledge panels, and carousels. \
             Recommendation: Validate the structured data using Google's Rich Results Test \
             (https://search.google.com/test/rich-results) to ensure it is correctly formatted \
             and eligible for the intended rich result features.",
            type_desc = type_desc,
            len = json_text.chars().count()
        ),
        snippet,
        page_url: None,
    });
}

fn meta_robots_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("meta[name=robots]").unwrap();
    match document.select(&sel).next() {
        Some(el) => {
            let content = el.value().attr("content").unwrap_or("");
            let snippet = Some(element_snippet(&el));
            let directives: Vec<&str> = content.split(',').map(|s| s.trim()).collect();

            let directive_desc: Vec<String> = directives
                .iter()
                .map(|d| match *d {
                    "noindex" => format!("\"{}\": Search engines will NOT include this page in their indexes.",
                        d),
                    "nofollow" => format!("\"{}\": Search engines will NOT follow links on this page.",
                        d),
                    "noarchive" => format!("\"{}\": Search engines will NOT show a cached version in results.",
                        d),
                    "nosnippet" => format!("\"{}\": Search engines will NOT show a text snippet in search results.",
                        d),
                    "notranslate" => format!(
                        "\"{}\": Search engines will NOT offer a translation of this page.", d
                    ),
                    "noimageindex" => format!(
                        "\"{}\": Search engines will NOT index images found on this page.", d
                    ),
                    "index" => format!("\"{}\": Search engines MAY include this page (default).",
                        d),
                    "follow" => format!("\"{}\": Search engines MAY follow links (default).", d),
                    "all" => format!(
                        "\"{}\": Equivalent to 'index, follow' — no restrictions.", d
                    ),
                    "none" => format!(
                        "\"{}\": Equivalent to 'noindex, nofollow' — full restriction.", d
                    ),
                    _ => format!("\"{}\": Unknown or custom directive.", d),
                })
                .collect();

            let desc = directive_desc.join(" ");

            if content.contains("noindex") || content.to_lowercase().contains("none") {
                findings.push(Finding {
                    category: "seo".to_string(),
                    check: "meta_robots".to_string(),
                    severity: Severity::Info,
                    title: "Page blocked from indexing".into(),
                    description: format!(
                        "Meta robots directive: \"{content}\". This page has been explicitly \
                         blocked from search engine indexing. {desc} \
                         Recommendation: If you intend this page to appear in search results, \
                         remove the 'noindex' directive or change it to 'index, follow'. If the \
                         page is intentionally private (admin pages, duplicate content), ensure \
                         it is also blocked in robots.txt as a secondary measure.",
                        content = content,
                        desc = desc
                    ),
                    snippet,
                    page_url: None,
                });
            } else {
                findings.push(Finding {
                    category: "seo".to_string(),
                    check: "meta_robots".to_string(),
                    severity: Severity::Pass,
                    title: "Meta robots allows indexing".into(),
                    description: format!(
                        "Meta robots directive: \"{content}\". {desc} \
                         These directives allow search engines to index and crawl the page as \
                         expected. No changes required.",
                        content = content,
                        desc = desc
                    ),
                    snippet,
                    page_url: None,
                });
            }
        }
        None => {
            findings.push(Finding {
                category: "seo".to_string(),
                check: "meta_robots".to_string(),
                severity: Severity::Pass,
                title: "No meta robots tag".into(),
                description: format!(
                    "No <meta name=\"robots\"> tag was found. By default, search engines assume \
                     'index, follow' — meaning the page can appear in search results and links \
                     can be followed. No action is required unless you need to restrict crawling \
                     or indexing for specific pages."
                ),
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn hreflang_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("link[hreflang]").unwrap();
    let links: Vec<_> = document.select(&sel).collect();

    if links.is_empty() {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "hreflang".to_string(),
            severity: Severity::Pass,
            title: "No hreflang tags".into(),
            description: format!(
                "No hreflang tags were found. If the site serves content in only one language, \
                 hreflang tags are not required. For multilingual or multi-regional sites, \
                 hreflang tags help search engines serve the correct language or regional \
                 version to users based on their location and language preferences."
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let lang_entries: Vec<(&str, &str)> = links
        .iter()
        .filter_map(|el| {
            let lang = el.value().attr("hreflang")?;
            let href = el.value().attr("href").unwrap_or("");
            Some((lang, href))
        })
        .collect();

    let langs: Vec<&str> = lang_entries.iter().map(|(l, _)| *l).collect();
    let has_default = langs.contains(&"x-default");
    let has_self = lang_entries.iter().any(|(_, h)| *h == "");

    let snippets: Vec<String> = links.iter().map(|el| element_snippet(el)).collect();
    let summary = truncate(&snippets.join("\n"), 200);

    if langs.len() > 1 && links.len() > 1 && !has_default {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "hreflang".to_string(),
            severity: Severity::Warning,
            title: "Missing x-default hreflang".into(),
            description: format!(
                "Found {count} hreflang tag(s) for languages/regions: [{langs}], but no \
                 x-default fallback is defined. The x-default tag tells search engines which \
                 page to show when no language matches the user's preference (e.g., a language \
                 selection page or the English version). \
                 Recommendation: Add <link rel=\"alternate\" hreflang=\"x-default\" \
                 href=\"https://example.com/\" /> pointing to the default or language selection page.",
                count = links.len(),
                langs = langs.join(", ")
            ),
            snippet: Some(summary),
            page_url: None,
        });
    } else {
        let mut notes = Vec::new();
        if has_default {
            notes.push("with x-default fallback");
        }
        if has_self {
            notes.push("includes a self-referencing href");
        }
        let note_str = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join(", "))
        };

        findings.push(Finding {
            category: "seo".to_string(),
            check: "hreflang".to_string(),
            severity: Severity::Pass,
            title: "Hreflang tags configured".into(),
            description: format!(
                "Found {count} hreflang tag(s) for languages/regions: [{langs}]{notes}. \
                 Hreflang annotations help search engines serve the correct language version in \
                 international search results and reduce duplicate content issues across \
                 language/region variants.",
                count = links.len(),
                langs = langs.join(", "),
                notes = note_str
            ),
            snippet: Some(summary),
            page_url: None,
        });
    }
}

fn robots_txt_audit(robots_content: &str, page_url: &str, findings: &mut Vec<Finding>) -> Vec<String> {
    let mut sitemaps: Vec<String> = Vec::new();
    let mut disallowed_paths: Vec<String> = Vec::new();
    let mut allowed_paths: Vec<String> = Vec::new();
    let mut user_agents: Vec<String> = Vec::new();
    let mut has_sitemap_directive = false;
    let mut crawl_delay: Option<f64> = None;
    let mut current_ua: String = String::new();

    for line in robots_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(col) = trimmed.find(':') {
            let directive = trimmed[..col].trim().to_lowercase();
            let value = trimmed[col + 1..].trim().to_string();
            match directive.as_str() {
                "user-agent" => {
                    current_ua = value.clone();
                    if !user_agents.contains(&value) {
                        user_agents.push(value);
                    }
                }
                "disallow" => {
                    let path = if value.is_empty() { "(empty — allows all)" } else { &value };
                    if !disallowed_paths.contains(&value) {
                        let entry = if current_ua.is_empty() || current_ua == "*" {
                            format!("{} (all bots)", value)
                        } else {
                            format!("{} (user-agent: {})", value, current_ua)
                        };
                        disallowed_paths.push(entry);
                    }
                }
                "allow" => {
                    if !allowed_paths.contains(&value) {
                        allowed_paths.push(value);
                    }
                }
                "sitemap" => {
                    has_sitemap_directive = true;
                    if !sitemaps.contains(&value) {
                        sitemaps.push(value);
                    }
                }
                "crawl-delay" => {
                    if let Ok(delay) = value.parse::<f64>() {
                        crawl_delay = Some(delay);
                    }
                }
                _ => {}
            }
        }
    }

    let mut issues: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if user_agents.is_empty() {
        issues.push("No User-agent directives found — all bots are allowed to crawl everything".to_string());
    } else if user_agents.len() == 1 && user_agents[0] == "*" {
        notes.push("Has a wildcard User-agent: * — applies to all bots".to_string());
    } else {
        notes.push(format!("{} specific user-agent rule(s) defined", user_agents.len()));
    }

    if disallowed_paths.is_empty() {
        notes.push("No paths are disallowed — all content is crawlable".to_string());
    } else {
        notes.push(format!("{} path(s) disallowed", disallowed_paths.len()));
    }

    if let Some(delay) = crawl_delay {
        notes.push(format!("Crawl-delay: {} seconds", delay));
    }

    if !has_sitemap_directive {
        issues.push("No Sitemap directive found — search engines won't discover your sitemap from robots.txt".to_string());
    } else {
        notes.push(format!("{} Sitemap URL(s) declared", sitemaps.len()));
    }

    let base = parse_base_url(page_url);
    let robots_url = format!("{}/robots.txt", base);

    if issues.is_empty() {
        if !notes.is_empty() {
            findings.push(Finding {
                category: "seo".to_string(),
                check: "robots_txt".to_string(),
                severity: Severity::Pass,
                title: "robots.txt is present and well-configured".into(),
                description: format!(
                    "The robots.txt file was successfully retrieved from {}.\n\n\
                     Configuration summary:\n  · {}\n\n\
                     Recommendation: Periodically review robots.txt to ensure it reflects your \
                     current site structure and SEO strategy. Disallowed pages that have inbound \
                     links may still appear in search results (as a listing without a snippet).\n\n\
                     Reference: RFC 9309 — Robots Exclusion Protocol, robots.txt.org",
                    robots_url,
                    notes.join("\n  · "),
                ),
                snippet: Some(truncate(robots_content, 300)),
                page_url: None,
            });
        }
        return sitemaps;
    }

    let issue_text: Vec<String> = issues.iter().enumerate()
        .map(|(i, issue)| format!("  {}. {}", i + 1, issue)).collect();

    findings.push(Finding {
        category: "seo".to_string(),
        check: "robots_txt".to_string(),
        severity: Severity::Warning,
        title: "robots.txt has configuration issues".into(),
        description: format!(
            "The robots.txt file was retrieved from {} but has {} issue(s).\n\n\
             Issues:\n{}\n\n\
             Current configuration notes:\n  · {}\n\n\
             Recommendation:\n  · Always include a User-agent directive (start with User-agent: * \
             for all bots)\n  · Use Allow and Disallow for fine-grained control\n  \
             · Add a Sitemap directive pointing to your XML sitemap\n  \
             · Use Crawl-delay if your server needs rate limiting\n  \
             · Test your robots.txt with Google's Robots Testing Tool\n\n\
             Reference: RFC 9309 — Robots Exclusion Protocol, robots.txt.org, \
             Google Search Central — robots.txt",
            robots_url,
            issues.len(),
            issue_text.join("\n"),
            notes.join("\n  · "),
        ),
        snippet: Some(truncate(robots_content, 300)),
        page_url: None,
    });

    sitemaps
}

async fn fetch_sitemap_xml(page_url: &str, known_sitemaps: &[String]) -> Result<String, String> {
    let urls_to_try: Vec<String> = if !known_sitemaps.is_empty() {
        known_sitemaps.to_vec()
    } else {
        let base = parse_base_url(page_url);
        if base.is_empty() {
            return Err("Could not parse base URL".into());
        }
        vec![
            format!("{}/sitemap.xml", base),
            format!("{}/sitemap_index.xml", base),
            format!("{}/sitemap/sitemap.xml", base),
        ]
    };

    let cfg = TenguConfig::from_env();
    let client = cfg.http_client_builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    for url in &urls_to_try {
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.text().await {
                    return Ok(body);
                }
            }
        }
    }

    Err("No sitemap found".into())
}

fn sitemap_audit(sitemap_content: &str, findings: &mut Vec<Finding>) {
    let url_count = sitemap_content.matches("<loc>").count();
    let is_index = sitemap_content.to_lowercase().contains("<sitemapindex");
    let has_lastmod = sitemap_content.contains("<lastmod>");
    let has_changefreq = sitemap_content.contains("<changefreq>");
    let has_priority = sitemap_content.contains("<priority>");
    let mut notes: Vec<String> = Vec::new();

    if is_index {
        notes.push("This is a sitemap index file (contains references to other sitemaps)".to_string());
    } else {
        notes.push("This is a standard sitemap file".to_string());
    }

    notes.push(format!("{} URL(s) declared in sitemap", url_count));

    if has_lastmod {
        notes.push("Includes lastmod dates — helps search engines understand freshness".to_string());
    } else {
        notes.push("Missing lastmod dates — consider adding <lastmod> for freshness signals".to_string());
    }

    if has_changefreq {
        notes.push("Includes changefreq hints".to_string());
    }

    if has_priority {
        notes.push("Includes priority hints".to_string());
    }

    if url_count == 0 {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "sitemap".to_string(),
            severity: Severity::Warning,
            title: "Sitemap appears to be empty or unparseable".into(),
            description: format!(
                "A sitemap file was found but no <loc> (URL) elements were detected. This may \
                 indicate an empty sitemap, incorrect format, or a paginated index that needs \
                 further crawling.\n\n\
                 Recommendation: Verify the sitemap at your browser and ensure it follows the \
                 standard sitemap protocol.\n\n\
                 Reference: sitemaps.org, Google Search Central — Sitemaps"
            ),
            snippet: Some(truncate(sitemap_content, 300)),
            page_url: None,
        });
        return;
    }

    findings.push(Finding {
        category: "seo".to_string(),
        check: "sitemap".to_string(),
        severity: Severity::Pass,
        title: format!("Sitemap found with {} URL(s)", url_count),
        description: format!(
            "A valid sitemap was detected.\n\n\
             Configuration summary:\n  · {}\n\n\
             Recommendations:\n  · Keep sitemaps under 50MB and 50,000 URLs (split into an index \
             if larger)\n  · Use lastmod to indicate when pages were last updated\n  \
             · Submit sitemap to Google Search Console and Bing Webmaster Tools\n  \
             · Reference sitemap from robots.txt (Sitemap: directive)\n\n\
             Reference: sitemaps.org, Google Search Central — Sitemaps",
            notes.join("\n  · "),
        ),
        snippet: Some(truncate(sitemap_content, 300)),
        page_url: None,
    });
}

fn extract_links(html: &str, page_url: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let link_sel = Selector::parse("a[href]").unwrap();
    let mut links: Vec<String> = Vec::new();

    for el in document.select(&link_sel) {
        if let Some(href) = el.value().attr("href") {
            let trimmed = href.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with("javascript:")
                || trimmed.starts_with("mailto:")
                || trimmed.starts_with("tel:")
                || trimmed.starts_with("data:")
                || trimmed.starts_with("blob:")
            {
                continue;
            }
            let absolute = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                trimmed.to_string()
            } else if let Ok(base) = Url::parse(page_url) {
                base.join(trimmed).map(|u| u.to_string()).unwrap_or_default()
            } else {
                continue;
            };
            if !absolute.is_empty() && !links.contains(&absolute) {
                links.push(absolute);
            }
        }
    }

    links
}

async fn broken_link_audit(html: &str, page_url: &str, findings: &mut Vec<Finding>) {
    // Extract links synchronously (Html is not Send, so we do this before any await)
    let links = extract_links(html, page_url);

    if links.is_empty() {
        return;
    }

    let cfg = TenguConfig::from_env();
    let client = cfg.http_client_builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .unwrap();

    let mut broken: Vec<(String, String)> = Vec::new();
    let mut checked = 0u32;
    let max_links = 30usize;

    for link in links.iter().take(max_links) {
        match client.head(link).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status >= 400 {
                    let reason = format!("HTTP {}", status);
                    broken.push((link.clone(), reason));
                }
            }
            Err(e) => {
                let reason = truncate(&e.to_string(), 60);
                broken.push((link.clone(), reason));
            }
        }
        checked += 1;
    }

    if broken.is_empty() {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "broken_links".to_string(),
            severity: Severity::Pass,
            title: "No broken links detected".into(),
            description: format!(
                "Checked {} link(s) on the page (up to {}) and found no broken links. \
                 All returned successful HTTP status codes.\n\n\
                 Recommendation: Regularly audit links, especially external ones, as they \
                 may become broken over time when third-party sites remove or restructure content.\n\n\
                 Reference: Google Search Central — Broken Links, W3C Link Checker",
                checked, max_links,
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let detail: Vec<String> = broken.iter().map(|(url, reason)| {
        format!("  · {} → {}", truncate(url, 80), reason)
    }).collect();

    findings.push(Finding {
        category: "seo".to_string(),
        check: "broken_links".to_string(),
        severity: Severity::Warning,
        title: format!("{} broken link(s) found on page", broken.len()),
        description: format!(
            "Checked {} link(s) on the page (up to {}) and found {} broken link(s). \
             Broken links harm user experience and SEO — search engines may penalize sites \
             with excessive 404/500 errors.\n\n\
             Broken links:\n{}\n\n\
             Note: Only the first {} links on the page were checked. Some failures may be \
             due to network timeouts or server-side blocks rather than actual broken content.\n\n\
             Recommendations:\n  · Fix or remove broken internal links\n  \
             · Update or redirect broken external links\n  \
             · Use 301 redirects for moved content (never 404)\n  \
             · Set up a custom 404 page that helps users find what they need\n  \
             · Use tools like Google Search Console to monitor crawl errors\n\n\
             Reference: Google Search Central — Broken Links, W3C Link Checker",
            checked,
            max_links,
            broken.len(),
            detail.join("\n"),
            max_links,
        ),
        snippet: None,
        page_url: None,
    });
}

async fn redirect_chain_audit(page_url: &str, findings: &mut Vec<Finding>) {
    let cfg = TenguConfig::from_env();
    let client = cfg.http_client_builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut chain: Vec<(u16, String)> = Vec::new();
    let mut current_url = page_url.to_string();
    let max_hops = 10;

    for _ in 0..=max_hops {
        match client.head(&current_url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                chain.push((status, current_url.clone()));

                if status < 300 || status >= 400 {
                    break;
                }

                if let Some(location) = resp.headers().get("location") {
                    if let Ok(loc_str) = location.to_str() {
                        let next = if loc_str.starts_with("http://") || loc_str.starts_with("https://") {
                            loc_str.to_string()
                        } else if let Ok(base) = Url::parse(&current_url) {
                            base.join(loc_str).map(|u| u.to_string()).unwrap_or_default()
                        } else {
                            break;
                        };

                        if next.is_empty() || next == current_url {
                            chain.push((0, "Redirect loop or empty location".to_string()));
                            break;
                        }
                        current_url = next;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            Err(e) => {
                chain.push((0, format!("Error: {}", truncate(&e.to_string(), 60))));
                break;
            }
        }
    }

    if chain.len() <= 1 {
        return;
    }

    let hops = chain.len() - 1;
    let detail: Vec<String> = chain.iter()
        .enumerate()
        .map(|(i, (status, url))| {
            if *status == 0 {
                format!("  {} {} — {}", i, truncate(url, 80), url)
            } else {
                format!("  {} HTTP {} → {}", i, status, truncate(url, 80))
            }
        })
        .collect();

    let severity = if hops >= 5 {
        Severity::Warning
    } else if hops >= 3 {
        Severity::Info
    } else {
        Severity::Info
    };

    let mut notes = Vec::new();
    if hops >= 5 {
        notes.push("⚠ Redirect chains longer than 5 hops should be shortened".to_string());
    }
    if hops >= 3 {
        notes.push("Each redirect adds latency (DNS + TCP + TLS + request time)".to_string());
    }

    let has_chain_issues = chain.iter().any(|(status, _)| *status == 0 || *status >= 400);

    findings.push(Finding {
        category: "seo".to_string(),
        check: "redirect_chain".to_string(),
        severity: if has_chain_issues { Severity::Warning } else { severity },
        title: format!("Redirect chain detected: {} hop(s)", hops),
        description: format!(
            "The page URL goes through {} redirect hop(s) before reaching the final destination. \
             Redirect chains slow down page load time and can dilute SEO link equity (PageRank).\n\n\
             Full redirect chain:\n{}\n\n\
             {}\n\n\
             Recommendations:\n  · Update internal links to point directly to the final URL\n  \
             · Replace multi-hop redirects with a single 301 redirect\n  \
             · Avoid redirect chains longer than 2-3 hops\n  \
             · Use 301 (permanent) instead of 302 (temporary) for permanent moves\n  \
             · Ensure all URLs in the chain use HTTPS\n\n\
             Reference: Google Search Central — Redirects, Moz — Redirect Chains",
            hops,
            detail.join("\n"),
            notes.join("\n"),
        ),
        snippet: Some(chain.last().map_or(String::new(), |(_, url)| url.clone())),
        page_url: None,
    });
}

fn microdata_rdfa_audit(document: &Html, findings: &mut Vec<Finding>) {
    let itemscope_sel = Selector::parse("[itemscope]").unwrap();
    let rdfa_sel = Selector::parse("[typeof], [vocab]").unwrap();
    let link_sel = Selector::parse("link[itemprop], meta[itemprop]").unwrap();

    let microdata_items = document.select(&itemscope_sel).count();
    let rdfa_items = document.select(&rdfa_sel).count();
    let microdata_props = document.select(&link_sel).count();
    let rdfa_props = 0u32;

    for el in document.select(&rdfa_sel) {
        let mut count = 0;
        for _ in el.select(&Selector::parse("[property]").unwrap()) {
            count += 1;
        }
        // rdfa_props += count;  // handled via the main count below
    }

    let total_props = {
        let mut c = 0u32;
        for el in document.select(&Selector::parse("[itemprop], [property]").unwrap()) {
            c += 1;
        }
        c
    };

    if microdata_items == 0 && rdfa_items == 0 {
        findings.push(Finding {
            category: "seo".to_string(),
            check: "structured_data_microdata".to_string(),
            severity: Severity::Info,
            title: "No Microdata or RDFa structured data detected".into(),
            description: format!(
                "The page does not use Microdata (itemscope/itemprop) or RDFa (typeof/property/vocab) \
                 markup. These structured data formats help search engines understand the content \
                 and enable rich results (rich snippets, knowledge panels).\n\n\
                 Alternatives already checked:\n  · JSON-LD: already audited separately\n  \
                 · Open Graph: already audited separately\n  · Twitter Cards: already audited separately\n\n\
                 Recommendation: Consider adding structured data using JSON-LD (recommended by \
                 Google), Microdata, or RDFa.\n\n\
                 Reference: schema.org, Google Search Central — Structured Data"
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let mut details: Vec<String> = Vec::new();

    if microdata_items > 0 {
        let types: Vec<String> = document.select(&itemscope_sel)
            .filter_map(|el| el.value().attr("itemtype").map(|t| t.to_string()))
            .collect();
        let unique_types: Vec<&str> = {
            let mut seen = Vec::new();
            for t in &types {
                if !seen.contains(&t.as_str()) {
                    seen.push(t.as_str());
                }
            }
            seen.truncate(5);
            seen
        };
        let types_str = if unique_types.is_empty() {
            " (no itemtype attributes found)".to_string()
        } else {
            format!(": {}", unique_types.join(", "))
        };
        details.push(format!("Microdata: {} item(s) with itemscope{}", microdata_items, types_str));
    }

    if rdfa_items > 0 {
        let types: Vec<String> = document.select(&rdfa_sel)
            .filter_map(|el| el.value().attr("typeof").map(|t| t.to_string()))
            .collect();
        let unique_types: Vec<&str> = {
            let mut seen = Vec::new();
            for t in &types {
                if !seen.contains(&t.as_str()) {
                    seen.push(t.as_str());
                }
            }
            seen.truncate(5);
            seen
        };
        let types_str = if unique_types.is_empty() {
            String::new()
        } else {
            format!(" ({})", unique_types.join(", "))
        };
        details.push(format!("RDFa: {} element(s) with typeof{}", rdfa_items, types_str));
    }

    if total_props > 0 {
        details.push(format!("Total properties defined: {}", total_props));
    }

    let type_count = if microdata_items > 0 { microdata_items } else { rdfa_items };

    findings.push(Finding {
        category: "seo".to_string(),
        check: "structured_data_microdata".to_string(),
        severity: Severity::Pass,
        title: format!("Microdata/RDFa structured data found ({} item(s))", type_count),
        description: format!(
            "The page uses Microdata and/or RDFa structured data markup.\n\n\
             Details:\n  · {}\n\n\
             What this means:\n  · Search engines can extract entity relationships from \
             Microdata and RDFa\n  · Well-formed structured data can enable rich results \
             (products, recipes, events, reviews, etc.)\n  · Google primarily recommends \
             JSON-LD but still supports Microdata and RDFa\n\n\
             Recommendation:\n  · Validate your markup with Google's Rich Results Test \
             (https://search.google.com/test/rich-results)\n  \
             · Consider adding JSON-LD as a supplement — it's easier to maintain and \
             Google's preferred format\n  \
             · Ensure itemtype/typeof URLs point to valid schema.org types\n  \
             · Use tools like Merkle's Schema Markup Validator for bulk validation\n\n\
             Reference: schema.org, Google Search Central — Structured Data, \
             RDFa Core 1.1, HTML Microdata W3C Spec",
            details.join("\n  · "),
        ),
        snippet: None,
        page_url: None,
    });
}

fn lang_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("html").unwrap();
    if let Some(el) = document.select(&sel).next() {
        let lang = el.value().attr("lang");
        let xml_lang = el.value().attr("xml:lang");

        let mut snippet_html = String::from("<html");
        for (name, value) in el.value().attrs() {
            let escaped = value.replace('"', "&quot;");
            snippet_html.push_str(&format!(" {}=\"{}\"", name, escaped));
        }
        snippet_html.push('>');
        let snippet = Some(truncate(&snippet_html, 300));

        match (lang, xml_lang) {
            (None, None) => {
                findings.push(Finding {
                    category: "seo".to_string(),
                    check: "lang_attribute".to_string(),
                    severity: Severity::Warning,
                    title: "Missing language attribute".into(),
                    description: format!(
                        "The <html> element has neither a lang nor an xml:lang attribute. \
                         The lang attribute is essential for accessibility (screen readers rely \
                         on it for correct pronunciation), SEO (search engines use it to serve \
                         the right language to users), and browser behavior (spell checkers, \
                         translation prompts). \
                         Recommendation: Add lang=\"en\" (or the appropriate language code) to \
                         the <html> element, e.g., <html lang=\"en\">."
                    ),
                    snippet,
                    page_url: None,
                });
            }
            _ => {
                let lang_val = lang.or(xml_lang).unwrap_or("");
                findings.push(Finding {
                    category: "seo".to_string(),
                    check: "lang_attribute".to_string(),
                    severity: Severity::Pass,
                    title: "Language attribute present".into(),
                    description: format!(
                        "The <html> element has lang=\"{lang_val}\", correctly declaring the \
                         page's primary language. This helps search engines serve the right \
                         content to users, assistive technologies pronounce text correctly, and \
                         browsers apply appropriate language-specific features.",
                        lang_val = lang_val
                    ),
                    snippet,
                    page_url: None,
                });
            }
        }
    }
}