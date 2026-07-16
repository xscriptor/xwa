import extruct
import json
import re
from typing import Optional
from bs4 import BeautifulSoup
from urllib.parse import urljoin


def extract_structured(html: str, url: str) -> dict:
    result = {}

    try:
        data = extruct.extract(
            html,
            base_url=url,
            syntaxes=["json-ld", "microdata", "opengraph", "rdfa"],
            errors="ignore",
        )
        for key in data:
            items = data[key]
            if items:
                if len(items) == 1:
                    result[key] = items[0]
                else:
                    result[key] = items
    except Exception as e:
        result["_error"] = str(e)

    if not result.get("opengraph"):
        try:
            soup = BeautifulSoup(html, "lxml")
            og = {}
            for tag in soup.find_all("meta"):
                prop = tag.get("property", "") or tag.get("name", "")
                content = tag.get("content", "")
                if prop.startswith(("og:", "twitter:", "article:", "profile:")):
                    og[prop] = content
            if og:
                result["opengraph"] = og
        except Exception:
            pass

    return result


def extract_custom(html: str, selectors: list[str]) -> dict:
    soup = BeautifulSoup(html, "lxml")
    result = {}
    for sel in selectors:
        sel = sel.strip()
        if not sel:
            continue
        try:
            elements = soup.select(sel)
            result[sel] = [
                el.get_text(strip=True)
                if el.name in ("p", "span", "div", "h1", "h2", "h3", "h4", "h5", "h6", "li", "td", "th", "a", "blockquote", "label", "small", "strong", "em", "b", "i", "code", "pre")
                else str(el)
                for el in elements[:100]
            ]
        except Exception as e:
            result[sel] = f"[error: {e}]"
    return result


def extract_metadata(html: str, url: str) -> dict:
    soup = BeautifulSoup(html, "lxml")
    meta = {}

    for tag in soup.find_all("meta"):
        name = tag.get("name", "") or tag.get("property", "") or ""
        content = tag.get("content", "")
        if name and content:
            meta[name] = content

    title_tag = soup.find("title")
    meta["title"] = title_tag.get_text(strip=True) if title_tag else ""

    desc = meta.get("description", "") or meta.get("og:description", "")
    meta["description"] = desc

    keywords = meta.get("keywords", "")
    meta["keywords"] = [k.strip() for k in keywords.split(",")] if keywords else []

    canonical = soup.find("link", rel="canonical")
    meta["canonical"] = canonical.get("href", "") if canonical else ""

    return meta


def extract_headings(html: str) -> dict:
    soup = BeautifulSoup(html, "lxml")
    headings = {"h1": [], "h2": [], "h3": [], "h4": [], "h5": [], "h6": []}
    for tag in soup.find_all(["h1", "h2", "h3", "h4", "h5", "h6"]):
        level = tag.name
        text = tag.get_text(strip=True)
        if text:
            headings[level].append(text)
    return {k: v for k, v in headings.items() if v}


def extract_links(html: str, url: str) -> dict:
    soup = BeautifulSoup(html, "lxml")
    internal = []
    external = []
    anchor = []
    domain = re.sub(r"https?://", "", url).split("/")[0]

    for a in soup.find_all("a", href=True):
        href = a["href"].strip()
        text = a.get_text(strip=True)[:80]
        if not href or href.startswith(("#", "javascript:", "mailto:", "tel:")):
            continue
        try:
            full = urljoin(url, href)
        except Exception:
            continue
        entry = {"href": full, "text": text}
        if domain in full:
            internal.append(entry)
        else:
            external.append(entry)
        if text:
            anchor.append(entry)

    return {
        "internal_count": len(internal),
        "external_count": len(external),
        "internal": internal[:50],
        "external": external[:50],
    }


def extract_tables(html: str) -> list[dict]:
    soup = BeautifulSoup(html, "lxml")
    tables = []
    for table in soup.find_all("table"):
        rows = []
        headers = []
        for th in table.find_all("th"):
            headers.append(th.get_text(strip=True))
        for tr in table.find_all("tr"):
            cells = [td.get_text(strip=True) for td in tr.find_all(["td", "th"])]
            if cells:
                rows.append(cells)
        if rows:
            tables.append({"headers": headers, "rows": rows[:50]})
    return tables


def extract_images(html: str, url: str) -> list[dict]:
    soup = BeautifulSoup(html, "lxml")
    images = []
    for img in soup.find_all("img", src=True):
        src = img["src"]
        try:
            full_src = urljoin(url, src)
        except Exception:
            full_src = src
        images.append({
            "src": full_src,
            "alt": img.get("alt", ""),
            "width": img.get("width", ""),
            "height": img.get("height", ""),
        })
    return images[:50]
