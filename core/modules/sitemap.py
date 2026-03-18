import asyncio
import aiohttp
from bs4 import BeautifulSoup
from urllib.parse import urlparse
import requests
from typing import List, Dict, Any
from core.utils.logger import logger

async def fetch_html_links(session: aiohttp.ClientSession, url: str, base_domain: str, parsed_base_scheme: str) -> List[str]:
    """Fetches HTML and parses base-domain hrefs."""
    try:
        async with session.get(url, timeout=5) as response:
            if response.status == 200:
                html = await response.text()
                soup = BeautifulSoup(html, "html.parser")
                links = set()
                for a_tag in soup.find_all("a", href=True):
                    href = a_tag["href"].strip()
                    if href.startswith("/"):
                        links.add(f"{parsed_base_scheme}://{base_domain}{href}")
                    elif base_domain in href:
                        links.add(href)
                return list(links)
    except Exception:
        pass
    return []

async def async_deep_scrape(base_url: str, max_pages: int = 150) -> set[str]:
    """Performs a BFS concurrent deep scrape."""
    visited = set()
    queue = [base_url]
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
            sitemap_urls = [loc.text.strip() for loc in soup.find_all("loc") if loc.text]
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

async def check_url(session: aiohttp.ClientSession, url: str, semaphore: asyncio.Semaphore) -> Dict[str, Any]:
    """Checks the status of a single URL asynchronously."""
    async with semaphore:
        try:
            # We use GET with stream=True or HEAD to save bandwidth. HEAD is faster.
            async with session.head(url, allow_redirects=True, timeout=10) as response:
                return {
                    "url": url, 
                    "status": response.status, 
                    "ok": response.status < 400
                }
        except Exception as e:
            return {
                "url": url, 
                "status": 0, 
                "ok": False, 
                "error": str(e)
            }

async def crawl_urls_concurrently(urls: List[str], max_concurrent: int = 10) -> List[Dict[str, Any]]:
    """Crawls a list of URLs concurrently using aiohttp and a semaphore."""
    semaphore = asyncio.Semaphore(max_concurrent)
    
    # We use a custom connector to limit per-host connections if needed
    connector = aiohttp.TCPConnector(limit_per_host=max_concurrent)
    
    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [check_url(session, url, semaphore) for url in urls]
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
        return {"urls_found": 0, "tree": {}, "broken_links": [], "scanned_count": 0, "all_urls": []}
        
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
    
    return {
        "urls_found": len(urls),
        "tree_root_children": list(tree.keys()),
        "broken_links": broken_links,
        "scanned_count": len(test_urls),
        "all_urls": urls,
        "crawl_results": crawl_results
    }
