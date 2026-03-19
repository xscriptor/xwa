"use client";

import { useMemo, useState } from "react";
import type { SeoData, SeoPerUrl } from "@/lib/types";

function SeoUrlAccordion({ page, defaultOpen = false }: { page: SeoPerUrl; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  const ogCount = Object.keys(page.social_meta?.og || {}).length;
  const twCount = Object.keys(page.social_meta?.twitter || {}).length;
  const h1Count = page.headings?.counts?.h1 || 0;

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{page.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className="status-badge badge-2xx">OG:{ogCount}</span>
          <span className="status-badge badge-2xx">TW:{twCount}</span>
          <span className={`status-badge ${h1Count === 1 ? "badge-2xx" : "badge-3xx"}`}>H1:{h1Count}</span>
        </div>
      </div>

      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">Title:</span><span className="value">{page.standard_meta?.title || "-"}</span></div>
          <div className="detail-item"><span className="label">Description:</span><span className="value">{page.standard_meta?.description || "-"}</span></div>
          <div className="detail-item"><span className="label">Canonical:</span><span className="value">{page.canonical || "-"}</span></div>
          <div className="detail-item"><span className="label">Words:</span><span className="value">{page.text_ratio?.word_count || 0}</span></div>
          <div className="detail-item"><span className="label">Missing Alt:</span><span className="value">{page.image_alts?.missing_alt || 0}</span></div>
        </div>
      )}
    </div>
  );
}

export default function SeoTab({ seo }: { seo: SeoData }) {
  const structuredData = seo.structured_data || {};
  const linkTags = seo.link_tags || [];
  const headingDetails = seo.headings?.details || [];
  const perUrl = useMemo(() => seo.per_url || [], [seo.per_url]);
  const ogCount = Object.keys(seo.social_meta?.og || {}).length;
  const twCount = Object.keys(seo.social_meta?.twitter || {}).length;
  const h1Count = seo.headings?.counts?.h1 || 0;
  const missingCritical = [
    !seo.standard_meta?.title,
    !seo.standard_meta?.description,
    !seo.canonical,
    h1Count !== 1,
  ].filter(Boolean).length;
  const seoScore = Math.max(15, 100 - missingCritical * 18);
  const backlinkRows = perUrl.slice(0, 8).map((row, idx) => ({
    key: row.url,
    anchor: row.headings?.details?.find((h) => h.tag === "H1")?.text || `node_${idx + 1}`,
    signal: (row.text_ratio?.word_count || 0) > 300 ? "authoritative" : "light",
    state: row.standard_meta?.title ? "ok" : "warn",
  }));

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>SEO_COMMAND_DECK</h2>
          <p className="mono-subline">Primary search integrity index with metadata, heading and social graph checks.</p>
          <div className="status-strip">
            <div className="status-cell"><span>title</span><strong>{seo.standard_meta?.title ? "present" : "missing"}</strong></div>
            <div className="status-cell"><span>description</span><strong>{seo.standard_meta?.description ? "present" : "missing"}</strong></div>
            <div className="status-cell"><span>canonical</span><strong>{seo.canonical ? "defined" : "missing"}</strong></div>
            <div className="status-cell"><span>h1 policy</span><strong>{h1Count === 1 ? "valid" : "review"}</strong></div>
          </div>

          <h3>BACKLINK_GRAPH_SIM</h3>
          <div className="matrix-panel">
            {backlinkRows.length > 0 ? (
              backlinkRows.map((row, idx) => (
                <div className="matrix-row" key={`${row.key}-${idx}`}>
                  <span className="matrix-key">{row.signal}</span>
                  <span className="matrix-value">{row.anchor}</span>
                  <span className={`matrix-state tag-${row.state}`}>{row.state}</span>
                </div>
              ))
            ) : (
              <div className="matrix-row">
                <span className="matrix-key">idle</span>
                <span className="matrix-value">no per-url seo nodes in this report</span>
                <span className="matrix-state tag-warn">wait</span>
              </div>
            )}
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>SEO_SCORE_RING</h3>
          <div className="radial-micro-grid">
            <div className="radial-micro-card">
              <h4>global score</h4>
              <div className="radial-ring" style={{ ["--pct" as string]: seoScore, ["--ring-color" as string]: "var(--primary)" }}>
                <span className="radial-ring-label">{seoScore}</span>
              </div>
            </div>
            <div className="radial-micro-card">
              <h4>og tags</h4>
              <div className="radial-ring" style={{ ["--pct" as string]: Math.min(100, ogCount * 10), ["--ring-color" as string]: "var(--secondary)" }}>
                <span className="radial-ring-label">{ogCount}</span>
              </div>
            </div>
            <div className="radial-micro-card">
              <h4>twitter tags</h4>
              <div className="radial-ring" style={{ ["--pct" as string]: Math.min(100, twCount * 10), ["--ring-color" as string]: "var(--tertiary)" }}>
                <span className="radial-ring-label">{twCount}</span>
              </div>
            </div>
          </div>

          <h3>STATUS_PANEL</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">critical misses</span><span className="value">{missingCritical}</span></div>
            <div className="mini-stat"><span className="label">json-ld nodes</span><span className="value">{(structuredData.json_ld || []).length}</span></div>
            <div className="mini-stat"><span className="label">link tags</span><span className="value">{linkTags.length}</span></div>
            <div className="mini-stat"><span className="label">headings</span><span className="value">{headingDetails.length}</span></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>STANDARD_META</h2>
        <div className="details-content">
          {[
            ["Title", seo.standard_meta?.title, true],
            ["Description", seo.standard_meta?.description, true],
            ["Author", seo.standard_meta?.author, false],
            ["Robots", seo.standard_meta?.robots, false],
            ["Canonical", seo.canonical, true],
            ["Viewport", seo.standard_meta?.viewport, true],
            ["Charset", seo.standard_meta?.charset, true],
            ["Generator", seo.standard_meta?.generator, false],
            ["Theme_Color", seo.standard_meta?.theme_color, false],
          ].map(([label, value, warn]) => (
            <div className="detail-item" key={String(label)}>
              <span className="label">{String(label)}:</span>
              <span className={`value ${!value && warn ? "text-warning" : ""}`}>
                {value || (warn ? "MISSING" : <span className="text-muted">not_set</span>)}
              </span>
            </div>
          ))}
          <div className="detail-item">
            <span className="label">Keywords:</span>
            <span className="value">
              {seo.standard_meta?.keywords
                ? <div className="tag-chips">{seo.standard_meta.keywords.split(",").map((k, i) => (
                    <span key={i} className="tag-chip">{k.trim()}</span>
                  ))}</div>
                : <span className="text-muted">not_set</span>
              }
            </span>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>OPENGRAPH [{Object.keys(seo.social_meta?.og || {}).length}]</h2>
        <div className="details-content">
          {Object.keys(seo.social_meta?.og || {}).length > 0 ? (
            <div className="table-responsive">
              <table className="link-table">
                <thead><tr><th>Property</th><th>Value</th></tr></thead>
                <tbody>
                  {Object.entries(seo.social_meta?.og || {}).map(([key, val]) => (
                    <tr key={key}><td className="font-medium">og:{key}</td><td className="url-cell">{val}</td></tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="alert-box alert-warning">No Open Graph meta tags found. Social sharing previews will not display correctly.</div>
          )}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>TWITTER_CARDS [{Object.keys(seo.social_meta?.twitter || {}).length}]</h2>
        <div className="details-content">
          {Object.keys(seo.social_meta?.twitter || {}).length > 0 ? (
            <div className="table-responsive">
              <table className="link-table">
                <thead><tr><th>Property</th><th>Value</th></tr></thead>
                <tbody>
                  {Object.entries(seo.social_meta?.twitter || {}).map(([key, val]) => (
                    <tr key={key}><td className="font-medium">twitter:{key}</td><td className="url-cell">{val}</td></tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="alert-box alert-warning">No Twitter Card meta tags found.</div>
          )}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>STRUCTURED_DATA</h2>
        <div className="details-content">
          <div className="detail-item vertical">
            <span className="label">JSON-LD [{(structuredData.json_ld || []).length}]</span>
            {(structuredData.json_ld || []).length > 0 ? (
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>@type</th><th>Name</th><th>Keys</th></tr></thead>
                  <tbody>
                    {(structuredData.json_ld || []).map((item, i) => (
                      <tr key={i}>
                        <td className="font-medium text-accent">{String((item as Record<string, unknown>).type || "Unknown")}</td>
                        <td className="url-cell">{String((item as Record<string, unknown>).name || "-")}</td>
                        <td className="text-muted">{Array.isArray((item as Record<string, unknown>).raw_keys) ? ((item as Record<string, unknown>).raw_keys as string[]).join(", ") : "-"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : <span className="text-muted">no JSON-LD</span>}
          </div>
          <div className="detail-item vertical mt-4">
            <span className="label">Microdata [{(structuredData.microdata_types || []).length}]</span>
            {(structuredData.microdata_types || []).length > 0 ? (
              <ul className="value-list">
                {(structuredData.microdata_types || []).map((t, i) => <li key={i}>{t}</li>)}
              </ul>
            ) : <span className="text-muted">no microdata</span>}
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>HEADINGS h1:{seo.headings?.counts?.h1 || 0} h2:{seo.headings?.counts?.h2 || 0} h3:{seo.headings?.counts?.h3 || 0}</h2>
        <div className="details-content">
          {seo.headings?.missing_h1 && <div className="alert-box alert-danger">No H1 tag found on the page</div>}
          {seo.headings?.multiple_h1 && <div className="alert-box alert-warning">Multiple H1 tags detected ({seo.headings?.counts?.h1})</div>}
          {headingDetails.length > 0 && (
            <div className="table-responsive" style={{ maxHeight: "250px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TAG</th><th>TEXT</th><th>ID</th></tr></thead>
                <tbody>
                  {headingDetails.map((h, i) => (
                    <tr key={i}>
                      <td className="font-medium">{h.tag || "-"}</td>
                      <td className="url-cell">{h.text || <span className="text-muted">empty</span>}</td>
                      <td className="text-accent">{h.id || "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>

      {linkTags.length > 0 && (
        <div className="details-card glass-panel">
          <h2>LINK_TAGS [{linkTags.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "250px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>REL</th><th>HREF</th><th>TYPE</th><th>LANG</th></tr></thead>
              <tbody>
                {linkTags.map((lt, i) => (
                  <tr key={i}>
                    <td className="font-medium">{String(lt.rel || "-")}</td>
                    <td className="url-cell">{String(lt.href || "-")}</td>
                    <td className="text-muted">{String(lt.type || "-")}</td>
                    <td className="text-muted">{String(lt.hreflang || "-")}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <div className="details-card glass-panel">
        <h2>ROBOTS_TXT</h2>
        <div className="details-content">
          <div className="detail-item">
            <span className="label">Presence:</span>
            <span className={`value ${seo.robots_txt?.presence ? "text-success" : "text-danger"}`}>
              {seo.robots_txt?.presence ? "FOUND" : "NOT_FOUND"}
            </span>
          </div>
          <div className="detail-item">
            <span className="label">URL:</span>
            <span className="value">{seo.robots_txt?.url || "N/A"}</span>
          </div>
          <div className="detail-item">
            <span className="label">Status:</span>
            <span className="value">{seo.robots_txt?.status_code || "N/A"}</span>
          </div>
          {(seo.robots_txt?.sitemaps || []).length > 0 && (
            <div className="detail-item vertical mt-4">
              <span className="label">Sitemaps [{(seo.robots_txt?.sitemaps || []).length}]</span>
              <ul className="value-list">
                {(seo.robots_txt?.sitemaps || []).map((s, i) => <li key={i}>{s}</li>)}
              </ul>
            </div>
          )}
          {(seo.robots_txt?.directives || []).length > 0 && (
            <div className="detail-item vertical mt-4">
              <span className="label">Directives [{(seo.robots_txt?.directives || []).length}]</span>
              <div className="struct-code-block">
                {(seo.robots_txt?.directives || []).map((d: string, i: number) => (
                  <div key={i} className="struct-code-line">{d}</div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>TEXT_RATIO</h2>
        <div className="details-content">
          {[
            ["Ratio", `${seo.text_ratio?.text_to_html_ratio || 0}%`],
            ["Words", seo.text_ratio?.word_count || 0],
            ["HTML_Size", `${((seo.text_ratio?.html_size_bytes || 0) / 1024).toFixed(1)} KB`],
            ["Text_Size", `${((seo.text_ratio?.text_size_bytes || 0) / 1024).toFixed(1)} KB`],
          ].map(([label, value]) => (
            <div className="detail-item" key={label}>
              <span className="label">{label}:</span>
              <span className="value">{value}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>SEO_PER_LINK [{perUrl.length}]</h2>
        {perUrl.length > 0 ? (
          <>
            <span className="text-muted">Expande cada URL para ver su análisis SEO específico.</span>
            <div style={{ marginTop: "1rem" }}>
              {perUrl.map((page, i) => (
                <SeoUrlAccordion key={`${page.url}-${i}`} page={page} defaultOpen={i === 0} />
              ))}
            </div>
          </>
        ) : (
          <span className="text-muted">No hay análisis SEO por enlace en este reporte.</span>
        )}
      </div>
    </div>
  );
}
