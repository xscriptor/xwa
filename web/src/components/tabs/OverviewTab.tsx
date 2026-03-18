"use client";

interface OverviewTabProps {
  seo: any;
  security: any;
  sitemap: any;
  accessibility: any;
  healthScore: number;
  warnings: string[];
}

function getScoreColor(score: number): string {
  if (score >= 80) return "var(--success-color)";
  if (score >= 50) return "var(--warning-color)";
  return "var(--danger-color)";
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

  return (
    <div className="overview-layout">
      <div className="glass-panel score-card">
        <h3>HEALTH_SCORE</h3>
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
