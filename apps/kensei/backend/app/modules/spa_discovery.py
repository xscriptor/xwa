import json
import re
from typing import Any, Callable, Dict, List, Optional
from urllib.parse import urljoin, urlparse

import httpx

LogCallback = Callable[[str], Any]

NG_ROUTE_PATTERNS = [
    (r"(?:path|component)\s*:\s*['\"]([a-z_\/][a-z0-9_\/\-:]*)['\"]", "static", "angular"),
    (r"loadChildren\s*[:=]\s*['\"]([^'\"]+?)['\"]", "lazy", "angular"),
    (r"loadComponent\s*[:=]\s*['\"]([^'\"]+?)['\"]", "lazy", "angular"),
    (r"redirectTo\s*[:=]\s*['\"]([^'\"]+?)['\"]", "redirect", "angular"),
    (r"canActivate\s*[:=]\s*\[([^\]]+)\]", "guard", "angular"),
    (r"canDeactivate\s*[:=]\s*\[([^\]]+)\]", "guard", "angular"),
    (r"canLoad\s*[:=]\s*\[([^\]]+)\]", "guard", "angular"),
    (r"children\s*:", "nested", "angular"),
]

REACT_ROUTE_PATTERNS = [
    (r"path\s*:\s*['\"]([a-z_\/][a-z0-9_\/\-:]*)['\"]", "static", "react"),
    (r"element\s*:\s*<([^>]+)>", "component", "react"),
    (r"lazy\s*\(\s*\(\)\s*=>\s*import\(", "lazy", "react"),
    (r"createBrowserRouter", "config", "react"),
    (r"createHashRouter", "config", "react"),
    (r"Route\s+path=['\"]([a-z_\/][a-z0-9_\/\-:]*)['\"]", "static", "react"),
    (r"PrivateRoute", "guard", "react"),
    (r"ProtectedRoute", "guard", "react"),
    (r"RequireAuth", "guard", "react"),
    (r"AuthGuard", "guard", "react"),
    (r"withAuth\s*\(", "guard", "react"),
    (r"useAuth\s*\(", "guard", "react"),
    (r"isAuthenticated", "guard", "react"),
    (r"<Navigate\s+to=['\"]([^'\"]+)['\"]", "redirect", "react"),
    (r"navigate\s*\(\s*['\"]([^'\"]+)['\"]", "redirect", "react"),
    (r"<Outlet\s*/?>", "nested", "react"),
    (r"children\s*:\s*\[", "nested", "react"),
]

VUE_ROUTE_PATTERNS = [
    (r"path\s*:\s*['\"]([a-z_\/][a-z0-9_\/\-:]*)['\"]", "static", "vue"),
    (r"component\s*:\s*\(\)\s*=>\s*import\(", "lazy", "vue"),
    (r"name\s*:\s*['\"]([^'\"]+)['\"]", "named", "vue"),
    (r"children\s*:", "nested", "vue"),
    (r"beforeEnter\s*:", "guard", "vue"),
    (r"meta\s*:\s*\{", "meta", "vue"),
    (r"requiresAuth", "guard", "vue"),
    (r"redirect\s*:\s*['\"]([^'\"]+)['\"]", "redirect", "vue"),
    (r"alias\s*:\s*['\"]([^'\"]+)['\"]", "alias", "vue"),
]

ALL_PATTERNS = NG_ROUTE_PATTERNS + REACT_ROUTE_PATTERNS + VUE_ROUTE_PATTERNS


async def run(target: str, log: LogCallback) -> Dict[str, Any]:
    base_url = target if target.startswith("http") else f"https://{target}"
    routes: List[Dict[str, Any]] = []
    detected_frameworks: List[str] = []
    guards: List[Dict[str, Any]] = []
    bundle_urls: List[str] = []

    await log(f"[spa] discovering SPA routes for {base_url}")

    async with httpx.AsyncClient(timeout=15, verify=False, follow_redirects=True) as client:
        try:
            response = await client.get(base_url)
            html = response.text
        except Exception as e:
            await log(f"[spa] fetch error: {str(e)}")
            return {"routes": [], "frameworks": [], "bundles": [], "guards": []}

    html_lower = html.lower()
    if "ng-version" in html_lower or "angular" in html_lower:
        detected_frameworks.append("angular")
    if "__react" in html_lower or "react." in html_lower or "createRoot" in html_lower:
        detected_frameworks.append("react")
    if "__vue__" in html_lower or "vue.esm" in html_lower or "__nuxt" in html_lower or "data-n-head" in html_lower or "v-application" in html_lower:
        detected_frameworks.append("vue")
    if "__next" in html_lower or "__NEXT_DATA__" in html or "next-data" in html_lower:
        detected_frameworks.append("next")

    await log(f"[spa] detected frameworks: {detected_frameworks}")

    script_src = re.compile(r'<script[^>]+src=["\']([^"\']+)["\']', re.IGNORECASE)
    preload_js = re.compile(r'<link[^>]+rel=["\']preload["\'][^>]+href=["\']([^"\']+\.js[^"\']*)["\']', re.IGNORECASE)
    module_script = re.compile(r'<script[^>]+type=["\']module["\'][^>]+src=["\']([^"\']+)["\']', re.IGNORECASE)
    for url in script_src.findall(html) + preload_js.findall(html) + module_script.findall(html):
        if url.startswith("//"):
            bundle_urls.append(f"https:{url}")
        elif url.startswith("/"):
            parsed = urlparse(base_url)
            bundle_urls.append(f"{parsed.scheme}://{parsed.netloc}{url}")
        elif url.startswith("http"):
            bundle_urls.append(url)
        else:
            bundle_urls.append(urljoin(base_url, url))

    def is_route_path(p: str) -> bool:
        if len(p) < 1: return False
        if p.startswith("#") or p.startswith("."): return False
        if not re.match(r'^[a-z_\/\*\@]', p, re.IGNORECASE): return False
        return True

    def extract_routes(text: str, patterns: list, source: str):
        for pat, route_type, framework in patterns:
            matches = re.finditer(pat, text, re.IGNORECASE)
            for m in matches:
                path = m.group(1) if m.lastindex and m.group(1) else ""
                if path and is_route_path(path) and path not in [r["path"] for r in routes]:
                    routes.append({
                        "path": path,
                        "framework": framework,
                        "route_type": route_type,
                        "module": None,
                    })
                if route_type == "guard":
                    guard_svc = m.group(1) if m.lastindex and m.group(1) else path
                    if guard_svc and is_route_path(guard_svc) and guard_svc not in [g["guard"] for g in guards]:
                        guards.append({
                            "guard": guard_svc,
                            "framework": framework,
                            "source": source,
                        })

    extract_routes(html, ALL_PATTERNS, "html")

    for url in bundle_urls[:5]:
        try:
            resp = await client.get(url, timeout=10)
            if resp.status_code == 200:
                extract_routes(resp.text, ALL_PATTERNS, url.split("/")[-1][:60])
        except Exception:
            continue

    for fw in detected_frameworks:
        fw_routes = [r for r in routes if r["framework"] == fw]
        fw_guards = [g for g in guards if g["framework"] == fw]
        if fw_routes:
            await log(f"[spa] {fw}: {len(fw_routes)} routes, {len(fw_guards)} guards")

    # Build route hierarchy map
    route_tree = _build_route_tree(routes)

    return {
        "routes": routes,
        "frameworks": detected_frameworks,
        "bundles": bundle_urls,
        "guards": guards,
        "route_tree": route_tree,
    }


def _build_route_tree(routes: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    tree: List[Dict[str, Any]] = []
    static_routes = [r for r in routes if r["route_type"] in ("static", "named")]
    parent_paths = set()
    for r in static_routes:
        parts = r["path"].strip("/").split("/")
        for i in range(1, len(parts) + 1):
            prefix = "/" + "/".join(parts[:i])
            parent_paths.add(prefix)

    children_map: Dict[str, List[Dict[str, Any]]] = {}
    for r in static_routes:
        path = "/" + r["path"].strip("/") if r["path"] not in ("", "/") else "/"
        parent = "/".join(path.rstrip("/").split("/")[:-1]) or "/"
        if parent not in children_map:
            children_map[parent] = []
        children_map[parent].append({**r, "_path": path})

    seen = set()
    for r in static_routes:
        path = "/" + r["path"].strip("/") if r["path"] not in ("", "/") else "/"
        if path not in seen:
            seen.add(path)
            children = children_map.get(path, [])
            entry: Dict[str, Any] = {
                "path": path,
                "framework": r["framework"],
                "route_type": r["route_type"],
            }
            if children:
                entry["children"] = [c["path"] for c in children]
            tree.append(entry)

    return tree
