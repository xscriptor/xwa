KNOWN_VERSIONS: dict[str, list[str]] = {
    "react": ["18.3.1", "18.2.0", "18.1.0", "18.0.0", "17.0.2", "17.0.1", "16.14.0"],
    "vue": ["3.5.13", "3.4.21", "3.3.4", "3.2.47", "3.0.11", "2.7.16", "2.6.14"],
    "angular": ["19.2.5", "19.1.7", "18.2.13", "17.3.12", "16.2.12", "15.2.10"],
    "svelte": ["5.25.3", "5.0.5", "4.2.19", "4.0.0", "3.59.2"],
    "jquery": ["3.7.1", "3.6.4", "3.5.1", "3.4.1", "2.2.4"],
    "nextjs": ["15.2.4", "15.1.7", "14.2.26", "14.0.4", "13.5.7"],
    "nuxt": ["3.16.0", "3.15.4", "3.14.1592", "3.0.0", "2.18.1"],
    "gatsby": ["5.14.3", "5.13.7", "5.12.0", "4.25.9"],
    "remix": ["2.16.0", "2.15.3", "2.14.0", "1.19.3"],
    "lodash": ["4.17.21", "4.17.20", "4.17.15", "4.17.11"],
    "moment": ["2.30.1", "2.29.4", "2.29.3", "2.24.0"],
    "dayjs": ["1.11.13", "1.11.10", "1.11.7", "1.10.7"],
    "axios": ["1.7.9", "1.6.8", "1.5.1", "1.4.0", "0.27.2"],
    "chart.js": ["4.4.8", "4.4.1", "4.3.0", "3.9.1"],
    "d3": ["7.9.0", "7.8.5", "7.6.1", "6.7.0"],
    "bootstrap": ["5.3.3", "5.2.3", "5.1.3", "4.6.2", "3.4.1"],
    "tailwindcss": ["4.1.2", "4.0.0", "3.4.17", "3.4.1", "3.3.0"],
    "swiper": ["11.2.5", "11.1.0", "10.3.1", "9.4.1"],
    "gsap": ["3.12.7", "3.12.5", "3.11.5", "3.10.4"],
    "three": ["0.174.0", "0.170.0", "0.164.0", "0.157.0"],
    "webpack": ["5.98.0", "5.94.0", "5.90.0", "5.88.0", "5.75.0"],
    "vite": ["6.2.4", "6.1.0", "5.4.14", "5.0.0", "4.5.9"],
    "esbuild": ["0.25.0", "0.24.0", "0.23.0", "0.19.0"],
    "rollup": ["4.34.0", "4.9.0", "3.29.0", "2.79.0"],
    "parcel": ["2.14.4", "2.12.0", "2.9.3", "1.12.5"],
    "nginx": ["1.27.4", "1.26.3", "1.24.0", "1.22.1"],
    "apache": ["2.4.63", "2.4.62", "2.4.57", "2.4.54"],
    "nodejs": ["22.14.0", "22.13.0", "20.18.0", "20.12.0", "18.20.0"],
    "python": ["3.13.2", "3.12.9", "3.11.11", "3.10.16"],
    "java": ["21.0.6", "21.0.2", "17.0.14", "17.0.2", "11.0.26"],
}


def _parse_version(version_str: str) -> tuple:
    parts = version_str.split(".")
    result = []
    for p in parts:
        try:
            result.append(int(p))
        except ValueError:
            result.append(0)
    while len(result) < 3:
        result.append(0)
    return tuple(result[:3])


def check(tech_name: str, detected_version: str | None) -> dict:
    if not detected_version:
        return {"status": "unknown", "current": None, "latest": None}

    versions = KNOWN_VERSIONS.get(tech_name.lower(), [])
    if not versions:
        return {"status": "unknown", "current": detected_version, "latest": None}

    latest = versions[0]
    latest_tuple = _parse_version(latest)

    try:
        detected_tuple = _parse_version(detected_version)
    except (ValueError, IndexError):
        return {"status": "unknown", "current": detected_version, "latest": latest}

    if detected_tuple >= latest_tuple:
        return {"status": "up_to_date", "current": detected_version, "latest": latest}

    minor_versions = _parse_minor(detected_version)
    latest_minor = _parse_minor(latest)
    if minor_versions != latest_minor:
        return {"status": "major_behind", "current": detected_version, "latest": latest}

    return {"status": "minor_behind", "current": detected_version, "latest": latest}


def _parse_minor(version_str: str) -> tuple:
    parts = version_str.split(".")
    try:
        major = int(parts[0]) if len(parts) > 0 else 0
        minor = int(parts[1]) if len(parts) > 1 else 0
        return (major, minor)
    except (ValueError, IndexError):
        return (0, 0)
