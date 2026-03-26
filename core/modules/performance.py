"""
Performance Analysis Module.

Provides TTFB estimation, resource/payload breakdown, unoptimized image
detection, and Core Web Vitals proxy estimates (LCP, CLS risk, INP risk).

NOTE: True CWV require a real browser engine. The values produced here are
heuristic estimates based on static HTML analysis and server timing.
"""

import re
from urllib.parse import urlparse, urljoin
from typing import Dict, Any, List
from bs4 import BeautifulSoup
from core.utils.logger import logger


# ==================== TTFB ====================

def measure_ttfb(response_time_ms: int, html_size_bytes: int) -> Dict[str, Any]:
    """Estimate Time to First Byte from total response time and payload size.

    Rough model: TTFB ~ total_time - transfer_time
    We assume ~5 MB/s effective throughput for a typical connection.
    """
    THROUGHPUT_BYTES_PER_MS = 5000  # ~5 MB/s
    transfer_estimate_ms = max(1, round(html_size_bytes / THROUGHPUT_BYTES_PER_MS))
    ttfb_estimate = max(1, response_time_ms - transfer_estimate_ms)

    rating = "good" if ttfb_estimate < 200 else "needs_improvement" if ttfb_estimate < 600 else "poor"

    return {
        "ttfb_estimate_ms": ttfb_estimate,
        "transfer_estimate_ms": transfer_estimate_ms,
        "total_response_ms": response_time_ms,
        "html_size_bytes": html_size_bytes,
        "rating": rating,
    }


# ==================== RESOURCE BREAKDOWN ====================

def analyze_resources(html: str, url: str) -> Dict[str, Any]:
    """Analyze page resources: scripts, stylesheets, images, fonts, iframes."""
    soup = BeautifulSoup(html, "html.parser")

    js_sources: List[str] = []
    css_sources: List[str] = []
    image_sources: List[str] = []
    font_sources: List[str] = []
    iframe_sources: List[str] = []

    # Scripts
    for tag in soup.find_all("script", src=True):
        js_sources.append(tag["src"])

    # Inline scripts count
    inline_scripts = len([s for s in soup.find_all("script") if not s.get("src") and s.string])

    # Stylesheets
    for tag in soup.find_all("link", rel="stylesheet"):
        href = tag.get("href", "")
        if href:
            css_sources.append(href)

    # Inline styles count
    inline_styles = len(soup.find_all("style"))

    # Images — standard src, lazy-loaded (data-src, data-lazy-src, data-original), noscript
    LAZY_ATTRS = ["src", "data-src", "data-lazy-src", "data-original", "data-lazy", "data-srcset"]
    seen_images: set = set()
    for tag in soup.find_all("img"):
        for attr in LAZY_ATTRS:
            val = tag.get(attr, "")
            if val and val not in seen_images and not val.startswith("data:"):
                if attr == "data-srcset":
                    val = val.split(",")[0].strip().split(" ")[0]
                seen_images.add(val)
                image_sources.append(val)
                break

    # Picture sources
    for tag in soup.find_all("source"):
        srcset = tag.get("srcset", "")
        if srcset:
            first_src = srcset.split(",")[0].strip().split(" ")[0]
            if first_src and first_src not in seen_images:
                seen_images.add(first_src)
                image_sources.append(first_src)

    # Noscript fallback images
    for noscript in soup.find_all("noscript"):
        noscript_soup = BeautifulSoup(str(noscript), "html.parser")
        for tag in noscript_soup.find_all("img"):
            src = tag.get("src", "")
            if src and src not in seen_images and not src.startswith("data:"):
                seen_images.add(src)
                image_sources.append(src)

    # CSS background-image in inline styles
    bg_images: List[str] = []
    for tag in soup.find_all(style=True):
        style_val = tag.get("style", "")
        if "url(" in style_val:
            urls = re.findall(r'url\(["\']?([^"\')\s]+)["\']?\)', style_val)
            for u in urls:
                if u not in seen_images and not u.startswith("data:"):
                    seen_images.add(u)
                    bg_images.append(u)
                    image_sources.append(u)

    # CSS background-image in <style> blocks
    for style in soup.find_all("style"):
        if style.string and "url(" in style.string:
            urls = re.findall(r'url\(["\']?([^"\')\s]+)["\']?\)', style.string)
            for u in urls:
                ext = u.rsplit(".", 1)[-1].lower() if "." in u else ""
                if ext in ("jpg", "jpeg", "png", "gif", "webp", "avif", "svg", "ico", "bmp"):
                    if u not in seen_images:
                        seen_images.add(u)
                        bg_images.append(u)
                        image_sources.append(u)

    # Fonts — preloaded, @font-face, Google Fonts, CSS @import
    seen_fonts: set = set()
    for tag in soup.find_all("link", rel="preload"):
        as_val = tag.get("as", "")
        href = tag.get("href", "")
        if as_val == "font" and href and href not in seen_fonts:
            seen_fonts.add(href)
            font_sources.append(href)

    # Google Fonts / external font stylesheets
    for tag in soup.find_all("link", rel="stylesheet"):
        href = tag.get("href", "")
        if href and ("fonts.googleapis.com" in href or "fonts.gstatic.com" in href or "use.typekit.net" in href):
            if href not in seen_fonts:
                seen_fonts.add(href)
                font_sources.append(href)

    # @font-face in <style> blocks
    for style in soup.find_all("style"):
        if style.string and "@font-face" in style.string:
            urls = re.findall(r'url\(["\']?([^"\')\s]+)["\']?\)', style.string)
            for u in urls:
                if u not in seen_fonts:
                    seen_fonts.add(u)
                    font_sources.append(u)

    # CSS @import in <style> blocks (may load font stylesheets)
    for style in soup.find_all("style"):
        if style.string and "@import" in style.string:
            imports = re.findall(r'@import\s+(?:url\()?["\']?([^"\')\s;]+)["\']?\)?', style.string)
            for imp in imports:
                if "font" in imp.lower() and imp not in seen_fonts:
                    seen_fonts.add(imp)
                    font_sources.append(imp)

    # Iframes — standard src + lazy-loaded (data-src, data-lazy-src)
    IFRAME_ATTRS = ["src", "data-src", "data-lazy-src"]
    seen_iframes: set = set()
    for tag in soup.find_all("iframe"):
        for attr in IFRAME_ATTRS:
            val = tag.get(attr, "")
            if val and val not in seen_iframes and not val.startswith("about:"):
                seen_iframes.add(val)
                iframe_sources.append(val)
                break

    total_external = len(js_sources) + len(css_sources) + len(image_sources) + len(font_sources) + len(iframe_sources)
    html_size = len(html.encode("utf-8", errors="ignore"))

    return {
        "total_external_requests": total_external,
        "html_size_bytes": html_size,
        "js": {
            "external_count": len(js_sources),
            "inline_count": inline_scripts,
            "sources": js_sources[:30],
        },
        "css": {
            "external_count": len(css_sources),
            "inline_count": inline_styles,
            "sources": css_sources[:20],
        },
        "images": {
            "count": len(image_sources),
            "sources": image_sources[:40],
            "bg_images_count": len(bg_images),
        },
        "fonts": {
            "count": len(font_sources),
            "sources": font_sources[:15],
        },
        "iframes": {
            "count": len(iframe_sources),
            "sources": iframe_sources[:10],
        },
    }


# ==================== UNOPTIMIZED IMAGES ====================

MODERN_FORMATS = {".webp", ".avif", ".svg"}


def detect_unoptimized_images(html: str, page_url: str) -> Dict[str, Any]:
    """Detect images not using modern formats (WebP, AVIF, SVG).
    
    Checks: <img src/data-src/data-lazy-src>, <source srcset>, <noscript> fallbacks,
    and CSS background-image in inline styles.
    """
    soup = BeautifulSoup(html, "html.parser")
    unoptimized: List[Dict[str, str]] = []
    all_sources: List[str] = []
    seen: set = set()

    LAZY_ATTRS = ["src", "data-src", "data-lazy-src", "data-original", "data-lazy"]

    # Collect all image sources
    for tag in soup.find_all("img"):
        for attr in LAZY_ATTRS:
            val = tag.get(attr, "")
            if val and val not in seen and not val.startswith("data:"):
                seen.add(val)
                all_sources.append(val)
                break

    # <source srcset>
    for tag in soup.find_all("source"):
        srcset = tag.get("srcset", "")
        if srcset:
            first_src = srcset.split(",")[0].strip().split(" ")[0]
            if first_src and first_src not in seen:
                seen.add(first_src)
                all_sources.append(first_src)

    # Noscript fallback images
    for noscript in soup.find_all("noscript"):
        noscript_soup = BeautifulSoup(str(noscript), "html.parser")
        for tag in noscript_soup.find_all("img"):
            src = tag.get("src", "")
            if src and src not in seen and not src.startswith("data:"):
                seen.add(src)
                all_sources.append(src)

    # CSS background-image in inline styles
    for tag in soup.find_all(style=True):
        style_val = tag.get("style", "")
        if "url(" in style_val:
            urls = re.findall(r'url\(["\']?([^"\')\s]+)["\']?\)', style_val)
            for u in urls:
                ext = u.rsplit(".", 1)[-1].lower() if "." in u else ""
                if ext in ("jpg", "jpeg", "png", "gif", "webp", "avif", "svg", "ico", "bmp"):
                    if u not in seen:
                        seen.add(u)
                        all_sources.append(u)

    # Check each source for format
    total = len(all_sources)
    for src in all_sources:
        parsed = urlparse(src)
        ext = parsed.path.rsplit(".", 1)[-1].lower() if "." in parsed.path else ""
        full_ext = f".{ext}" if ext else ""
        if full_ext and full_ext not in MODERN_FORMATS:
            absolute = urljoin(page_url, src)
            unoptimized.append({
                "src": absolute[:300],
                "format": ext.upper() if ext else "unknown",
            })

    # Check <picture> for modern format support
    has_picture_modern = False
    for pic in soup.find_all("picture"):
        for source in pic.find_all("source"):
            src_type = source.get("type", "")
            if "webp" in src_type or "avif" in src_type:
                has_picture_modern = True
                break

    return {
        "total_images": total,
        "unoptimized_count": len(unoptimized),
        "unoptimized": unoptimized[:30],
        "has_picture_element_modern": has_picture_modern,
        "recommendation": "Convert images to WebP or AVIF for smaller file sizes" if unoptimized else "All images use modern formats",
    }


# ==================== CORE WEB VITALS ESTIMATES ====================

def estimate_cwv(html: str, ttfb_ms: int, resources: Dict[str, Any]) -> Dict[str, Any]:
    """Estimate Core Web Vitals from static HTML analysis.

    LCP estimate: TTFB + rendering heuristic based on resource count and image count.
    CLS risk:     images/iframes without explicit width/height.
    INP risk:     synchronous JS scripts (missing async/defer).
    """
    soup = BeautifulSoup(html, "html.parser")

    # --- LCP Estimate ---
    # Heuristic: TTFB + base render (~200ms) + penalty per blocking resource
    blocking_css = resources.get("css", {}).get("external_count", 0)
    blocking_js = 0
    for script in soup.find_all("script", src=True):
        if not script.get("async") and not script.get("defer"):
            blocking_js += 1

    image_count = resources.get("images", {}).get("count", 0)
    render_penalty = (blocking_css * 80) + (blocking_js * 120) + (min(image_count, 10) * 30)
    lcp_estimate = ttfb_ms + 200 + render_penalty

    lcp_rating = "good" if lcp_estimate < 2500 else "needs_improvement" if lcp_estimate < 4000 else "poor"

    # --- CLS Risk ---
    unstable_elements = 0
    for tag in soup.find_all(["img", "iframe", "video"]):
        has_width = tag.get("width") or (tag.get("style") and "width" in tag.get("style", ""))
        has_height = tag.get("height") or (tag.get("style") and "height" in tag.get("style", ""))
        if not (has_width and has_height):
            unstable_elements += 1

    cls_risk = "low" if unstable_elements <= 1 else "medium" if unstable_elements <= 5 else "high"

    # --- INP Risk ---
    total_sync_scripts = blocking_js + resources.get("js", {}).get("inline_count", 0)
    inp_risk = "low" if total_sync_scripts <= 2 else "medium" if total_sync_scripts <= 6 else "high"

    return {
        "lcp": {
            "estimate_ms": lcp_estimate,
            "rating": lcp_rating,
            "blocking_js": blocking_js,
            "blocking_css": blocking_css,
            "hint": f"Estimated from TTFB ({ttfb_ms}ms) + {blocking_js} blocking JS + {blocking_css} blocking CSS + {image_count} images",
        },
        "cls": {
            "risk": cls_risk,
            "unstable_elements": unstable_elements,
            "hint": f"{unstable_elements} media element(s) without explicit width/height dimensions",
        },
        "inp": {
            "risk": inp_risk,
            "sync_scripts": total_sync_scripts,
            "hint": f"{total_sync_scripts} synchronous script(s) may delay interaction response",
        },
    }


# ==================== MAIN ORCHESTRATORS ====================

def run_performance_analysis(
    html: str, url: str, headers: Dict[str, str], response_time_ms: int
) -> Dict[str, Any]:
    """Full performance analysis for the main page."""
    logger.info("Executing Performance Analysis modules...")

    html_size = len(html.encode("utf-8", errors="ignore"))
    ttfb_data = measure_ttfb(response_time_ms, html_size)
    resources = analyze_resources(html, url)
    images = detect_unoptimized_images(html, url)
    cwv = estimate_cwv(html, ttfb_data["ttfb_estimate_ms"], resources)

    return {
        "url": url,
        "ttfb": ttfb_data,
        "resources": resources,
        "unoptimized_images": images,
        "cwv_estimates": cwv,
    }


def run_performance_snapshot(
    html: str, url: str, headers: Dict[str, str], response_time_ms: int
) -> Dict[str, Any]:
    """Lightweight per-URL performance snapshot."""
    html_size = len(html.encode("utf-8", errors="ignore"))
    ttfb_data = measure_ttfb(response_time_ms, html_size)
    resources = analyze_resources(html, url)
    images = detect_unoptimized_images(html, url)
    cwv = estimate_cwv(html, ttfb_data["ttfb_estimate_ms"], resources)

    return {
        "url": url,
        "ttfb": ttfb_data,
        "resources": resources,
        "unoptimized_images": images,
        "cwv_estimates": cwv,
    }
