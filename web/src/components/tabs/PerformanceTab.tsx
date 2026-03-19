"use client";

import { useState } from "react";
import type { SitemapCrawlResult, SitemapData } from "@/lib/types";

function PerformanceUrlRow({ row }: { row: SitemapCrawlResult }) {
  const [open, setOpen] = useState(false);
  const ms = row.response_time_ms || 0;
  const tone = ms > 1500 ? "badge-4xx" : ms > 700 ? "badge-3xx" : "badge-2xx";

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{row.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className={`status-badge ${tone}`}>{ms}ms</span>
          <span className="status-badge badge-2xx">WORDS:{row.word_count || 0}</span>
        </div>
      </div>
      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">Status:</span><span className="value">{row.status || "ERR"}</span></div>
          <div className="detail-item"><span className="label">Title:</span><span className="value">{row.title || "-"}</span></div>
          <div className="detail-item"><span className="label">H1:</span><span className="value">{row.h1 || "-"}</span></div>
          <div className="detail-item"><span className="label">Description:</span><span className="value">{row.description || "-"}</span></div>
        </div>
      )}
    </div>
  );
}

export default function PerformanceTab({ sitemap }: { sitemap: SitemapData }) {
  const rows = sitemap.crawl_results || [];
  const avg = sitemap.avg_response_time_ms || 0;
  const fastCount = rows.filter((r) => (r.response_time_ms || 0) < 700).length;
  const slowCount = rows.filter((r) => (r.response_time_ms || 0) > 1200).length;

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>PERFORMANCE_OVERVIEW</h2>
          <div className="crawl-stats-row">
            <div className="crawl-stat"><span className="crawl-stat-value text-accent">{rows.length}</span><span className="crawl-stat-label">links</span></div>
            <div className="crawl-stat"><span className={`crawl-stat-value ${avg > 1200 ? "text-danger" : avg > 700 ? "text-warning" : "text-success"}`}>{avg}</span><span className="crawl-stat-label">avg_ms</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-success">{fastCount}</span><span className="crawl-stat-label">fast</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-warning">{slowCount}</span><span className="crawl-stat-label">slow</span></div>
          </div>

          <h3>PERFORMANCE_TIMELINE</h3>
          <div className="telemetry-feed">
            {rows.slice(0, 7).map((row, idx) => (
              <div key={`${row.url}-${idx}`} className="telemetry-line">
                <span>[{String(idx + 1).padStart(2, "0")}]</span>
                <span className="url-cell">{row.url}</span>
                <span className={(row.response_time_ms || 0) > 1200 ? "text-warning" : "text-success"}>
                  {row.response_time_ms || 0}ms
                </span>
              </div>
            ))}
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>OPTIMIZATION_PROTOCOLS</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">FAST_RATIO</span><span className="value text-success">{rows.length ? Math.round((fastCount / rows.length) * 100) : 0}%</span></div>
            <div className="mini-stat"><span className="label">SLOW_RATIO</span><span className="value text-warning">{rows.length ? Math.round((slowCount / rows.length) * 100) : 0}%</span></div>
            <div className="mini-stat"><span className="label">AVG_STATUS</span><span className="value">{avg > 1200 ? "DEGRADED" : avg > 700 ? "WATCH" : "OPTIMAL"}</span></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>PERFORMANCE_PER_LINK [{rows.length}]</h2>
        <div className="terminal-action-panel" style={{ marginBottom: "0.8rem" }}>
          <div className="terminal-action-line"><span>cache_policy</span><strong>{avg > 1000 ? "enable edge caching" : "within target"}</strong></div>
          <div className="terminal-action-line"><span>image_strategy</span><strong>{slowCount > 0 ? "compress + modern formats" : "stable"}</strong></div>
          <div className="terminal-action-line"><span>critical_css</span><strong>{rows.length > 0 ? "inline first paint styles" : "n/a"}</strong></div>
        </div>
        {rows.length > 0 ? rows.map((row, i) => <PerformanceUrlRow key={`${row.url}-${i}`} row={row} />) : <span className="text-muted">No performance data</span>}
      </div>
    </div>
  );
}
