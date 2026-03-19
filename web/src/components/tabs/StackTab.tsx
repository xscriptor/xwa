"use client";

import { useMemo, useState } from "react";
import type { SecurityData, SitemapData } from "@/lib/types";

function StackAccordion({ url, tech, defaultOpen = false }: { url: string; tech: Array<{ name?: string; source?: string; detail?: string }>; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{url}</span>
        </div>
        <div className="a11y-page-badges"><span className="status-badge badge-2xx">TECH:{tech.length}</span></div>
      </div>
      {open && (
        <div className="details-content">
          {tech.length > 0 ? (
            <ul className="value-list">
              {tech.map((t, i) => <li key={`${t.name || "tech"}-${i}`}>{`${t.name || "Unknown"} (${t.source || "unknown"})`}</li>)}
            </ul>
          ) : <span className="text-muted">No stack fingerprints</span>}
        </div>
      )}
    </div>
  );
}

export default function StackTab({ security, sitemap }: { security: SecurityData; sitemap: SitemapData }) {
  const rows = useMemo(() => {
    const main = [{ url: "main", tech: security.technology?.technologies || [] }];
    const per = (security.per_url || []).map((p) => ({
      url: p.url,
      tech: p.technology?.technologies || [],
    }));
    return [...main, ...per];
  }, [security.technology, security.per_url]);

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>TECH_STACK_ANALYSIS</h2>
          <div className="telemetry-feed">
            {rows.slice(0, 10).map((row, idx) => (
              <div key={`${row.url}-${idx}`} className="telemetry-line">
                <span>[{String(idx + 1).padStart(2, "0")}]</span>
                <span>TS</span>
                <span className="url-cell">{row.url}</span>
                <span className={(row.tech?.length || 0) > 0 ? "text-success" : "text-warning"}>{row.tech?.length || 0} tech</span>
              </div>
            ))}
          </div>

          <h3>STACK_OVERVIEW</h3>
          <div className="crawl-stats-row">
            <div className="crawl-stat"><span className="crawl-stat-value text-accent">{rows.length}</span><span className="crawl-stat-label">links</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-success">{(security.technology?.count || 0)}</span><span className="crawl-stat-label">main_tech</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-warning">{sitemap.urls_found || 0}</span><span className="crawl-stat-label">urls_found</span></div>
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>INFRASTRUCTURE_RATING</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">STACK_ROWS</span><span className="value text-accent">{rows.length}</span></div>
            <div className="mini-stat"><span className="label">MAIN_TECH</span><span className="value text-success">{security.technology?.count || 0}</span></div>
            <div className="mini-stat"><span className="label">INDEXED_URLS</span><span className="value text-warning">{sitemap.urls_found || 0}</span></div>
            <div className="mini-stat"><span className="label">STATUS</span><span className="value">STABLE</span></div>
          </div>

          <h3>SERVER_CLUSTER</h3>
          <div className="cluster-grid">
            {rows.slice(0, 12).map((row, idx) => (
              <span key={`${row.url}-${idx}`} className="cluster-cell">srv_{idx + 1}</span>
            ))}
          </div>

          <h3>SECURITY_MATRIX</h3>
          <div className="matrix-panel">
            <div className="matrix-row"><span className="matrix-key">cookies</span><span className="matrix-value">{security.cookies?.total || 0} scanned</span><span className={`matrix-state ${(security.cookies?.issues?.length || 0) > 0 ? "tag-warn" : "tag-ok"}`}>{security.cookies?.issues?.length || 0}</span></div>
            <div className="matrix-row"><span className="matrix-key">headers</span><span className="matrix-value">missing security headers</span><span className={`matrix-state ${(security.headers?.missing_headers?.length || 0) > 0 ? "tag-warn" : "tag-ok"}`}>{security.headers?.missing_headers?.length || 0}</span></div>
            <div className="matrix-row"><span className="matrix-key">ssl</span><span className="matrix-value">certificate status</span><span className={`matrix-state ${security.ssl?.valid ? "tag-ok" : "tag-alert"}`}>{security.ssl?.valid ? "ok" : "risk"}</span></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>STACK_PER_LINK [{rows.length}]</h2>
        {rows.map((row, i) => <StackAccordion key={`${row.url}-${i}`} url={row.url} tech={row.tech} defaultOpen={i === 0} />)}
      </div>
    </div>
  );
}
