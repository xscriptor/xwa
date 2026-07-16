import re
import sys
import json
import logging
import os
from typing import Optional

import httpx
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from extractors.structured import extract_structured, extract_custom, extract_metadata, extract_headings, extract_links, extract_tables, extract_images
from extractors.nlp import analyze as nlp_analyze
from extractors.crawler import crawl_manager

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    stream=sys.stdout,
)
log = logging.getLogger("extractor")

app = FastAPI(title="Shinobi Extractor", version="0.3.0")


class ExtractRequest(BaseModel):
    url: str
    html: str = ""
    extract_structured: bool = True
    nlp_enabled: bool = False
    custom_selectors: list[str] = []


class ExtractResponse(BaseModel):
    url: str
    structured: dict | None = None
    nlp: dict | None = None
    emails: list[str] = []
    phones: list[str] = []
    custom: dict | None = None
    metadata: dict | None = None
    headings: dict | None = None
    links: dict | None = None
    tables: list | None = None
    images: list | None = None


class CrawlRequest(BaseModel):
    url: str
    depth: int = 3
    max_pages: int = 100
    same_domain: bool = True
    download_assets: bool = True
    file_types: list[str] = []
    extract_structured: bool = True
    nlp_enabled: bool = False
    custom_selectors: list[str] = []


class CrawlStartResponse(BaseModel):
    job_id: str
    url: str
    domain: str
    status: str


EMAIL_RE = re.compile(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
PHONE_RE = re.compile(r"(\+?\d[\s.-]?){1,3}\(\d{2,4}\)[\s.-]?\d{3,4}[\s.-]?\d{3,4}")


def extract_emails(text: str) -> list[str]:
    return sorted(set(EMAIL_RE.findall(text)))


def extract_phones(text: str) -> list[str]:
    return sorted(set(PHONE_RE.findall(text)))


def fetch_html(url: str) -> str:
    try:
        resp = httpx.get(url, timeout=30, follow_redirects=True,
                         headers={"User-Agent": "Mozilla/5.0 (compatible; Shinobi/0.1)"})
        resp.raise_for_status()
        return resp.text
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Failed to fetch {url}: {e}")


def run_extraction_pipeline(html: str, url: str,
                            extract_structured: bool = True,
                            nlp_enabled: bool = False,
                            custom_selectors: list[str] | None = None) -> dict:
    result = {}

    if extract_structured:
        try:
            result["structured"] = extract_structured(html, url)
        except Exception as e:
            log.error("Structured extraction failed: %s", e)
            result["structured"] = {"_error": str(e)}

    if nlp_enabled:
        try:
            result["nlp"] = nlp_analyze(html, url)
        except Exception as e:
            log.error("NLP analysis failed: %s", e)
            result["nlp"] = {"_error": str(e)}

    try:
        result["metadata"] = extract_metadata(html, url)
    except Exception:
        pass

    try:
        result["headings"] = extract_headings(html)
    except Exception:
        pass

    try:
        links_data = extract_links(html, url)
        if links_data:
            result["links"] = links_data
    except Exception:
        pass

    try:
        tables = extract_tables(html)
        if tables:
            result["tables"] = tables
    except Exception:
        pass

    try:
        images = extract_images(html, url)
        if images:
            result["images"] = images
    except Exception:
        pass

    if custom_selectors:
        try:
            result["custom"] = extract_custom(html, custom_selectors)
        except Exception as e:
            result["custom"] = {"_error": str(e)}

    text_content = html
    try:
        from bs4 import BeautifulSoup
        soup = BeautifulSoup(html, "lxml")
        text_content = soup.get_text(separator=" ", strip=True)
    except Exception:
        text_content = re.sub(r"<[^>]+>", " ", html)

    result["emails"] = extract_emails(text_content)
    result["phones"] = extract_phones(text_content)

    return result


@app.get("/health")
def health():
    return {"status": "ok", "service": "shinobi-extractor", "version": "0.3.0"}


@app.post("/extract", response_model=ExtractResponse)
def extract(req: ExtractRequest):
    log.info("Extracting: %s", req.url)
    html = req.html or fetch_html(req.url)
    data = run_extraction_pipeline(html, req.url, req.extract_structured, req.nlp_enabled, req.custom_selectors)

    return ExtractResponse(
        url=req.url,
        structured=data.get("structured"),
        nlp=data.get("nlp"),
        emails=data.get("emails", []),
        phones=data.get("phones", []),
        custom=data.get("custom"),
        metadata=data.get("metadata"),
        headings=data.get("headings"),
        links=data.get("links"),
        tables=data.get("tables"),
        images=data.get("images"),
    )


@app.post("/crawl")
def start_crawl(req: CrawlRequest):
    log.info("Starting crawl: %s (depth=%d, max=%d)", req.url, req.depth, req.max_pages)
    output_dir = os.environ.get("DATA_DIR", os.path.join(os.path.dirname(__file__), "..", "downloads"))

    def extract_cb(html, url):
        return run_extraction_pipeline(
            html, url,
            extract_structured=req.extract_structured,
            nlp_enabled=req.nlp_enabled,
            custom_selectors=req.custom_selectors,
        )

    job = crawl_manager.start(
        url=req.url,
        output_dir=output_dir,
        depth=req.depth,
        max_pages=req.max_pages,
        same_domain=req.same_domain,
        extract_callback=extract_cb,
    )

    from urllib.parse import urlparse
    domain = urlparse(req.url).netloc

    return CrawlStartResponse(job_id=job.job_id, url=req.url, domain=domain, status="queued")


@app.get("/crawl/{job_id}")
def get_crawl(job_id: str):
    job = crawl_manager.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail="Job not found")
    return job.to_dict()


@app.get("/crawl/{job_id}/status")
def get_crawl_status(job_id: str):
    job = crawl_manager.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail="Job not found")
    return {
        "job_id": job.job_id,
        "status": job.status,
        "pages": job.pages,
        "files": job.files,
        "progress_pct": job.progress_pct,
        "current_url": job.current_url,
        "errors": job.errors[-5:],
        "log": job.log_lines[-50:],
    }


@app.get("/crawl/{job_id}/results")
def get_crawl_results(job_id: str):
    job = crawl_manager.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail="Job not found")
    return {
        "job_id": job.job_id,
        "url": job.url,
        "domain": job.domain,
        "status": job.status,
        "pages": job.pages,
        "files": job.files,
        "emails": job.emails,
        "phones": job.phones,
        "zip_path": job.zip_path,
        "results": job.results,
        "output_dir": job.output_dir_path,
    }


@app.post("/crawl/{job_id}/cancel")
def cancel_crawl(job_id: str):
    if crawl_manager.cancel(job_id):
        return {"status": "cancelled"}
    raise HTTPException(status_code=404, detail="Job not found")


@app.get("/crawl-jobs")
def list_crawl_jobs():
    return crawl_manager.list()


if __name__ == "__main__":
    import uvicorn
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 9090
    uvicorn.run(app, host="0.0.0.0", port=port, log_level="info")
