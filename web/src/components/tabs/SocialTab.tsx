"use client";

import { useMemo, useState } from "react";
import type { SeoData } from "@/lib/types";

function SocialAccordion({ url, og, twitter, defaultOpen = false }: { url: string; og: Record<string, string>; twitter: Record<string, string>; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className="status-badge badge-2xx">OG:{Object.keys(og).length}</span>
          <span className="status-badge badge-2xx">TW:{Object.keys(twitter).length}</span>
        </div>
      </div>
      {open && (
        <div className="details-content">
          <div className="detail-item vertical">
            <span className="label">Open Graph</span>
            {Object.keys(og).length > 0 ? (
              <ul className="value-list">{Object.entries(og).map(([k, v]) => <li key={k}>{`og:${k} = ${v}`}</li>)}</ul>
            ) : <span className="text-muted">No OG tags</span>}
          </div>
          <div className="detail-item vertical mt-4">
            <span className="label">Twitter</span>
            {Object.keys(twitter).length > 0 ? (
              <ul className="value-list">{Object.entries(twitter).map(([k, v]) => <li key={k}>{`twitter:${k} = ${v}`}</li>)}</ul>
            ) : <span className="text-muted">No Twitter tags</span>}
          </div>
        </div>
      )}
    </div>
  );
}

export default function SocialTab({ seo }: { seo: SeoData }) {
  const rows = useMemo(() => {
    const base = [{
      url: "main",
      og: seo.social_meta?.og || {},
      twitter: seo.social_meta?.twitter || {},
    }];

    const per = (seo.per_url || []).map((p) => ({
      url: p.url,
      og: p.social_meta?.og || {},
      twitter: p.social_meta?.twitter || {},
    }));

    return [...base, ...per];
  }, [seo.social_meta, seo.per_url]);

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>SOCIAL_GRAPH_AUDIT</h2>
          <div className="telemetry-feed">
            {rows.slice(0, 10).map((row, idx) => (
              <div key={`${row.url}-${idx}`} className="telemetry-line">
                <span>[{String(idx + 1).padStart(2, "0")}]</span>
                <span>SG</span>
                <span className="url-cell">{row.url}</span>
                <span className={(Object.keys(row.og).length + Object.keys(row.twitter).length) > 0 ? "text-success" : "text-warning"}>
                  {(Object.keys(row.og).length + Object.keys(row.twitter).length) > 0 ? "OK" : "MISS"}
                </span>
              </div>
            ))}
          </div>

          <h3>SOCIAL_OVERVIEW</h3>
          <div className="crawl-stats-row">
            <div className="crawl-stat"><span className="crawl-stat-value text-accent">{rows.length}</span><span className="crawl-stat-label">links</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-success">{rows.reduce((s, r) => s + Object.keys(r.og).length, 0)}</span><span className="crawl-stat-label">og_tags</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-secondary">{rows.reduce((s, r) => s + Object.keys(r.twitter).length, 0)}</span><span className="crawl-stat-label">tw_tags</span></div>
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>REALTIME_SIGNALS</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">COVERAGE</span><span className="value text-success">{rows.filter((r) => Object.keys(r.og).length + Object.keys(r.twitter).length > 0).length}</span></div>
            <div className="mini-stat"><span className="label">NO_META</span><span className="value text-warning">{rows.filter((r) => Object.keys(r.og).length + Object.keys(r.twitter).length === 0).length}</span></div>
            <div className="mini-stat"><span className="label">STATUS</span><span className="value">ACTIVE</span></div>
          </div>

          <h3>ACTION_PANEL</h3>
          <div className="terminal-action-panel">
            <div className="terminal-action-line"><span>og:title</span><strong>{seo.social_meta?.og?.title ? "ok" : "add"}</strong></div>
            <div className="terminal-action-line"><span>og:image</span><strong>{seo.social_meta?.og?.image ? "ok" : "add"}</strong></div>
            <div className="terminal-action-line"><span>twitter:card</span><strong>{seo.social_meta?.twitter?.card ? "ok" : "add"}</strong></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>SOCIAL_PER_LINK [{rows.length}]</h2>
        {rows.map((row, i) => <SocialAccordion key={`${row.url}-${i}`} url={row.url} og={row.og} twitter={row.twitter} defaultOpen={i === 0} />)}
      </div>
    </div>
  );
}
