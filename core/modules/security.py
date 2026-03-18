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
        "http_methods": http_methods,
    }
