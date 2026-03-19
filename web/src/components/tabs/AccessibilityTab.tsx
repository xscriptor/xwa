"use client";

import { useState } from "react";
import type { AccessibilityData, AccessibilityPage } from "@/lib/types";

function SeverityBadge({ severity }: { severity: string }) {
  const cls = severity === "error" ? "badge-4xx" : severity === "warning" ? "badge-3xx" : "badge-2xx";
  return <span className={`status-badge ${cls}`}>{severity.toUpperCase()}</span>;
}

function PageA11yCard({ data, defaultOpen = false }: { data: AccessibilityPage; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  const summary = data.summary || {};
  const url = data.url || "Main Page";

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{url}</span>
        </div>
        <div className="a11y-page-badges">
          {(summary.errors || 0) > 0 && <span className="status-badge badge-4xx">ERR:{summary.errors}</span>}
          {(summary.warnings || 0) > 0 && <span className="status-badge badge-3xx">WARN:{summary.warnings}</span>}
          {(summary.info || 0) > 0 && <span className="status-badge badge-2xx">INFO:{summary.info}</span>}
          {(summary.total_issues || 0) === 0 && <span className="status-badge badge-2xx">PASS</span>}
        </div>
      </div>

      {open && (
        <div className="details-content a11y-page-body">
          <div className="a11y-section">
            <h3>LANG_CHARSET</h3>
            <div className="detail-item">
              <span className="label">html[lang]:</span>
              <span className={`value ${data.language?.has_lang ? "text-success" : "text-danger"}`}>
                {data.language?.lang || "MISSING"}
              </span>
            </div>
            <div className="detail-item">
              <span className="label">charset:</span>
              <span className={`value ${data.language?.has_charset ? "text-success" : "text-danger"}`}>
                {data.language?.charset || "MISSING"}
              </span>
            </div>
            <div className="detail-item">
              <span className="label">encoding:</span>
              <span className={`value ${data.language?.encoding_valid ? "text-success" : "text-danger"}`}>
                {data.language?.encoding_valid ? "VALID" : "INVALID"}
              </span>
            </div>
          </div>

          <div className="a11y-section">
            <h3>HEADING_STRUCTURE [{data.headings?.total_headings || 0}]</h3>
            {(data.headings?.headings || []).length > 0 ? (
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>Tag</th><th>Text</th><th>Content Below</th></tr></thead>
                  <tbody>
                    {(data.headings?.headings || []).map((h, i) => (
                      <tr key={i}>
                        <td className="font-medium">{h.tag}</td>
                        <td className="url-cell">{h.text || <span className="text-muted">empty</span>}</td>
                        <td className={h.has_content_below ? "text-success" : "text-warning"}>
                          {h.has_content_below ? "YES" : "NO"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : <span className="text-muted">no headings found</span>}
            {(data.headings?.issues || []).length > 0 && (
              <div className="a11y-issues-list mt-4">
                {(data.headings?.issues || []).map((issue, i) => (
                  <div key={i} className="a11y-issue-row">
                    <SeverityBadge severity={issue.severity} />
                    <span>{issue.message}</span>
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="a11y-section">
            <h3>IMAGES [{data.images?.total_images || 0}] missing_alt:{data.images?.missing_alt || 0} empty_alt:{data.images?.empty_alt || 0}</h3>
            {(data.images?.images || []).length > 0 ? (
              <div className="table-responsive" style={{ maxHeight: "250px", overflowY: "auto" }}>
                <table className="link-table">
                  <thead><tr><th>#</th><th>Source</th><th>Alt</th><th>Status</th></tr></thead>
                  <tbody>
                    {(data.images?.images || []).map((img, i) => (
                      <tr key={i}>
                        <td className="text-muted">{i + 1}</td>
                        <td className="url-cell">{img.src}</td>
                        <td className="url-cell">{img.alt_text || <span className="text-muted">-</span>}</td>
                        <td className={!img.has_alt ? "text-danger" : img.alt_empty ? "text-warning" : "text-success"}>
                          {!img.has_alt ? "MISSING" : img.alt_empty ? "EMPTY" : "OK"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : <span className="text-muted">no images</span>}
          </div>

          <div className="a11y-section">
            <h3>ARIA_LABELS total:{data.aria?.total_interactive || 0} missing:{data.aria?.missing_labels || 0}</h3>
            {(data.aria?.issues || []).length > 0 ? (
              <div className="a11y-issues-list">
                {(data.aria?.issues || []).map((issue, i) => (
                  <div key={i} className="a11y-issue-row">
                    <SeverityBadge severity={issue.severity} />
                    <div>
                      <div>{issue.message}</div>
                      {issue.html_snippet && <code className="a11y-snippet">{issue.html_snippet}</code>}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="alert-box alert-success">All interactive elements have accessible labels</div>
            )}
          </div>

          <div className="a11y-section">
            <h3>FORMS [{data.forms?.total_forms || 0}]</h3>
            {(data.forms?.forms || []).length > 0 ? (
              (data.forms?.forms || []).map((form, i) => (
                <div key={i} className="a11y-form-card glass-panel">
                  <div className="detail-item">
                    <span className="label">form[{form.index}]</span>
                    <span className="value">action=&quot;{form.action || "/"}&quot; method={form.method}</span>
                  </div>
                  <div className="detail-item">
                    <span className="label">inputs:</span>
                    <span className="value">{form.total_inputs} total, {form.labeled_inputs} labeled</span>
                  </div>
                  <div className="detail-item">
                    <span className="label">submit:</span>
                    <span className={`value ${form.has_submit ? "text-success" : "text-danger"}`}>
                      {form.has_submit ? "YES" : "MISSING"}
                    </span>
                  </div>
                  {(form.issues || []).length > 0 && (
                    <div className="a11y-issues-list mt-4">
                      {(form.issues || []).map((issue, j) => (
                        <div key={j} className="a11y-issue-row">
                          <SeverityBadge severity={issue.severity} />
                          <span>{issue.message}</span>
                        </div>
                      ))}
                    </div>
                  )}
                  {(form.unlabeled_inputs?.length || 0) > 0 && (
                    <div className="table-responsive mt-4">
                      <table className="link-table">
                        <thead><tr><th>Tag</th><th>Type</th><th>Name</th><th>HTML</th></tr></thead>
                        <tbody>
                          {(form.unlabeled_inputs || []).map((inp, j) => (
                            <tr key={j}>
                              <td>{inp.tag}</td>
                              <td>{inp.type}</td>
                              <td>{inp.name || "-"}</td>
                              <td className="url-cell">{inp.html}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              ))
            ) : <span className="text-muted">no forms</span>}
          </div>
        </div>
      )}
    </div>
  );
}

export default function AccessibilityTab({ accessibility }: { accessibility: AccessibilityData }) {
  const mainPage = accessibility?.main_page;
  const perUrl = accessibility?.per_url || [];

  if (!mainPage) {
    return (
      <div className="details-card glass-panel">
        <h2>ACCESSIBILITY</h2>
        <span className="text-muted">No accessibility data available</span>
      </div>
    );
  }

  const totalErrors = (mainPage.summary?.errors || 0) + perUrl.reduce((s, p) => s + (p.summary?.errors || 0), 0);
  const totalWarnings = (mainPage.summary?.warnings || 0) + perUrl.reduce((s, p) => s + (p.summary?.warnings || 0), 0);
  const totalPages = 1 + perUrl.length;
  const focusNodes = [
    ...(mainPage.headings?.headings || []).slice(0, 12).map((h, i) => `${h.tag || "node"}_${i + 1}`),
    ...(mainPage.forms?.forms || []).slice(0, 4).map((f) => `form_${f.index ?? 0}`),
  ].slice(0, 16);
  const readerSignals = [
    { key: "lang", value: mainPage.language?.has_lang ? "ok" : "missing", state: mainPage.language?.has_lang ? "ok" : "alert" },
    { key: "charset", value: mainPage.language?.has_charset ? "ok" : "missing", state: mainPage.language?.has_charset ? "ok" : "alert" },
    { key: "aria labels", value: `${mainPage.aria?.total_interactive || 0} total`, state: (mainPage.aria?.missing_labels || 0) > 0 ? "warn" : "ok" },
    { key: "forms", value: `${mainPage.forms?.total_forms || 0} forms`, state: "ok" },
  ];

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>FOCUS_ORDER_MAP</h2>
          <p className="mono-subline">Tab-stop projection based on heading landmarks and form controls.</p>
          <div className="focus-map-grid">
            {focusNodes.length > 0 ? focusNodes.map((node, idx) => (
              <span key={`${node}-${idx}`} className="focus-cell">{node}</span>
            )) : (
              <span className="focus-cell">no_focus_nodes</span>
            )}
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>READER_ENGINE</h3>
          <div className="matrix-panel">
            {readerSignals.map((item) => (
              <div className="matrix-row" key={item.key}>
                <span className="matrix-key">{item.key}</span>
                <span className="matrix-value">{item.value}</span>
                <span className={`matrix-state tag-${item.state}`}>{item.state}</span>
              </div>
            ))}
          </div>

          <h3>A11Y_RADIAL</h3>
          <div className="radial-micro-grid">
            <div className="radial-micro-card">
              <h4>errors</h4>
              <div className="radial-ring" style={{ ["--pct" as string]: Math.min(100, totalErrors * 10), ["--ring-color" as string]: "var(--danger)" }}>
                <span className="radial-ring-label">{totalErrors}</span>
              </div>
            </div>
            <div className="radial-micro-card">
              <h4>warnings</h4>
              <div className="radial-ring" style={{ ["--pct" as string]: Math.min(100, totalWarnings * 8), ["--ring-color" as string]: "var(--warning)" }}>
                <span className="radial-ring-label">{totalWarnings}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>A11Y_OVERVIEW</h2>
        <div className="crawl-stats-row">
          <div className="crawl-stat">
            <span className="crawl-stat-value text-accent">{totalPages}</span>
            <span className="crawl-stat-label">pages</span>
          </div>
          <div className="crawl-stat">
            <span className={`crawl-stat-value ${totalErrors > 0 ? "text-danger" : "text-success"}`}>{totalErrors}</span>
            <span className="crawl-stat-label">errors</span>
          </div>
          <div className="crawl-stat">
            <span className={`crawl-stat-value ${totalWarnings > 0 ? "text-warning" : "text-success"}`}>{totalWarnings}</span>
            <span className="crawl-stat-label">warnings</span>
          </div>
        </div>
      </div>

      <PageA11yCard data={mainPage} defaultOpen={true} />

      {perUrl.length > 0 && (
        <>
          <div className="details-card glass-panel">
            <h2>PER_URL_ANALYSIS [{perUrl.length} sub-pages]</h2>
            <span className="text-muted">Click each URL to expand details</span>
          </div>
          {perUrl.map((page, idx) => (
            <PageA11yCard key={idx} data={page} defaultOpen={false} />
          ))}
        </>
      )}
    </div>
  );
}
