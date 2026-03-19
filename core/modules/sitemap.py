import asyncio
import time
import re
import aiohttp
from bs4 import BeautifulSoup
from urllib.parse import urlparse, urlunparse, urlencode, parse_qs
import requests
from typing import List, Dict, Any, Set
from core.utils.logger import logger

# ---------------------------------------------------------------------------
# URL Normalization & Exclusion (ported from sitemap-generator/crawl.js)
# ---------------------------------------------------------------------------

EXCLUDED_EXTENSIONS = {
    '.pdf', '.jpg', '.jpeg', '.png', '.gif', '.svg', '.webp', '.ico',
    '.zip', '.gz', '.tar', '.rar', '.7z',
    '.mp3', '.mp4', '.avi', '.mov', '.wmv', '.flv', '.webm',
    '.css', '.js', '.json', '.xml', '.woff', '.woff2', '.ttf', '.eot',
    '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx',
}

EXCLUDED_PATH_PATTERNS = [
    re.compile(r'/wp-admin/', re.IGNORECASE),
    re.compile(r'/wp-includes/', re.IGNORECASE),
    re.compile(r'/wp-content/plugins/', re.IGNORECASE),
    re.compile(r'/node_modules/', re.IGNORECASE),
    re.compile(r'/\.git/', re.IGNORECASE),
    re.compile(r'/(login|logout|register|signup|signin|auth)(/|$)', re.IGNORECASE),
    re.compile(r'/feed(/|$)', re.IGNORECASE),
    re.compile(r'/xmlrpc\.php', re.IGNORECASE),
]


def normalize_url(url: str) -> str:
    """Normalizes a URL: strips fragments, trailing slashes, sorts query params."""
    try:
        parsed = urlparse(url)
        # Remove fragment
        path = parsed.path.rstrip('/') or '/'
        # Sort query params
        query_params = parse_qs(parsed.query, keep_blank_values=True)
        sorted_query = urlencode(
            {k: v[0] if len(v) == 1 else v for k, v in sorted(query_params.items())},
            doseq=True
        ) if query_params else ''
        return urlunparse((
            parsed.scheme.lower(),
            parsed.netloc.lower(),
            path,
            parsed.params,
            sorted_query,
            ''  # no fragment
        ))
    except Exception:
        return url


def should_exclude_url(url: str) -> bool:
    """Checks if a URL should be excluded from crawling."""
    parsed = urlparse(url)
    path_lower = parsed.path.lower()

    # Check file extensions
    for ext in EXCLUDED_EXTENSIONS:
        if path_lower.endswith(ext):
            return True

    # Check path patterns
    for pattern in EXCLUDED_PATH_PATTERNS:
        if pattern.search(parsed.path):
            return True

    return False


# ---------------------------------------------------------------------------
# Metadata Extraction (ported from sitemap-generator/crawl.js)
# ---------------------------------------------------------------------------

def extract_page_metadata(html: str, url: str) -> Dict[str, Any]:
    """Extracts title, description, h1, word_count, and canonical from HTML."""
    soup = BeautifulSoup(html, "html.parser")

    # Title
    title_tag = soup.find("title")
    title = title_tag.get_text(strip=True) if title_tag else ""

    # Meta description
    desc_tag = soup.find("meta", attrs={"name": "description"})
    description = desc_tag.get("content", "").strip() if desc_tag else ""

    # H1
    h1_tag = soup.find("h1")
    h1 = h1_tag.get_text(strip=True) if h1_tag else ""

    # Word count (visible text only)
    for tag in soup(["script", "style", "noscript"]):
        tag.decompose()
    text = soup.get_text(separator=" ", strip=True)
    word_count = len(text.split()) if text else 0

    # Canonical
    canonical_tag = soup.find("link", attrs={"rel": "canonical"})
    canonical = canonical_tag.get("href", "").strip() if canonical_tag else ""

    return {
        "title": title,
        "description": description,
        "h1": h1,
        "word_count": word_count,
        "canonical": canonical,
    }


# ---------------------------------------------------------------------------
# Enhanced HTML Link Fetching with Metadata
# ---------------------------------------------------------------------------

async def fetch_html_links(
    session: aiohttp.ClientSession,
    url: str,
    base_domain: str,
    parsed_base_scheme: str
) -> List[str]:
    """Fetches HTML and parses base-domain hrefs, with URL normalization and filtering."""
    try:
        async with session.get(url, timeout=aiohttp.ClientTimeout(total=10)) as response:
            if response.status == 200:
                content_type = response.headers.get('Content-Type', '')
                if 'text/html' not in content_type:
                    return []
                html = await response.text()
                soup = BeautifulSoup(html, "html.parser")
                links = set()
                for a_tag in soup.find_all("a", href=True):
                    href = a_tag["href"].strip()
                    if not href or href.startswith(('#', 'mailto:', 'tel:', 'javascript:')):
                        continue
                    if href.startswith("/"):
                        full_url = f"{parsed_base_scheme}://{base_domain}{href}"
                    elif base_domain in href:
                        full_url = href
                    else:
                        continue

                    normalized = normalize_url(full_url)
                    if not should_exclude_url(normalized):
                        links.add(normalized)
                return list(links)
    except Exception:
        pass
    return []


async def async_deep_scrape(base_url: str, max_pages: int = 150) -> Set[str]:
    """Performs a BFS concurrent deep scrape with URL normalization."""
    visited: Set[str] = set()
    queue = [normalize_url(base_url)]
    parsed_base = urlparse(base_url)
    base_domain = parsed_base.netloc

    connector = aiohttp.TCPConnector(limit_per_host=10)
    async with aiohttp.ClientSession(connector=connector) as session:
        while queue and len(visited) < max_pages:
            batch = queue[:20]
            queue = queue[20:]

            tasks = []
            for url in batch:
                if url not in visited:
                    visited.add(url)
                    tasks.append(fetch_html_links(session, url, base_domain, parsed_base.scheme))

            if not tasks:
                continue

            results = await asyncio.gather(*tasks)
            for links in results:
                for link in links:
                    if link not in visited and len(visited) + len(queue) < max_pages:
                        queue.append(link)

    return visited


def fetch_all_urls(base_url: str) -> List[str]:
    """Finds URLs from sitemap and performs deep recursive scrape on the site."""
    urls = set()
    parsed_base = urlparse(base_url)
    base_domain = parsed_base.netloc

    # 1. Try Sitemap
    sitemap_url = f"{parsed_base.scheme}://{base_domain}/sitemap.xml"
    logger.info(f"Attempting to fetch sitemap from: {sitemap_url}")
    try:
        resp = requests.get(sitemap_url, timeout=10)
        if resp.status_code == 200:
            soup = BeautifulSoup(resp.content, "xml")
            sitemap_urls = [normalize_url(loc.text.strip()) for loc in soup.find_all("loc") if loc.text]
            urls.update(sitemap_urls)
            logger.info(f"Extracted {len(sitemap_urls)} URLs from sitemap.")
    except Exception as e:
        logger.error(f"Sitemap fetch failed: {e}")

    # 2. Deep Scrape site
    logger.info(f"Deep scraping site {base_url} (max 150 pages)...")
    scraped = asyncio.run(async_deep_scrape(base_url))
    urls.update(scraped)
    logger.info(f"Total internal links discovered: {len(urls)}.")

    return list(urls)


# ---------------------------------------------------------------------------
# Enhanced URL Checker with Metadata & Response Time
# ---------------------------------------------------------------------------

async def check_url_with_metadata(
    session: aiohttp.ClientSession,
    url: str,
    semaphore: asyncio.Semaphore
) -> Dict[str, Any]:
    """Checks URL status AND extracts page metadata + response time."""
    async with semaphore:
        start_time = time.monotonic()
        try:
            async with session.get(
                url,
                allow_redirects=True,
                timeout=aiohttp.ClientTimeout(total=15)
            ) as response:
                elapsed_ms = round((time.monotonic() - start_time) * 1000)
                result = {
                    "url": url,
                    "status": response.status,
                    "ok": response.status < 400,
                    "response_time_ms": elapsed_ms,
                }

                # Extract metadata only for successful HTML responses
                content_type = response.headers.get('Content-Type', '')
                if response.status == 200 and 'text/html' in content_type:
                    try:
                        html = await response.text()
                        metadata = extract_page_metadata(html, url)
                        result.update(metadata)
                    except Exception:
                        pass

                return result
        except Exception as e:
            elapsed_ms = round((time.monotonic() - start_time) * 1000)
            return {
                "url": url,
                "status": 0,
                "ok": False,
                "error": str(e),
                "response_time_ms": elapsed_ms,
            }


async def crawl_urls_concurrently(urls: List[str], max_concurrent: int = 10) -> List[Dict[str, Any]]:
    """Crawls a list of URLs concurrently, extracting metadata and timing."""
    semaphore = asyncio.Semaphore(max_concurrent)
    connector = aiohttp.TCPConnector(limit_per_host=max_concurrent)

    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [check_url_with_metadata(session, url, semaphore) for url in urls]
        results = await asyncio.gather(*tasks)

    return list(results)


def build_url_tree(urls: List[str]) -> Dict[str, Any]:
    """Transforms a flat list of URLs into a hierarchical tree structure."""
    tree = {"_paths": []}

    for url in urls:
        parsed = urlparse(url)
        path = parsed.path.strip("/")

        if not path:
            tree["_paths"].append(url)
            continue

        parts = path.split("/")
        current_node = tree

        for part in parts:
            if part not in current_node:
                current_node[part] = {"_paths": []}
            current_node = current_node[part]

        current_node["_paths"].append(url)

    return tree


def run_sitemap_analysis(base_url: str) -> Dict[str, Any]:
    """Main entry point for sitemap and crawler analysis."""
    urls = fetch_all_urls(base_url)

    if not urls:
        return {"urls_found": 0, "tree": {}, "broken_links": [], "scanned_count": 0, "all_urls": [], "crawl_results": []}

    tree = build_url_tree(urls)

    # Limit max scans to defaults (e.g., 50) for fast CLI testing
    test_urls = urls[:50]
    logger.info(f"Validating {len(test_urls)} URLs concurrently (rate limit: 5 concurrent)...")

    crawl_results = asyncio.run(crawl_urls_concurrently(test_urls, max_concurrent=5))

    broken_links = [res for res in crawl_results if not res["ok"]]
    if broken_links:
        logger.warning(f"Found {len(broken_links)} broken links!")
    else:
        logger.info("No broken links found among scanned URLs.")

    # Compute aggregate stats
    avg_response_time = 0
    ok_results = [r for r in crawl_results if r.get("ok")]
    if ok_results:
        avg_response_time = round(
            sum(r.get("response_time_ms", 0) for r in ok_results) / len(ok_results)
        )

    return {
        "urls_found": len(urls),
        "tree_root_children": list(tree.keys()),
        "broken_links": broken_links,
        "scanned_count": len(test_urls),
        "all_urls": urls,
        "crawl_results": crawl_results,
        "avg_response_time_ms": avg_response_time,
    }
