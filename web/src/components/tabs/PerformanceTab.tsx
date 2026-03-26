"use client";

import { useState } from "react";
import type { PerformanceData, PerformancePage, SitemapData, SitemapCrawlResult } from "@/lib/types";

/* ==================== HELPERS ==================== */

function riskTag(risk?: string) {
  if (!risk) return <span className="tag-ok">N/A</span>;
  const r = risk.toLowerCase();
  if (r === "good" || r === "low") return <span className="tag-ok">{risk.toUpperCase()}</span>;
  if (r === "needs_improvement" || r === "medium") return <span className="tag-warn">{risk.toUpperCase()}</span>;
  return <span className="tag-alert">{risk.toUpperCase()}</span>;
}

function formatKB(bytes?: number) {
  if (!bytes) return "0 KB";
  return `${(bytes / 1024).toFixed(1)} KB`;
}

/* ==================== PER-LINK ROW ==================== */

function ResourceList({ label, items }: { label: string; items?: string[] }) {
  if (!items || items.length === 0) return null;
  return (
    <div style={{ marginTop: "0.5rem" }}>
      <h4 style={{ fontFamily: "var(--font-mono)", fontSize: "0.66rem", color: "var(--secondary)", textTransform: "uppercase", letterSpacing: "0.06em", marginBottom: "0.3rem" }}>
        {label} [{items.length}]
      </h4>
      <div className="matrix-panel">
        {items.map((src, idx) => {
          const ext = src.includes(".") ? src.split(".").pop()?.split("?")[0]?.toUpperCase() || "" : "";
          return (
            <div key={idx} className="matrix-row">
              <span className="matrix-key">{ext || "---"}</span>
              <span className="matrix-value" title={src}>{src}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function PerformanceUrlRow({ perf, crawl }: { perf: PerformancePage; crawl?: SitemapCrawlResult }) {
  const [open, setOpen] = useState(false);
  const ttfb = perf.ttfb?.ttfb_estimate_ms || 0;
  const lcp = perf.cwv_estimates?.lcp?.estimate_ms || 0;
  const requests = perf.resources?.total_external_requests || 0;
  const unopt = perf.unoptimized_images?.unoptimized_count || 0;
  const totalImg = perf.resources?.images?.count || 0;
  const responseMs = crawl?.response_time_ms || perf.ttfb?.total_response_ms || 0;
  const tone = responseMs > 1500 ? "badge-4xx" : responseMs > 700 ? "badge-3xx" : "badge-2xx";

  const res = perf.resources;

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{perf.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className={`status-badge ${tone}`}>{responseMs}ms</span>
          <span className="status-badge badge-2xx">IMG:{totalImg}</span>
          {riskTag(perf.cwv_estimates?.lcp?.rating)}
        </div>
      </div>
      {open && (
        <div className="details-content">
          {/* Summary stats */}
          <div className="status-strip">
            <div className="status-cell"><span>TTFB</span><strong>{ttfb}ms</strong></div>
            <div className="status-cell"><span>LCP_EST</span><strong>{lcp}ms</strong></div>
            <div className="status-cell"><span>CLS_RISK</span><strong>{(perf.cwv_estimates?.cls?.risk || "n/a").toUpperCase()}</strong></div>
            <div className="status-cell"><span>INP_RISK</span><strong>{(perf.cwv_estimates?.inp?.risk || "n/a").toUpperCase()}</strong></div>
          </div>
          <div className="status-strip" style={{ marginTop: "0.4rem" }}>
            <div className="status-cell"><span>REQUESTS</span><strong>{requests}</strong></div>
            <div className="status-cell"><span>PAGE_SIZE</span><strong>{formatKB(res?.html_size_bytes)}</strong></div>
            <div className="status-cell"><span>IMAGES</span><strong>{totalImg}</strong></div>
            <div className="status-cell"><span>UNOPT_IMG</span><strong>{unopt}</strong></div>
            <div className="status-cell"><span>JS</span><strong>{(res?.js?.external_count || 0) + (res?.js?.inline_count || 0)}</strong></div>
            <div className="status-cell"><span>CSS</span><strong>{(res?.css?.external_count || 0) + (res?.css?.inline_count || 0)}</strong></div>
            <div className="status-cell"><span>FONTS</span><strong>{res?.fonts?.count || 0}</strong></div>
            <div className="status-cell"><span>IFRAMES</span><strong>{res?.iframes?.count || 0}</strong></div>
          </div>

          {/* Unoptimized images list */}
          {(perf.unoptimized_images?.unoptimized?.length || 0) > 0 && (
            <div style={{ marginTop: "0.5rem" }}>
              <h4 style={{ fontFamily: "var(--font-mono)", fontSize: "0.66rem", color: "var(--danger)", textTransform: "uppercase", letterSpacing: "0.06em", marginBottom: "0.3rem" }}>
                UNOPTIMIZED_IMAGES [{perf.unoptimized_images!.unoptimized!.length}]
              </h4>
              <div className="matrix-panel">
                {perf.unoptimized_images!.unoptimized!.map((img, idx) => (
                  <div key={idx} className="matrix-row">
                    <span className="matrix-key">{img.format}</span>
                    <span className="matrix-value" title={img.src}>{img.src}</span>
                    <span className="matrix-state tag-warn">CONVERT</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Full resource lists */}
          <ResourceList label="IMAGES" items={res?.images?.sources} />
          <ResourceList label="JAVASCRIPT" items={res?.js?.sources} />
          <ResourceList label="CSS" items={res?.css?.sources} />
          <ResourceList label="FONTS" items={res?.fonts?.sources} />
          <ResourceList label="IFRAMES" items={res?.iframes?.sources} />
        </div>
      )}
    </div>
  );
}

/* ==================== MAIN TAB ==================== */

export default function PerformanceTab({
  performance,
  sitemap,
}: {
  performance: PerformanceData;
  sitemap: SitemapData;
}) {
  const main = performance?.main_page;
  const perUrl = performance?.per_url || [];
  const crawlResults = sitemap.crawl_results || [];

  // Sitemap-level stats (kept for backward compat)
  const rows = crawlResults;
  const avg = sitemap.avg_response_time_ms || 0;
  const fastCount = rows.filter((r) => (r.response_time_ms || 0) < 700).length;
  const slowCount = rows.filter((r) => (r.response_time_ms || 0) > 1200).length;

  // Main page data
  const ttfb = main?.ttfb;
  const cwv = main?.cwv_estimates;
  const res = main?.resources;
  const unopt = main?.unoptimized_images;

  return (
    <div className="tab-sections">

      {/* ===== CWV HERO ===== */}
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>CORE_WEB_VITALS</h2>
          <p className="mono-subline">Estimated from static HTML analysis &middot; Without chrome browser</p>

          <div className="radial-micro-grid">
            {/* LCP */}
            <div className="radial-micro-card">
              <h4>LCP</h4>
              <div className="radial-ring" style={{
                "--pct": Math.min(100, Math.round(((cwv?.lcp?.estimate_ms || 0) / 5000) * 100)),
                "--ring-color": (cwv?.lcp?.rating === "good") ? "var(--success)" : (cwv?.lcp?.rating === "needs_improvement") ? "var(--warning)" : "var(--danger)",
              } as React.CSSProperties}>
                <span className="radial-ring-label">{cwv?.lcp?.estimate_ms || 0}ms</span>
              </div>
              {riskTag(cwv?.lcp?.rating)}
            </div>

            {/* CLS */}
            <div className="radial-micro-card">
              <h4>CLS_RISK</h4>
              <div className="radial-ring" style={{
                "--pct": cwv?.cls?.risk === "low" ? 15 : cwv?.cls?.risk === "medium" ? 50 : 85,
                "--ring-color": cwv?.cls?.risk === "low" ? "var(--success)" : cwv?.cls?.risk === "medium" ? "var(--warning)" : "var(--danger)",
              } as React.CSSProperties}>
                <span className="radial-ring-label">{cwv?.cls?.unstable_elements || 0}</span>
              </div>
              {riskTag(cwv?.cls?.risk)}
            </div>

            {/* INP */}
            <div className="radial-micro-card">
              <h4>INP_RISK</h4>
              <div className="radial-ring" style={{
                "--pct": cwv?.inp?.risk === "low" ? 15 : cwv?.inp?.risk === "medium" ? 50 : 85,
                "--ring-color": cwv?.inp?.risk === "low" ? "var(--success)" : cwv?.inp?.risk === "medium" ? "var(--warning)" : "var(--danger)",
              } as React.CSSProperties}>
                <span className="radial-ring-label">{cwv?.inp?.sync_scripts || 0}</span>
              </div>
              {riskTag(cwv?.inp?.risk)}
            </div>
          </div>

          {/* CWV hints */}
          <div className="terminal-action-panel" style={{ marginTop: "0.8rem" }}>
            <div className="terminal-action-line"><span>lcp_hint</span><strong>{cwv?.lcp?.hint || "n/a"}</strong></div>
            <div className="terminal-action-line"><span>cls_hint</span><strong>{cwv?.cls?.hint || "n/a"}</strong></div>
            <div className="terminal-action-line"><span>inp_hint</span><strong>{cwv?.inp?.hint || "n/a"}</strong></div>
          </div>
        </div>

        {/* Right rail: Server / TTFB */}
        <div className="details-card glass-panel protocol-hero-rail">
          <h3>SERVER_RESPONSE</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat">
              <span className="label">TTFB</span>
              <span className={`value ${ttfb?.rating === "good" ? "text-success" : ttfb?.rating === "needs_improvement" ? "text-warning" : "text-danger"}`}>{ttfb?.ttfb_estimate_ms || 0}ms</span>
            </div>
            <div className="mini-stat">
              <span className="label">TOTAL_RESPONSE</span>
              <span className="value">{ttfb?.total_response_ms || 0}ms</span>
            </div>
            <div className="mini-stat">
              <span className="label">TRANSFER_TIME</span>
              <span className="value">{ttfb?.transfer_estimate_ms || 0}ms</span>
            </div>
            <div className="mini-stat">
              <span className="label">HTML_SIZE</span>
              <span className="value">{formatKB(ttfb?.html_size_bytes)}</span>
            </div>
            <div className="mini-stat">
              <span className="label">TTFB_RATING</span>
              <span className="value">{(ttfb?.rating || "n/a").toUpperCase()}</span>
            </div>
          </div>

          <h3 style={{ marginTop: "0.7rem" }}>CRAWL_STATS</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat">
              <span className="label">LINKS_SCANNED</span>
              <span className="value text-accent">{rows.length}</span>
            </div>
            <div className="mini-stat">
              <span className="label">AVG_RESPONSE</span>
              <span className={`value ${avg > 1200 ? "text-danger" : avg > 700 ? "text-warning" : "text-success"}`}>{avg}ms</span>
            </div>
            <div className="mini-stat">
              <span className="label">FAST</span>
              <span className="value text-success">{fastCount}</span>
            </div>
            <div className="mini-stat">
              <span className="label">SLOW</span>
              <span className="value text-warning">{slowCount}</span>
            </div>
          </div>
        </div>
      </div>

      {/* ===== RESOURCE BREAKDOWN ===== */}
      <div className="details-card glass-panel">
        <h2>RESOURCE_BREAKDOWN</h2>
        <p className="mono-subline">External requests and payload analysis for the main page</p>

        <div className="status-strip">
          <div className="status-cell"><span>TOTAL_REQUESTS</span><strong>{res?.total_external_requests || 0}</strong></div>
          <div className="status-cell"><span>HTML_SIZE</span><strong>{formatKB(res?.html_size_bytes)}</strong></div>
          <div className="status-cell"><span>JS_FILES</span><strong>{res?.js?.external_count || 0}</strong></div>
          <div className="status-cell"><span>CSS_FILES</span><strong>{res?.css?.external_count || 0}</strong></div>
          <div className="status-cell"><span>IMAGES</span><strong>{res?.images?.count || 0}</strong></div>
          <div className="status-cell"><span>FONTS</span><strong>{res?.fonts?.count || 0}</strong></div>
        </div>

        {/* Waterfall-style bar for each type */}
        <div className="waterfall-table" style={{ marginTop: "0.7rem" }}>
          {[
            { label: "JavaScript", count: (res?.js?.external_count || 0) + (res?.js?.inline_count || 0), color: "var(--warning)" },
            { label: "CSS", count: (res?.css?.external_count || 0) + (res?.css?.inline_count || 0), color: "var(--secondary)" },
            { label: "Images", count: res?.images?.count || 0, color: "var(--primary)" },
            { label: "Fonts", count: res?.fonts?.count || 0, color: "var(--tertiary)" },
            { label: "Iframes", count: res?.iframes?.count || 0, color: "var(--danger)" },
          ].map((item) => {
            const max = Math.max(
              (res?.js?.external_count || 0) + (res?.js?.inline_count || 0),
              res?.images?.count || 0,
              (res?.css?.external_count || 0) + (res?.css?.inline_count || 0),
              1
            );
            const pct = Math.round((item.count / max) * 100);
            return (
              <div key={item.label} className="waterfall-row">
                <span>{item.label}</span>
                <div className="waterfall-track">
                  <div className="waterfall-fill" style={{ width: `${pct}%`, background: item.color }} />
                </div>
                <span>{item.count}</span>
              </div>
            );
          })}
        </div>

        {/* Inline vs external breakdown */}
        <div className="terminal-action-panel" style={{ marginTop: "0.6rem" }}>
          <div className="terminal-action-line"><span>inline_scripts</span><strong>{res?.js?.inline_count || 0}</strong></div>
          <div className="terminal-action-line"><span>external_scripts</span><strong>{res?.js?.external_count || 0}</strong></div>
          <div className="terminal-action-line"><span>inline_styles</span><strong>{res?.css?.inline_count || 0}</strong></div>
          <div className="terminal-action-line"><span>external_styles</span><strong>{res?.css?.external_count || 0}</strong></div>
        </div>
      </div>

      {/* ===== UNOPTIMIZED IMAGES ===== */}
      <div className="details-card glass-panel">
        <h2>UNOPTIMIZED_IMAGES [{unopt?.unoptimized_count || 0}/{unopt?.total_images || 0}]</h2>
        <p className="mono-subline">{unopt?.recommendation || "No image data"}</p>

        {(unopt?.unoptimized?.length || 0) > 0 ? (
          <div className="matrix-panel">
            {unopt!.unoptimized!.map((img, idx) => (
              <div key={idx} className="matrix-row">
                <span className="matrix-key">{img.format}</span>
                <span className="matrix-value">{img.src}</span>
                <span className="matrix-state tag-warn">CONVERT</span>
              </div>
            ))}
          </div>
        ) : (
          <div className="terminal-action-panel">
            <div className="terminal-action-line"><span>status</span><strong>All images use modern formats</strong></div>
          </div>
        )}
      </div>

      {/* ===== PERFORMANCE TIMELINE (from crawl) ===== */}
      <div className="details-card glass-panel">
        <h2>RESPONSE_TIMELINE</h2>
        <div className="telemetry-feed">
          {rows.slice(0, 10).map((row, idx) => (
            <div key={`${row.url}-${idx}`} className="telemetry-line">
              <span>[{String(idx + 1).padStart(2, "0")}]</span>
              <span className={(row.response_time_ms || 0) > 1200 ? "text-warning" : "text-success"}>
                {row.response_time_ms || 0}ms
              </span>
              <span className="url-cell">{row.url}</span>
              <span>{row.status || "ERR"}</span>
            </div>
          ))}
        </div>
      </div>

      {/* ===== PER-LINK ANALYSIS ===== */}
      <div className="details-card glass-panel">
        <h2>PERFORMANCE_PER_LINK [{perUrl.length}]</h2>
        <div className="terminal-action-panel" style={{ marginBottom: "0.8rem" }}>
          <div className="terminal-action-line"><span>cache_policy</span><strong>{avg > 1000 ? "enable edge caching" : "within target"}</strong></div>
          <div className="terminal-action-line"><span>image_strategy</span><strong>{slowCount > 0 ? "compress + modern formats" : "stable"}</strong></div>
          <div className="terminal-action-line"><span>critical_css</span><strong>{rows.length > 0 ? "inline first paint styles" : "n/a"}</strong></div>
        </div>
        {perUrl.length > 0 ? (
          perUrl.map((p, i) => {
            const matchedCrawl = crawlResults.find((cr) => cr.url === p.url);
            return <PerformanceUrlRow key={`${p.url}-${i}`} perf={p} crawl={matchedCrawl} />;
          })
        ) : (
          // Fallback to crawl-only data if no performance per-url
          rows.length > 0 ? (
            rows.map((row, i) => (
              <div key={`${row.url}-${i}`} className="details-card glass-panel">
                <div className="a11y-page-header">
                  <div className="a11y-page-title">
                    <span className="url-cell" style={{ fontSize: "0.85rem" }}>{row.url}</span>
                  </div>
                  <div className="a11y-page-badges">
                    <span className={`status-badge ${(row.response_time_ms || 0) > 1500 ? "badge-4xx" : (row.response_time_ms || 0) > 700 ? "badge-3xx" : "badge-2xx"}`}>
                      {row.response_time_ms || 0}ms
                    </span>
                  </div>
                </div>
              </div>
            ))
          ) : (
            <span className="text-muted">No performance data</span>
          )
        )}
      </div>
    </div>
  );
}
