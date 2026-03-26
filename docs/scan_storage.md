# Scan Storage & Future Improvements

## Where Are Scan Results Stored?

Every scan triggered through the API is persisted in a local **SQLite** database.

| Detail           | Value                                              |
| ---------------- | -------------------------------------------------- |
| **Engine**       | SQLite (via SQLModel / SQLAlchemy)                 |
| **File**         | `xwa.db` in the project root                       |
| **Connection**   | Defined in `api/db/database.py` → `sqlite:///./xwa.db` |
| **Table**        | `ScanRecord`                                       |
| **Migrations**   | Managed with Alembic (`alembic.ini` + `api/db/migrations/`) |

### ScanRecord Schema

| Column                    | Type       | Description                                        |
| ------------------------- | ---------- | -------------------------------------------------- |
| `id`                      | Integer PK | Auto-incremented identifier                        |
| `target_url`              | String     | The URL that was scanned (indexed)                 |
| `scan_timestamp`          | DateTime   | UTC timestamp of when the scan started             |
| `urls_found`              | Integer    | Number of URLs discovered during crawl             |
| `broken_links_count`      | Integer    | Count of broken (404) links                        |
| `missing_security_headers`| Integer    | Count of absent security headers                   |
| `is_ssl_valid`            | Boolean    | Whether the SSL certificate is valid               |
| `raw_results`             | JSON       | Full scan report stored as a serialized JSON blob  |

The lightweight summary columns (`urls_found`, `broken_links_count`, etc.) power the dashboard list view, while `raw_results` holds the complete `FullScanReport` for the detail page.

