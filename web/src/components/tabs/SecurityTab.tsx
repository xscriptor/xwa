"use client";

import { useMemo, useState } from "react";
import type { SecurityData, SecurityPerUrl } from "@/lib/types";

function SecurityUrlAccordion({ page, defaultOpen = false }: { page: SecurityPerUrl; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  const missingHeaders = page.headers?.missing_headers?.length || 0;
  const mixed = page.mixed_content?.total_mixed || 0;
  const techCount = page.technology?.count || 0;

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{page.url}</span>
        </div>
        <div className="a11y-page-badges">
          <span className={`status-badge ${missingHeaders === 0 ? "badge-2xx" : "badge-3xx"}`}>MISS:{missingHeaders}</span>
          <span className={`status-badge ${mixed === 0 ? "badge-2xx" : "badge-4xx"}`}>MIX:{mixed}</span>
          <span className="status-badge badge-2xx">TECH:{techCount}</span>
        </div>
      </div>
      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">Missing Headers:</span><span className="value">{missingHeaders}</span></div>
          <div className="detail-item"><span className="label">CORS:</span><span className="value">{String(Boolean(page.cors && Object.keys(page.cors).length > 0)).toUpperCase()}</span></div>
          <div className="detail-item"><span className="label">CSP Present:</span><span className="value">{page.csp?.present ? "YES" : "NO"}</span></div>
          <div className="detail-item"><span className="label">SRI Missing:</span><span className="value">{page.sri?.missing_integrity || 0}</span></div>
          <div className="detail-item"><span className="label">Exposed Emails:</span><span className="value">{page.exposure?.email_count || 0}</span></div>
          <div className="detail-item"><span className="label">Internal IPs:</span><span className="value">{page.exposure?.ip_count || 0}</span></div>
        </div>
      )}
    </div>
  );
}

export default function SecurityTab({ security: sec }: { security: SecurityData }) {
  const perUrl = useMemo(() => sec.per_url || [], [sec.per_url]);
  const missingHeaderCount = sec.headers?.missing_headers?.length || 0;
  const exposedCount = sec.sensitive_paths_found?.length || 0;
  const threatCells = [
    `ssl:${sec.ssl?.valid ? "ok" : "risk"}`,
    `headers:${missingHeaderCount}`,
    `cookies:${sec.cookies?.issues?.length || 0}`,
    `paths:${exposedCount}`,
    `cors:${Object.keys(sec.cors || {}).length}`,
    `csp:${sec.csp?.present ? "on" : "off"}`,
    `mixed:${sec.mixed_content?.total_mixed || 0}`,
    `sri:${sec.sri?.missing_integrity || 0}`,
  ];

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>VULNERABILITY_ANALYSIS</h2>
          <div className="telemetry-feed">
            <div className="telemetry-line"><span>[01]</span><span>SSL</span><span className="url-cell">certificate validation</span><span className={sec.ssl?.valid ? "text-success" : "text-danger"}>{sec.ssl?.valid ? "SECURE" : "RISK"}</span></div>
            <div className="telemetry-line"><span>[02]</span><span>HEADERS</span><span className="url-cell">security headers check</span><span className={missingHeaderCount > 0 ? "text-warning" : "text-success"}>{missingHeaderCount} missing</span></div>
            <div className="telemetry-line"><span>[03]</span><span>PATHS</span><span className="url-cell">sensitive paths exposure</span><span className={exposedCount > 0 ? "text-danger" : "text-success"}>{exposedCount} exposed</span></div>
            <div className="telemetry-line"><span>[04]</span><span>COOKIES</span><span className="url-cell">flags + hardening</span><span className={(sec.cookies?.issues?.length || 0) > 0 ? "text-warning" : "text-success"}>{sec.cookies?.issues?.length || 0} issues</span></div>
          </div>

          <h3>SSL_TLS_CERT</h3>
          <div className="details-content">
            {[
              ["Status", sec.ssl?.valid ? "VALID" : "INVALID", sec.ssl?.valid ? "text-success" : "text-danger"],
              ["Issuer", sec.ssl?.issuer || "Unknown", ""],
              ["Subject", sec.ssl?.subject || "N/A", ""],
              ["Expires", sec.ssl?.expires_on || "N/A", ""],
              ["Days_Left", sec.ssl?.days_remaining ?? "N/A", (sec.ssl?.days_remaining || 0) < 30 ? "text-warning" : "text-success"],
              ["Expired", sec.ssl?.is_expired ? "YES" : "NO", sec.ssl?.is_expired ? "text-danger" : "text-success"],
            ].map(([label, value, cls]) => (
              <div className="detail-item" key={label}>
                <span className="label">{label}:</span>
                <span className={`value ${cls}`}>{value}</span>
              </div>
            ))}
            {sec.ssl?.error && (
              <div className="detail-item">
                <span className="label">Error:</span>
                <span className="value text-danger">{sec.ssl.error}</span>
              </div>
            )}
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>THREAT_STATUS</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">SSL_STATE</span><span className={`value ${sec.ssl?.valid ? "text-success" : "text-danger"}`}>{sec.ssl?.valid ? "VALID" : "INVALID"}</span></div>
            <div className="mini-stat"><span className="label">HDR_MISSING</span><span className={`value ${missingHeaderCount > 0 ? "text-warning" : "text-success"}`}>{missingHeaderCount}</span></div>
            <div className="mini-stat"><span className="label">EXPOSED_PATHS</span><span className={`value ${exposedCount > 0 ? "text-danger" : "text-success"}`}>{exposedCount}</span></div>
            <div className="mini-stat"><span className="label">COOKIE_ISSUES</span><span className={`value ${(sec.cookies?.issues?.length || 0) > 0 ? "text-warning" : "text-success"}`}>{sec.cookies?.issues?.length || 0}</span></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>SECURITY_HEADERS</h2>
        <div className="details-content">
          <div className="detail-item vertical">
            <span className="label text-success">present [{Object.keys(sec.headers?.headers_present || {}).length}]</span>
            {Object.keys(sec.headers?.headers_present || {}).length > 0 ? (
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>Header</th><th>Value</th></tr></thead>
                  <tbody>
                    {Object.entries(sec.headers?.headers_present || {}).map(([k, v]) => (
                      <tr key={k}><td className="font-medium">{k}</td><td className="url-cell text-success">{v}</td></tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : <span className="text-muted">none</span>}
          </div>
          <div className="detail-item vertical mt-4">
            <span className="label text-danger">missing [{sec.headers?.missing_headers?.length || 0}]</span>
            <ul className="value-list error-list">
              {(sec.headers?.missing_headers || []).length > 0
                ? (sec.headers?.missing_headers || []).map((h: string) => <li key={h}>{h}</li>)
                : <li className="text-success">all present</li>
              }
            </ul>
          </div>
          <div className="detail-item vertical mt-4">
            <span className="label text-warning">leaked_info</span>
            {Object.keys(sec.headers?.leaked_server_info || {}).length > 0 ? (
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>Header</th><th>Value</th></tr></thead>
                  <tbody>
                    {Object.entries(sec.headers?.leaked_server_info || {}).map(([k, v]) => (
                      <tr key={k}><td className="font-medium">{k}</td><td className="url-cell text-warning">{v}</td></tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : <span className="text-success">none leaked</span>}
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>COOKIES [{sec.cookies?.total || 0}]</h2>
        <div className="details-content">
          {(sec.cookies?.cookies || []).length > 0 ? (
            <>
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>Name</th><th>Secure</th><th>HttpOnly</th><th>SameSite</th></tr></thead>
                  <tbody>
                    {(sec.cookies?.cookies || []).map((c, i) => (
                      <tr key={i}>
                        <td className="font-medium">{c.name}</td>
                        <td className={c.secure ? "text-success" : "text-danger"}>{c.secure ? "Y" : "N"}</td>
                        <td className={c.httponly ? "text-success" : "text-danger"}>{c.httponly ? "Y" : "N"}</td>
                        <td>{c.samesite || "N/A"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              {(sec.cookies?.issues || []).length > 0 && (
                <div className="detail-item vertical mt-4">
                  <span className="label text-warning">issues</span>
                  <ul className="value-list error-list">
                    {(sec.cookies?.issues || []).map((issue, i) => <li key={i}>{issue}</li>)}
                  </ul>
                </div>
              )}
            </>
          ) : <span className="text-muted">no cookies</span>}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>SENSITIVE_PATHS</h2>
        <div className="details-content">
          {(sec.sensitive_paths_found || []).length > 0 ? (
            <div className="detail-item vertical">
              <span className="label text-danger">exposed [{(sec.sensitive_paths_found || []).length}]</span>
              <ul className="value-list error-list">
                {(sec.sensitive_paths_found || []).map((p) => <li key={p}>{p}</li>)}
              </ul>
            </div>
          ) : (
            <div className="alert-box alert-success">All paths returned 404. No exposures.</div>
          )}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>THREAT_VIEWPORT</h2>
        <div className="threat-map-grid">
          {threatCells.map((cell, idx) => (
            <span key={`${cell}-${idx}`} className="threat-cell">{cell}</span>
          ))}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>OPERATOR_CONSOLE</h2>
        <div className="terminal-action-panel">
          <div className="terminal-action-line"><span>run::tls_policy</span><strong>{sec.ssl?.valid ? "monitor" : "renew immediately"}</strong></div>
          <div className="terminal-action-line"><span>run::header_hardening</span><strong>{missingHeaderCount > 0 ? `add ${missingHeaderCount} headers` : "compliant"}</strong></div>
          <div className="terminal-action-line"><span>run::path_protection</span><strong>{exposedCount > 0 ? "restrict exposed endpoints" : "clean"}</strong></div>
          <div className="terminal-action-line"><span>run::cookie_review</span><strong>{(sec.cookies?.issues?.length || 0) > 0 ? "set secure/httponly" : "stable"}</strong></div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>SECURITY_PER_LINK [{perUrl.length}]</h2>
        {perUrl.length > 0 ? (
          <>
            <span className="text-muted">Expande cada URL para ver su snapshot de seguridad.</span>
            <div style={{ marginTop: "1rem" }}>
              {perUrl.map((page, i) => (
                <SecurityUrlAccordion key={`${page.url}-${i}`} page={page} defaultOpen={i === 0} />
              ))}
            </div>
          </>
        ) : (
          <span className="text-muted">No hay análisis de seguridad por enlace en este reporte.</span>
        )}
      </div>
    </div>
  );
}
