use scraper::node::Node;
use scraper::{ElementRef, Html, Selector};

use crate::auditor::{Finding, Severity};

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

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max.saturating_sub(1)])
    } else {
        s.to_string()
    }
}

fn srgb_channel(c: u8) -> f64 {
    let v = c as f64 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    0.2126 * srgb_channel(r) + 0.7152 * srgb_channel(g) + 0.0722 * srgb_channel(b)
}

fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let l1 = relative_luminance(a.0, a.1, a.2);
    let l2 = relative_luminance(b.0, b.1, b.2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    if h.len() == 3 {
        let r = u8::from_str_radix(&h[0..1], 16).ok()? * 17;
        let g = u8::from_str_radix(&h[1..2], 16).ok()? * 17;
        let b = u8::from_str_radix(&h[2..3], 16).ok()? * 17;
        Some((r, g, b))
    } else if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).ok()?;
        let g = u8::from_str_radix(&h[2..4], 16).ok()?;
        let b = u8::from_str_radix(&h[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

fn parse_rgb_function(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim();
    let inner = s
        .strip_prefix("rgb(")
        .or_else(|| s.strip_prefix("rgba("))?
        .strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(|c| c == ',' || c == ' ' || c == '/').filter(|p| !p.is_empty()).collect();
    if parts.len() < 3 {
        return None;
    }
    let r = parts[0].trim().parse::<f64>().ok()?;
    let g = parts[1].trim().parse::<f64>().ok()?;
    let b = parts[2].trim().parse::<f64>().ok()?;
    Some((r as u8, g as u8, b as u8))
}

fn parse_color_value(value: &str) -> Option<(u8, u8, u8)> {
    let v = value.trim();
    if v.starts_with('#') {
        return hex_to_rgb(v);
    }
    if v.starts_with("rgb") {
        return parse_rgb_function(v);
    }
    let named = match v.to_ascii_lowercase().as_str() {
        "black" => (0, 0, 0),
        "silver" => (192, 192, 192),
        "gray" | "grey" => (128, 128, 128),
        "white" => (255, 255, 255),
        "maroon" => (128, 0, 0),
        "red" => (255, 0, 0),
        "purple" => (128, 0, 128),
        "fuchsia" | "magenta" => (255, 0, 255),
        "green" => (0, 128, 0),
        "lime" => (0, 255, 0),
        "olive" => (128, 128, 0),
        "yellow" => (255, 255, 0),
        "navy" => (0, 0, 128),
        "blue" => (0, 0, 255),
        "teal" => (0, 128, 128),
        "aqua" | "cyan" => (0, 255, 255),
        "aliceblue" => (240, 248, 255),
        "antiquewhite" => (250, 235, 215),
        "aquamarine" => (127, 255, 212),
        "azure" => (240, 255, 255),
        "beige" => (245, 245, 220),
        "bisque" => (255, 228, 196),
        "blanchedalmond" => (255, 235, 205),
        "blueviolet" => (138, 43, 226),
        "brown" => (165, 42, 42),
        "burlywood" => (222, 184, 135),
        "cadetblue" => (95, 158, 160),
        "chartreuse" => (127, 255, 0),
        "chocolate" => (210, 105, 30),
        "coral" => (255, 127, 80),
        "cornflowerblue" => (100, 149, 237),
        "cornsilk" => (255, 248, 220),
        "crimson" => (220, 20, 60),
        "darkblue" => (0, 0, 139),
        "darkcyan" => (0, 139, 139),
        "darkgoldenrod" => (184, 134, 11),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "darkgreen" => (0, 100, 0),
        "darkmagenta" => (139, 0, 139),
        "darkolivegreen" => (85, 107, 47),
        "darkorange" => (255, 140, 0),
        "darkorchid" => (153, 50, 204),
        "darkred" => (139, 0, 0),
        "darksalmon" => (233, 150, 122),
        "darkseagreen" => (143, 188, 143),
        "darkslateblue" => (72, 61, 139),
        "darkslategray" | "darkslategrey" => (47, 79, 79),
        "darkturquoise" => (0, 206, 209),
        "darkviolet" => (148, 0, 211),
        "deeppink" => (255, 20, 147),
        "deepskyblue" => (0, 191, 255),
        "dimgray" | "dimgrey" => (105, 105, 105),
        "dodgerblue" => (30, 144, 255),
        "firebrick" => (178, 34, 34),
        "floralwhite" => (255, 250, 240),
        "forestgreen" => (34, 139, 34),
        "gainsboro" => (220, 220, 220),
        "ghostwhite" => (248, 248, 255),
        "gold" => (255, 215, 0),
        "goldenrod" => (218, 165, 32),
        "greenyellow" => (173, 255, 47),
        "honeydew" => (240, 255, 240),
        "hotpink" => (255, 105, 180),
        "indianred" => (205, 92, 92),
        "indigo" => (75, 0, 130),
        "ivory" => (255, 255, 240),
        "khaki" => (240, 230, 140),
        "lavender" => (230, 230, 250),
        "lavenderblush" => (255, 240, 245),
        "lawngreen" => (124, 252, 0),
        "lemonchiffon" => (255, 250, 205),
        "lightblue" => (173, 216, 230),
        "lightcoral" => (128, 128, 128),
        "lightcyan" => (224, 255, 255),
        "lightgoldenrodyellow" => (250, 250, 210),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "lightgreen" => (144, 238, 144),
        "lightpink" => (255, 182, 193),
        "lightsalmon" => (255, 160, 122),
        "lightseagreen" => (32, 178, 170),
        "lightskyblue" => (135, 206, 250),
        "lightslategray" | "lightslategrey" => (119, 136, 153),
        "lightsteelblue" => (176, 196, 222),
        "lightyellow" => (255, 255, 224),
        "limegreen" => (50, 205, 50),
        "linen" => (250, 240, 230),
        "mediumaquamarine" => (102, 205, 170),
        "mediumblue" => (0, 0, 205),
        "mediumorchid" => (186, 85, 211),
        "mediumpurple" => (147, 112, 219),
        "mediumseagreen" => (60, 179, 113),
        "mediumslateblue" => (123, 104, 238),
        "mediumspringgreen" => (0, 250, 154),
        "mediumturquoise" => (72, 209, 204),
        "mediumvioletred" => (199, 21, 133),
        "midnightblue" => (25, 25, 112),
        "mintcream" => (245, 255, 250),
        "mistyrose" => (255, 228, 225),
        "moccasin" => (255, 228, 181),
        "navajowhite" => (255, 222, 173),
        "oldlace" => (253, 245, 230),
        "olivedrab" => (107, 142, 35),
        "orange" => (255, 165, 0),
        "orangered" => (255, 69, 0),
        "orchid" => (218, 112, 214),
        "palegoldenrod" => (238, 232, 170),
        "palegreen" => (152, 251, 152),
        "paleturquoise" => (175, 238, 238),
        "palevioletred" => (219, 112, 147),
        "papayawhip" => (255, 239, 213),
        "peachpuff" => (255, 218, 185),
        "peru" => (205, 133, 63),
        "pink" => (255, 192, 203),
        "plum" => (221, 160, 221),
        "powderblue" => (176, 224, 230),
        "rebeccapurple" => (102, 51, 153),
        "rosybrown" => (188, 143, 143),
        "royalblue" => (65, 105, 225),
        "saddlebrown" => (139, 69, 19),
        "salmon" => (250, 128, 114),
        "sandybrown" => (244, 164, 96),
        "seagreen" => (46, 139, 87),
        "seashell" => (255, 245, 238),
        "sienna" => (160, 82, 45),
        "skyblue" => (135, 206, 235),
        "slateblue" => (106, 90, 205),
        "slategray" | "slategrey" => (112, 128, 144),
        "snow" => (255, 250, 250),
        "springgreen" => (0, 255, 127),
        "steelblue" => (70, 130, 180),
        "tan" => (210, 180, 140),
        "thistle" => (216, 191, 216),
        "tomato" => (255, 99, 71),
        "turquoise" => (64, 224, 208),
        "violet" => (238, 130, 238),
        "wheat" => (245, 222, 179),
        "whitesmoke" => (245, 245, 245),
        "yellowgreen" => (154, 205, 50),
        "transparent" => (255, 255, 255),
        _ => return None,
    };
    Some(named)
}

fn extract_declaration(style: &str, name: &str) -> Option<String> {
    for decl in style.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        let colon = decl.find(':')?;
        let prop = decl[..colon].trim();
        if prop.eq_ignore_ascii_case(name) {
            let val = decl[colon + 1..].trim();
            return Some(val.to_string());
        }
    }
    None
}

fn is_large_text(style: &str) -> bool {
    let fs = extract_declaration(style, "font-size");
    let fw = extract_declaration(style, "font-weight");

    let size_pt = fs.as_deref().and_then(|v| {
        let v = v.trim();
        if let Ok(num) = v.trim_end_matches("px").trim().parse::<f64>() {
            Some(num * 0.75)
        } else if let Ok(num) = v.trim_end_matches("pt").trim().parse::<f64>() {
            Some(num)
        } else if let Ok(num) = v.trim_end_matches("em").trim().parse::<f64>() {
            Some(num * 12.0)
        } else if let Ok(num) = v.trim_end_matches("rem").trim().parse::<f64>() {
            Some(num * 16.0)
        } else if let Ok(num) = v.trim_end_matches("%").trim().parse::<f64>() {
            Some(num * 0.16)
        } else {
            None
        }
    }).unwrap_or(16.0);

    let bold = fw.as_deref().map_or(false, |v| {
        let v = v.trim().to_lowercase();
        v == "bold" || v == "bolder" || v == "700" || v == "800" || v == "900"
    });

    size_pt >= 18.0 || (size_pt >= 14.0 && bold)
}

// ---------------------------------------------------------------------------
// CSS <style> block parser + simple selector matching
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CssRule {
    raw_selectors: Vec<String>,
    declarations: Vec<(String, String)>,
}

fn parse_style_rules(css: &str) -> Vec<CssRule> {
    let mut rules = Vec::new();
    let mut remaining = css.trim();

    while let Some(open) = remaining.find('{') {
        let before = remaining[..open].trim();
        let mut depth = 1u32;
        let mut close = open + 1;
        while depth > 0 && close < remaining.len() {
            match remaining.as_bytes()[close] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            close += 1;
        }
        let body = &remaining[open + 1..close - 1];

        for selector in before.split(',') {
            let sel = selector.trim();
            if sel.is_empty() || sel.starts_with("@media") || sel.starts_with('@') {
                continue;
            }
            let mut decls = Vec::new();
            for d in body.split(';') {
                let d = d.trim();
                if d.is_empty() {
                    continue;
                }
                if let Some(col) = d.find(':') {
                    let prop = d[..col].trim();
                    let val = d[col + 1..].trim();
                    if !prop.is_empty() && !val.is_empty() {
                        decls.push((prop.to_lowercase(), val.to_string()));
                    }
                }
            }
            if !decls.is_empty() {
                let parts: Vec<String> = sel.split_whitespace().map(|s| s.to_string()).collect();
                rules.push(CssRule {
                    raw_selectors: parts,
                    declarations: decls,
                });
            }
        }

        remaining = remaining[close..].trim();
    }

    rules
}

fn selector_matches_element(selector_parts: &[String], el: &ElementRef) -> bool {
    if selector_parts.is_empty() {
        return false;
    }
    let last = &selector_parts[selector_parts.len() - 1];
    let simple = last.trim_start_matches(':');
    let tag = el.value().name();

    if simple.starts_with('.') {
        let cls = &simple[1..];
        if !el.value().attr("class").map_or(false, |c| c.split_whitespace().any(|p| p.eq_ignore_ascii_case(cls))) {
            return false;
        }
    } else if simple.starts_with('#') {
        let id = &simple[1..];
        if el.value().attr("id") != Some(id) {
            return false;
        }
    } else if simple.starts_with('[') {
        let attr_end = simple.find('=').or_else(|| simple.find(']')).unwrap_or(simple.len() - 1);
        let attr_name = &simple[1..attr_end];
        if el.value().attr(attr_name).is_none() {
            return false;
        }
    } else if simple != tag && simple != "body" && simple != "html" && simple != "*" {
        return false;
    }

    if selector_parts.len() > 1 {
        let parent_sel = &selector_parts[..selector_parts.len() - 1];
        let parent = el.parent().and_then(|n| ElementRef::wrap(n));
        match parent {
            Some(p) if selector_matches_element(parent_sel, &p) => {}
            _ => return selector_parts.len() == 1,
        }
    }

    true
}

fn collect_style_rules(document: &Html) -> Vec<CssRule> {
    let style_sel = Selector::parse("style").unwrap();
    let mut all_rules = Vec::new();
    for style_el in document.select(&style_sel) {
        let css = style_el.text().collect::<String>();
        if !css.trim().is_empty() {
            all_rules.extend(parse_style_rules(&css));
        }
    }
    all_rules
}

fn matching_declarations<'a>(el: &ElementRef, rules: &'a [CssRule]) -> Vec<&'a (String, String)> {
    let mut decls = Vec::new();
    for rule in rules {
        if selector_matches_element(&rule.raw_selectors, el) {
            for d in &rule.declarations {
                decls.push(d);
            }
        }
    }
    decls
}

fn find_decl<'a>(decls: &[&'a (String, String)], name: &str) -> Option<&'a str> {
    decls.iter().find(|d| d.0 == name).map(|d| d.1.as_str())
}

// ---------------------------------------------------------------------------
// DOM tree walking for inherited colors
// ---------------------------------------------------------------------------

fn inherited_color(el: &ElementRef, rules: &[CssRule]) -> Option<String> {
    let mut current = el.parent().and_then(|n| ElementRef::wrap(n));
    while let Some(ref ce) = current {
        if let Some(inline) = ce.value().attr("style").and_then(|s| extract_declaration(s, "color")) {
            if parse_color_value(&inline).is_some() {
                return Some(inline);
            }
        }
        let matched = matching_declarations(ce, rules);
        if let Some(val) = find_decl(&matched, "color") {
            if parse_color_value(val).is_some() {
                return Some(val.to_string());
            }
        }
        current = ce.parent().and_then(|n| ElementRef::wrap(n));
    }
    None
}

fn inherited_bg(el: &ElementRef, rules: &[CssRule]) -> Option<String> {
    let mut current = el.parent().and_then(|n| ElementRef::wrap(n));
    while let Some(ref ce) = current {
        if let Some(inline) = ce.value().attr("style").and_then(|s| extract_declaration(s, "background-color")) {
            let v = inline.trim().to_lowercase();
            if v != "transparent" && parse_color_value(&inline).is_some() {
                return Some(inline);
            }
        }
        let matched = matching_declarations(ce, rules);
        if let Some(val) = find_decl(&matched, "background-color") {
            let v = val.trim().to_lowercase();
            if v != "transparent" && parse_color_value(val).is_some() {
                return Some(val.to_string());
            }
        }
        current = ce.parent().and_then(|n| ElementRef::wrap(n));
    }
    None
}

fn has_bg_image(el: &ElementRef, rules: &[CssRule]) -> bool {
    if let Some(inline) = el.value().attr("style") {
        if let Some(bgi) = extract_declaration(inline, "background-image") {
            let v = bgi.trim().to_lowercase();
            if v != "none" && !v.is_empty() {
                return true;
            }
        }
    }
    let matched = matching_declarations(el, rules);
    if let Some(val) = find_decl(&matched, "background-image") {
        let v = val.trim().to_lowercase();
        return v != "none" && !v.is_empty();
    }
    false
}

// ---------------------------------------------------------------------------
// Theme detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum DetectedTheme {
    Light,
    Dark,
}

fn detect_themes(document: &Html) -> Vec<DetectedTheme> {
    let mut themes = Vec::new();
    let style_sel = Selector::parse("style").unwrap();

    for style_el in document.select(&style_sel) {
        let css = style_el.text().collect::<String>();
        let lower = css.to_lowercase();
        if lower.contains("@media (prefers-color-scheme: dark)") || lower.contains("@media(prefers-color-scheme:dark)") {
            if !themes.contains(&DetectedTheme::Dark) {
                themes.push(DetectedTheme::Dark);
            }
        }
        if lower.contains("@media (prefers-color-scheme: light)") || lower.contains("@media(prefers-color-scheme:light)") {
            if !themes.contains(&DetectedTheme::Light) {
                themes.push(DetectedTheme::Light);
            }
        }
    }

    let html_sel = Selector::parse("html, body").unwrap();
    for el in document.select(&html_sel) {
        if let Some(cls) = el.value().attr("class") {
            let lower = cls.to_lowercase();
            if lower.contains("theme-dark") || lower.contains("dark-theme") || lower.contains("dark-mode") || lower == "dark" {
                if !themes.contains(&DetectedTheme::Dark) {
                    themes.push(DetectedTheme::Dark);
                }
            }
            if lower.contains("theme-light") || lower.contains("light-theme") || lower.contains("light-mode") || lower == "light" {
                if !themes.contains(&DetectedTheme::Light) {
                    themes.push(DetectedTheme::Light);
                }
            }
        }
        if let Some(data) = el.value().attr("data-theme") {
            match data.trim().to_lowercase().as_str() {
                "dark" if !themes.contains(&DetectedTheme::Dark) => themes.push(DetectedTheme::Dark),
                "light" if !themes.contains(&DetectedTheme::Light) => themes.push(DetectedTheme::Light),
                _ => {}
            }
        }
    }

    if themes.is_empty() {
        themes.push(DetectedTheme::Light);
        themes.push(DetectedTheme::Dark);
    }

    themes
}

fn theme_default_bg(theme: DetectedTheme) -> (u8, u8, u8) {
    match theme {
        DetectedTheme::Light => (255, 255, 255),
        DetectedTheme::Dark => (17, 17, 17),
    }
}

fn theme_default_fg(theme: DetectedTheme) -> (u8, u8, u8) {
    match theme {
        DetectedTheme::Light => (0, 0, 0),
        DetectedTheme::Dark => (232, 232, 232),
    }
}

fn theme_label(theme: DetectedTheme) -> &'static str {
    match theme {
        DetectedTheme::Light => "light mode",
        DetectedTheme::Dark => "dark mode",
    }
}

// ---------------------------------------------------------------------------
// Resolved style per element
// ---------------------------------------------------------------------------

struct ResolvedStyle {
    color: Option<(u8, u8, u8)>,
    bg: Option<(u8, u8, u8)>,
    has_bg_image: bool,
}

fn resolve_style(
    el: &ElementRef,
    rules: &[CssRule],
    _theme: DetectedTheme,
) -> ResolvedStyle {
    let inline_style = el.value().attr("style").unwrap_or("");

    // Inline
    let inline_color = extract_declaration(inline_style, "color").and_then(|v| parse_color_value(&v));
    let inline_bg = extract_declaration(inline_style, "background-color").and_then(|v| parse_color_value(&v));

    // Matched rules
    let matched = matching_declarations(el, rules);
    let rule_color = find_decl(&matched, "color").and_then(parse_color_value);
    let rule_bg = find_decl(&matched, "background-color").and_then(parse_color_value);

    // Inherited
    let inherited_color_val = if inline_color.is_none() && rule_color.is_none() {
        inherited_color(el, rules).and_then(|v| parse_color_value(&v))
    } else {
        None
    };

    let inherited_bg_val = if inline_bg.is_none() && rule_bg.is_none() {
        let candidate = inherited_bg(el, rules).and_then(|v| parse_color_value(&v));
        if candidate.is_none() {
            // Try <html> direct
            let html_sel = Selector::parse("html").unwrap();
            if let Some(html_el) = el.select(&html_sel).next() {
                html_el.value().attr("style").and_then(|s| extract_declaration(s, "background-color")).and_then(|v| parse_color_value(&v))
            } else {
                None
            }
        } else {
            candidate
        }
    } else {
        None
    };

    let color = inline_color.or(rule_color).or(inherited_color_val);
    let bg = inline_bg.or(rule_bg).or(inherited_bg_val);

    let bg_image = has_bg_image(el, rules);

    ResolvedStyle { color, bg, has_bg_image: bg_image }
}

// ---------------------------------------------------------------------------
// Updated analyze function
// ---------------------------------------------------------------------------

pub async fn analyze(html: &str) -> Vec<Finding> {
    let document = Html::parse_document(html);
    let mut findings = Vec::new();

    alt_text_audit(&document, &mut findings);
    heading_structure_audit(&document, &mut findings);
    aria_audit(&document, &mut findings);
    landmark_audit(&document, &mut findings);
    form_label_audit(&document, &mut findings);
    keyboard_audit(&document, &mut findings);
    link_text_audit(&document, &mut findings);
    table_audit(&document, &mut findings);
    iframe_audit(&document, &mut findings);
    viewport_audit(&document, &mut findings);
    language_audit(&document, &mut findings);
    color_contrast_audit(&document, &mut findings);
    media_transcript_audit(&document, &mut findings);
    focus_indicator_audit(&document, &mut findings);

    findings
}

// ---------------------------------------------------------------------------
// Original audits (unchanged from existing code)
// ---------------------------------------------------------------------------

fn alt_text_audit(document: &Html, findings: &mut Vec<Finding>) {
    let img_sel = Selector::parse("img").unwrap();
    let mut examples: Vec<ElementRef> = Vec::new();

    for img in document.select(&img_sel) {
        if img.value().attr("alt").is_none() {
            examples.push(img);
            if examples.len() >= 5 {
                break;
            }
        }
    }

    if examples.is_empty() {
        return;
    }

    let total = document
        .select(&img_sel)
        .filter(|img| img.value().attr("alt").is_none())
        .count();

    let detail_lines: Vec<String> = examples
        .iter()
        .map(|el| {
            let src = el.value().attr("src").unwrap_or("");
            format!("  · `{}` src=\"{}\"", el_snippet(el).unwrap_or_default(), truncate(src, 80))
        })
        .collect();

    findings.push(Finding {
        category: "accessibility".to_string(),
        check: "alt_text".to_string(),
        severity: Severity::Error,
        title: format!("{} image(s) missing alt text", total),
        description: format!(
            "Found {} <img> element(s) without an alt attribute. Screen readers cannot describe image content to blind or low-vision users without alt text, making these images inaccessible.\n\nExamples of missing alt:\n{}\n\nRecommendations:\n  · Informative images: alt=\"brief description of the content\"\n  · Decorative images: alt=\"\" (empty string) to hide from assistive technology\n  · Functional images (icon links): alt=\"link purpose\"\n  · Never omit the alt attribute entirely\n\nReference: WCAG 2.2 Success Criterion 1.1.1 Non-text Content (Level A)",
            total,
            detail_lines.join("\n"),
        ),
        snippet: examples.first().and_then(el_snippet),
        page_url: None,
    });
}

fn heading_structure_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    let headings: Vec<ElementRef> = document.select(&sel).collect();

    if headings.is_empty() {
        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "headings_outline".to_string(),
            severity: Severity::Error,
            title: "Page has no heading structure".into(),
            description: format!(
                "The document has no heading elements (h1–h6). Screen reader users rely on headings to navigate and understand the page outline — without them every section sounds like a flat wall of text.\n\nRecommendation: Add a logical hierarchy:\n  1. One <h1> for the page title\n  2. <h2> for each major section\n  3. <h3> for subsections, etc.\n\nReference: WCAG 2.2 Success Criterion 1.3.1 Info and Relationships (Level A), 2.4.10 Section Headings (Level AAA)"
            ),
            snippet: None,
            page_url: None,
        });
        return;
    }

    let outline: Vec<String> = headings
        .iter()
        .map(|h| {
            let tag = h.value().name();
            let text: String = h.text().collect::<Vec<_>>().concat().trim().to_string();
            let display = if text.is_empty() { "(empty)" } else { &text };
            format!("  <{}>{}</{}>", tag, display, tag)
        })
        .collect();

    let mut prev: u32 = 0;
    let mut skip = false;
    for h in &headings {
        let name = h.value().name();
        if let Ok(n) = name[1..].parse::<u32>() {
            if n > prev + 1 {
                skip = true;
            }
            prev = n;
        }
    }

    if skip {
        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "headings_outline".to_string(),
            severity: Severity::Warning,
            title: "Heading hierarchy skips one or more levels".into(),
            description: format!(
                "The heading outline jumps over levels (e.g., h1 → h3 with no h2). This breaks the logical document structure for screen reader users who navigate by heading level.\n\nCurrent heading outline:\n{}\n\nRecommendation: Ensure headings form a strict hierarchy without gaps. Every h3 should be preceded by an h2, every h2 by an h1, etc.\n\nReference: WCAG 2.2 Success Criterion 1.3.1 Info and Relationships (Level A), 2.4.6 Headings and Labels (Level AA)",
                outline.join("\n"),
            ),
            snippet: Some(outline.join("\n")),
            page_url: None,
        });
    }
}

fn aria_audit(document: &Html, findings: &mut Vec<Finding>) {
    let role_btn = Selector::parse("[role=button]").unwrap();
    for el in document.select(&role_btn) {
        if el.value().name() != "button" && el.value().attr("tabindex").is_none() {
            findings.push(Finding {
                category: "accessibility".to_string(),
                check: "aria_roles".to_string(),
                severity: Severity::Error,
                title: "role='button' on non-interactive element without tabindex".into(),
                description: format!(
                    "A `<{}>` element has role=\"button\" but is not natively interactive and lacks a tabindex attribute. Keyboard users cannot focus or activate this element — it is effectively invisible to assistive technology.\n\nRecommendation: Add tabindex=\"0\" to make it keyboard-focusable, or better, use a native <button> which provides focus and activation semantics automatically.\n\nReference: WCAG 2.2 Success Criterion 4.1.2 Name, Role, Value (Level A)",
                    el.value().name(),
                ),
                snippet: el_snippet(&el),
                page_url: None,
            });
        }
    }

    let role_link = Selector::parse("[role=link]").unwrap();
    for el in document.select(&role_link) {
        if el.value().name() != "a" && el.value().attr("tabindex").is_none() {
            findings.push(Finding {
                category: "accessibility".to_string(),
                check: "aria_roles".to_string(),
                severity: Severity::Error,
                title: "role='link' on non-anchor element without tabindex".into(),
                description: format!(
                    "A `<{}>` element has role=\"link\" but no tabindex. It is not reachable via keyboard navigation and assistive technology may not present it as an interactive control.\n\nRecommendation: Add tabindex=\"0\" or use a native <a href=\"…\"> element.\n\nReference: WCAG 2.2 Success Criterion 4.1.2 Name, Role, Value (Level A)",
                    el.value().name(),
                ),
                snippet: el_snippet(&el),
                page_url: None,
            });
        }
    }

    let aria_sel = Selector::parse("[role], [aria-label], [aria-labelledby], [aria-describedby]").unwrap();
    let total = document.select(&aria_sel).count();
    if total == 0 {
        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "aria_usage".to_string(),
            severity: Severity::Info,
            title: "No ARIA attributes detected on the page".into(),
            description: format!(
                "The page does not use any ARIA attributes (role, aria-label, aria-labelledby, aria-describedby). While this is acceptable for simple static content, complex interactive widgets must use ARIA to communicate semantics, states, and properties to assistive technology.\n\nReference: WAI-ARIA 1.2 Authoring Practices"
            ),
            snippet: None,
            page_url: None,
        });
    }
}

fn landmark_audit(document: &Html, findings: &mut Vec<Finding>) {
    let landmarks: [(&str, Selector); 6] = [
        ("<header> or role=\"banner\"", Selector::parse("header, [role=banner]").unwrap()),
        ("<nav> or role=\"navigation\"", Selector::parse("nav, [role=navigation]").unwrap()),
        ("<main> or role=\"main\"", Selector::parse("main, [role=main]").unwrap()),
        ("<footer> or role=\"contentinfo\"", Selector::parse("footer, [role=contentinfo]").unwrap()),
        ("<aside> or role=\"complementary\"", Selector::parse("aside, [role=complementary]").unwrap()),
        ("<form> with aria-label or role=\"search\"", Selector::parse("form[aria-label], form[aria-labelledby], [role=search]").unwrap()),
    ];

    let mut present: Vec<&str> = Vec::new();
    let mut absent: Vec<&str> = Vec::new();

    for (label, sel) in &landmarks {
        if document.select(sel).next().is_some() {
            present.push(label);
        } else {
            absent.push(label);
        }
    }

    if present.is_empty() {
        let link_sel = Selector::parse("a[href]").unwrap();
        let link_count = document.select(&link_sel).count();
        let nav_note = if link_count >= 3 {
            " The page contains several links — consider wrapping navigation in a <nav> element."
        } else {
            ""
        };

        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "landmarks".to_string(),
            severity: Severity::Warning,
            title: "No landmark regions found on the page".into(),
            description: format!(
                "The page uses none of the HTML5 landmark elements or ARIA landmark roles. Screen reader users cannot quickly jump to key sections — they must manually tab through the entire page.\n\nRecommendation: Add landmark elements:\n  · <header> for the site header / banner\n  · <nav> for navigation menus{}\n  · <main> for primary content\n  · <footer> for footer / contentinfo\n  · <aside> for complementary content\n\nReference: WCAG 2.2 Success Criterion 1.3.1 Info and Relationships (Level A)",
                nav_note,
            ),
            snippet: None,
            page_url: None,
        });
    } else if !absent.is_empty() {
        let link_sel = Selector::parse("a[href]").unwrap();
        let link_count = document.select(&link_sel).count();
        let has_nav = present.iter().any(|s| s.contains("<nav>"));
        let nav_note = if !has_nav && link_count >= 3 {
            "\n  · Consider adding <nav> — the page has multiple navigation links"
        } else {
            ""
        };

        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "landmarks".to_string(),
            severity: Severity::Info,
            title: "Some landmark regions are missing".into(),
            description: format!(
                "Present landmarks:\n  · {}\n\nMissing landmarks:\n  · {}{}\n\nReference: WCAG 2.2 Success Criterion 1.3.1 Info and Relationships (Level A)",
                present.join("\n  · "),
                absent.join("\n  · "),
                nav_note,
            ),
            snippet: None,
            page_url: None,
        });
    }
}

fn form_label_audit(document: &Html, findings: &mut Vec<Finding>) {
    let control_sel = Selector::parse(
        "input:not([type=hidden]):not([type=submit]):not([type=button]):not([type=reset]), select, textarea",
    )
    .unwrap();
    let label_for_sel = Selector::parse("label[for]").unwrap();

    let for_ids: Vec<String> = document
        .select(&label_for_sel)
        .filter_map(|el| el.value().attr("for").map(String::from))
        .collect();

    let mut examples: Vec<ElementRef> = Vec::new();
    let mut total = 0u32;

    for ctrl in document.select(&control_sel) {
        let id = ctrl.value().attr("id").map(String::from);
        let linked = id.as_ref().map_or(false, |id| for_ids.contains(id));
        if linked {
            continue;
        }

        let mut wrapped = false;
        let mut current = ctrl.parent();
        while let Some(node) = current {
            match node.value() {
                Node::Element(e) if e.name() == "label" => {
                    wrapped = true;
                    break;
                }
                Node::Element(_) => {
                    current = node.parent();
                }
                _ => break,
            }
        }
        if wrapped {
            continue;
        }

        if ctrl.value().attr("aria-label").is_some() || ctrl.value().attr("aria-labelledby").is_some() {
            continue;
        }

        total += 1;
        if examples.len() < 5 {
            examples.push(ctrl);
        }
    }

    if total == 0 {
        return;
    }

    let detail: Vec<String> = examples
        .iter()
        .map(|el| {
            let html = el_snippet(el).unwrap_or_default();
            let name = el.value().name();
            let type_attr = el.value().attr("type").unwrap_or("");
            if name == "input" && !type_attr.is_empty() {
                format!("  · `{}` (type=\"{}\")", html, type_attr)
            } else {
                format!("  · `<{}>` {}", name, html)
            }
        })
        .collect();

    findings.push(Finding {
        category: "accessibility".to_string(),
        check: "form_labels".to_string(),
        severity: Severity::Error,
        title: format!("{} form control(s) without an accessible label", total),
        description: format!(
            "Found {} form control(s) that are not associated with a <label> (via for/id), not wrapped in a <label>, and lack aria-label / aria-labelledby. Screen readers cannot announce the purpose of these controls, making the form unusable for blind users.\n\nExamples of unlabeled controls:\n{}\n\nRecommendation:\n  · Add <label for=\"id\"> referencing the control's id\n  · Wrap the control in a <label> element\n  · Or add aria-label=\"Purpose\" or aria-labelledby=\"id\"\n\nReference: WCAG 2.2 Success Criterion 3.3.2 Labels or Instructions (Level A), 4.1.2 Name, Role, Value (Level A)",
            total,
            detail.join("\n"),
        ),
        snippet: examples.first().and_then(el_snippet),
        page_url: None,
    });
}

fn keyboard_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("[tabindex]").unwrap();
    let mut positive: Vec<ElementRef> = Vec::new();

    for el in document.select(&sel) {
        if let Some(val) = el.value().attr("tabindex") {
            if let Ok(n) = val.parse::<i32>() {
                if n > 0 {
                    positive.push(el);
                    if positive.len() >= 5 {
                        break;
                    }
                }
            }
        }
    }

    if positive.is_empty() {
        return;
    }

    let lines: Vec<String> = positive
        .iter()
        .map(|el| {
            let t = el.value().attr("tabindex").unwrap_or("");
            format!("  · `{}` (tabindex=\"{}\")", el_snippet(el).unwrap_or_default(), t)
        })
        .collect();

    findings.push(Finding {
        category: "accessibility".to_string(),
        check: "keyboard_nav".to_string(),
        severity: Severity::Warning,
        title: format!("{} element(s) with positive tabindex values found", positive.len()),
        description: format!(
            "Positive tabindex values (tabindex=\"1\", \"2\", etc.) force elements into a custom focus order that overrides the natural DOM sequence. This creates a confusing navigation experience for keyboard-only users.\n\nAffected elements:\n{}\n\nRecommendation:\n  · Use tabindex=\"0\" to make an element focusable in DOM order\n  · Use tabindex=\"-1\" to make it script-focusable but removed from tab order\n  · Never use tabindex=\"1+\" — reorder the HTML source instead\n\nReference: WCAG 2.2 Success Criterion 2.4.3 Focus Order (Level A)",
            lines.join("\n"),
        ),
        snippet: positive.first().and_then(el_snippet),
        page_url: None,
    });
}

fn link_text_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("a[href]").unwrap();
    let generic = [
        "click here", "here", "read more", "more", "learn more", "this",
        "link", "go", "details", "info", "continue", "start",
    ];

    for link in document.select(&sel) {
        let text: String = link.text().collect::<Vec<_>>().join(" ").trim().to_lowercase();
        if text.len() < 2 {
            continue;
        }
        let is_generic = generic.iter().any(|g| text == *g || text.starts_with(&format!("{} ", g)));
        if !is_generic {
            continue;
        }

        let href = link.value().attr("href").unwrap_or("");
        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "link_text".to_string(),
            severity: Severity::Warning,
            title: "Link text is generic or non-descriptive".into(),
            description: format!(
                "Link text \"{}\" (href=\"{}\") is too vague. Screen reader users often navigate by cycling through links or pulling up a list of all links — generic text gives no clue about the destination.\n\nRecommendation: Replace with descriptive text that makes sense out of context, e.g., \"View complete audit report\" instead of \"Click here\".\n\nReference: WCAG 2.2 Success Criterion 2.4.4 Link Purpose (In Context) (Level A)",
                text,
                truncate(href, 100),
            ),
            snippet: el_snippet(&link),
            page_url: None,
        });
    }
}

fn table_audit(document: &Html, findings: &mut Vec<Finding>) {
    let table_sel = Selector::parse("table").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let caption_sel = Selector::parse("caption").unwrap();
    let scope_sel = Selector::parse("th[scope]").unwrap();

    for table in document.select(&table_sel) {
        let fragment = Html::parse_fragment(&table.inner_html());

        let th_count = fragment.select(&th_sel).count();
        if th_count == 0 {
            continue;
        }

        let has_caption = fragment.select(&caption_sel).next().is_some();
        let th_with_scope = fragment.select(&scope_sel).count();
        let missing_scope = th_count > th_with_scope;

        let table_html = el_snippet(&table);

        if !has_caption {
            findings.push(Finding {
                category: "accessibility".to_string(),
                check: "tables".to_string(),
                severity: Severity::Info,
                title: "Data table lacks a <caption>".into(),
                description: format!(
                    "A data <table> with {} header cell(s) (<th>) has no <caption>. Screen readers announce the caption before entering the table, giving users context about its content and purpose.\n\nRecommendation: Add <caption>Brief summary of the table</caption> as the first child of <table>.\n\nReference: WCAG 2.2 Success Criterion 1.3.1 Info and Relationships (Level A)",
                    th_count,
                ),
                snippet: table_html.clone(),
                page_url: None,
            });
        }

        if missing_scope {
            findings.push(Finding {
                category: "accessibility".to_string(),
                check: "tables".to_string(),
                severity: Severity::Warning,
                title: "Table <th> cells missing scope attribute".into(),
                description: format!(
                    "This <table> contains {} <th> element(s), but only {} have a scope attribute. Without scope=\"col\" or scope=\"row\", screen readers may fail to associate headers with their corresponding data cells, especially in complex tables.\n\nRecommendation: Add scope=\"col\" to column headers and scope=\"row\" to row headers.\n\nReference: WCAG 2.2 Success Criterion 1.3.1 Info and Relationships (Level A)",
                    th_count,
                    th_with_scope,
                ),
                snippet: table_html,
                page_url: None,
            });
        }
    }
}

fn iframe_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("iframe").unwrap();

    for iframe in document.select(&sel) {
        let src = iframe.value().attr("src").unwrap_or("");
        if src.starts_with("data:") {
            continue;
        }

        let title = iframe.value().attr("title");
        let has_title = title.map_or(false, |t| !t.trim().is_empty());

        if !has_title {
            findings.push(Finding {
                category: "accessibility".to_string(),
                check: "iframes".to_string(),
                severity: Severity::Warning,
                title: "Iframe missing or has an empty title attribute".into(),
                description: format!(
                    "An <iframe> with src=\"{}\" has no title attribute (or the title is empty). Screen readers announce the title when entering the iframe; without it users have no context about the embedded content.\n\nRecommendation: Add title=\"Description of the iframe content\" to the <iframe>.\n\nReference: WCAG 2.2 Success Criterion 2.4.1 Bypass Blocks (Level A), 4.1.2 Name, Role, Value (Level A)",
                    truncate(src, 100),
                ),
                snippet: el_snippet(&iframe),
                page_url: None,
            });
        }
    }
}

fn viewport_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("meta[name=viewport]").unwrap();

    match document.select(&sel).next() {
        Some(el) => {
            let content = el.value().attr("content").unwrap_or("");
            let zoom_blocked = content.contains("user-scalable=no")
                || content.contains("maximum-scale=1")
                || content.contains("maximum-scale=1.0");

            if zoom_blocked {
                findings.push(Finding {
                    category: "accessibility".to_string(),
                    check: "viewport".to_string(),
                    severity: Severity::Warning,
                    title: "Viewport meta tag prevents user zoom".into(),
                    description: format!(
                        "The viewport meta tag content \"{}\" disables pinch-to-zoom (user-scalable=no or maximum-scale=1.0). Low-vision users must be able to zoom text to at least 200% without loss of content or functionality.\n\nRecommendation: Remove user-scalable=no and set maximum-scale=5.0 or higher:\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\nReference: WCAG 2.2 Success Criterion 1.4.4 Resize Text (Level AA)",
                        truncate(content, 200),
                    ),
                    snippet: el_snippet(&el),
                    page_url: None,
                });
            }
        }
        None => {
            findings.push(Finding {
                category: "accessibility".to_string(),
                check: "viewport".to_string(),
                severity: Severity::Warning,
                title: "Missing viewport meta tag".into(),
                description: format!(
                    "The page does not include a <meta name=\"viewport\"> tag. Mobile browsers will render the page at a desktop width and shrink it, making text too small to read without manual zoom.\n\nRecommendation: Add to <head>:\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\nReference: WCAG 2.2 Success Criterion 1.4.4 Resize Text (Level AA)"
                ),
                snippet: None,
                page_url: None,
            });
        }
    }
}

fn language_audit(document: &Html, findings: &mut Vec<Finding>) {
    let sel = Selector::parse("html").unwrap();

    if let Some(el) = document.select(&sel).next() {
        let has_lang = el.value().attr("lang").is_some();
        let has_xml_lang = el.value().attr("xml:lang").is_some();

        if !has_lang && !has_xml_lang {
            findings.push(Finding {
                category: "accessibility".to_string(),
                check: "lang_attribute".to_string(),
                severity: Severity::Error,
                title: "<html> element missing lang attribute".into(),
                description: format!(
                    "The <html> tag lacks both lang and xml:lang attributes. Assistive technology cannot determine the page's language, which causes incorrect pronunciation, wrong voice selection, and braille translation errors.\n\nRecommendation: Add lang=\"en\" (or the appropriate language code) to the <html> element:\n  <html lang=\"en\">\n\nReference: WCAG 2.2 Success Criterion 3.1.1 Language of Page (Level A)"
                ),
                snippet: el_snippet(&el),
                page_url: None,
            });
        }
    }
}

fn media_transcript_audit(document: &Html, findings: &mut Vec<Finding>) {
    let video_sel = Selector::parse("video, audio").unwrap();
    let track_sel = Selector::parse("track").unwrap();
    let mut media_count = 0u32;
    let mut missing_captions = 0u32;
    let mut examples: Vec<String> = Vec::new();

    for el in document.select(&video_sel) {
        media_count += 1;
        let tag = el.value().name();
        let src = el.value().attr("src").unwrap_or("");
        let has_track = el.select(&track_sel).any(|t| {
            t.value()
                .attr("kind")
                .map_or(false, |k| k.eq_ignore_ascii_case("captions") || k.eq_ignore_ascii_case("subtitles"))
        });

        if !has_track {
            missing_captions += 1;
        }

        if examples.len() < 5 {
            examples.push(format!(
                "  · <{}> src=\"{}\" {}",
                tag,
                truncate(src, 80),
                if has_track { "✓ has <track>" } else { "✗ missing captions/subtitles" },
            ));
        }
    }

    if media_count == 0 {
        return;
    }

    if missing_captions > 0 {
        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "media_captions".to_string(),
            severity: Severity::Warning,
            title: format!("{} media element(s) missing captions or subtitles", missing_captions),
            description: format!(
                "Found {} <video>/<audio> element(s) that lack a <track> element with kind=\"captions\" or kind=\"subtitles\". Deaf and hard-of-hearing users cannot access audio content without synchronized text alternatives.\n\nAffected elements:\n{}\n\nRecommendation: Add a <track> element for each media source:\n  <track kind=\"captions\" src=\"captions.vtt\" srclang=\"en\" label=\"English\">\n\nReference: WCAG 2.2 Success Criterion 1.2.2 Captions (Prerecorded) (Level A), 1.2.4 Captions (Live) (Level AA)",
                missing_captions,
                examples.join("\n"),
            ),
            snippet: None,
            page_url: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Enhanced color contrast audit
// ---------------------------------------------------------------------------

fn color_contrast_audit(document: &Html, findings: &mut Vec<Finding>) {
    let text_sel = Selector::parse(
        "p, span, div, h1, h2, h3, h4, h5, h6, a, li, td, th, label, legend, blockquote, figcaption, cite, small, strong, em, b, i, u, code, pre",
    )
    .unwrap();
    let rules = collect_style_rules(document);
    let themes = detect_themes(document);
    let mut all_examples: Vec<(String, u8, u8, u8, u8, u8, u8, f64, bool)> = Vec::new();
    let mut bg_image_examples: Vec<String> = Vec::new();
    let mut unmeasurable_count = 0u32;

    for theme in &themes {
        for el in document.select(&text_sel) {
            let resolved = resolve_style(&el, &rules, *theme);
            let inline_style = el.value().attr("style").unwrap_or("");

            if resolved.has_bg_image {
                if bg_image_examples.len() < 5 {
                    let tag = el.value().name();
                    let text: String = el.text().collect::<Vec<_>>().concat().trim().to_string();
                    let preview = truncate(&text, 40);
                    let line = format!("  · <{}> \"{}\" (bg-image, cannot measure)", tag, preview);
                    if !bg_image_examples.contains(&line) {
                        bg_image_examples.push(line);
                    }
                }
                unmeasurable_count += 1;
                continue;
            }

            let fg = resolved.color.or_else(|| {
                let inherited = inherited_color(&el, &rules);
                inherited.as_ref().and_then(|v| parse_color_value(v))
            }).unwrap_or_else(|| theme_default_fg(*theme));

            let bg = resolved.bg.or_else(|| {
                let inherited = inherited_bg(&el, &rules);
                inherited.as_ref().and_then(|v| parse_color_value(v))
            }).unwrap_or_else(|| theme_default_bg(*theme));

            let ratio = contrast_ratio(fg, bg);
            let large = is_large_text(inline_style);

            let aa_pass = if large { ratio >= 3.0 } else { ratio >= 4.5 };

            if aa_pass {
                continue;
            }

            if all_examples.len() >= 10 {
                continue;
            }

            all_examples.push((
                theme_label(*theme).to_string(),
                fg.0, fg.1, fg.2, bg.0, bg.1, bg.2, ratio, large,
            ));
        }
    }

    if !bg_image_examples.is_empty() {
        findings.push(Finding {
            category: "accessibility".to_string(),
            check: "color_contrast_bg_image".to_string(),
            severity: Severity::Info,
            title: format!("{} element(s) use background-image — contrast cannot be measured statically", unmeasurable_count),
            description: format!(
                "Found {} text element(s) with a CSS `background-image`. Tengu's static analysis cannot extract the effective colour from an image, so contrast ratio cannot be computed for these elements. A browser-based audit is required.\n\nAffected elements (up to 5):\n{}\n\nReference: WCAG 2.2 Success Criterion 1.4.3 Contrast (Minimum) (Level AA)",
                unmeasurable_count,
                bg_image_examples.join("\n"),
            ),
            snippet: None,
            page_url: None,
        });
    }

    if all_examples.is_empty() {
        return;
    }

    let theme_groups: Vec<DetectedTheme> = themes.iter().filter(|t| {
        all_examples.iter().any(|(th, _, _, _, _, _, _, _, _)| th == theme_label(**t))
    }).copied().collect();

    let has_multi_theme = theme_groups.len() > 1;

    let detail: Vec<String> = all_examples
        .iter()
        .map(|(theme_tag, fr, fg, fb, br, bg, bb, ratio, large)| {
            let size_tag = if *large { " (large text)" } else { "" };
            let theme_prefix = if has_multi_theme { format!("[{}] ", theme_tag) } else { String::new() };
            format!(
                "  {}. #{:02x}{:02x}{:02x} on #{:02x}{:02x}{:02x} → ratio {:.2}:1{} — fails AA",
                theme_prefix, fr, fg, fb, br, bg, bb, ratio, size_tag,
            )
        })
        .collect();

    let theme_note = if has_multi_theme {
        let detected: Vec<String> = themes.iter().map(|t| theme_label(*t).to_string()).collect();
        format!("\n\nThemes detected: {}. Contrast checked for each theme independently.", detected.join(", "))
    } else {
        String::new()
    };

    let description = format!(
        "Found {} element(s) with insufficient color contrast between text and background.{} \
         \n\nExamples:\n{}\n\nThresholds (WCAG 2.2):\n  · AA normal text: 4.5:1\n  · AA large text (≥18pt or ≥14pt bold): 3:1\n  · AAA normal text: 7:1\n  · AAA large text: 4.5:1\n\n\
         Color sources checked (in order): inline `style` → matched CSS rules from `<style>` blocks → inherited from parent elements → \
         theme default (dark: #111111 bg / #E8E8E8 fg, light: #FFFFFF bg / #000000 fg).\n\n\
         For full CSS analysis including external stylesheets and `var()` resolution, use a browser-based tool.\n\n\
         Reference: WCAG 2.2 Success Criterion 1.4.3 Contrast (Minimum) (Level AA), 1.4.6 Contrast (Enhanced) (Level AAA)",
        all_examples.len(),
        theme_note,
        detail.join("\n"),
    );

    findings.push(Finding {
        category: "accessibility".to_string(),
        check: "color_contrast".to_string(),
        severity: Severity::Warning,
        title: format!("{} element(s) with insufficient color contrast", all_examples.len()),
        description,
        snippet: None,
        page_url: None,
    });
}

fn focus_indicator_audit(document: &Html, findings: &mut Vec<Finding>) {
    let focusable_sel = Selector::parse(
        "a[href], button, input:not([type=hidden]), select, textarea, [tabindex], [contenteditable]",
    )
    .unwrap();
    let mut examples: Vec<ElementRef> = Vec::new();

    for el in document.select(&focusable_sel) {
        let style_attr = match el.value().attr("style") {
            Some(s) if !s.trim().is_empty() => s,
            _ => continue,
        };

        let outline = extract_declaration(style_attr, "outline");
        let outline_none = outline.as_deref().map_or(false, |v| {
            let v = v.trim().to_lowercase();
            v == "none" || v == "0" || v.starts_with("none ") || v.starts_with("0 ")
        });

        if !outline_none {
            continue;
        }

        examples.push(el);
        if examples.len() >= 5 {
            break;
        }
    }

    if examples.is_empty() {
        return;
    }

    let lines: Vec<String> = examples
        .iter()
        .map(|el| {
            let tag = el.value().name();
            let id = el.value().attr("id").map(|v| format!("#{}", v)).unwrap_or_default();
            let cls = el
                .value()
                .attr("class")
                .map(|v| {
                    let c: Vec<&str> = v.split_whitespace().take(2).collect();
                    format!(".{}", c.join("."))
                })
                .unwrap_or_default();
            let snippet = el_snippet(el).unwrap_or_default();
            format!("  · <{}{}{}> {}", tag, id, cls, snippet)
        })
        .collect();

    findings.push(Finding {
        category: "accessibility".to_string(),
        check: "focus_indicator".to_string(),
        severity: Severity::Warning,
        title: format!("{} focusable element(s) have outline:none in inline style", examples.len()),
        description: format!(
            "Found {} focusable element(s) with outline:none or outline:0 set via inline style. Removing the focus outline makes it impossible for keyboard users to see which element is currently focused, violating WCAG focus visibility requirements.\n\nAffected elements:\n{}\n\nRecommendation: Replace outline:none with a visible focus style:\n  /* Preferred: custom focus ring */\n  outline: 2px solid #4A90D9;\n  outline-offset: 2px;\n\n  /* If you must remove the default outline, always provide a replacement */\n\nReference: WCAG 2.2 Success Criterion 2.4.7 Focus Visible (Level AA), 2.4.11 Focus Appearance (Level AA)",
            examples.len(),
            lines.join("\n"),
        ),
        snippet: examples.first().and_then(el_snippet),
        page_url: None,
    });
}
