from fastapi import APIRouter, Depends, HTTPException
from fastapi.responses import StreamingResponse, FileResponse
from sqlmodel import Session, select
from api.db.database import get_session
from api.db.models import ScanRecord
from api.routers.scan import scan_progress
from core.export.jsonc import export_jsonc
from core.export.markdown import export_markdown
import asyncio
from typing import List
import os

router = APIRouter()


def _delete_generated_export_files(scan_id: int) -> List[str]:
    deleted_paths: List[str] = []
    candidates = [
        f"reports/scan_{scan_id}.md",
        f"reports/scan_{scan_id}.jsonc",
    ]

    for path in candidates:
        if os.path.exists(path):
            os.remove(path)
            deleted_paths.append(path)

    return deleted_paths

@router.get("/reports")
def list_reports(skip: int = 0, limit: int = 20, db: Session = Depends(get_session)):
    """List recent scan history with pagination."""
    scans = db.exec(select(ScanRecord).order_by(ScanRecord.id.desc()).offset(skip).limit(limit)).all()
    
    # We strip the full `raw_results` dump to save bandwidth on the list view
    return [
        {
            "id": s.id, 
            "target_url": s.target_url, 
            "urls_found": s.urls_found,
            "broken_links_count": s.broken_links_count,
            "missing_security_headers": s.missing_security_headers,
            "is_ssl_valid": s.is_ssl_valid,
            "timestamp": s.scan_timestamp
        } 
        for s in scans
    ]

@router.get("/reports/{scan_id}")
def get_report(scan_id: int, db: Session = Depends(get_session)):
    """Fetch the complete JSON results of a specific scan."""
    scan = db.get(ScanRecord, scan_id)
    if not scan:
        raise HTTPException(status_code=404, detail="Scan not found.")
    
    if "Completed" not in scan_progress.get(scan_id, "Completed"):
        return {"status": "In Progress", "current_step": scan_progress.get(scan_id)}
        
    return scan.raw_results

@router.get("/progress/{scan_id}")
async def stream_progress(scan_id: int):
    """Server-Sent Events (SSE) to stream live scan progress to frontend."""
    async def event_generator():
        while True:
            # Check the in-memory progress tracker
            status = scan_progress.get(scan_id, "Unknown")
            yield f"data: {status}\n\n"
            
            if "Completed" in status or "Error" in status:
                break
                
            await asyncio.sleep(1) # Send update every 1 second
            
    return StreamingResponse(event_generator(), media_type="text/event-stream")

@router.get("/export/md/{scan_id}")
def download_md(scan_id: int, db: Session = Depends(get_session)):
    """Export the report as Markdown."""
    scan = db.get(ScanRecord, scan_id)
    if not scan:
        raise HTTPException(status_code=404, detail="Scan not found.")
        
    path = f"reports/scan_{scan_id}.md"
    export_markdown(scan.raw_results, path)
    return FileResponse(path, filename=f"xwa_scan_{scan_id}.md")

@router.get("/export/jsonc/{scan_id}")
def download_jsonc(scan_id: int, db: Session = Depends(get_session)):
    """Export the report as JSONC."""
    scan = db.get(ScanRecord, scan_id)
    if not scan:
        raise HTTPException(status_code=404, detail="Scan not found.")
        
    path = f"reports/scan_{scan_id}.jsonc"
    export_jsonc(scan.raw_results, path)
    return FileResponse(path, filename=f"xwa_scan_{scan_id}.jsonc")


@router.delete("/reports/{scan_id}")
def delete_report(scan_id: int, db: Session = Depends(get_session)):
    """Delete a scan record and any generated export files for that scan."""
    scan = db.get(ScanRecord, scan_id)
    if not scan:
        raise HTTPException(status_code=404, detail="Scan not found.")

    deleted_files = _delete_generated_export_files(scan_id)
    db.delete(scan)
    db.commit()
    scan_progress.pop(scan_id, None)

    return {
        "message": "Scan deleted successfully.",
        "deleted_scan_id": scan_id,
        "deleted_files": deleted_files,
    }


@router.delete("/reports")
def delete_all_reports(db: Session = Depends(get_session)):
    """Delete all stored scan records and their generated export files."""
    scans = db.exec(select(ScanRecord)).all()
    scan_ids = [scan.id for scan in scans if scan.id is not None]
    deleted_files: List[str] = []

    for scan_id in scan_ids:
        deleted_files.extend(_delete_generated_export_files(scan_id))

    for scan in scans:
        db.delete(scan)

    db.commit()

    for scan_id in scan_ids:
        scan_progress.pop(scan_id, None)

    return {
        "message": "All scans deleted successfully.",
        "deleted_scan_ids": scan_ids,
        "deleted_files": deleted_files,
    }
