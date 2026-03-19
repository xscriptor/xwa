"use client";

import { useMemo, useState } from "react";
import type { SeoData, SitemapData } from "@/lib/types";

interface LinkItem {
  url: string;
  status: number;
  broken: boolean;
  canonical?: string;
}

function LinkAccordion({ item, defaultOpen = false }: { item: LinkItem; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{item.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className={`status-badge ${item.broken ? "badge-4xx" : "badge-2xx"}`}>{item.status || 200}</span>
        </div>
      </div>
      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">Broken:</span><span className={`value ${item.broken ? "text-danger" : "text-success"}`}>{item.broken ? "YES" : "NO"}</span></div>
          <div className="detail-item"><span className="label">Canonical:</span><span className="value">{item.canonical || "-"}</span></div>
        </div>
      )}
    </div>
  );
}

export default function LinksTab({ sitemap, seo }: { sitemap: SitemapData; seo: SeoData }) {
  const links = useMemo<LinkItem[]>(() => {
    const broken = new Set((sitemap.broken_links || []).map((b) => b.url));
    const canonicalMap = new Map((seo.per_url || []).map((r) => [r.url, r.canonical || ""]));
    return (sitemap.crawl_results || []).map((r) => ({
      url: r.url,
      status: r.status || 0,
      broken: broken.has(r.url),
      canonical: canonicalMap.get(r.url),
    }));
  }, [sitemap.broken_links, sitemap.crawl_results, seo.per_url]);

  const brokenCount = links.filter((l) => l.broken).length;
  const okCount = links.length - brokenCount;
  const anchorCloud = links
    .slice(0, 18)
    .map((link) => {
      try {
        const parsed = new URL(link.url);
        return parsed.pathname.split("/").filter(Boolean).pop() || parsed.hostname;
      } catch {
        return link.url;
      }
    });

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>HYPERLINK_ARCHITECTURE</h2>
          <div className="telemetry-feed">
            {links.slice(0, 10).map((item, idx) => (
              <div key={`${item.url}-${idx}`} className="telemetry-line">
                <span>[{String(idx + 1).padStart(2, "0")}]</span>
                <span>{item.status || 0}</span>
                <span className="url-cell">{item.url}</span>
                <span className={item.broken ? "text-danger" : "text-success"}>{item.broken ? "BROKEN" : "OK"}</span>
              </div>
            ))}
          </div>

          <h3>LINKS_OVERVIEW</h3>
          <div className="crawl-stats-row">
            <div className="crawl-stat"><span className="crawl-stat-value text-accent">{links.length}</span><span className="crawl-stat-label">links</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-success">{okCount}</span><span className="crawl-stat-label">healthy</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-danger">{brokenCount}</span><span className="crawl-stat-label">broken</span></div>
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>DISTRIBUTION_STATS</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">TOTAL</span><span className="value text-accent">{links.length}</span></div>
            <div className="mini-stat"><span className="label">HEALTHY</span><span className="value text-success">{okCount}</span></div>
            <div className="mini-stat"><span className="label">BROKEN</span><span className="value text-danger">{brokenCount}</span></div>
            <div className="mini-stat"><span className="label">BROKEN_%</span><span className="value text-warning">{links.length ? Math.round((brokenCount / links.length) * 100) : 0}%</span></div>
          </div>

          <h3>ANCHOR_CLOUD</h3>
          <div className="anchor-cloud">
            {anchorCloud.length > 0 ? anchorCloud.map((anchor, idx) => (
              <span key={`${anchor}-${idx}`} className="anchor-pill">{anchor}</span>
            )) : (
              <span className="anchor-pill">no_anchor_data</span>
            )}
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>LINKS_PER_LINK [{links.length}]</h2>
        {links.length > 0 ? links.map((link, i) => <LinkAccordion key={`${link.url}-${i}`} item={link} defaultOpen={i === 0} />) : <span className="text-muted">No links data</span>}
      </div>
    </div>
  );
}
