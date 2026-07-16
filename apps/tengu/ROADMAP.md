# Tengu Development Roadmap

This document tracks the strategic steps required to evolve Tengu into a full-scale web quality auditing platform.
This file is formatted to be synced automatically with GitHub Issues using the `xgh` roadmap standard.

## Core Engine <!-- phase:core -->

- [ ] HTTP client with configurable timeouts, redirect handling, and retry logic
- [ ] HTML parser with DOM tree extraction and serialization
- [ ] URL normalization and canonicalization
- [x] Batch URL analysis from sitemap, CSV, or single entry
- [ ] Crawl mode for full-site auditing with depth/pages configuration
- [ ] Request watermarking (request ID, timestamp, duration per analysis)

## Performance Analysis <!-- phase:performance -->

- [x] Page weight audit (total bytes, DOM node count, request count)
- [x] Largest Contentful Paint (LCP) — noted, requires browser
- [x] Cumulative Layout Shift (CLS) — noted, requires browser
- [x] Interaction to Next Paint (INP) — noted, requires browser
- [x] Resource loading waterfall (blocking vs deferred, render-blocking resources)
- [x] Image optimization audit (missing dimensions, wrong format, oversized)
- [x] Font loading audit (swap/block/fallback behavior, variable font usage)
- [x] Cache policy audit (Cache-Control, ETag, Last-Modified headers)
- [x] Compression audit (gzip/brotli negotiation, content encoding)
- [x] Third-party script performance impact scoring

## SEO Analysis <!-- phase:seo -->

- [x] Title tag presence, length, and uniqueness analysis
- [x] Meta description presence and quality scoring
- [x] Heading hierarchy validation (h1-h6 order, missing levels, multiple h1)
- [x] Canonical URL detection and cross-page consistency check
- [x] Open Graph tag audit (og:title, og:description, og:image, og:type)
- [x] Twitter Card tag audit (card, site, title, description, image)
- [x] JSON-LD structured data extraction and schema validation
- [x] Microdata and RDFa extraction
- [x] Sitemap.xml discovery, parsing, and URL coverage analysis
- [x] Robots.txt parsing, rule interpretation, and directives audit
- [x] Meta robots tag analysis (index/noindex, follow/nofollow)
- [x] hreflang tag audit for multilingual sites
- [x] Redirect chain analysis (301/302 chain length, circular redirect detection)
- [x] Broken link detection (404/410 discovery within crawled pages)

## Accessibility Analysis <!-- phase:a11y -->

- [x] Image alt text presence and quality analysis
- [x] Heading structure and document outline validation
- [x] ARIA attribute usage audit (roles, labels, descriptions)
- [x] Landmark element detection and structure analysis
- [x] Color contrast ratio calculation (WCAG AA/AAA compliance) — inline, <style> rules, inherited, themes, bg-image detection
- [x] Keyboard navigation audit (focusable elements, tab order, focus indicators)
- [x] Form label association validation (label-for, aria-label, aria-labelledby)
- [x] Video/audio transcript and caption detection
- [x] Language attribute validation (html lang attribute)
- [x] Viewport and zoom configuration audit
- [x] Link text quality analysis (descriptive vs generic text like "click here")
- [x] Table structure validation (headers, captions, scope attributes)
- [x] Iframe title attribute audit
- [x] Focus indicator visibility (outline:none detection in inline styles)

## Best Practices & Compliance <!-- phase:best-practices -->

- [x] HTTPS enforcement and certificate validity audit
- [x] Security headers audit (HSTS, CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)
- [x] Cookie audit (Secure, HttpOnly, SameSite attributes, third-party cookies)
- [ ] GDPR cookie consent banner detection and pattern analysis
- [x] Doctype and HTML validation (W3C standards compliance)
- [x] Deprecated HTML element and attribute detection
- [x] Mixed content detection (HTTPS page loading HTTP resources)
- [x] Subresource Integrity (SRI) audit for external scripts and stylesheets
- [x] Console error detection (noted: requires browser runtime)
- [x] Content Security Policy parsing and directive coverage analysis
- [x] Permissions-Policy / Feature-Policy audit (camera, microphone, geolocation usage)
- [x] GDPR cookie consent banner detection and pattern analysis

## Web Interface <!-- phase:web-ui -->

- [x] Angular standalone project scaffold with Nothing Design tokens
- [x] Audit configuration form (URL input, category toggles, crawl depth)
- [x] Real-time audit progress via WebSocket streaming
- [x] Score dashboard with per-category breakdown
- [x] Detailed findings list with severity, category, and check-level filtering
- [x] HTML source snippets for each finding with line highlighting
- [x] History page with past audit results
- [x] Sidebar navigation (TENGU / XWA - MODULE)
- [x] Dark/light theme toggle
- [x] Terminal panel showing real-time scan progress

## Reporting <!-- phase:reporting -->

- [x] PDF report generation with score summary and findings table
- [x] CSV export of all findings
- [x] JSON export for programmatic consumption
- [x] HTML export
- [x] Markdown export
- [x] Score history trend chart (SVG chart, no external deps)
- [x] Lighthouse-compatible JSON output format
- [x] Comparison report between two audits (before/after)

## Backend API <!-- phase:backend -->

- [ ] FastAPI project scaffold with health check endpoint
- [ ] REST endpoints for audit CRUD (POST /api/audit, GET /api/audits, GET /api/audits/{id}, DELETE /api/audits/{id})
- [ ] WebSocket endpoint for real-time audit streaming (/api/audit/live)
- [ ] PostgreSQL models for audits, findings, and pages
- [ ] Database export (raw + encrypted) following Samurai pattern
- [ ] Celery or asyncio task queue for long-running audits

## Integration <!-- phase:xwa -->

- [ ] Shared Nothing Design component library with Samurai
- [ ] Cross-compatible database schema conventions
- [ ] Unified XWA docker-compose orchestration
- [ ] XWA API gateway integration

## Production Hardening <!-- phase:production -->

- [ ] Authentication middleware for API endpoints
- [ ] Rate limiting for audit requests
- [x] Audit history retention policies and cleanup (TENGU_MAX_HISTORY env var)
- [x] Structured logging with request tracing (tracing crate with request watermarking)
- [ ] Prometheus metrics endpoint
- [x] Docker multi-stage build for minimal image size
