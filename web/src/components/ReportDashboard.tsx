"use client";

import { useEffect, useState } from "react";
import "./ReportDashboard.css";
import OverviewTab from "./tabs/OverviewTab";
import SeoTab from "./tabs/SeoTab";
import SecurityTab from "./tabs/SecurityTab";
import SitemapTab from "./tabs/SitemapTab";
import AccessibilityTab from "./tabs/AccessibilityTab";
import StructureTab from "./tabs/StructureTab";

interface ScanReport {
  target_url: string;
  scan_timestamp: string;
  seo: any;
  sitemap: any;
  security: any;
  accessibility: any;
  structure: any;
}

type TabKey = "overview" | "seo" | "security" | "sitemap" | "accessibility" | "structure";

function computeHealthScore(r: ScanReport): number {
  let s = 100;
  if (!r.seo.standard_meta?.title) s -= 10;
  if (!r.seo.standard_meta?.description) s -= 10;
  if (r.seo.headings?.missing_h1) s -= 8;
  if (r.seo.headings?.multiple_h1) s -= 5;
  if ((r.seo.image_alts?.missing_alt || 0) > 0) s -= Math.min(r.seo.image_alts.missing_alt * 2, 10);
  if (!r.seo.canonical) s -= 5;
  if (!r.seo.robots_txt?.presence) s -= 3;
  s -= (r.security.headers?.missing_headers?.length || 0) * 3;
  if (!r.security.ssl?.valid) s -= 15;
  if (r.security.ssl?.is_expired) s -= 10;
  if ((r.security.cookies?.issues?.length || 0) > 0) s -= Math.min(r.security.cookies.issues.length * 2, 10);
  if ((r.security.sensitive_paths_found?.length || 0) > 0) s -= r.security.sensitive_paths_found.length * 5;
  if ((r.sitemap.broken_links?.length || 0) > 0) s -= Math.min(r.sitemap.broken_links.length * 2, 15);
  const a11y = r.accessibility?.main_page?.summary;
  if (a11y) s -= Math.min((a11y.errors || 0) * 3 + (a11y.warnings || 0), 15);
  const struct = r.structure?.main_page?.summary;
  if (struct) s -= Math.min((struct.total_issues || 0) * 2, 10);
  return Math.max(0, Math.min(100, s));
}

function collectWarnings(r: ScanReport): string[] {
  const w: string[] = [];
  if (!r.seo.standard_meta?.title) w.push("Page title is missing");
  if (!r.seo.standard_meta?.description) w.push("Meta description is missing");
  if (r.seo.headings?.missing_h1) w.push("No H1 tag found");
  if (r.seo.headings?.multiple_h1) w.push("Multiple H1 tags");
  if ((r.seo.image_alts?.missing_alt || 0) > 0) w.push(`${r.seo.image_alts.missing_alt} image(s) missing alt`);
  if (!r.seo.canonical) w.push("No canonical URL");
  if (!r.seo.robots_txt?.presence) w.push("robots.txt not found");
  if (!r.security.ssl?.valid) w.push("SSL invalid/missing");
  if (r.security.ssl?.is_expired) w.push("SSL expired");
  if ((r.security.headers?.missing_headers?.length || 0) > 0) w.push(`${r.security.headers.missing_headers.length} security header(s) missing`);
  if ((r.security.sensitive_paths_found?.length || 0) > 0) w.push(`${r.security.sensitive_paths_found.length} sensitive path(s) exposed`);
  if ((r.security.cookies?.issues?.length || 0) > 0) w.push(`${r.security.cookies.issues.length} cookie issue(s)`);
  if ((r.sitemap.broken_links?.length || 0) > 0) w.push(`${r.sitemap.broken_links.length} broken link(s)`);
  const a11y = r.accessibility?.main_page;
  if (a11y?.language && !a11y.language.has_lang) w.push("Missing html lang attribute");
  if (a11y?.language && !a11y.language.has_charset) w.push("Missing charset declaration");
  if ((a11y?.aria?.missing_labels || 0) > 0) w.push(`${a11y.aria.missing_labels} element(s) missing ARIA label`);
  const struct = r.structure?.main_page;
  if (struct?.summary?.total_issues > 0) w.push(`${struct.summary.total_issues} structure issue(s) found`);
  if (struct?.semantic && !struct.semantic.has_main) w.push("Missing <main> landmark");
  return w;
}

export default function ReportDashboard({ scanId }: { scanId: string }) {
  const [status, setStatus] = useState("Connecting to engine...");
  const [isCompleted, setIsCompleted] = useState(false);
  const [reportData, setReportData] = useState<ScanReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabKey>("overview");

  const handleQuickScan = async (targetUrl: string) => {
    try {
      const res = await fetch("http://localhost:8000/api/scan", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: targetUrl })
      });
      if (res.ok) {
        const data = await res.json();
        window.location.href = `/reports/${data.scan_id}`;
      }
    } catch (err) {
      console.error("Scan trigger failed", err);
    }
  };

  useEffect(() => {
    const es = new EventSource(`http://localhost:8000/api/progress/${scanId}`);
    es.onmessage = (event) => {
      const s = event.data;
      setStatus(s);
      if (s.includes("Completed")) { setIsCompleted(true); es.close(); }
      else if (s.includes("Error")) { setError(s); es.close(); }
    };
    es.onerror = () => { setIsCompleted(true); es.close(); };
    return () => { es.close(); };
  }, [scanId]);

  useEffect(() => {
    if (isCompleted && !error) {
      fetch(`http://localhost:8000/api/reports/${scanId}`)
        .then(r => r.json())
        .then(data => {
          if (data.status === "In Progress") setIsCompleted(false);
          else setReportData(data);
        })
        .catch(() => setError("Failed to load report data"));
    }
  }, [isCompleted, error, scanId]);

  if (error) {
    return (
      <div className="dashboard-container">
        <div className="glass-panel error-panel"><h2>SCAN_FAILED</h2><p>{error}</p></div>
      </div>
    );
  }

  if (!isCompleted || !reportData) {
    return (
      <div className="dashboard-container loading-state">
        <div className="glass-panel loader-panel">
          <div className="pulse-ring"></div>
          <h2>ANALYSIS_IN_PROGRESS</h2>
          <p className="status-text">{status}</p>
          <div className="progress-bar"><div className="progress-fill indeterminate"></div></div>
        </div>
      </div>
    );
  }

  const healthScore = computeHealthScore(reportData);
  const warnings = collectWarnings(reportData);

  const tabs: { key: TabKey; label: string }[] = [
    { key: "overview", label: "Overview" },
    { key: "seo", label: "SEO" },
    { key: "security", label: "Security" },
    { key: "sitemap", label: "Sitemap" },
    { key: "accessibility", label: "A11y" },
    { key: "structure", label: "Structure" },
  ];

  return (
    <div className="dashboard-container fade-in">
      <header className="dashboard-header glass-panel">
        <div>
          <h1>// {reportData.target_url}</h1>
          <p className="scan-time">{new Date(reportData.scan_timestamp).toLocaleString()}</p>
        </div>
        <div className="header-actions">
          <a href={`http://localhost:8000/api/export/md/${scanId}`} className="btn-secondary" download>.md</a>
          <a href={`http://localhost:8000/api/export/jsonc/${scanId}`} className="btn-primary" download>.jsonc</a>
        </div>
      </header>

      <div className="tab-nav glass-panel">
        {tabs.map(t => (
          <button
            key={t.key}
            className={`tab-btn ${activeTab === t.key ? "active" : ""}`}
            onClick={() => setActiveTab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="tab-content">
        {activeTab === "overview" && (
          <OverviewTab
            seo={reportData.seo}
            security={reportData.security}
            sitemap={reportData.sitemap}
            accessibility={reportData.accessibility}
            healthScore={healthScore}
            warnings={warnings}
          />
        )}
        {activeTab === "seo" && <SeoTab seo={reportData.seo} />}
        {activeTab === "security" && <SecurityTab security={reportData.security} />}
        {activeTab === "sitemap" && <SitemapTab sitemap={reportData.sitemap} onQuickScan={handleQuickScan} />}
        {activeTab === "accessibility" && <AccessibilityTab accessibility={reportData.accessibility} />}
        {activeTab === "structure" && <StructureTab structure={reportData.structure} />}
      </div>
    </div>
  );
}
