from fastapi import APIRouter, BackgroundTasks, HTTPException, Depends
from sqlmodel import Session
from api.db.database import get_session
from api.db.models import ScanRecord
from core.models.scan_results import (
    FullScanReport, SEOResults, SitemapResults, SecurityResults,
    AccessibilityResults, StructureResults, PerformanceResults
)
from core.modules.security import run_security_analysis
from core.modules.security import run_security_snapshot
from core.modules.sitemap import run_sitemap_analysis
from core.modules.accessibility import run_accessibility_analysis
from core.modules.structure import run_structure_analysis
from core.modules.performance import run_performance_analysis, run_performance_snapshot
from core.modules.seo import (
    extract_standard_meta_tags,
    extract_social_meta_tags,
    extract_structured_data,
    extract_link_tags,
    analyze_headings,
    analyze_image_alts,
    analyze_text_ratio,
    extract_canonical,
    check_robots_txt
)
from core.utils.http import fetch_url
from datetime import datetime
import asyncio
import aiohttp
from pydantic import BaseModel

router = APIRouter()

class ScanRequest(BaseModel):
    url: str

# Simple in-memory tracker for SSE (for a real app use Redis)
scan_progress = {}


async def fetch_page_html(session: aiohttp.ClientSession, url: str) -> dict:
    """Fetch HTML content and headers from a URL for per-page analysis."""
    try:
        async with session.get(url, timeout=aiohttp.ClientTimeout(total=8)) as response:
            if response.status == 200:
                content_type = response.headers.get("Content-Type", "")
                if "text/html" not in content_type:
                    return {"url": url, "html": None, "headers": {}, "ok": False}
                html = await response.text()
                return {
                    "url": url,
                    "html": html,
                    "headers": dict(response.headers),
                    "ok": True,
                }
    except Exception:
        pass
    return {"url": url, "html": None, "headers": {}, "ok": False}


async def batch_fetch_html(urls: list, max_concurrent: int = 5) -> list:
    """Fetch HTML content for multiple URLs concurrently."""
    semaphore = asyncio.Semaphore(max_concurrent)
    connector = aiohttp.TCPConnector(limit_per_host=max_concurrent)

    async def limited_fetch(session, url):
        async with semaphore:
            return await fetch_page_html(session, url)

    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [limited_fetch(session, url) for url in urls]
        return await asyncio.gather(*tasks)


def sync_core_execution(url: str, session: Session, scan_id: int):
    """Executes the entire core analysis pipeline synchronously and saves it to DB."""
    try:
        scan_progress[scan_id] = "Connecting to target..."
        response = fetch_url(url)
        if not response:
            scan_progress[scan_id] = "Error: Connection Failed"
            return

        scan_progress[scan_id] = "Running SEO Analysis..."
        html_content = response.text

        standard_meta = extract_standard_meta_tags(html_content)
        social_meta = extract_social_meta_tags(html_content)
        structured_data = extract_structured_data(html_content)
        link_tags = extract_link_tags(html_content)
        headings = analyze_headings(html_content)
        alts = analyze_image_alts(html_content)
        ratio = analyze_text_ratio(html_content)
        canonical = extract_canonical(html_content)
        robots = check_robots_txt(url)

        scan_progress[scan_id] = "Running Sitemap & Crawler Analysis..."
        sitemap_results = run_sitemap_analysis(url)

        scan_progress[scan_id] = "Running Security Analysis..."
        security_results = run_security_analysis(url, response.headers, response.cookies, html_content)

        scan_progress[scan_id] = "Running Accessibility Analysis..."
        main_accessibility = run_accessibility_analysis(html_content, url)

        scan_progress[scan_id] = "Running Structure Analysis..."
        main_structure = run_structure_analysis(html_content, url)

        scan_progress[scan_id] = "Running Performance Analysis..."
        main_performance = run_performance_analysis(
            html_content, url, dict(response.headers), int((response.elapsed.total_seconds()) * 1000)
        )

        # Per-URL analysis: sample up to 25 discovered URLs
        per_url_accessibility = []
        per_url_seo = []
        per_url_security = []
        per_url_structure = []
        per_url_performance = []
        crawled_urls = sitemap_results.get("all_urls", [])
        sample_urls = [u for u in crawled_urls if u != url][:25]
        if sample_urls:
            scan_progress[scan_id] = f"Running per-link analysis on {len(sample_urls)} sub-pages..."
            page_results = asyncio.run(batch_fetch_html(sample_urls))
            for page in page_results:
                if page["ok"] and page["html"]:
                    page_html = page["html"]
                    page_url = page["url"]

                    # Accessibility per URL
                    a11y = run_accessibility_analysis(page_html, page_url)
                    per_url_accessibility.append(a11y)

                    # SEO per URL (no robots check here to keep performance stable)
                    per_url_seo.append({
                        "url": page_url,
                        "standard_meta": extract_standard_meta_tags(page_html),
                        "social_meta": extract_social_meta_tags(page_html),
                        "structured_data": extract_structured_data(page_html),
                        "link_tags": extract_link_tags(page_html),
                        "headings": analyze_headings(page_html),
                        "image_alts": analyze_image_alts(page_html),
                        "text_ratio": analyze_text_ratio(page_html),
                        "canonical": extract_canonical(page_html),
                    })

                    # Security snapshot per URL
                    per_url_security.append(
                        run_security_snapshot(page_url, page.get("headers", {}), page_html)
                    )

                    # Structure per URL
                    per_url_structure.append(run_structure_analysis(page_html, page_url))

                    # Performance per URL
                    # Use crawl_results response_time if available
                    page_response_ms = 0
                    for cr in sitemap_results.get("crawl_results", []):
                        if cr.get("url") == page_url:
                            page_response_ms = cr.get("response_time_ms", 0)
                            break
                    per_url_performance.append(
                        run_performance_snapshot(page_html, page_url, page.get("headers", {}), page_response_ms)
                    )


        scan_progress[scan_id] = "Formatting Final Report..."

        report = FullScanReport(
            target_url=url,
            scan_timestamp=datetime.utcnow().isoformat() + "Z",
            seo=SEOResults(
                standard_meta=standard_meta,
                social_meta=social_meta,
                structured_data=structured_data,
                link_tags=link_tags,
                headings=headings,
                image_alts=alts,
                text_ratio=ratio,
                canonical=canonical,
                robots_txt=robots,
                per_url=per_url_seo,
            ),
            sitemap=SitemapResults(**sitemap_results),
            security=SecurityResults(
                **security_results,
                per_url=per_url_security,
            ),
            accessibility=AccessibilityResults(
                main_page=main_accessibility,
                per_url=per_url_accessibility
            ),
            structure=StructureResults(
                main_page=main_structure,
                per_url=per_url_structure,
            ),
            performance=PerformanceResults(
                main_page=main_performance,
                per_url=per_url_performance,
            )
        )

        # Save to DB
        scan_record = session.get(ScanRecord, scan_id)
        if scan_record:
            scan_record.urls_found = sitemap_results["urls_found"]
            scan_record.broken_links_count = len(sitemap_results["broken_links"])
            scan_record.missing_security_headers = len(security_results["headers"]["missing_headers"])
            scan_record.is_ssl_valid = security_results["ssl"].get("valid", False)
            scan_record.raw_results = report.serialize()

            session.add(scan_record)
            session.commit()

        scan_progress[scan_id] = "Completed"

    except Exception as e:
        scan_progress[scan_id] = f"Error: {str(e)}"

@router.post("/scan")
def trigger_scan(req: ScanRequest, background_tasks: BackgroundTasks, db: Session = Depends(get_session)):
    """Receives target URLs and triggers the core engine."""

    nuevo_scan = ScanRecord(target_url=req.url)
    db.add(nuevo_scan)
    db.commit()
    db.refresh(nuevo_scan)

    scan_id = nuevo_scan.id
    scan_progress[scan_id] = "Initiating scan..."

    background_tasks.add_task(sync_core_execution, req.url, db, scan_id)

    return {"message": "Scan started.", "scan_id": scan_id}
