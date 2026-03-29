"""
Security Analysis Module.

Analyzes security headers, SSL certificates, cookies, sensitive paths,
CORS policy, CSP directives, SRI, mixed content, email/IP exposure,
HTTP methods, and technology detection.
"""

import ssl
import socket
import asyncio
import aiohttp
import re
import os
import requests
import dns.resolver
from urllib.parse import urlparse
from typing import Dict, Any, List, Optional
from datetime import datetime
from bs4 import BeautifulSoup
from core.utils.logger import logger


# ==================== HEADERS ====================

def analyze_security_headers(headers: Dict[str, str]) -> Dict[str, Any]:
    """Analyzes response headers for security best practices and leaked info."""
    h = {k.lower(): v for k, v in headers.items()}

    security_headers = {
        'strict-transport-security': h.get('strict-transport-security'),
        'content-security-policy': h.get('content-security-policy'),
        'x-frame-options': h.get('x-frame-options'),
        'x-content-type-options': h.get('x-content-type-options'),
        'referrer-policy': h.get('referrer-policy'),
        'permissions-policy': h.get('permissions-policy'),
        'x-xss-protection': h.get('x-xss-protection'),
        'cross-origin-opener-policy': h.get('cross-origin-opener-policy'),
        'cross-origin-resource-policy': h.get('cross-origin-resource-policy'),
    }

    leaked_info = {}
    for key in ['server', 'x-powered-by', 'x-aspnet-version', 'x-generator', 'x-drupal-cache']:
        val = h.get(key)
        if val:
            leaked_info[key] = val

    missing_headers = [k for k, v in security_headers.items() if v is None]

    return {
        'headers_present': {k: v for k, v in security_headers.items() if v is not None},
        'missing_headers': missing_headers,
        'leaked_server_info': leaked_info,
        'score_penalty': len(missing_headers) * 10
    }


# ==================== CORS ====================

def analyze_cors(headers: Dict[str, str]) -> Dict[str, Any]:
    """Analyzes CORS headers for misconfigurations."""
    h = {k.lower(): v for k, v in headers.items()}

    origin = h.get('access-control-allow-origin', '')
    methods = h.get('access-control-allow-methods', '')
    allow_headers = h.get('access-control-allow-headers', '')
    credentials = h.get('access-control-allow-credentials', '')
    expose = h.get('access-control-expose-headers', '')

    issues = []
    if origin == '*':
        issues.append({"severity": "warning", "message": "CORS allows all origins (*) -- potential security risk"})
    if origin == '*' and credentials.lower() == 'true':
        issues.append({"severity": "error", "message": "CORS wildcard with credentials enabled -- critical vulnerability"})
    if 'DELETE' in methods.upper() or 'PUT' in methods.upper():
        issues.append({"severity": "info", "message": f"CORS allows destructive methods: {methods}"})

    return {
        "allow_origin": origin or None,
        "allow_methods": methods or None,
        "allow_headers": allow_headers or None,
        "allow_credentials": credentials or None,
        "expose_headers": expose or None,
        "has_cors": bool(origin),
        "issues": issues
    }


# ==================== CSP DEEP PARSE ====================

def parse_csp(headers: Dict[str, str]) -> Dict[str, Any]:
    """Deep parse Content-Security-Policy directives."""
    h = {k.lower(): v for k, v in headers.items()}
    csp_raw = h.get('content-security-policy', '')

    if not csp_raw:
        return {"present": False, "directives": {}, "issues": []}

    directives: Dict[str, List[str]] = {}
    issues = []

    for part in csp_raw.split(';'):
        part = part.strip()
        if not part:
            continue
        tokens = part.split()
        if tokens:
            directive_name = tokens[0].lower()
            directive_values = tokens[1:]
            directives[directive_name] = directive_values

    # Check for risky directives
    risky_values = {"'unsafe-inline'", "'unsafe-eval'", "*", "data:", "blob:"}
    for directive, values in directives.items():
        for val in values:
            if val.lower() in risky_values:
                issues.append({
                    "severity": "warning" if val.lower() != "'unsafe-eval'" else "error",
                    "directive": directive,
                    "value": val,
                    "message": f"{directive} contains risky value: {val}"
                })

    if 'default-src' not in directives and 'script-src' not in directives:
        issues.append({"severity": "warning", "directive": "default-src", "message": "No default-src or script-src defined"})

    return {
        "present": True,
        "raw": csp_raw[:500],
        "directives": directives,
        "issues": issues
    }


# ==================== SRI ====================

def check_subresource_integrity(html_content: str) -> Dict[str, Any]:
    """Check external scripts and stylesheets for integrity attributes."""
    soup = BeautifulSoup(html_content, 'html.parser')
    results = []
    missing_count = 0

    for script in soup.find_all('script', src=True):
        src = script.get('src', '')
        has_integrity = bool(script.get('integrity'))
        is_external = src.startswith('http') or src.startswith('//')
        if is_external:
            results.append({"type": "script", "src": src[:200], "has_integrity": has_integrity})
            if not has_integrity:
                missing_count += 1

    for link in soup.find_all('link', rel='stylesheet'):
        href = link.get('href', '')
        has_integrity = bool(link.get('integrity'))
        is_external = href.startswith('http') or href.startswith('//')
        if is_external:
            results.append({"type": "stylesheet", "src": href[:200], "has_integrity": has_integrity})
            if not has_integrity:
                missing_count += 1

    return {
        "total_external": len(results),
        "missing_integrity": missing_count,
        "resources": results
    }


# ==================== MIXED CONTENT ====================

def check_mixed_content(html_content: str, page_url: str) -> Dict[str, Any]:
    """Detect HTTP resources loaded on HTTPS pages."""
    parsed = urlparse(page_url)
    if parsed.scheme != 'https':
        return {"applicable": False, "issues": []}

    soup = BeautifulSoup(html_content, 'html.parser')
    mixed_resources = []

    checks = [
        ('script', 'src'), ('img', 'src'), ('link', 'href'),
        ('iframe', 'src'), ('video', 'src'), ('audio', 'src'),
        ('source', 'src'), ('object', 'data'),
    ]

    for tag_name, attr in checks:
        for el in soup.find_all(tag_name):
            val = el.get(attr, '')
            if val.startswith('http://'):
                mixed_resources.append({
                    "tag": tag_name,
                    "attribute": attr,
                    "url": val[:200]
                })

    return {
        "applicable": True,
        "total_mixed": len(mixed_resources),
        "resources": mixed_resources
    }


# ==================== EMAIL / IP EXPOSURE ====================

def detect_exposure(html_content: str) -> Dict[str, Any]:
    """Detect exposed emails and internal IPs in HTML source."""
    emails = list(set(re.findall(r'[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}', html_content)))
    # Internal IPs: 10.x, 172.16-31.x, 192.168.x
    internal_ips = list(set(re.findall(
        r'\b(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3})\b',
        html_content
    )))

    return {
        "emails": emails[:20],
        "internal_ips": internal_ips[:10],
        "email_count": len(emails),
        "ip_count": len(internal_ips)
    }


# ==================== TECHNOLOGY DETECTION ====================

def detect_technology(html_content: str, headers: Dict[str, str]) -> Dict[str, Any]:
    """Detect technologies, frameworks, and CMS from headers and HTML."""
    h = {k.lower(): v for k, v in headers.items()}
    soup = BeautifulSoup(html_content, 'html.parser')
    detected = []

    # Header-based detection
    server = h.get('server', '').lower()
    powered_by = h.get('x-powered-by', '').lower()

    tech_map = {
        'nginx': 'Nginx', 'apache': 'Apache', 'cloudflare': 'Cloudflare',
        'iis': 'Microsoft IIS', 'litespeed': 'LiteSpeed',
    }
    for key, name in tech_map.items():
        if key in server:
            detected.append({"name": name, "source": "server header", "detail": h.get('server', '')})

    if 'php' in powered_by:
        detected.append({"name": "PHP", "source": "x-powered-by", "detail": powered_by})
    if 'asp.net' in powered_by:
        detected.append({"name": "ASP.NET", "source": "x-powered-by", "detail": powered_by})
    if 'express' in powered_by:
        detected.append({"name": "Express.js", "source": "x-powered-by", "detail": powered_by})

    # HTML meta generator
    gen = soup.find('meta', attrs={'name': 'generator'})
    if gen and gen.get('content'):
        detected.append({"name": gen['content'], "source": "meta[generator]", "detail": gen['content']})

    # Framework/CMS detection from HTML patterns
    html_lower = html_content[:50000].lower()
    patterns = [
        ('wp-content/', 'WordPress'), ('wp-includes/', 'WordPress'),
        ('drupal.js', 'Drupal'), ('joomla', 'Joomla'),
        ('shopify', 'Shopify'), ('squarespace', 'Squarespace'),
        ('wix.com', 'Wix'), ('_next/', 'Next.js'),
        ('__nuxt', 'Nuxt.js'), ('ng-version', 'Angular'),
        ('data-reactroot', 'React'), ('data-v-', 'Vue.js'),
        ('gatsby', 'Gatsby'), ('svelte', 'Svelte'),
        ('bootstrap', 'Bootstrap'), ('tailwindcss', 'TailwindCSS'),
        ('jquery', 'jQuery'), ('laravel', 'Laravel'),
    ]
    seen = set()
    for pattern, name in patterns:
        if pattern in html_lower and name not in seen:
            detected.append({"name": name, "source": "html pattern", "detail": pattern})
            seen.add(name)

    return {
        "technologies": detected,
        "count": len(detected)
    }


def _extract_version(value: str) -> Optional[str]:
    if not value:
        return None
    match = re.search(r'(\d+\.\d+(?:\.\d+)?)', value)
    if match:
        return match.group(1)
    return None


def analyze_technology_vulnerabilities(technology_info: Dict[str, Any]) -> Dict[str, Any]:
    technologies = technology_info.get("technologies", []) if technology_info else []
    if not technologies:
        return {
            "available": False,
            "provider": "nvd",
            "status": "no_technology_detected",
            "total_cves": 0,
            "findings": [],
            "errors": [],
        }

    excluded_keywords = {
        "nginx",
        "apache",
        "cloudflare",
        "litespeed",
        "microsoft iis",
        "php",
        "asp.net",
    }
    query_candidates = []
    seen = set()
    for tech in technologies:
        name = (tech.get("name") or "").strip()
        detail = (tech.get("detail") or "").strip()
        if not name:
            continue
        keyword = name.lower()
        if keyword in excluded_keywords:
            continue
        version = _extract_version(name) or _extract_version(detail)
        dedupe_key = (keyword, version or "")
        if dedupe_key in seen:
            continue
        seen.add(dedupe_key)
        query_candidates.append({
            "technology": name,
            "keyword": keyword,
            "version": version,
        })

    findings = []
    errors = []
    max_candidates = 5
    for candidate in query_candidates[:max_candidates]:
        keyword = candidate["keyword"]
        version = candidate["version"]
        query = f"{keyword} {version}" if version else keyword
        try:
            response = requests.get(
                "https://services.nvd.nist.gov/rest/json/cves/2.0",
                params={"keywordSearch": query, "resultsPerPage": 5},
                timeout=5,
            )
            response.raise_for_status()
            payload = response.json()
            vulnerabilities = payload.get("vulnerabilities", [])
            cves = []
            for entry in vulnerabilities[:5]:
                cve_data = entry.get("cve", {})
                metrics = cve_data.get("metrics", {})
                severity = None
                score = None
                for metric_key in ["cvssMetricV31", "cvssMetricV30", "cvssMetricV2"]:
                    metric_values = metrics.get(metric_key, [])
                    if metric_values:
                        metric = metric_values[0].get("cvssData", {})
                        severity = metric.get("baseSeverity") or severity
                        score = metric.get("baseScore") or score
                        break

                description = ""
                for desc in cve_data.get("descriptions", []):
                    if desc.get("lang") == "en":
                        description = desc.get("value", "")
                        break

                cves.append({
                    "id": cve_data.get("id"),
                    "published": cve_data.get("published"),
                    "last_modified": cve_data.get("lastModified"),
                    "severity": severity,
                    "cvss_score": score,
                    "description": description[:300],
                })

            if cves:
                findings.append({
                    "technology": candidate["technology"],
                    "keyword": keyword,
                    "version": version,
                    "count": len(cves),
                    "cves": cves,
                })
        except Exception as exc:
            errors.append(f"{candidate['technology']}: {str(exc)}")

    total_cves = sum(item.get("count", 0) for item in findings)
    status = "ok"
    if not findings and errors:
        status = "unavailable"
    elif findings and errors:
        status = "partial"

    return {
        "available": bool(findings) or not errors,
        "provider": "nvd",
        "status": status,
        "total_cves": total_cves,
        "findings": findings,
        "errors": errors[:5],
    }


def _check_google_safe_browsing(url: str) -> Dict[str, Any]:
    api_key = os.getenv("GOOGLE_SAFE_BROWSING_API_KEY", "").strip()
    if not api_key:
        return {
            "provider": "google_safe_browsing",
            "available": False,
            "listed": None,
            "status": "skipped",
            "reason": "missing_api_key",
        }

    endpoint = f"https://safebrowsing.googleapis.com/v4/threatMatches:find?key={api_key}"
    payload = {
        "client": {"clientId": "xwa", "clientVersion": "1.0.0"},
        "threatInfo": {
            "threatTypes": ["MALWARE", "SOCIAL_ENGINEERING", "UNWANTED_SOFTWARE"],
            "platformTypes": ["ANY_PLATFORM"],
            "threatEntryTypes": ["URL"],
            "threatEntries": [{"url": url}],
        },
    }
    try:
        response = requests.post(endpoint, json=payload, timeout=8)
        response.raise_for_status()
        body = response.json()
        matches = body.get("matches", [])
        return {
            "provider": "google_safe_browsing",
            "available": True,
            "listed": len(matches) > 0,
            "matches": matches[:5],
            "status": "ok",
        }
    except Exception as exc:
        return {
            "provider": "google_safe_browsing",
            "available": False,
            "listed": None,
            "status": "error",
            "error": str(exc),
        }


def _check_urlhaus(url: str) -> Dict[str, Any]:
    try:
        response = requests.post(
            "https://urlhaus-api.abuse.ch/v1/url/",
            data={"url": url},
            timeout=8,
        )
        response.raise_for_status()
        body = response.json()
        query_status = body.get("query_status")
        listed = query_status == "ok"
        return {
            "provider": "urlhaus",
            "available": True,
            "listed": listed,
            "status": "ok",
            "threat": body.get("threat"),
            "url_status": body.get("url_status"),
            "tags": body.get("tags", []),
        }
    except Exception as exc:
        return {
            "provider": "urlhaus",
            "available": False,
            "listed": None,
            "status": "error",
            "error": str(exc),
        }


def analyze_blacklists(url: str) -> Dict[str, Any]:
    checks = [
        _check_google_safe_browsing(url),
        _check_urlhaus(url),
    ]
    available = [c for c in checks if c.get("available")]
    listed = [c for c in available if c.get("listed") is True]
    status = "ok"
    if not available:
        status = "unavailable"
    elif listed:
        status = "listed"

    return {
        "status": status,
        "is_listed": len(listed) > 0,
        "providers": checks,
    }


def _dns_txt_records(name: str) -> List[str]:
    try:
        answers = dns.resolver.resolve(name, "TXT", lifetime=4)
        records = []
        for rdata in answers:
            value = "".join(part.decode("utf-8", errors="ignore") for part in rdata.strings)
            records.append(value)
        return records
    except Exception:
        return []


def analyze_dns_security(url: str) -> Dict[str, Any]:
    parsed = urlparse(url)
    domain = parsed.hostname or ""
    if not domain:
        return {
            "domain": None,
            "status": "unavailable",
            "error": "invalid_domain",
        }

    dnskey_records = []
    rrsig_records = []
    dnssec_error = None
    try:
        dnskey_answers = dns.resolver.resolve(domain, "DNSKEY", lifetime=4)
        dnskey_records = [str(r)[:200] for r in dnskey_answers]
    except Exception as exc:
        dnssec_error = str(exc)

    try:
        rrsig_answers = dns.resolver.resolve(domain, "RRSIG", lifetime=4)
        rrsig_records = [str(r)[:200] for r in rrsig_answers]
    except Exception:
        pass

    spf_records = [r for r in _dns_txt_records(domain) if r.lower().startswith("v=spf1")]
    dmarc_records = [r for r in _dns_txt_records(f"_dmarc.{domain}") if r.lower().startswith("v=dmarc1")]

    dkim_selectors = ["default", "selector1", "selector2", "google", "k1", "mail", "dkim"]
    dkim_records = []
    for selector in dkim_selectors:
        selector_name = f"{selector}._domainkey.{domain}"
        for record in _dns_txt_records(selector_name):
            if "dkim" in record.lower() or record.lower().startswith("v=dkim1"):
                dkim_records.append({"selector": selector, "record": record[:250]})

    dnssec_present = bool(dnskey_records) and bool(rrsig_records)
    configured = []
    missing = []
    for name, present in [
        ("DNSSEC", dnssec_present),
        ("SPF", bool(spf_records)),
        ("DKIM", bool(dkim_records)),
        ("DMARC", bool(dmarc_records)),
    ]:
        if present:
            configured.append(name)
        else:
            missing.append(name)

    summary_status = "good"
    if len(configured) == 0:
        summary_status = "critical"
    elif missing:
        summary_status = "partial"

    return {
        "domain": domain,
        "status": summary_status,
        "dnssec": {
            "present": dnssec_present,
            "dnskey_records": dnskey_records[:3],
            "rrsig_records": rrsig_records[:3],
            "error": dnssec_error,
        },
        "spf": {
            "present": bool(spf_records),
            "records": spf_records[:3],
        },
        "dkim": {
            "present": bool(dkim_records),
            "selectors_checked": dkim_selectors,
            "records": dkim_records[:5],
        },
        "dmarc": {
            "present": bool(dmarc_records),
            "records": dmarc_records[:3],
        },
        "summary": {
            "configured": configured,
            "missing": missing,
        },
    }


# ==================== SSL ====================

def analyze_ssl_certificate(url: str) -> Dict[str, Any]:
    """Connects to the host using sockets to extract and verify the SSL cert."""
    parsed = urlparse(url)
    hostname = parsed.hostname
    port = parsed.port or 443

    if not hostname or parsed.scheme != 'https':
        return {"valid": False, "error": "Not an HTTPS URL or invalid hostname."}

    context = ssl.create_default_context()

    try:
        with socket.create_connection((hostname, port), timeout=5) as sock:
            with context.wrap_socket(sock, server_hostname=hostname) as ssock:
                cert = ssock.getpeercert()

                expire_date = datetime.strptime(cert['notAfter'], '%b %d %H:%M:%S %Y %Z')
                days_remaining = (expire_date - datetime.utcnow()).days

                issuer = dict(x[0] for x in cert['issuer'])
                subject = dict(x[0] for x in cert['subject'])

                # SANs
                sans = []
                for item in cert.get('subjectAltName', []):
                    if item[0] == 'DNS':
                        sans.append(item[1])

                return {
                    "valid": True,
                    "issuer": issuer.get('organizationName', issuer.get('commonName', 'Unknown')),
                    "subject": subject.get('commonName', hostname),
                    "days_remaining": days_remaining,
                    "expires_on": str(expire_date),
                    "is_expired": days_remaining < 0,
                    "san": sans[:10],
                    "protocol": ssock.version(),
                }
    except ssl.SSLCertVerificationError as e:
        return {"valid": False, "error": f"Certificate verification failed: {str(e)}"}
    except Exception as e:
        return {"valid": False, "error": f"SSL connection error: {str(e)}"}


# ==================== COOKIES ====================

def analyze_cookies(cookies: Any) -> Dict[str, Any]:
    """Analyzes cookies for Secure, HttpOnly, and SameSite flags."""
    issues = []
    analyzed_cookies = []

    for cookie in cookies:
        c_info = {
            "name": cookie.name,
            "secure": cookie.secure,
            "httponly": cookie.has_nonstandard_attr('HttpOnly') or 'HttpOnly' in cookie._rest,
            "samesite": cookie._rest.get('SameSite', 'Not Set') if hasattr(cookie, '_rest') else 'Not Set'
        }
        analyzed_cookies.append(c_info)

        if not c_info["secure"]:
            issues.append(f"Cookie '{cookie.name}' is missing 'Secure' flag.")
        if not c_info["httponly"]:
            issues.append(f"Cookie '{cookie.name}' is missing 'HttpOnly' flag.")

    return {
        "total": len(analyzed_cookies),
        "cookies": analyzed_cookies,
        "issues": issues
    }


# ==================== SENSITIVE PATHS ====================

SENSITIVE_PATHS = [
    '/.git/', '/.git/config', '/.env', '/.env.bak',
    '/wp-admin/', '/wp-login.php', '/wp-config.php.bak',
    '/admin/', '/administrator/', '/login/',
    '/phpinfo.php', '/info.php',
    '/phpmyadmin/', '/adminer.php',
    '/backup.zip', '/backup.tar.gz', '/db.sql',
    '/.DS_Store', '/.htaccess', '/.htpasswd',
    '/config.php.bak', '/config.yml', '/config.json',
    '/api/docs', '/api/swagger.json', '/swagger-ui/', '/openapi.json',
    '/server-status', '/server-info',
    '/debug/', '/_debug/', '/trace/',
    '/.well-known/security.txt',
    '/robots.txt', '/sitemap.xml',
    '/crossdomain.xml', '/clientaccesspolicy.xml',
    '/elmah.axd', '/web.config',
]

async def check_sensitive_path(session: aiohttp.ClientSession, base_url: str, path: str) -> Optional[str]:
    """Checks a single sensitive path."""
    url = f"{base_url.rstrip('/')}{path}"
    try:
        async with session.head(url, allow_redirects=False, timeout=aiohttp.ClientTimeout(total=5)) as response:
            if response.status in [200, 401, 403]:
                return f"{path} (HTTP {response.status})"
    except Exception:
        pass
    return None

async def brute_force_sensitive_paths(base_url: str) -> List[str]:
    """Concurrently scans for common sensitive directories and files."""
    found_paths = []
    connector = aiohttp.TCPConnector(limit_per_host=10)

    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [check_sensitive_path(session, base_url, path) for path in SENSITIVE_PATHS]
        results = await asyncio.gather(*tasks)

        for res in results:
            if res:
                found_paths.append(res)

    return found_paths


# ==================== HTTP METHODS ====================

async def check_http_methods(url: str) -> Dict[str, Any]:
    """Test which HTTP methods are enabled on the server."""
    methods_to_test = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS', 'TRACE', 'HEAD']
    allowed = []
    dangerous = []

    connector = aiohttp.TCPConnector(limit_per_host=5)
    async with aiohttp.ClientSession(connector=connector) as session:
        # First try OPTIONS
        try:
            async with session.options(url, timeout=aiohttp.ClientTimeout(total=5)) as resp:
                allow_header = resp.headers.get('Allow', '')
                if allow_header:
                    allowed = [m.strip().upper() for m in allow_header.split(',')]
        except Exception:
            pass

        # If OPTIONS didn't give us info, test individual methods
        if not allowed:
            for method in methods_to_test:
                try:
                    async with session.request(method, url, timeout=aiohttp.ClientTimeout(total=3)) as resp:
                        if resp.status not in [405, 501]:
                            allowed.append(method)
                except Exception:
                    pass

    dangerous_methods = {'PUT', 'DELETE', 'TRACE', 'PATCH'}
    dangerous = [m for m in allowed if m in dangerous_methods]

    return {
        "allowed": allowed,
        "dangerous": dangerous,
        "has_dangerous": len(dangerous) > 0
    }


# ==================== MAIN ====================

def run_security_analysis(base_url: str, headers: Dict[str, str], cookies: Any,
                          html_content: str = "") -> Dict[str, Any]:
    """Main execution entry point for Security module."""
    logger.info("Executing Security Analysis modules...")

    sec_headers = analyze_security_headers(headers)
    ssl_info = analyze_ssl_certificate(base_url)
    cookie_info = analyze_cookies(cookies)
    cors_info = analyze_cors(headers)
    csp_info = parse_csp(headers)

    # HTML-based checks
    sri_info = check_subresource_integrity(html_content) if html_content else {}
    mixed_info = check_mixed_content(html_content, base_url) if html_content else {}
    exposure_info = detect_exposure(html_content) if html_content else {}
    tech_info = detect_technology(html_content, headers) if html_content else {}
    vulnerabilities_info = analyze_technology_vulnerabilities(tech_info)
    blacklist_info = analyze_blacklists(base_url)
    dns_security_info = analyze_dns_security(base_url)

    logger.info("Brute forcing sensitive paths (async)...")
    sensitive_paths = asyncio.run(brute_force_sensitive_paths(base_url))

    logger.info("Testing HTTP methods (async)...")
    http_methods = asyncio.run(check_http_methods(base_url))

    return {
        "headers": sec_headers,
        "ssl": ssl_info,
        "cookies": cookie_info,
        "sensitive_paths_found": sensitive_paths,
        "cors": cors_info,
        "csp": csp_info,
        "sri": sri_info,
        "mixed_content": mixed_info,
        "exposure": exposure_info,
        "technology": tech_info,
        "vulnerabilities": vulnerabilities_info,
        "blacklist": blacklist_info,
        "dns_security": dns_security_info,
        "http_methods": http_methods,
    }


def run_security_snapshot(url: str, headers: Dict[str, str], html_content: str = "") -> Dict[str, Any]:
    """Fast per-URL security snapshot for UI drill-downs.

    This avoids expensive checks like SSL sockets, sensitive path brute-force,
    and HTTP method probing for every discovered URL.
    """
    sec_headers = analyze_security_headers(headers)
    cors_info = analyze_cors(headers)
    csp_info = parse_csp(headers)

    sri_info = check_subresource_integrity(html_content) if html_content else {}
    mixed_info = check_mixed_content(html_content, url) if html_content else {}
    exposure_info = detect_exposure(html_content) if html_content else {}
    tech_info = detect_technology(html_content, headers) if html_content else {}

    return {
        "url": url,
        "headers": sec_headers,
        "cors": cors_info,
        "csp": csp_info,
        "sri": sri_info,
        "mixed_content": mixed_info,
        "exposure": exposure_info,
        "technology": tech_info,
    }
