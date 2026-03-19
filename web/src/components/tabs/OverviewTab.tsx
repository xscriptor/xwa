"use client";

import type { AccessibilityData, SecurityData, SeoData, SitemapData } from "@/lib/types";

interface OverviewTabProps {
  seo: SeoData;
  security: SecurityData;
  sitemap: SitemapData;
  accessibility: AccessibilityData;
  healthScore: number;
  warnings: string[];
}

function getScoreColor(score: number): string {
  if (score >= 80) return "var(--success)";
  if (score >= 50) return "var(--warning)";
  return "var(--danger)";
}

function getScoreLabel(score: number): string {
  if (score >= 90) return "EXCELLENT";
  if (score >= 80) return "GOOD";
  if (score >= 60) return "NEEDS_WORK";
  if (score >= 40) return "POOR";
  return "CRITICAL";
}

export default function OverviewTab({ seo, security: sec, sitemap: sit, accessibility: a11y, healthScore, warnings }: OverviewTabProps) {
  const a11ySummary = a11y?.main_page?.summary || {};
  const headerMissing = sec.headers?.missing_headers || [];
  const telemetryRows = [
    { tag: "SEO", level: seo.standard_meta?.title ? "ok" : "warn", detail: seo.standard_meta?.title ? "title present" : "title missing", metric: `${Object.keys(seo.social_meta?.og || {}).length} og` },
    { tag: "SSL", level: sec.ssl?.valid ? "ok" : "alert", detail: sec.ssl?.valid ? "certificate valid" : "certificate invalid", metric: sec.ssl?.days_remaining ? `${sec.ssl.days_remaining}d` : "n/a" },
    { tag: "HEAD", level: headerMissing.length === 0 ? "ok" : "warn", detail: headerMissing.length === 0 ? "headers complete" : `${headerMissing.length} headers missing`, metric: `${6 - headerMissing.length}/6` },
    { tag: "A11Y", level: (a11ySummary.errors || 0) > 0 ? "alert" : "ok", detail: `${a11ySummary.total_issues || 0} total findings`, metric: `e${a11ySummary.errors || 0}:w${a11ySummary.warnings || 0}` },
    { tag: "SMAP", level: (sit.broken_links?.length || 0) > 0 ? "warn" : "ok", detail: `${sit.urls_found || 0} urls discovered`, metric: `${sit.broken_links?.length || 0} broken` },
    { tag: "COOK", level: (sec.cookies?.issues?.length || 0) > 0 ? "warn" : "ok", detail: `${sec.cookies?.total || 0} cookies scanned`, metric: `${sec.cookies?.issues?.length || 0} issues` },
  ];

  return (
    <div className="overview-layout">
      <div className="glass-panel protocol-hero-grid">
        <div className="protocol-hero-main score-card">
          <h2>OPERATIONS_OVERVIEW</h2>
          <p className="mono-subline">Cross-module status strip for SEO, security, crawl and accessibility.</p>
          <div className="score-gauge">
            <svg viewBox="0 0 120 120" className="score-ring">
              <circle cx="60" cy="60" r="52" className="score-track" />
              <circle
                cx="60" cy="60" r="52"
                className="score-fill"
                style={{
                  strokeDasharray: `${(healthScore / 100) * 327} 327`,
                  stroke: getScoreColor(healthScore)
                }}
              />
            </svg>
            <div className="score-number" style={{ color: getScoreColor(healthScore) }}>
              {healthScore}
            </div>
          </div>
          <span className="score-label" style={{ color: getScoreColor(healthScore) }}>{getScoreLabel(healthScore)}</span>

          <h3>SYSTEM_TELEMETRY</h3>
          <div className="telemetry-feed">
            {telemetryRows.map((row, index) => (
              <div key={`${row.tag}-${index}`} className="telemetry-line">
                <span>{`00:${String(index + 1).padStart(2, "0")}`}</span>
                <strong className={`tag-${row.level}`}>{row.tag}</strong>
                <span>{row.detail}</span>
                <strong>{row.metric}</strong>
              </div>
            ))}
          </div>
        </div>

        <div className="protocol-hero-rail">
          <h2>GLOBAL_STATUS</h2>
          <p className="mono-subline">Fast command summary for current scan posture.</p>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">urls found</span><span className="value">{sit.urls_found}</span></div>
            <div className="mini-stat"><span className="label">broken links</span><span className="value">{sit.broken_links?.length || 0}</span></div>
            <div className="mini-stat"><span className="label">ssl validity</span><span className="value">{sec.ssl?.valid ? "valid" : "invalid"}</span></div>
            <div className="mini-stat"><span className="label">missing headers</span><span className="value">{headerMissing.length}</span></div>
            <div className="mini-stat"><span className="label">cookie issues</span><span className="value">{sec.cookies?.issues?.length || 0}</span></div>
            <div className="mini-stat"><span className="label">a11y issues</span><span className="value">{a11ySummary.total_issues || 0}</span></div>
          </div>
        </div>
      </div>

      <div className="metrics-grid">
        <div className="metric-card glass-panel">
          <h3>URLS_FOUND</h3>
          <div className="metric-value text-accent">{sit.urls_found}</div>
          <p className="metric-sub">scanned: {sit.scanned_count || 0}</p>
        </div>

        <div className="metric-card glass-panel">
          <h3>BROKEN_LINKS</h3>
          <div className={`metric-value ${(sit.broken_links?.length || 0) > 0 ? "text-danger" : "text-success"}`}>
            {sit.broken_links?.length || 0}
          </div>
          <p className="metric-sub">4xx/5xx responses</p>
        </div>

        <div className="metric-card glass-panel">
          <h3>SSL_CERT</h3>
          <div className={`metric-value ${sec.ssl?.valid ? "text-success" : "text-danger"}`}>
            {sec.ssl?.valid ? "VALID" : "INVALID"}
          </div>
          <p className="metric-sub">
            {sec.ssl?.days_remaining ? `${sec.ssl.days_remaining}d remaining` : sec.ssl?.error || "N/A"}
          </p>
        </div>

        <div className="metric-card glass-panel">
          <h3>SEC_HEADERS</h3>
          <div className={`metric-value ${(sec.headers?.missing_headers?.length || 0) === 0 ? "text-success" : "text-warning"}`}>
            {(sec.headers?.missing_headers?.length || 0)} MISS
          </div>
          <p className="metric-sub">of 6 checked</p>
        </div>

        <div className="metric-card glass-panel">
          <h3>EXPOSED_PATHS</h3>
          <div className={`metric-value ${(sec.sensitive_paths_found?.length || 0) > 0 ? "text-danger" : "text-success"}`}>
            {sec.sensitive_paths_found?.length || 0}
          </div>
          <p className="metric-sub">sensitive dirs</p>
        </div>

        <div className="metric-card glass-panel">
          <h3>SEO_META</h3>
          <div className={`metric-value ${seo.standard_meta?.title ? "text-success" : "text-warning"}`}>
            {seo.standard_meta?.title ? "OK" : "MISS"}
          </div>
          <p className="metric-sub">
            og:{Object.keys(seo.social_meta?.og || {}).length} | tw:{Object.keys(seo.social_meta?.twitter || {}).length}
          </p>
        </div>

        <div className="metric-card glass-panel">
          <h3>COOKIES</h3>
          <div className={`metric-value ${(sec.cookies?.issues?.length || 0) === 0 ? "text-success" : "text-warning"}`}>
            {sec.cookies?.issues?.length || 0}
          </div>
          <p className="metric-sub">{sec.cookies?.total || 0} analyzed</p>
        </div>

        <div className="metric-card glass-panel">
          <h3>A11Y_ISSUES</h3>
          <div className={`metric-value ${(a11ySummary.errors || 0) > 0 ? "text-danger" : (a11ySummary.warnings || 0) > 0 ? "text-warning" : "text-success"}`}>
            {a11ySummary.total_issues || 0}
          </div>
          <p className="metric-sub">err:{a11ySummary.errors || 0} warn:{a11ySummary.warnings || 0}</p>
        </div>
      </div>

      {warnings.length > 0 && (
        <div className="glass-panel warnings-panel">
          <h3>ISSUES [{warnings.length}]</h3>
          <div className="terminal-action-panel" style={{ marginBottom: "0.8rem" }}>
            <div className="terminal-action-line"><span>priority_1</span><strong>{warnings[0] || "none"}</strong></div>
            <div className="terminal-action-line"><span>ssl_action</span><strong>{sec.ssl?.valid ? "monitor cert expiration" : "fix certificate chain"}</strong></div>
            <div className="terminal-action-line"><span>crawl_action</span><strong>{(sit.broken_links?.length || 0) > 0 ? "repair broken links" : "stable"}</strong></div>
          </div>
          <ul className="warnings-list">
            {warnings.map((w, i) => (
              <li key={i} className="warning-item">
                <span className="warning-dot"></span>
                {w}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
