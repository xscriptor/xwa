import re
from typing import Any, Callable, Dict, List, Optional

import httpx

LogCallback = Callable[[str], Any]

SERVER_SIGNATURES: Dict[str, List[str]] = {
    "nginx": [r"nginx(?:/([\d.]+))?"],
    "apache": [r"Apache(?:/([\d.]+))?"],
    "iis": [r"IIS(?:/([\d.]+))?", r"Microsoft-IIS(?:/([\d.]+))?"],
    "caddy": [r"Caddy(?:/([\d.]+))?"],
    "openresty": [r"openresty(?:/([\d.]+))?"],
    "gunicorn": [r"gunicorn(?:/([\d.]+))?"],
    "uwsgi": [r"uWSGI(?:/([\d.]+))?"],
    "nodejs": [r"Node\.?(?:\.?js)?(?:/([\d.]+))?", r"Express(?:/([\d.]+))?"],
    "python": [r"Python(?:/([\d.]+))?", r"Werkzeug(?:/([\d.]+))?"],
    "java": [r"Java(?:/([\d.]+))?", r"Apache\-Tomcat(?:/([\d.]+))?", r"Jetty(?:/([\d.]+))?"],
    "cloudflare": [r"cloudflare"],
    "cloudfront": [r"AmazonS3", r"CloudFront"],
    "fastly": [r"Fastly"],
}


def _extract_version(header_value: str, pattern: str) -> Optional[str]:
    match = re.search(pattern, header_value, re.IGNORECASE)
    if match and match.lastindex and match.group(1):
        return match.group(1)
    return None


async def run(target: str, log: LogCallback) -> Dict[str, Any]:
    base_url = target if target.startswith("http") else f"https://{target}"
    servers: List[Dict[str, Any]] = []
    cdn: Optional[Dict[str, Any]] = None
    headers_raw: Dict[str, str] = {}

    await log(f"[fingerprint] probing {base_url}")

    async with httpx.AsyncClient(timeout=15, verify=False, follow_redirects=True) as client:
        try:
            response = await client.get(base_url)
            headers_raw = dict(response.headers)
            await log(f"[fingerprint] HTTP {response.status_code}")
        except Exception as e:
            await log(f"[fingerprint] HTTP error: {str(e)}")
            return {"servers": [], "cdn": None, "headers": {}}

    lower_headers = {k.lower(): v for k, v in headers_raw.items()}

    # Server header
    server_val = lower_headers.get("server", "")
    if server_val:
        for name, patterns in SERVER_SIGNATURES.items():
            for pat in patterns:
                if re.search(pat, server_val, re.IGNORECASE):
                    version = _extract_version(server_val, pat)
                    servers.append({
                        "name": name,
                        "version": version,
                        "confidence": "high",
                        "evidence": f"Server: {server_val}",
                    })

    # X-Powered-By
    xpb = lower_headers.get("x-powered-by", "")
    if xpb:
        for name, patterns in SERVER_SIGNATURES.items():
            for pat in patterns:
                if re.search(pat, xpb, re.IGNORECASE):
                    version = _extract_version(xpb, pat)
                    servers.append({
                        "name": name,
                        "version": version,
                        "confidence": "medium",
                        "evidence": f"X-Powered-By: {xpb}",
                    })

    # CDN detection via headers
    cfn_ray = lower_headers.get("cf-ray")
    cfn_cache = lower_headers.get("cf-cache-status")
    if cfn_ray or cfn_cache:
        cdn = {"name": "cloudflare", "version": None, "confidence": "high", "evidence": "cf-ray / cf-cache-status detected"}

    x_cache = lower_headers.get("x-cache")
    if x_cache and not cdn:
        cdn = {"name": "fastly", "version": None, "confidence": "high", "evidence": f"X-Cache: {x_cache}"}

    x_amz = lower_headers.get("x-amz-cf-id")
    if x_amz and not cdn:
        cdn = {"name": "cloudfront", "version": None, "confidence": "high", "evidence": "x-amz-cf-id detected"}

    # Detect hosting platform
    via = lower_headers.get("via", "")
    if "amazon" in via.lower() and not cdn:
        cdn = {"name": "cloudfront", "version": None, "confidence": "medium", "evidence": f"Via: {via}"}

    await log(f"[fingerprint] servers: {[s['name'] for s in servers]}")
    await log(f"[fingerprint] cdn: {cdn['name'] if cdn else 'none'}")

    return {
        "servers": servers,
        "cdn": cdn,
        "headers": {k: v for k, v in headers_raw.items() if k.lower() in ("server", "x-powered-by", "x-aspnet-version", "x-robots-tag", "via", "x-frame-options")},
    }
