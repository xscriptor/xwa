import asyncio
import json
from typing import Dict
from fastapi import FastAPI, Depends, WebSocket, WebSocketDisconnect, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import Response
from sqlalchemy.orm import Session, joinedload
from . import models, database, profiler

app = FastAPI(title="Kensei API", description="Web Technology Stack Profiler API", version="0.2.0")

@app.on_event("startup")
def init_database():
    database.wait_for_db()
    models.Base.metadata.create_all(bind=database.engine)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/")
def read_root():
    return {"status": "ok", "message": "Kensei Engine Running with WebSockets enabled"}

@app.websocket("/api/profile/live")
async def websocket_profile(
    websocket: WebSocket,
    target: str,
    timeout: int = 180,
    db: Session = Depends(database.get_db)
):
    await websocket.accept()
    profile_record = None

    try:
        await asyncio.wait_for(websocket.send_text("[LOG] [init] profiling session established"), timeout=10)
    except (asyncio.TimeoutError, Exception):
        return

    try:
        profile_record = models.Profile(
            domain_target=target,
            status="RUNNING"
        )
        db.add(profile_record)
        db.commit()
        db.refresh(profile_record)

        await websocket.send_text(f"[PROFILE_META] profile_id={profile_record.id}")

        results = await profiler.run_full_profile(
            target, websocket, db, profile_record.id, timeout_seconds=timeout
        )

        profile_record.status = "COMPLETED"
        db.commit()
        await websocket.send_text("[done] profiling complete and saved to history")

    except WebSocketDisconnect:
        if profile_record:
            profile_record.status = "CANCELLED"
            db.commit()
    except Exception as e:
        if profile_record:
            profile_record.status = "ERROR"
            db.commit()
        try:
            await websocket.send_text(f"[!] CRITICAL ERROR: {str(e)}")
        except Exception:
            pass
        try:
            await websocket.close()
        except Exception:
            pass


@app.get("/api/profiles")
def list_profiles(db: Session = Depends(database.get_db)):
    profiles = db.query(models.Profile).order_by(models.Profile.id.desc()).all()
    return profiles


@app.get("/api/profiles/{profile_id}")
def get_profile_details(profile_id: int, db: Session = Depends(database.get_db)):
    profile = db.query(models.Profile)\
             .options(
                 joinedload(models.Profile.technologies),
                 joinedload(models.Profile.routes),
                 joinedload(models.Profile.js_dependencies),
             )\
             .filter(models.Profile.id == profile_id)\
             .first()
    if not profile:
        raise HTTPException(status_code=404, detail="Profile not found")
    return profile


@app.delete("/api/profiles/{profile_id}")
def delete_profile(profile_id: int, db: Session = Depends(database.get_db)):
    profile = db.query(models.Profile).filter(models.Profile.id == profile_id).first()
    if not profile:
        raise HTTPException(status_code=404, detail="Profile not found")
    db.delete(profile)
    db.commit()
    return {"status": "deleted", "profile_id": profile_id}


@app.get("/api/profiles/{profile_id}/export/json")
def export_profile_json(profile_id: int, db: Session = Depends(database.get_db)):
    profile = db.query(models.Profile)\
             .options(
                 joinedload(models.Profile.technologies),
                 joinedload(models.Profile.routes),
                 joinedload(models.Profile.js_dependencies),
             )\
             .filter(models.Profile.id == profile_id)\
             .first()
    if not profile:
        raise HTTPException(status_code=404, detail="Profile not found")

    payload = {
        "domain": profile.domain_target,
        "status": profile.status,
        "created_at": profile.created_at.isoformat() if profile.created_at else None,
        "technologies": [
            {
                "category": t.category,
                "name": t.name,
                "version": t.version,
                "confidence": t.confidence,
                "evidence": t.evidence,
            }
            for t in profile.technologies
        ],
        "routes": [
            {
                "path": r.path,
                "framework": r.framework,
                "route_type": r.route_type,
                "module": r.module,
            }
            for r in profile.routes
        ],
        "js_dependencies": [
            {
                "name": d.name,
                "version": d.version,
                "source": d.source,
                "package_manager": d.package_manager,
            }
            for d in profile.js_dependencies
        ],
    }

    json_bytes = json.dumps(payload, indent=2, ensure_ascii=False).encode("utf-8")
    return Response(
        content=json_bytes,
        media_type="application/json",
        headers={
            "Content-Disposition": f'attachment; filename=kensei-profile-{profile_id}.json'
        },
    )


@app.get("/api/version-db")
def get_version_db():
    from .modules.version_db import KNOWN_VERSIONS
    return {
        name: {"latest": versions[0], "all": versions}
        for name, versions in KNOWN_VERSIONS.items()
    }


@app.get("/api/profiles/{profile_id}/report")
def build_report(profile_id: int, db: Session = Depends(database.get_db)):
    profile = db.query(models.Profile)\
             .options(
                 joinedload(models.Profile.technologies),
                 joinedload(models.Profile.routes),
                 joinedload(models.Profile.js_dependencies),
             )\
             .filter(models.Profile.id == profile_id)\
             .first()
    if not profile:
        raise HTTPException(status_code=404, detail="Profile not found")

    tech_by_category: Dict[str, list] = {}
    for t in profile.technologies:
        tech_by_category.setdefault(t.category, []).append({
            "name": t.name, "version": t.version, "confidence": t.confidence,
        })

    guard_count = len([r for r in profile.routes if r.route_type == "guard"])
    route_count = len([r for r in profile.routes if r.route_type != "guard"])
    dep_count = len(profile.js_dependencies)

    report = {
        "domain": profile.domain_target,
        "profile_id": profile_id,
        "created_at": profile.created_at.isoformat() if profile.created_at else None,
        "summary": {
            "technologies_found": len(profile.technologies),
            "routes_discovered": route_count,
            "guards_detected": guard_count,
            "js_dependencies_found": dep_count,
            "categories": list(tech_by_category.keys()),
        },
        "technologies_by_category": tech_by_category,
    }

    outdated = [t for t in profile.technologies if t.category == "outdated"]
    if outdated:
        report["outdated_technologies"] = [
            {"name": t.name, "version": t.version, "evidence": t.evidence}
            for t in outdated
        ]

    return report


@app.delete("/api/profiles")
def delete_all_profiles(db: Session = Depends(database.get_db)):
    count = db.query(models.Profile).delete()
    db.commit()
    return {"status": "deleted", "count": count}

@app.get("/api/profiles/compare")
def compare_profiles(
    ids: str,
    db: Session = Depends(database.get_db),
):
    id_list = [int(x.strip()) for x in ids.split(",") if x.strip().isdigit()]
    if len(id_list) < 2:
        raise HTTPException(status_code=400, detail="Provide at least 2 profile IDs (comma-separated)")

    profiles = db.query(models.Profile).filter(models.Profile.id.in_(id_list)).all()
    if len(profiles) != len(id_list):
        raise HTTPException(status_code=404, detail="One or more profiles not found")

    result = []
    for p in sorted(profiles, key=lambda x: x.id):
        tech_names = set()
        for t in p.technologies:
            if t.category not in ("outdated",):
                tech_names.add(f"{t.name}@{t.version or '?'}")

        result.append({
            "profile_id": p.id,
            "domain": p.domain_target,
            "created_at": p.created_at.isoformat() if p.created_at else None,
            "technology_count": len(p.technologies),
            "technologies": sorted(tech_names),
        })

    diffs = []
    for i in range(1, len(result)):
        prev_techs = set(result[i - 1]["technologies"])
        curr_techs = set(result[i]["technologies"])
        added = curr_techs - prev_techs
        removed = prev_techs - curr_techs
        if added or removed:
            diffs.append({
                "from_profile": result[i - 1]["profile_id"],
                "to_profile": result[i]["profile_id"],
                "added": sorted(added),
                "removed": sorted(removed),
            })

    return {
        "profiles": result,
        "changes": diffs,
    }
