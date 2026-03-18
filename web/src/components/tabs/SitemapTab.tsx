"use client";

import { useState } from "react";

function getStatusBadgeClass(status: number): string {
  if (status >= 200 && status < 300) return "badge-2xx";
  if (status >= 300 && status < 400) return "badge-3xx";
  if (status >= 400 && status < 500) return "badge-4xx";
  if (status >= 500) return "badge-5xx";
  return "badge-err";
}

function URLTreeNode({ name, node, depth = 0 }: { name: string; node: any; depth?: number }) {
  const [open, setOpen] = useState(depth < 2);
  const childKeys = Object.keys(node).filter(k => k !== "_paths");
  const paths: string[] = node._paths || [];
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
          {childKeys.map(key => <URLTreeNode key={key} name={key} node={node[key]} depth={depth + 1} />)}
        </div>
      )}
    </div>
  );
}

interface SitemapTabProps {
  sitemap: any;
  onQuickScan: (url: string) => void;
}

export default function SitemapTab({ sitemap: sit, onQuickScan }: SitemapTabProps) {
  const crawlResults: any[] = sit.crawl_results || [];
  const statusStats: Record<string, number> = {};
  crawlResults.forEach((r: any) => {
    const code = r.status || 0;
    const key = code === 0 ? "ERR" : `${code}`;
    statusStats[key] = (statusStats[key] || 0) + 1;
  });

  const urlTree: Record<string, any> = {};
  (sit.all_urls || []).forEach((url: string) => {
    try {
      const u = new URL(url);
      const parts = u.pathname.split("/").filter(Boolean);
      let node = urlTree;
      parts.forEach(part => {
        if (!node[part]) node[part] = { _paths: [] };
        node = node[part];
      });
      node._paths = node._paths || [];
      node._paths.push(url);
    } catch { /* skip */ }
  });

  return (
    <div className="tab-sections">
      <div className="details-card glass-panel">
        <h2>// CRAWL_SUMMARY</h2>
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
          <h2>// URL_STATUS [{crawlResults.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "400px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>#</th><th>CODE</th><th>URL</th><th>ERR</th></tr></thead>
              <tbody>
                {crawlResults.map((r: any, idx: number) => (
                  <tr key={idx} className={r.ok ? "" : "row-error"}>
                    <td className="text-muted">{idx + 1}</td>
                    <td><span className={`status-badge ${getStatusBadgeClass(r.status)}`}>{r.status || "ERR"}</span></td>
                    <td className="url-cell">{r.url}</td>
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
          <h2>// BROKEN_LINKS [{sit.broken_links.length}]</h2>
          <div className="table-responsive">
            <table className="link-table">
              <thead><tr><th>CODE</th><th>URL</th><th>ERR</th></tr></thead>
              <tbody>
                {sit.broken_links.map((link: any, idx: number) => (
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
          <h2>// ALL_URLS [{sit.all_urls.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "400px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>#</th><th>URL</th><th>ACT</th></tr></thead>
              <tbody>
                {sit.all_urls.map((link: string, idx: number) => (
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
          <h2>// URL_TREE</h2>
          <div className="url-tree">
            {Object.keys(urlTree).filter(k => k !== "_paths").map(key => (
              <URLTreeNode key={key} name={key} node={urlTree[key]} depth={0} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
