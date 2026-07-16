import json
import os
from datetime import datetime, timezone
from sqlalchemy.orm import Session, joinedload
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from . import models


SAMURAI_VERSION = "2.5.0"


def build_export_payload(db: Session) -> dict:
    all_scans = (
        db.query(models.Scan)
        .options(
            joinedload(models.Scan.findings),
            joinedload(models.Scan.discovered_links).joinedload(
                models.DiscoveredLink.findings
            ),
        )
        .order_by(models.Scan.id.desc())
        .all()
    )

    scans_data = []
    total_findings = 0
    total_links = 0

    for scan in all_scans:
        findings_list = []
        for f in scan.findings:
            findings_list.append(
                {
                    "id": f.id,
                    "severity": f.severity,
                    "finding_type": f.finding_type,
                    "description": f.description,
                    "poc_payload": f.poc_payload,
                    "cvss_score": f.cvss_score,
                    "link_id": f.link_id,
                }
            )

        links_list = []
        for link in scan.discovered_links:
            link_findings = []
            for lf in link.findings:
                link_findings.append(
                    {
                        "id": lf.id,
                        "severity": lf.severity,
                        "finding_type": lf.finding_type,
                        "description": lf.description,
                        "poc_payload": lf.poc_payload,
                        "cvss_score": lf.cvss_score,
                    }
                )
            links_list.append(
                {
                    "id": link.id,
                    "url": link.url,
                    "status_code": link.status_code,
                    "content_type": link.content_type,
                    "findings": link_findings,
                }
            )

        scans_data.append(
            {
                "id": scan.id,
                "domain_target": scan.domain_target,
                "status": scan.status,
                "scan_type": scan.scan_type,
                "created_at": scan.created_at.isoformat() if scan.created_at else None,
                "findings": findings_list,
                "discovered_links": links_list,
            }
        )
        total_findings += len(findings_list) + sum(
            len(link.get("findings", [])) for link in links_list
        )
        total_links += len(links_list)

    payload = {
        "export_metadata": {
            "exported_at": datetime.now(timezone.utc).isoformat(),
            "samurai_version": SAMURAI_VERSION,
            "scan_count": len(all_scans),
            "finding_count": total_findings,
            "link_count": total_links,
        },
        "scans": scans_data,
    }

    return payload


def encrypt_export_payload(payload: dict, password: str) -> bytes:
    salt = os.urandom(16)
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=32,
        salt=salt,
        iterations=600_000,
    )
    key = kdf.derive(password.encode("utf-8"))

    aesgcm = AESGCM(key)
    nonce = os.urandom(12)

    plaintext = json.dumps(payload, indent=2, ensure_ascii=False).encode("utf-8")
    ciphertext = aesgcm.encrypt(nonce, plaintext, None)

    return b"SAMURAI_DB_EXPORT_V1" + salt + nonce + ciphertext


def decrypt_export_payload(file_bytes: bytes, password: str) -> dict:
    header = b"SAMURAI_DB_EXPORT_V1"
    header_len = len(header)

    if not file_bytes.startswith(header):
        raise ValueError("Formato de archivo no válido")

    salt = file_bytes[header_len : header_len + 16]
    nonce = file_bytes[header_len + 16 : header_len + 28]
    ciphertext = file_bytes[header_len + 28 :]

    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=32,
        salt=salt,
        iterations=600_000,
    )
    key = kdf.derive(password.encode("utf-8"))

    aesgcm = AESGCM(key)
    plaintext = aesgcm.decrypt(nonce, ciphertext, None)
    return json.loads(plaintext.decode("utf-8"))
