import asyncio
import re
import ssl
import socket
from typing import Any, Callable, Dict, List, Optional
from urllib.parse import urlparse

LogCallback = Callable[[str], Any]

TLS_VERSIONS = {
    ssl.TLSVersion.TLSv1: "TLS 1.0",
    ssl.TLSVersion.TLSv1_1: "TLS 1.1",
    ssl.TLSVersion.TLSv1_2: "TLS 1.2",
    ssl.TLSVersion.TLSv1_3: "TLS 1.3",
}

CIPHER_STRENGTH: Dict[str, str] = {
    "CHACHA20": "high", "AES_256": "high", "AES_128": "high",
    "CAMELLIA": "medium", "3DES": "low", "DES": "low",
    "RC4": "low", "NULL": "none", "EXPORT": "none",
}

CERT_SIGNATURE_ALGOS = {
    "sha256WithRSAEncryption": "SHA-256 + RSA",
    "sha384WithRSAEncryption": "SHA-384 + RSA",
    "sha512WithRSAEncryption": "SHA-512 + RSA",
    "sha256WithECDSA": "SHA-256 + ECDSA",
    "sha384WithECDSA": "SHA-384 + ECDSA",
    "sha512WithECDSA": "SHA-512 + ECDSA",
    "sha1WithRSAEncryption": "SHA-1 + RSA (weak)",
    "sha1WithECDSA": "SHA-1 + ECDSA (weak)",
    "md5WithRSAEncryption": "MD5 + RSA (insecure)",
}


async def run(target: str, log: LogCallback) -> Dict[str, Any]:
    hostname = target
    if target.startswith("http"):
        hostname = urlparse(target).hostname or target

    await log(f"[ssl] probing TLS for {hostname}")

    result: Dict[str, Any] = {
        "hostname": hostname, "tls_version": None, "cipher_suite": None,
        "cipher_strength": None, "certificate": None, "cert_issuer": None,
        "cert_subject": None, "cert_expiry": None,
        "cert_signature_algorithm": None, "cert_serial": None,
        "cert_fingerprint_sha256": None, "sni_supported": True, "errors": [],
    }

    try:
        max_version = await _probe_tls_version(hostname, ssl.TLSVersion.TLSv1_3)
        if max_version:
            result["tls_version"] = TLS_VERSIONS.get(max_version, str(max_version))
            await log(f"[ssl] max TLS version: {result['tls_version']}")

        supported = await _probe_all_versions(hostname)
        result["tls_versions_supported"] = [TLS_VERSIONS.get(v, str(v)) for v in supported]
        await log(f"[ssl] supported TLS: {', '.join(result['tls_versions_supported'])}")

        cipher, cipher_data = await _get_negotiated_cipher(hostname)
        if cipher and cipher_data:
            result["cipher_suite"] = cipher
            result["tls_version_negotiated"] = TLS_VERSIONS.get(
                cipher_data.get("version"), str(cipher_data.get("version"))
            )
            for name, strength in CIPHER_STRENGTH.items():
                if name in cipher.upper():
                    result["cipher_strength"] = strength
                    break
            if not result["cipher_strength"]:
                result["cipher_strength"] = "unknown"
            await log(f"[ssl] negotiated: {cipher} ({result['cipher_strength']})")

        cert_info = await _get_certificate_info(hostname)
        if cert_info:
            result.update(cert_info)
            await log(f"[ssl] cert: {result.get('cert_subject', 'unknown')} "
                      f"expires {result.get('cert_expiry', 'unknown')}")

        remaining = _days_until_expiry(result.get("cert_expiry"))
        result["cert_days_remaining"] = remaining
        if remaining is not None and remaining < 30:
            result["cert_warning"] = f"Certificate expires in {remaining} days"
            await log(f"[ssl] WARNING: cert expires in {remaining} days")

    except ssl.SSLCertVerificationError as e:
        result["errors"].append(f"cert verification: {str(e)[:100]}")
        await log(f"[ssl] cert verification error: {str(e)[:60]}")
    except ssl.SSLError as e:
        result["errors"].append(f"SSL error: {str(e)[:100]}")
        await log(f"[ssl] SSL error: {str(e)[:60]}")
    except socket.gaierror as e:
        result["errors"].append(f"DNS resolution failed: {str(e)}")
        await log(f"[ssl] DNS error: {str(e)}")
    except socket.timeout:
        result["errors"].append("connection timed out")
        await log("[ssl] connection timed out")
    except Exception as e:
        result["errors"].append(str(e)[:100])
        await log(f"[ssl] error: {str(e)[:60]}")

    return result


async def _probe_tls_version(hostname: str, target_version: int) -> Optional[int]:
    def _probe():
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        ctx.maximum_version = target_version
        with socket.create_connection((hostname, 443), timeout=5) as sock:
            with ctx.wrap_socket(sock, server_hostname=hostname) as ssock:
                return ssock.version()
    try:
        return await asyncio.to_thread(_probe)
    except Exception:
        return None


async def _probe_all_versions(hostname: str) -> List[int]:
    async def _probe_one(ver: int) -> Optional[int]:
        def _probe():
            ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
            ctx.check_hostname = False
            ctx.verify_mode = ssl.CERT_NONE
            ctx.minimum_version = ver
            ctx.maximum_version = ver
            with socket.create_connection((hostname, 443), timeout=3) as sock:
                with ctx.wrap_socket(sock, server_hostname=hostname) as ssock:
                    return ver
        try:
            return await asyncio.to_thread(_probe)
        except Exception:
            return None

    tasks = [
        _probe_one(ssl.TLSVersion.TLSv1),
        _probe_one(ssl.TLSVersion.TLSv1_1),
        _probe_one(ssl.TLSVersion.TLSv1_2),
        _probe_one(ssl.TLSVersion.TLSv1_3),
    ]
    results = await asyncio.gather(*tasks)
    return [r for r in results if r is not None]


async def _get_negotiated_cipher(hostname: str):
    def _probe():
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        with socket.create_connection((hostname, 443), timeout=5) as sock:
            with ctx.wrap_socket(sock, server_hostname=hostname) as ssock:
                cipher_name = ssock.cipher()
                if cipher_name:
                    return cipher_name[0], {"version": ssock.version()}
        return None, None
    try:
        return await asyncio.to_thread(_probe)
    except Exception:
        return None, None


async def _get_certificate_info(hostname: str) -> Optional[Dict[str, Any]]:
    def _probe():
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        with socket.create_connection((hostname, 443), timeout=5) as sock:
            with ctx.wrap_socket(sock, server_hostname=hostname) as ssock:
                cert = ssock.getpeercert()
                if not cert:
                    return None
                info: Dict[str, Any] = {}
                subject = dict(x[0] for x in cert.get("subject", []))
                info["cert_subject"] = subject
                issuer = dict(x[0] for x in cert.get("issuer", []))
                info["cert_issuer"] = issuer
                info["cert_expiry"] = cert.get("notAfter")
                info["cert_serial"] = hex(cert.get("serialNumber", 0)) if cert.get("serialNumber") else None
                sig_algo = _extract_signature_algo(cert)
                if sig_algo:
                    info["cert_signature_algorithm"] = CERT_SIGNATURE_ALGOS.get(sig_algo, sig_algo)
                sans = cert.get("subjectAltName", [])
                if sans:
                    info["cert_san"] = [san[1] for san in sans if san[0] == "DNS"]
                try:
                    import hashlib
                    der = ssock.getpeercert(binary_form=True)
                    info["cert_fingerprint_sha256"] = hashlib.sha256(der).hexdigest()
                except Exception:
                    pass
                return info
    try:
        return await asyncio.to_thread(_probe)
    except Exception:
        return None


def _extract_signature_algo(cert: Dict) -> Optional[str]:
    try:
        sig = cert.get("signatureAlgorithm", "")
        if sig:
            return sig
    except Exception:
        pass
    return None


def _days_until_expiry(expiry_str: Optional[str]) -> Optional[int]:
    if not expiry_str:
        return None
    try:
        from datetime import datetime
        exp = datetime.strptime(expiry_str, "%b %d %H:%M:%S %Y %Z")
        remaining = (exp - datetime.now()).days
        return max(remaining, 0)
    except Exception:
        return None
