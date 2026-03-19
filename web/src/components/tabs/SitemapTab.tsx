"use client";

import { useState } from "react";
import type { SitemapData, SitemapCrawlResult } from "@/lib/types";

function getStatusBadgeClass(status: number): string {
  if (status >= 200 && status < 300) return "badge-2xx";
  if (status >= 300 && status < 400) return "badge-3xx";
  if (status >= 400 && status < 500) return "badge-4xx";
  if (status >= 500) return "badge-5xx";
  return "badge-err";
}

function URLTreeNode({ name, node, depth = 0 }: { name: string; node: Record<string, unknown>; depth?: number }) {
  const [open, setOpen] = useState(depth < 2);
  const childKeys = Object.keys(node).filter((k) => k !== "_paths");
  const paths = (node._paths as string[] | undefined) || [];
  const hasChildren = childKeys.length > 0;
  if (name === "_paths") return null;

  return (
    <div className="tree-node" style={{ paddingLeft: `${depth * 1.25}rem` }}>
      <div className={`tree-label ${hasChildren ? "expandable" : ""}`} onClick={() => hasChildren && setOpen(!open)}>
        {hasChildren && <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>}
        <span className="tree-name">/{name}</span>
        {paths.length > 0 && <span className="tree-count">{paths.length}</span>}
      </div>
      {open && hasChildren && (
        <div className="tree-children">
          {childKeys.map((key) => <URLTreeNode key={key} name={key} node={(node[key] as Record<string, unknown>) || {}} depth={depth + 1} />)}
        </div>
      )}
    </div>
  );
}

interface SitemapTabProps {
  sitemap: SitemapData;
  onQuickScan: (url: string) => void;
}

export default function SitemapTab({ sitemap: sit, onQuickScan }: SitemapTabProps) {
  const crawlResults: SitemapCrawlResult[] = sit.crawl_results || [];
  const statusStats: Record<string, number> = {};
  crawlResults.forEach((r) => {
    const code = r.status || 0;
    const key = code === 0 ? "ERR" : `${code}`;
    statusStats[key] = (statusStats[key] || 0) + 1;
  });

  const urlTree: Record<string, unknown> = {};
  (sit.all_urls || []).forEach((url: string) => {
    try {
      const u = new URL(url);
      const parts = u.pathname.split("/").filter(Boolean);
      let node: Record<string, unknown> = urlTree;
      parts.forEach(part => {
        if (!node[part]) node[part] = { _paths: [] };
        node = node[part] as Record<string, unknown>;
      });
      const existing = (node._paths as string[] | undefined) || [];
      node._paths = [...existing, url];
    } catch { /* skip */ }
  });

  const feedRows = crawlResults.slice(0, 14).map((row, index) => {
    const state = row.ok ? "ok" : "alert";
    return {
      ts: `01:${String(index + 10).padStart(2, "0")}`,
      code: row.status || 0,
      url: row.url,
      time: row.response_time_ms ?? "-",
      state,
    };
  });

  const uniqueHosts = new Set(
    (sit.all_urls || []).map((url) => {
      try {
        return new URL(url).host;
      } catch {
        return "invalid";
      }
    })
  ).size;

  return (
    <div className="tab-sections">
      <div className="glass-panel protocol-hero-grid">
        <div className="protocol-hero-main">
          <h2>SITEMAP_CONTROL_CENTER</h2>
          <p className="mono-subline">Live crawl and sitemap index diagnostics for detected routes.</p>

          <h3>LIVE_ANALYSIS_FEED</h3>
          <div className="telemetry-feed">
            {feedRows.length > 0 ? (
              feedRows.map((item, idx) => (
                <div key={`${item.url}-${idx}`} className="telemetry-line">
                  <span>{item.ts}</span>
                  <strong className={`tag-${item.state}`}>{item.code || "ERR"}</strong>
                  <span>{item.url}</span>
                  <strong>{item.time}ms</strong>
                </div>
              ))
            ) : (
              <div className="telemetry-line">
                <span>--:--</span>
                <strong className="tag-warn">idle</strong>
                <span>no crawl results available in this report</span>
                <strong>n/a</strong>
              </div>
            )}
          </div>
        </div>

        <div className="protocol-hero-rail">
          <h2>PROTOCOL_WIDGETS</h2>
          <p className="mono-subline">Route and status matrix compressed for operator review.</p>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">urls found</span><span className="value">{sit.urls_found || 0}</span></div>
            <div className="mini-stat"><span className="label">scanned</span><span className="value">{sit.scanned_count || 0}</span></div>
            <div className="mini-stat"><span className="label">broken</span><span className="value">{sit.broken_links?.length || 0}</span></div>
            <div className="mini-stat"><span className="label">avg response</span><span className="value">{sit.avg_response_time_ms ?? "-"}</span></div>
            <div className="mini-stat"><span className="label">hosts</span><span className="value">{uniqueHosts}</span></div>
            <div className="mini-stat"><span className="label">status classes</span><span className="value">{Object.keys(statusStats).length}</span></div>
          </div>

          <h3>INDEX_HIERARCHY</h3>
          <div className="matrix-panel">
            {(sit.tree_root_children || []).slice(0, 8).map((node, idx) => (
              <div key={`${node}-${idx}`} className="matrix-row">
                <span className="matrix-key">root_{idx + 1}</span>
                <span className="matrix-value">{node}</span>
                <span className="matrix-state tag-ok">linked</span>
              </div>
            ))}
            {(sit.tree_root_children || []).length === 0 && (
              <div className="matrix-row">
                <span className="matrix-key">root</span>
                <span className="matrix-value">tree root unavailable for this scan</span>
                <span className="matrix-state tag-warn">pending</span>
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>CRAWL_SUMMARY</h2>
        <div className="crawl-stats-row">
          <div className="crawl-stat">
            <span className="crawl-stat-value text-accent">{sit.urls_found}</span>
            <span className="crawl-stat-label">total</span>
          </div>
          <div className="crawl-stat">
            <span className="crawl-stat-value text-success">{sit.scanned_count || 0}</span>
            <span className="crawl-stat-label">scanned</span>
          </div>
          <div className="crawl-stat">
            <span className={`crawl-stat-value ${(sit.broken_links?.length || 0) > 0 ? "text-danger" : "text-success"}`}>
              {sit.broken_links?.length || 0}
            </span>
            <span className="crawl-stat-label">broken</span>
          </div>
          {sit.avg_response_time_ms != null && (
            <div className="crawl-stat">
              <span className="crawl-stat-value text-warning">{sit.avg_response_time_ms}</span>
              <span className="crawl-stat-label">avg_ms</span>
            </div>
          )}
        </div>
        {Object.keys(statusStats).length > 0 && (
          <div className="detail-item vertical mt-4">
            <span className="label">status_distribution</span>
            <div className="status-badges-row">
              {Object.entries(statusStats).sort().map(([code, count]) => (
                <span key={code} className={`status-badge ${getStatusBadgeClass(parseInt(code) || 0)}`}>
                  {code}:{count}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>

      {crawlResults.length > 0 && (
        <div className="details-card glass-panel">
          <h2>URL_STATUS [{crawlResults.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "500px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>#</th><th>CODE</th><th>URL</th><th>TITLE</th><th>WORDS</th><th>MS</th><th>ERR</th></tr></thead>
              <tbody>
                {crawlResults.map((r, idx) => (
                  <tr key={idx} className={r.ok ? "" : "row-error"}>
                    <td className="text-muted">{idx + 1}</td>
                    <td><span className={`status-badge ${getStatusBadgeClass(r.status || 0)}`}>{r.status || "ERR"}</span></td>
                    <td className="url-cell">{r.url}</td>
                    <td className="text-muted" style={{ maxWidth: '200px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {r.title || "-"}
                    </td>
                    <td className="text-muted">{r.word_count || "-"}</td>
                    <td className="text-muted">{r.response_time_ms != null ? r.response_time_ms : "-"}</td>
                    <td className="text-muted">{r.error || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {(sit.broken_links?.length || 0) > 0 && (
        <div className="details-card glass-panel">
          <h2>BROKEN_LINKS [{(sit.broken_links || []).length}]</h2>
          <div className="table-responsive">
            <table className="link-table">
              <thead><tr><th>CODE</th><th>URL</th><th>ERR</th></tr></thead>
              <tbody>
                {(sit.broken_links || []).map((link, idx) => (
                  <tr key={idx}>
                    <td className="text-danger font-medium">{link.status || "FAIL"}</td>
                    <td className="url-cell">{link.url}</td>
                    <td className="text-muted">{link.error || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {(sit.all_urls?.length || 0) > 0 && (
        <div className="details-card glass-panel">
          <h2>ALL_URLS [{(sit.all_urls || []).length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "400px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>#</th><th>URL</th><th>ACT</th></tr></thead>
              <tbody>
                {(sit.all_urls || []).map((link: string, idx: number) => (
                  <tr key={idx}>
                    <td className="text-muted">{idx + 1}</td>
                    <td className="url-cell">{link}</td>
                    <td><button className="scan-mini-btn" onClick={() => onQuickScan(link)}>scan</button></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {Object.keys(urlTree).length > 0 && (
        <div className="details-card glass-panel">
          <h2>URL_TREE</h2>
          <div className="url-tree">
            {Object.keys(urlTree).filter(k => k !== "_paths").map(key => (
              <URLTreeNode key={key} name={key} node={(urlTree[key] as Record<string, unknown>) || {}} depth={0} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
