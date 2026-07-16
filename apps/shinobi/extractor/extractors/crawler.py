import os
import re
import json
import zipfile
import logging
import subprocess
import tempfile
import shutil
import threading
import time
import uuid
from pathlib import Path
from typing import Optional
from urllib.parse import urlparse

from bs4 import BeautifulSoup

log = logging.getLogger("extractor.crawler")

HTML_EXTS = {".html", ".htm", ".php", ".asp", ".aspx", ".jsp", ".shtml"}


def _find_httrack() -> Optional[str]:
    for cmd in ["httrack", "httrack-cli", "/usr/bin/httrack", "/usr/local/bin/httrack"]:
        if shutil.which(cmd):
            return cmd
    return None


class CrawlJob:
    def __init__(self, job_id: str, url: str, output_dir: str, depth: int, max_pages: int,
                 same_domain: bool, extract_callback=None):
        self.job_id = job_id
        self.url = url
        self.domain = urlparse(url).netloc
        self.output_dir = Path(output_dir)
        self.depth = depth
        self.max_pages = max_pages
        self.same_domain = same_domain
        self.extract_callback = extract_callback
        self.httrack_path = _find_httrack()

        self.status = "queued"
        self.pages = 0
        self.files = 0
        self.errors: list[str] = []
        self.log_lines: list[str] = []
        self.progress_pct = 0
        self.current_url = ""
        self.results: list[dict] = []
        self.emails: list[str] = []
        self.phones: list[str] = []
        self.zip_path = ""
        self.output_dir_path = ""
        self._proc: Optional[subprocess.Popen] = None
        self._tmpdir: Optional[str] = None

    def to_dict(self) -> dict:
        return {
            "job_id": self.job_id,
            "url": self.url,
            "domain": self.domain,
            "status": self.status,
            "pages": self.pages,
            "files": self.files,
            "progress_pct": self.progress_pct,
            "current_url": self.current_url,
            "errors": self.errors[-10:],
            "emails": self.emails,
            "phones": self.phones,
            "zip_path": self.zip_path,
            "output_dir": self.output_dir_path,
            "results_count": len(self.results),
            "results": self.results[-50:],
            "log": self.log_lines[-200:],
        }

    def _build_args(self, output_root: Path) -> list[str]:
        args = [
            self.httrack_path,
            self.url,
            "-O", str(output_root),
            "-w",
            f"-r{self.depth}",
            "-N0",
            "-v",
            "-s2",
            "-q",
            "-Q",
            "-I0",
        ]
        if self.same_domain:
            args.append("-d")
        return args

    def run(self):
        if not self.httrack_path:
            self.status = "failed"
            self.errors.append("httrack not found")
            return

        self.status = "running"
        self._tmpdir = tempfile.mkdtemp(prefix="shinobi_")
        output_root = Path(self._tmpdir) / "mirror"
        output_root.mkdir(parents=True)

        args = self._build_args(output_root)
        log.info("Starting httrack: %s", " ".join(str(a) for a in args[:6]) + " ...")

        try:
            self._proc = subprocess.Popen(
                args, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, bufsize=1,
            )

            for line in iter(self._proc.stderr.readline, ""):
                line = line.rstrip("\n\r")
                if not line:
                    continue
                self.log_lines.append(line)

                m = re.search(r"(\d+)\s+pages?\s+maj", line, re.IGNORECASE)
                if m:
                    self.pages = int(m.group(1))
                    self.progress_pct = min(95, int(self.pages / max(self.max_pages, 1) * 100))
                m = re.search(r"(\d+)\s+files?\s+maj", line, re.IGNORECASE)
                if m:
                    self.files = int(m.group(1))
                m = re.search(r"Error:\s*(.+)", line, re.IGNORECASE)
                if m:
                    err = m.group(1).strip()
                    if err not in self.errors:
                        self.errors.append(err)
                m = re.search(r"https?://\S+", line)
                if m and not re.search(r"Error|Warning", line):
                    self.current_url = m.group(0)

            self._proc.wait()

            hts_dir = output_root / self.domain
            if not hts_dir.exists():
                alt = list(output_root.iterdir()) if output_root.exists() else []
                if alt:
                    hts_dir = alt[0]

            if hts_dir.exists():
                self.output_dir_path = str(hts_dir)
                self._extract_results(hts_dir)
                self._create_zip(hts_dir)
                self.progress_pct = 100
                self.status = "completed"
            else:
                self.status = "failed"
                self.errors.append("httrack produced no output")
                last = "\n".join(self.log_lines[-15:])
                self.errors.append(f"Last output: {last[:300]}")

        except Exception as e:
            self.status = "failed"
            self.errors.append(str(e))
            log.error("Crawl failed: %s", e)
        finally:
            if self._tmpdir and os.path.exists(self._tmpdir):
                shutil.rmtree(self._tmpdir, ignore_errors=True)
            log.info("Crawl %s: %s pages, %s files", self.job_id, self.pages, self.files)

    def cancel(self):
        if self._proc and self._proc.poll() is None:
            self._proc.terminate()
            self.status = "cancelled"
            log.info("Crawl %s cancelled", self.job_id)

    def _extract_results(self, mirror_dir: Path):
        html_files = []
        for ext in HTML_EXTS:
            html_files.extend(mirror_dir.rglob(f"*{ext}"))

        for html_path in html_files:
            try:
                with open(html_path, "r", encoding="utf-8", errors="replace") as f:
                    html = f.read()
            except Exception:
                continue
            if not html.strip():
                continue

            rel_path = html_path.relative_to(mirror_dir.parent)
            url = f"https://{self.domain}/{rel_path}"

            if self.extract_callback:
                try:
                    result = self.extract_callback(html, url)
                    if result:
                        result["url"] = url
                        result["saved_path"] = str(rel_path)
                        for e in result.get("emails", []):
                            if e not in self.emails:
                                self.emails.append(e)
                        for p in result.get("phones", []):
                            if p not in self.phones:
                                self.phones.append(p)
                        self.results.append(result)
                except Exception as e:
                    log.error("Extract failed %s: %s", url, e)

        meta = {
            "url": self.url,
            "domain": self.domain,
            "pages": self.pages,
            "files": self.files,
            "errors": self.errors[:30],
            "emails": self.emails,
            "phones": self.phones,
            "results_count": len(self.results),
        }
        with open(mirror_dir / "_crawl.json", "w") as f:
            json.dump(meta, f, indent=2)

    def _create_zip(self, mirror_dir: Path):
        zip_path = self.output_dir / f"{self.domain}.zip"
        with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
            for file_path in sorted(mirror_dir.rglob("*")):
                if file_path.is_file() and file_path.suffix not in {".zip", ".log"}:
                    arcname = str(file_path.relative_to(mirror_dir.parent))
                    zf.write(file_path, arcname)
        # Move to persistent storage
        final_zip = self.output_dir / f"{self.domain}.zip"
        if zip_path != final_zip:
            shutil.move(str(zip_path), str(final_zip))
        self.zip_path = str(final_zip)


class CrawlManager:
    def __init__(self):
        self._jobs: dict[str, CrawlJob] = {}
        self._lock = threading.Lock()
        self._queue: list[str] = []
        self._current: Optional[str] = None

    def start(self, url: str, output_dir: str, depth: int, max_pages: int,
              same_domain: bool, extract_callback=None) -> CrawlJob:
        job_id = str(uuid.uuid4())[:8]
        job = CrawlJob(job_id, url, output_dir, depth, max_pages, same_domain, extract_callback)
        with self._lock:
            self._jobs[job_id] = job
            self._queue.append(job_id)
        threading.Thread(target=self._worker, daemon=True).start()
        return job

    def _worker(self):
        with self._lock:
            if self._current is not None:
                return
            if not self._queue:
                return
            job_id = self._queue.pop(0)
            self._current = job_id

        job = self._jobs.get(job_id)
        if job:
            job.run()

        with self._lock:
            self._current = None
            if self._queue:
                threading.Thread(target=self._worker, daemon=True).start()

    def get(self, job_id: str) -> Optional[CrawlJob]:
        with self._lock:
            return self._jobs.get(job_id)

    def cancel(self, job_id: str) -> bool:
        with self._lock:
            job = self._jobs.get(job_id)
        if job:
            job.cancel()
            return True
        return False

    def list(self) -> list[dict]:
        with self._lock:
            return [j.to_dict() for j in self._jobs.values()]


crawl_manager = CrawlManager()
