# Web Analysis & SEO Dashboard - Roadmap

This document outlines the step-by-step development plan for the project, separated into detailed phases. Progress marked in this file will automatically sync with GitHub Issues.

## Phase 1: Project Setup & Core CLI <!-- phase:project-setup -->

- [x] Initialize the project repository and set up version control conventions. (#1)
- [x] Define the base directory structure (separating core engine, api, and web frontend). (#2)
- [x] Set up Python virtual environment and basic dependencies (e.g., `requests`, `beautifulsoup4`). (#3)
- [x] Create a basic Command Line Interface (CLI) entry point using `argparse` or `Typer`. (#4)
- [x] Implement robust error handling for HTTP requests and invalid URLs in the core engine. (#5)
- [x] Set up standard logging configuration for the core engine execution. (#6)

## Phase 2: SEO Analysis Engine <!-- phase:seo-engine -->

- [x] Implement extractor for standard meta tags (Title, Description, Keywords). (#7)
- [x] Implement extractor for Open Graph (OG) and Twitter Card meta tags. (#8)
- [x] Create a module to analyze header tag hierarchy (H1, H2, H3) and detect missing H1s. (#9)
- [x] Implement a checker for missing `alt` attributes on image tags. (#10)
- [x] Analyze text-to-HTML ratio and calculate basic word count statistics. (#11)
- [x] Create a module to check for robots.txt presence and basic allow/disallow rules. (#12)
- [x] Implement canonical URL verification. (#13)

## Phase 3: Sitemap & Crawler Module <!-- phase:sitemap-crawler -->

- [x] Implement function to automatically locate and download `sitemap.xml` (or `sitemap_index.xml`). (#14)
- [x] Parse XML content to extract all listed URLs efficiently. (#15)
- [x] Implement a concurrent HTTP crawler (e.g., using `asyncio` and `aiohttp`) to validate URL status codes. (#16)
- [x] Detect and report broken links (404s) and redirect chains (301/302). (#17)
- [x] Develop an algorithm to transform a flat list of URL paths into a hierarchical tree structure. (#18)
- [x] Implement rate limiting and sensible timeouts to avoid overloading target servers. (#19)

## Phase 4: Security Analysis Module <!-- phase:security-engine -->

- [x] Scanner for missing or misconfigured security headers (CORS, HSTS, X-Frame-Options, CSP). (#20)
- [x] Implement basic SSL/TLS certificate analysis (expiration date, issuer, validation status). (#21)
- [x] Create a passive directory brute-forcer to detect common sensitive paths (`/.git/`, `/.env`, `/wp-admin/`, etc.). (#22)
- [x] Detect leaked server information headers (e.g., `X-Powered-By`, `Server`). (#23)
- [x] Check for basic cookie security flags (`HttpOnly`, `Secure`, `SameSite`). (#24)

## Phase 5: Export & Serialization Engine <!-- phase:export-engine -->

- [x] Define standardized internal data classes (e.g., Pydantic models) to hold all analysis results. (#25)
- [x] Create a unified serializer that converts internal data models to primitive Python dictionaries. (#26)
- [x] Implement JSONC (JSON with Comments) formatting and file generation. (#27)
- [x] Develop a Markdown (MD) report generator using Jinja2 templates or string formatting. (#28)
- [x] Add CLI flags to specify output formats and export destination paths. (#29)

## Phase 6: Backend API Configuration (FastAPI) <!-- phase:api-setup -->

- [x] Initialize the FastAPI application structure within the project. (#30)
- [x] Configure standard middlewares (CORS for Next.js frontend frontend, GZip). (#31)
- [x] Set up SQLite database connection using SQLAlchemy or SQLModel. (#32)
- [x] Define database schema and ORM models (Analyses, URLs, SEO Results, Security Results). (#33)
- [x] Implement Alembic for database migrations. (#34)

## Phase 7: API Endpoints & Background Tasks <!-- phase:api-endpoints -->

- [x] Create `POST /api/scan` endpoint to receive target URLs and trigger the core engine. (#35)
- [x] Implement background task processing (e.g., Celery, RQ, or FastAPI `BackgroundTasks`) for long-running scans. (#36)
- [x] Create endpoints for WebSocket or Server-Sent Events (SSE) to stream live scan progress to the frontend. (#37)
- [x] Create `GET /api/reports/{id}` endpoint to fetch the complete JSON results of a specific scan. (#38)
- [x] Create `GET /api/reports` endpoint with pagination to list scan history. (#39)
- [x] Implement direct download endpoints for generated MD and JSONC files (e.g., `/api/export/md/{id}`). (#40)

## Phase 8: Web Frontend Setup (Next.js) <!-- phase:frontend-setup -->

- [x] Initialize Next.js project inside the `web/` directory using TypeScript and chosen framework. (#41)
- [x] Set up global styles (`globals.css`) and basic layout components (Navbar, Footer, Sidebar). (#42)
- [x] Create UI components for entering target URLs and initiating scans. (#43)
- [x] Implement Server-Sent Events (SSE) consumer in the frontend to display live progress bars and real-time logs. (#44)
- [x] Build Dashboard Overview page featuring high-level metrics (cards for total URLs, security score, broken links). (#45)

## Phase 9: Web Frontend Features & Views <!-- phase:frontend-features -->

- [x] Create detailed SEO Analysis view component (Meta tags, Structure, Images). (#46)
- [x] Create detailed Sitemap Analysis view component (Visual tree, Broken links table). (#47)
- [x] Create detailed Security Analysis view component (Headers, SSL status, Exposures). (#48)
- [x] Build the Scan History page (`/reports`) displaying past scans in a paginated table. (#49)
- [x] Implement responsive design for mobile and tablet views, including Light/Dark themes. (#50)

## Phase 10: Local Multiplatform Tooling <!-- phase:local-tooling -->

- [ ] Research and initialize a Terminal UI (TUI) library for Python (e.g., `Textual` or `Rich`). (#54)
- [ ] Port the core CLI functionality to the TUI, creating visually distinct panes for SEO, Security, and Sitemaps natively in the terminal. (#55)
- [ ] Configure PyInstaller, Nuitka, or PyOxidizer to package the Python engine into a single executable binary. (#56)
- [ ] Automate build processes for cross-compiling the binary for Linux, Windows, and macOS. (#57)
- [ ] Write comprehensive installation and usage documentation for running the tool locally without the web dashboard. (#58)
- [ ] Explore native GUI alternatives (e.g., PyQt/PySide6) for a standalone desktop application experience. (#59)


## Phase 11: Template Parity & UX Alignment <!-- phase:template-parity-ux -->

- [x] Move report modules to independent routes under `/reports/{id}/{section}`. (#61)
- [x] Keep one top navigation bar and show report page tabs in top nav when viewing a report. (#62)
- [x] Add dynamic right-side quick navigation links that point to sections inside each report page. (#63)
- [x] Keep footer developer names as direct profile links. (#64)
- [x] Add report cleanup actions for single-scan delete and full history delete. (#65)
- [x] Add repository ignore rules for generated local scan artifacts. (#66)
- [ ] Complete full section-by-section parity audit against all `design/dark/*/code.html` templates. (#67)
- [ ] Finalize visual parity refinements for cards, micro-panels, and telemetry blocks by section. (#68)

## Phase x: colavorative development <!-- phase:colaborative-development -->

- [/] Modify frontend (#69)
- [/] Add translations to spanish (#70)
- [/] Add design templates (#71)
- [/] init dev branch (#72)
- [ ] Improve the Scan tab.
- [ ] Improve the Overview tab.
- [ ] Improve the Performance tab.
- [ ] Improve the Security tab.
- [ ] Improve the Network tab.
- [ ] Improve the Sitemaps tab.
- [ ] Improve the Stack tab.
- [ ] Improve the Reports tab.
- [ ] Improve the SEO tab.
- [ ] Improve the Accessibility tab.
- [ ] Improve the Structure tab.
- [ ] Improve the Content tab.
- [ ] Improve the Links tab.
- [ ] Improve the Compliance tab.
- [ ] Improve the Social tab.
