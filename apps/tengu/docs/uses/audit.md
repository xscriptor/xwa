# Audit Usage Guide

## Running an Audit

### From the Web UI

1. Navigate to `http://localhost:8080`
2. Enter the target URL
3. Select audit categories using the tab bar (ALL runs all four categories)
4. Choose audit mode:
   - **SINGLE URL**: audits one page
   - **FULL SITE**: crawls from the given URL and audits all discovered pages (max 50)
5. For FULL SITE mode, optionally enable INCLUDE SUBDOMAINS
6. Click START AUDIT
7. Watch results stream in real-time

### From the API

Open a WebSocket to:

```
ws://localhost:8080/api/audit/live?url=<URL>&mode=single&subdomains=false&checks=performance,seo,accessibility,best_practices
```

## Understanding Audit Categories

### Performance

Evaluates page loading efficiency:

- Page weight (total bytes, request count)
- Resource loading waterfall (blocking vs deferred)
- Image optimization (missing dimensions, wrong format)
- Font loading strategy (swap vs block)
- Cache policy (Cache-Control, ETag, Last-Modified)
- Compression negotiation (Brotli, gzip)
- Render-blocking resources
- Core Web Vitals candidates (LCP element, CLS detection)

### SEO

Evaluates search engine visibility:

- Title tag presence, length, uniqueness
- Meta description presence and quality
- Heading hierarchy validation (h1-h6 order, missing levels)
- Canonical URL detection
- Open Graph tags (og:title, og:type, og:image, og:url, og:description)
- Twitter Card tags
- JSON-LD structured data extraction
- Meta robots directives
- hreflang tag presence
- HTML lang attribute

### Accessibility

Evaluates WCAG 2.2 compliance:

- Image alt text presence
- Heading structure and document outline
- ARIA attribute usage (roles, labels, descriptions)
- Landmark element detection
- Form label associations (label-for, aria-label)
- Keyboard navigation (tabindex values)
- Link text quality (descriptive vs generic)
- Table structure (caption, scope, headers)
- Iframe title attributes
- Viewport zoom configuration
- Language attribute
- Color contrast (informational note)

### Best Practices

Evaluates security and standards compliance:

- HTTPS enforcement
- Security headers (HSTS, CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)
- Cookie attributes (Secure, HttpOnly, SameSite)
- Doctype declaration
- Deprecated HTML elements
- Mixed content detection
- Subresource Integrity (SRI)
- Console error detection (informational note)

## Interpreting Findings

Each finding has:

- **Severity**: Error (must fix), Warning (should fix), Info (consider), Pass (no issues found)
- **Check**: Short identifier for the audit check
- **Title**: One-line summary
- **Description**: Detailed explanation, impact, and actionable recommendation
- **Snippet**: The relevant HTML element (if applicable)
- **Line**: Line number in the pretty-printed HTML source

## Exporting Results

After an audit completes, export buttons appear below the findings list. Five formats are available:

| Format | Use Case |
|---|---|
| CSV | Spreadsheet analysis, filtering, reporting |
| JSON | Programmatic processing, CI pipelines |
| PDF | Client deliverables, printed reports |
| HTML | Self-contained styled report |
| Markdown | Documentation, issue trackers |
