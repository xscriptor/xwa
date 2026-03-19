"use client";

import { useMemo, useState } from "react";
import type { SeoData, SitemapData, StructureData } from "@/lib/types";

interface ContentRow {
  url: string;
  title?: string;
  h1?: string;
  description?: string;
  words?: number;
}

function ContentAccordion({ row, defaultOpen = false }: { row: ContentRow; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{row.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className="status-badge badge-2xx">WORDS:{row.words || 0}</span>
        </div>
      </div>
      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">Title:</span><span className="value">{row.title || "-"}</span></div>
          <div className="detail-item"><span className="label">H1:</span><span className="value">{row.h1 || "-"}</span></div>
          <div className="detail-item"><span className="label">Description:</span><span className="value">{row.description || "-"}</span></div>
        </div>
      )}
    </div>
  );
}

export default function ContentTab({ seo, sitemap, structure }: { seo: SeoData; sitemap: SitemapData; structure: StructureData }) {
  const rows = useMemo<ContentRow[]>(() => {
    const fromCrawl = (sitemap.crawl_results || []).map((r) => ({
      url: r.url,
      title: r.title,
      h1: r.h1,
      description: r.description,
      words: r.word_count,
    }));

    const fromSeo = (seo.per_url || []).map((r) => ({
      url: r.url,
      title: r.standard_meta?.title || undefined,
      h1: r.headings?.details?.find((h) => h.tag === "H1")?.text,
      description: r.standard_meta?.description || undefined,
      words: r.text_ratio?.word_count,
    }));

    const map = new Map<string, ContentRow>();
    [...fromCrawl, ...fromSeo].forEach((row) => {
      const current = map.get(row.url) || { url: row.url };
      map.set(row.url, { ...current, ...row });
    });

    return Array.from(map.values());
  }, [seo.per_url, sitemap.crawl_results]);

  const totalWords = rows.reduce((sum, r) => sum + (r.words || 0), 0) + (structure.main_page?.clean_text?.total_words || 0);
  const maxWords = Math.max(1, ...rows.map((r) => r.words || 0));

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>CONTENT_HEATMAP</h2>
          <p className="mono-subline">Word-density heat by discovered URL blocks.</p>
          <div className="heatmap-grid">
            {rows.length > 0 ? rows.slice(0, 20).map((row, idx) => {
              const pct = Math.max(10, Math.round(((row.words || 0) / maxWords) * 100));
              return (
                <span key={`${row.url}-${idx}`} className="heat-cell" style={{ opacity: pct / 100 }}>
                  {(row.words || 0).toString().slice(0, 3)}w
                </span>
              );
            }) : (
              <span className="heat-cell">no_content</span>
            )}
          </div>

          <h3>LIVE_SCAN_FEED</h3>
          <div className="telemetry-feed">
            {(sitemap.crawl_results || []).slice(0, 10).map((result, idx) => (
              <div key={`${result.url}-${idx}`} className="telemetry-line">
                <span>{`02:${String(idx + 10).padStart(2, "0")}`}</span>
                <strong className={result.ok ? "tag-ok" : "tag-warn"}>{result.status || "ERR"}</strong>
                <span className="url-cell">{result.url}</span>
                <strong>{result.word_count || 0}w</strong>
              </div>
            ))}
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>READABILITY_MATRIX</h3>
          <div className="matrix-panel">
            {rows.slice(0, 6).map((row, idx) => (
              <div className="matrix-row" key={`${row.url}-${idx}`}>
                <span className="matrix-key">node_{idx + 1}</span>
                <span className="matrix-value">{row.title || row.url}</span>
                <span className={`matrix-state ${row.words && row.words > 350 ? "tag-ok" : "tag-warn"}`}>{row.words || 0}w</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>CONTENT_OVERVIEW</h2>
        <div className="crawl-stats-row">
          <div className="crawl-stat"><span className="crawl-stat-value text-accent">{rows.length}</span><span className="crawl-stat-label">links</span></div>
          <div className="crawl-stat"><span className="crawl-stat-value text-success">{totalWords}</span><span className="crawl-stat-label">words_total</span></div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>CONTENT_PER_LINK [{rows.length}]</h2>
        {rows.length > 0 ? rows.map((row, i) => <ContentAccordion key={`${row.url}-${i}`} row={row} defaultOpen={i === 0} />) : <span className="text-muted">No content data</span>}
      </div>
    </div>
  );
}
