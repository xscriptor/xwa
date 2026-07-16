# Shinobi Development Roadmap

This document tracks the strategic steps required to evolve Shinobi into a full-scale silent web scraping platform.
This file is formatted to be synced automatically with GitHub Issues using the `xgh` roadmap standard.

## Core Engine <!-- phase:core -->

- [x] BFS recursive crawler with configurable depth and max pages
- [x] HTML link extraction (a[href], link[href], img[src], script[src], etc.)
- [x] Asset download (CSS, JS, images, PDFs, archives, media, fonts)
- [x] Same-domain scoping
- [x] File type filtering
- [x] URL canonicalization (strip fragments, normalize trailing slashes)
- [x] Content deduplication by SHA-256 hash
- [x] Sitemap.xml parsing for URL discovery
- [x] robots.txt parsing and compliance

## Anti-Blocking System <!-- phase:anti-blocking -->

- [x] User-Agent rotation (15 real browser UAs)
- [x] Header randomisation (Accept, Accept-Language, Sec-CH-UA, Sec-Fetch-*)
- [x] Request delay with jitter
- [x] Exponential backoff on retry
- [x] HTTP 429/503 rate-limit detection and handling
- [x] HTTP/HTTPS/SOCKS5 proxy support
- [x] Proxy rotation on rate-limit

## Web Interface <!-- phase:web-ui -->

- [x] Scrape configuration form
- [x] Real-time SSE progress streaming
- [x] Jobs list with status badges
- [x] Downloaded files browser
- [x] Cancel running job
- [x] Dark instrument-panel UI theme
- [x] Angular 19 SPA (standalone components)
- [x] Built by build.rs automatically
- [x] Two-mode tabs: Fast Test / Deep Research
- [x] ZIP download for completed jobs
- [ ] Job scheduler (cron-like recurring scrapes)

## Deep Research (Python Sidecar) <!-- phase:deep -->

### Extraction
- [x] Structured data extraction (JSON-LD, microdata, Open Graph, RDFa) via extruct
- [x] Metadata extraction (title, description, keywords, canonical)
- [x] Headings extraction (document outline)
- [x] Link analysis (internal vs external, anchor text)
- [x] HTML table extraction to structured data
- [x] Image extraction (src, alt, dimensions)
- [x] Email and phone extraction (regex)
- [x] Custom CSS selector extraction
- [ ] Content extraction (Readability / Mozilla Readability)
- [ ] Article extraction (news article, blog post body)
- [ ] Form field detection and analysis
- [ ] Schema.org validation and normalization
- [ ] Price / product data extraction
- [ ] Review / rating extraction
- [ ] API endpoint discovery from web pages

### NLP & Analysis
- [x] Text extraction from HTML (strip markup)
- [x] Extractive summarization (TF + position scoring)
- [x] Named entity recognition (pattern-based)
- [x] Keyword extraction with TF, bigrams, density
- [x] Sentiment analysis (dictionary-based)
- [x] Readability scoring (Flesch Reading Ease)
- [x] Text statistics (word count, sentence count)
- [ ] Spacy integration for full NLP pipeline
- [ ] Language detection (lingua / langdetect)
- [ ] Translation (NLLB / deep-translator)
- [ ] Topic modeling (LDA / BERTopic)
- [ ] Text classification (zero-shot / LLM)
- [ ] Keyphrase extraction (RAKE / TextRank)
- [ ] Named entity linking (Wikipedia / Wikidata)
- [ ] Relation extraction between entities
- [ ] Content similarity / clustering across pages
- [ ] Change detection (diff between scrapes)

### Infrastructure
- [x] FastAPI server with /extract endpoint
- [x] Health check endpoint
- [x] Docker support with docker-compose
- [x] Auto venv creation and pip install
- [ ] Async endpoint for batch processing
- [ ] WebSocket for real-time extraction progress
- [ ] Extraction pipeline configuration (YAML)
- [ ] Plugin system for custom extractors
- [ ] Redis queue for job distribution
- [ ] Result caching with TTL
- [ ] Rate limiting for external APIs

## Advanced Features <!-- phase:advanced -->

- [x] JavaScript rendering via headless Chromium (chromiumoxide)
- [x] Screenshot capture of scraped pages
- [x] Email and phone number extraction from scraped content
- [x] Webhook notification on scrape completion
- [x] JSON export of job results
- [x] Batch URL extraction
- [x] Crawl + Extract mode (combine BFS with Python sidecar)
- [x] CSV export of deep research results
- [ ] WARC/ARC archive format output
- [ ] Request fingerprint randomisation (TLS client hello)
- [ ] Readability/article extraction
- [ ] Screenshot gallery in the UI
- [ ] Full-text search across scraped content
- [ ] Diff view between scrape versions
- [ ] Visual comparison of scraped pages

## Production Hardening <!-- phase:production -->

- [ ] Authentication middleware for API endpoints
- [ ] CORS origin restrictability
- [ ] Output size limits and disk usage monitoring
- [ ] Structured logging with span-based request tracing
- [ ] Pause/resume jobs
- [ ] Job queue with priority levels
- [ ] Email/Slack/Discord notifications
- [ ] Prometheus metrics endpoint
- [ ] Rate limiting across all jobs (global throttle)

## XWA Integration <!-- phase:xwa -->

- [ ] Angular shared component library with Samurai
- [ ] Cross-compatible database schema with Samurai
- [ ] Unified XWA docker-compose orchestration
- [ ] XWA API gateway integration
