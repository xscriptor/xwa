import json
import re
from typing import Any, Callable, Dict, List, Optional
from urllib.parse import urljoin, urlparse

import httpx

from . import version_db

LogCallback = Callable[[str], Any]

FRAMEWORK_SIGNATURES: Dict[str, List[str]] = {
    "angular": [r"ng-version", r"@angular/core"],
    "react": [r"__REACT_DEVTOOLS_GLOBAL_HOOK__", r"__reactFiber", r"React\.createElement", r"createRoot", r"react@(\d+\.\d+\.\d+)"],
    "vue": [r"vue\.esm", r"__vue__", r"Vue\.js", r"vue@(\d+\.\d+\.\d+)"],
    "nextjs": [r"__NEXT_DATA__"],
    "nuxt": [r"__NUXT__"],
    "svelte": [r"svelte", r"__svelte"],
    "jquery": [r"jquery@(\d+\.\d+\.\d+)", r"jquery\.min\.js"],
    "gatsby": [r"gatsby"],
    "remix": [r"remix"],
    "sapper": [r"sapper"],
}

BUILD_TOOL_SIGNATURES: Dict[str, List[str]] = {
    "webpack": [r"webpack", r"__webpack_require__"],
    "vite": [r"vite", r"__vite__"],
    "esbuild": [r"esbuild"],
    "rollup": [r"rollup"],
    "parcel": [r"parcel"],
}

COMMON_LIBRARIES: List[tuple] = [
    ("lodash", r"lodash@(\d+\.\d+\.\d+)", 1),
    ("moment", r"moment@(\d+\.\d+\.\d+)", 1),
    ("dayjs", r"dayjs@(\d+\.\d+\.\d+)", 1),
    ("axios", r"axios@(\d+\.\d+\.\d+)", 1),
    ("chart.js", r"chart\.js@(\d+\.\d+\.\d+)", 1),
    ("d3", r"d3@(\d+\.\d+\.\d+)", 1),
    ("bootstrap", r"bootstrap@(\d+\.\d+\.\d+)", 1),
    ("tailwindcss", r"tailwindcss@(\d+\.\d+\.\d+)", 1),
    ("material-ui", r"@material-ui", 0),
    ("antd", r"antd", 0),
    ("swiper", r"swiper@(\d+\.\d+\.\d+)", 1),
    ("gsap", r"gsap@(\d+\.\d+\.\d+)", 1),
    ("three", r"three@(\d+\.\d+\.\d+)", 1),
]

MODULE_FEDERATION_SIGNATURES: Dict[str, List[str]] = {
    "exposes": [r"exposes\s*:", r"exposes\s*="],
    "remotes": [r"remotes\s*:", r"remotes\s*="],
    "shared": [r"shared\s*:", r"shared\s*=", r"__webpack_share__"],
    "module_federation_plugin": [r"ModuleFederationPlugin", r"ModuleFederation"],
}


async def run(target: str, log: LogCallback) -> Dict[str, Any]:
    base_url = target if target.startswith("http") else f"https://{target}"
    technologies: List[Dict[str, Any]] = []
    dependencies: List[Dict[str, Any]] = []
    bundle_urls: List[str] = []
    source_maps: List[Dict[str, Any]] = []
    module_federation: Dict[str, Any] = {}
    outdated: List[Dict[str, Any]] = []

    await log(f"[js] analyzing JS bundle for {base_url}")

    async with httpx.AsyncClient(timeout=15, verify=False, follow_redirects=True) as client:
        try:
            response = await client.get(base_url)
            html = response.text
        except Exception as e:
            await log(f"[js] fetch error: {str(e)}")
            return {"technologies": [], "dependencies": [], "bundles": [], "source_maps": [], "module_federation": {}, "outdated": []}

    script_pattern = re.compile(r'<script[^>]+src=["\']([^"\']+)["\']', re.IGNORECASE)
    preload_pattern = re.compile(r'<link[^>]+rel=["\']preload["\'][^>]+href=["\']([^"\']+\.js[^"\']*)["\']', re.IGNORECASE)
    module_pattern = re.compile(r'<script[^>]+type=["\']module["\'][^>]+src=["\']([^"\']+)["\']', re.IGNORECASE)
    bundle_urls = [
        u for u in
        script_pattern.findall(html) + preload_pattern.findall(html) + module_pattern.findall(html)
        if not u.startswith("data:")
    ]

    await log(f"[js] found {len(bundle_urls)} script references")

    resolved_urls = []
    for url in bundle_urls:
        if url.startswith("//"):
            resolved_urls.append(f"https:{url}")
        elif url.startswith("/"):
            parsed = urlparse(base_url)
            resolved_urls.append(f"{parsed.scheme}://{parsed.netloc}{url}")
        elif url.startswith("http"):
            resolved_urls.append(url)
        else:
            resolved_urls.append(urljoin(base_url, url))

    html_lower = html.lower()

    for name, patterns in FRAMEWORK_SIGNATURES.items():
        for pat in patterns:
            match = re.search(pat, html_lower, re.IGNORECASE)
            if match:
                version = match.group(1) if match.lastindex and match.group(1) else None
                technologies.append({
                    "name": name,
                    "version": version,
                    "confidence": "high",
                    "evidence": f"matched pattern: {pat}",
                })
                break

    for name, patterns in BUILD_TOOL_SIGNATURES.items():
        for pat in patterns:
            if re.search(pat, html_lower, re.IGNORECASE):
                technologies.append({
                    "name": name,
                    "version": None,
                    "confidence": "medium",
                    "evidence": f"matched build tool pattern: {pat}",
                })
                break

    for lib_name, pat, vg in COMMON_LIBRARIES:
        match = re.search(pat, html_lower, re.IGNORECASE)
        if match:
            version = match.group(vg) if vg > 0 else None
            dependencies.append({
                "name": lib_name,
                "version": version,
                "source": "inline",
                "package_manager": "unknown",
            })

    bundle_text = ""
    for url in resolved_urls[:5]:
        try:
            resp = await client.get(url, timeout=10)
            if resp.status_code == 200:
                bundle_text += resp.text + "\n"
                await log(f"[js] fetched bundle: {url.split('/')[-1][:60]} ({len(resp.text)}b)")

                # Check for sourceMappingURL
                sm_match = re.search(r'//#\s*sourceMappingURL\s*=\s*([^\s]+)', resp.text)
                if sm_match:
                    sm_url = sm_match.group(1)
                    if not sm_url.startswith("http"):
                        sm_url = urljoin(url, sm_url)
                    source_maps.append({"bundle": url, "source_map_url": sm_url})
                    await log(f"[js] found source map ref: {sm_url.split('/')[-1][:60]}")

                    try:
                        sm_resp = await client.get(sm_url, timeout=10)
                        if sm_resp.status_code == 200:
                            try:
                                sm_data = sm_resp.json()
                                sources = sm_data.get("sources", [])
                                source_maps[-1]["sources"] = sources
                                source_maps[-1]["source_count"] = len(sources)
                                await log(f"[js] source map contains {len(sources)} original files")

                                for dep_name, dep_pat, dep_vg in COMMON_LIBRARIES:
                                    for src in sources:
                                        if dep_name in src.lower():
                                            if not any(d["name"] == dep_name for d in dependencies):
                                                dependencies.append({
                                                    "name": dep_name,
                                                    "version": None,
                                                    "source": "sourcemap",
                                                    "package_manager": "unknown",
                                                })
                                                break
                            except (json.JSONDecodeError, Exception):
                                pass
                    except Exception:
                        pass
        except Exception:
            continue

    if bundle_text:
        bundle_lower = bundle_text.lower()
        for lib_name, lib_pat, _ in COMMON_LIBRARIES:
            if re.search(lib_pat, bundle_text, re.IGNORECASE):
                if not any(d["name"] == lib_name for d in dependencies):
                    dependencies.append({
                        "name": lib_name,
                        "version": None,
                        "source": "bundle",
                        "package_manager": "unknown",
                    })

        for name, patterns in FRAMEWORK_SIGNATURES.items():
            if any(p in bundle_lower for p in patterns):
                if not any(t["name"] == name for t in technologies):
                    technologies.append({
                        "name": name,
                        "version": None,
                        "confidence": "medium",
                        "evidence": "detected in JS bundle",
                    })

        for name, patterns in BUILD_TOOL_SIGNATURES.items():
            if any(p in bundle_lower for p in patterns):
                if not any(t["name"] == name for t in technologies):
                    technologies.append({
                        "name": name,
                        "version": None,
                        "confidence": "low",
                        "evidence": "detected in JS bundle",
                    })

        for name, patterns in MODULE_FEDERATION_SIGNATURES.items():
            for pat in patterns:
                if re.search(pat, bundle_text, re.IGNORECASE):
                    module_federation[name] = True
                    break

        if module_federation:
            await log(f"[js] module federation detected: {list(module_federation.keys())}")

            mf_remotes = re.findall(r'["\']([^"\']+)["\']\s*:\s*["\']([^"\']+)["\']', bundle_text)
            if mf_remotes:
                module_federation["remote_entries"] = [{"name": r[0], "url": r[1]} for r in mf_remotes]

            mf_exposes = re.findall(r'["\']\./([^"\']+)["\']', bundle_text)
            if mf_exposes and module_federation.get("exposes"):
                module_federation["exposed_modules"] = mf_exposes[:20]

    seen = set()
    unique_techs = []
    for t in technologies:
        key = t["name"]
        if key not in seen:
            seen.add(key)
            unique_techs.append(t)
            version_check = version_db.check(t["name"], t.get("version"))
            if version_check["status"] != "unknown" and version_check["status"] != "up_to_date":
                outdated.append({
                    "name": t["name"],
                    "current": t.get("version"),
                    "latest": version_check["latest"],
                    "status": version_check["status"],
                })

    for d in dependencies:
        version_check = version_db.check(d["name"], d.get("version"))
        if version_check["status"] != "unknown" and version_check["status"] != "up_to_date":
            outdated.append({
                "name": d["name"],
                "current": d.get("version"),
                "latest": version_check["latest"],
                "status": version_check["status"],
            })

    if outdated:
        await log(f"[js] outdated deps: {[o['name'] + '@' + (o['current'] or '?') + ' -> ' + o['latest'] for o in outdated]}")

    await log(f"[js] identified technologies: {[t['name'] for t in unique_techs]}")
    await log(f"[js] identified dependencies: {[d['name'] for d in dependencies]}")
    if source_maps:
        await log(f"[js] source maps found: {len(source_maps)}")

    return {
        "technologies": unique_techs,
        "dependencies": dependencies,
        "bundles": resolved_urls,
        "source_maps": source_maps,
        "module_federation": module_federation,
        "outdated": outdated,
    }
