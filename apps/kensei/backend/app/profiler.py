import asyncio
from typing import Any, Callable, Dict, List, Optional
from sqlalchemy.orm import Session
from . import models
from .modules import server_fingerprint, js_analyzer, spa_discovery, ssl_tls

LogCallback = Callable[[str], Any]


async def run_full_profile(
    target: str,
    websocket: Any,
    db: Session,
    profile_id: int,
    timeout_seconds: int = 180,
) -> Dict[str, Any]:
    results: Dict[str, Any] = {}

    async def log(msg: str):
        await websocket.send_text(msg)

    await log(f"[kensei] starting full profile for {target}")

    # Phase 1: SSL/TLS handshake
    await log("[kensei] phase 1/4 — SSL/TLS handshake analysis")
    try:
        ssl_result = await ssl_tls.run(target, log)
        results["ssl_tls"] = ssl_result
        _save_ssl_findings(db, profile_id, ssl_result)
        await log("[kensei] SSL/TLS analysis complete")
    except Exception as e:
        await log(f"[kensei] SSL/TLS error: {str(e)}")
        results["ssl_tls"] = {"error": str(e)}

    # Phase 2: Server Fingerprinting
    await log("[kensei] phase 2/4 — server fingerprinting")
    try:
        fingerprint = await server_fingerprint.run(target, log)
        results["server"] = fingerprint
        _save_server_findings(db, profile_id, fingerprint)
        await log("[kensei] server fingerprinting complete")
    except Exception as e:
        await log(f"[kensei] server fingerprinting error: {str(e)}")
        results["server"] = {"error": str(e)}

    # Phase 3: JS Bundle Analysis
    await log("[kensei] phase 3/4 — JS bundle analysis")
    try:
        js_results = await js_analyzer.run(target, log)
        results["js_analysis"] = js_results
        _save_js_findings(db, profile_id, js_results)
        await log("[kensei] JS bundle analysis complete")
    except Exception as e:
        await log(f"[kensei] JS bundle analysis error: {str(e)}")
        results["js_analysis"] = {"error": str(e)}

    # Phase 4: SPA Route Discovery
    await log("[kensei] phase 4/4 — SPA route discovery")
    try:
        routes = await spa_discovery.run(target, log)
        results["spa_routes"] = routes
        _save_route_findings(db, profile_id, routes)
        await log("[kensei] SPA route discovery complete")
    except Exception as e:
        await log(f"[kensei] SPA route discovery error: {str(e)}")
        results["spa_routes"] = {"error": str(e)}

    await log("[kensei] profile complete")
    return results


def _save_ssl_findings(db: Session, profile_id: int, ssl_result: Dict[str, Any]):
    tls_version = ssl_result.get("tls_version")
    if tls_version:
        tech = models.Technology(
            profile_id=profile_id,
            category="security",
            name=f"TLS/{tls_version}",
            version=tls_version,
            confidence="high",
            evidence=ssl_result.get("cipher_suite", ""),
        )
        db.add(tech)

    cipher = ssl_result.get("cipher_suite")
    if cipher:
        tech = models.Technology(
            profile_id=profile_id,
            category="security",
            name=f"Cipher/{cipher.split('-')[0] if '-' in cipher else cipher}",
            version=cipher,
            confidence="high",
            evidence=f"strength: {ssl_result.get('cipher_strength', 'unknown')}",
        )
        db.add(tech)

    cert_subject = ssl_result.get("cert_subject")
    if cert_subject:
        cn = cert_subject.get("commonName", "unknown")
        tech = models.Technology(
            profile_id=profile_id,
            category="certificate",
            name=f"CN={cn}",
            version=None,
            confidence="high",
            evidence=f"issuer: {ssl_result.get('cert_issuer', {}).get('organizationName', 'unknown')} | expires: {ssl_result.get('cert_expiry', 'unknown')}",
        )
        db.add(tech)

    cert_fp = ssl_result.get("cert_fingerprint_sha256")
    if cert_fp:
        tech = models.Technology(
            profile_id=profile_id,
            category="certificate",
            name="SHA-256 Fingerprint",
            version=cert_fp[:16] + "...",
            confidence="high",
            evidence=f"SHA256:{cert_fp}",
        )
        db.add(tech)

    db.commit()


def _save_server_findings(db: Session, profile_id: int, fingerprint: Dict[str, Any]):
    servers = fingerprint.get("servers", [])
    for svr in servers:
        tech = models.Technology(
            profile_id=profile_id,
            category="backend",
            name=svr.get("name", "unknown"),
            version=svr.get("version"),
            confidence=svr.get("confidence", "medium"),
            evidence=svr.get("evidence"),
        )
        db.add(tech)

    cdn = fingerprint.get("cdn")
    if cdn:
        tech = models.Technology(
            profile_id=profile_id,
            category="cdn",
            name=cdn.get("name", "unknown"),
            version=cdn.get("version"),
            confidence=cdn.get("confidence", "medium"),
            evidence=cdn.get("evidence"),
        )
        db.add(tech)

    db.commit()


def _save_js_findings(db: Session, profile_id: int, js_results: Dict[str, Any]):
    deps = js_results.get("dependencies", [])
    for dep in deps:
        js_dep = models.JsDependency(
            profile_id=profile_id,
            name=dep.get("name", "unknown"),
            version=dep.get("version"),
            source=dep.get("source"),
            package_manager=dep.get("package_manager", "unknown"),
        )
        db.add(js_dep)

    techs = js_results.get("technologies", [])
    for t in techs:
        tech = models.Technology(
            profile_id=profile_id,
            category="frontend",
            name=t.get("name", "unknown"),
            version=t.get("version"),
            confidence=t.get("confidence", "medium"),
            evidence=t.get("evidence"),
        )
        db.add(tech)

    outdated = js_results.get("outdated", [])
    for o in outdated:
        tech = models.Technology(
            profile_id=profile_id,
            category="outdated",
            name=o.get("name", "unknown"),
            version=f"{o.get('current', '?')} -> {o.get('latest', '?')}",
            confidence="medium",
            evidence=f"status: {o.get('status', 'unknown')}",
        )
        db.add(tech)

    mf = js_results.get("module_federation", {})
    if mf:
        for key in mf:
            if key in ("remote_entries", "exposed_modules"):
                continue
            tech = models.Technology(
                profile_id=profile_id,
                category="architecture",
                name=f"ModuleFederation/{key}",
                version=None,
                confidence="high",
                evidence=f"module federation {key} detected",
            )
            db.add(tech)

    db.commit()


def _save_route_findings(db: Session, profile_id: int, routes: Dict[str, Any]):
    route_list = routes.get("routes", [])
    for r in route_list:
        route = models.DiscoveredRoute(
            profile_id=profile_id,
            path=r.get("path", ""),
            framework=r.get("framework"),
            route_type=r.get("route_type"),
            module=r.get("module"),
        )
        db.add(route)

    guards = routes.get("guards", [])
    for g in guards:
        route = models.DiscoveredRoute(
            profile_id=profile_id,
            path=g.get("guard", ""),
            framework=g.get("framework"),
            route_type="guard",
            module=g.get("source"),
        )
        db.add(route)

    db.commit()
