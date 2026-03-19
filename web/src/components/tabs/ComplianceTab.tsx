"use client";

import { useMemo, useState } from "react";
import type { AccessibilityData, SecurityData, SeoData } from "@/lib/types";

interface ComplianceRow {
  url: string;
  a11yErrors: number;
  a11yWarnings: number;
  missingHeaders: number;
  hasLang: boolean;
  hasCharset: boolean;
}

function ComplianceAccordion({ row, defaultOpen = false }: { row: ComplianceRow; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{row.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className={`status-badge ${row.a11yErrors > 0 ? "badge-4xx" : "badge-2xx"}`}>A11Y_ERR:{row.a11yErrors}</span>
          <span className={`status-badge ${row.missingHeaders > 0 ? "badge-3xx" : "badge-2xx"}`}>HDR_MISS:{row.missingHeaders}</span>
        </div>
      </div>
      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">A11y warnings:</span><span className="value">{row.a11yWarnings}</span></div>
          <div className="detail-item"><span className="label">html[lang]:</span><span className={`value ${row.hasLang ? "text-success" : "text-danger"}`}>{row.hasLang ? "OK" : "MISSING"}</span></div>
          <div className="detail-item"><span className="label">charset:</span><span className={`value ${row.hasCharset ? "text-success" : "text-danger"}`}>{row.hasCharset ? "OK" : "MISSING"}</span></div>
        </div>
      )}
    </div>
  );
}

export default function ComplianceTab({ security, accessibility, seo }: { security: SecurityData; accessibility: AccessibilityData; seo: SeoData }) {
  const rows = useMemo<ComplianceRow[]>(() => {
    const secMap = new Map((security.per_url || []).map((p) => [p.url, p]));
    const a11y = accessibility.per_url || [];

    const combined = a11y.map((page) => {
      const sec = page.url ? secMap.get(page.url) : undefined;
      return {
        url: page.url || "main",
        a11yErrors: page.summary?.errors || 0,
        a11yWarnings: page.summary?.warnings || 0,
        missingHeaders: sec?.headers?.missing_headers?.length || 0,
        hasLang: Boolean(page.language?.has_lang),
        hasCharset: Boolean(page.language?.has_charset),
      };
    });

    if (!combined.length && seo.per_url?.length) {
      return seo.per_url.map((s) => ({
        url: s.url,
        a11yErrors: 0,
        a11yWarnings: 0,
        missingHeaders: secMap.get(s.url)?.headers?.missing_headers?.length || 0,
        hasLang: Boolean(s.standard_meta?.viewport),
        hasCharset: Boolean(s.standard_meta?.charset),
      }));
    }

    return combined;
  }, [security.per_url, accessibility.per_url, seo.per_url]);

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>LIVE_DOCUMENT_SCAN</h2>
          <div className="telemetry-feed">
            {rows.slice(0, 8).map((row, idx) => (
              <div key={`${row.url}-${idx}`} className="telemetry-line">
                <span>[{String(idx + 1).padStart(2, "0")}]</span>
                <span className="url-cell">{row.url}</span>
                <span className={row.a11yErrors > 0 || row.missingHeaders > 0 ? "text-warning" : "text-success"}>
                  {row.a11yErrors > 0 || row.missingHeaders > 0 ? "REVIEW" : "OK"}
                </span>
              </div>
            ))}
          </div>

          <h3>COMPLIANCE_OVERVIEW</h3>
          <div className="crawl-stats-row">
            <div className="crawl-stat"><span className="crawl-stat-value text-accent">{rows.length}</span><span className="crawl-stat-label">links</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-danger">{rows.reduce((s, r) => s + r.a11yErrors, 0)}</span><span className="crawl-stat-label">a11y_errors</span></div>
            <div className="crawl-stat"><span className="crawl-stat-value text-warning">{rows.reduce((s, r) => s + r.missingHeaders, 0)}</span><span className="crawl-stat-label">hdr_missing</span></div>
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>POLICY_COVERAGE</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">LANG_OK</span><span className="value text-success">{rows.filter((r) => r.hasLang).length}</span></div>
            <div className="mini-stat"><span className="label">CHARSET_OK</span><span className="value text-success">{rows.filter((r) => r.hasCharset).length}</span></div>
            <div className="mini-stat"><span className="label">AUDIT_TIMELINE</span><span className="value">ACTIVE</span></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>COMPLIANCE_PER_LINK [{rows.length}]</h2>
        <div className="protocol-hero-grid" style={{ marginBottom: "0.85rem" }}>
          <div className="glass-panel" style={{ padding: "0.75rem" }}>
            <h3>POLICY_COVERAGE_CARD</h3>
            <div className="status-strip">
              <div className="status-cell"><span>lang ok</span><strong>{rows.filter((r) => r.hasLang).length}</strong></div>
              <div className="status-cell"><span>charset ok</span><strong>{rows.filter((r) => r.hasCharset).length}</strong></div>
              <div className="status-cell"><span>a11y errors</span><strong>{rows.reduce((s, r) => s + r.a11yErrors, 0)}</strong></div>
              <div className="status-cell"><span>missing headers</span><strong>{rows.reduce((s, r) => s + r.missingHeaders, 0)}</strong></div>
            </div>
          </div>
          <div className="glass-panel" style={{ padding: "0.75rem" }}>
            <h3>AUDIT_TIMELINE_CARD</h3>
            <div className="waterfall-table">
              {rows.slice(0, 8).map((row, idx) => {
                const risk = row.a11yErrors + row.missingHeaders;
                return (
                  <div key={`${row.url}-${idx}`} className="waterfall-row">
                    <span className="url-cell">{row.url}</span>
                    <div className="waterfall-track"><div className="waterfall-fill" style={{ width: `${Math.max(8, Math.min(100, risk * 20))}%` }} /></div>
                    <strong>{risk}</strong>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
        {rows.length > 0 ? rows.map((row, i) => <ComplianceAccordion key={`${row.url}-${i}`} row={row} defaultOpen={i === 0} />) : <span className="text-muted">No compliance data</span>}
      </div>
    </div>
  );
}
