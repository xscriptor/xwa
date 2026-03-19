"use client";

import { useState } from "react";
import type { SecurityData, SitemapCrawlResult, SitemapData } from "@/lib/types";

function NetworkAccordion({ row, defaultOpen = false }: { row: SitemapCrawlResult; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{row.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className="status-badge badge-2xx">{row.response_time_ms || 0}ms</span>
          <span className={`status-badge ${row.ok ? "badge-2xx" : "badge-4xx"}`}>{row.status || "ERR"}</span>
        </div>
      </div>
      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">Title:</span><span className="value">{row.title || "-"}</span></div>
          <div className="detail-item"><span className="label">Canonical:</span><span className="value">{row.canonical || "-"}</span></div>
          <div className="detail-item"><span className="label">Error:</span><span className="value">{row.error || "-"}</span></div>
        </div>
      )}
    </div>
  );
}

export default function NetworkTab({ sitemap, security }: { sitemap: SitemapData; security: SecurityData }) {
  const rows = sitemap.crawl_results || [];
  const okCount = rows.filter((r) => r.ok).length;
  const timeoutLike = rows.filter((r) => !r.ok).length;
  const maxMs = Math.max(1, ...rows.map((r) => r.response_time_ms || 0));

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>LIVE_STREAM_DATA</h2>
          <div className="telemetry-feed">
            {rows.slice(0, 10).map((row, i) => (
              <div key={`${row.url}-${i}`} className="telemetry-line">
                <span>[{String(i + 1).padStart(2, "0")}]</span>
                <span>{row.status || "ERR"}</span>
                <span className="url-cell">{row.url}</span>
                <span className={row.ok ? "text-success" : "text-danger"}>{row.response_time_ms || 0}ms</span>
              </div>
            ))}
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>DISTRIBUTION_STATS</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">TOTAL_REQ</span><span className="value text-accent">{rows.length}</span></div>
            <div className="mini-stat"><span className="label">OK</span><span className="value text-success">{okCount}</span></div>
            <div className="mini-stat"><span className="label">FAILED</span><span className="value text-danger">{timeoutLike}</span></div>
            <div className="mini-stat"><span className="label">AVG_MS</span><span className="value text-warning">{sitemap.avg_response_time_ms || 0}</span></div>
            <div className="mini-stat"><span className="label">SEC_HEADERS</span><span className="value text-success">{Object.keys(security.headers?.headers_present || {}).length}</span></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>NETWORK_PER_LINK [{rows.length}]</h2>
        <h3>WATERFALL_VIEW</h3>
        <div className="waterfall-table" style={{ marginBottom: "0.85rem" }}>
          {rows.slice(0, 10).map((row, idx) => {
            const ms = row.response_time_ms || 0;
            return (
              <div key={`${row.url}-${idx}`} className="waterfall-row">
                <span className="url-cell">{row.url}</span>
                <div className="waterfall-track"><div className="waterfall-fill" style={{ width: `${Math.max(6, Math.round((ms / maxMs) * 100))}%` }} /></div>
                <strong>{ms}ms</strong>
              </div>
            );
          })}
        </div>

        <div className="status-strip" style={{ marginBottom: "0.85rem" }}>
          <div className="status-cell"><span>transport ok</span><strong>{okCount}</strong></div>
          <div className="status-cell"><span>transport fail</span><strong>{timeoutLike}</strong></div>
          <div className="status-cell"><span>avg latency</span><strong>{sitemap.avg_response_time_ms || 0}ms</strong></div>
          <div className="status-cell"><span>security headers</span><strong>{Object.keys(security.headers?.headers_present || {}).length}</strong></div>
        </div>
        {rows.length > 0 ? rows.map((row, i) => <NetworkAccordion key={`${row.url}-${i}`} row={row} defaultOpen={i === 0} />) : <span className="text-muted">No network data</span>}
      </div>
    </div>
  );
}
