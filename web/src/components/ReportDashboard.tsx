"use client";

import { useEffect, useState } from "react";
import { getReport, openProgressStream, triggerScan } from "@/lib/api";
import type { ScanReport } from "@/lib/types";
import { type ReportTabKey } from "@/lib/navigation";
import "./ReportDashboard.css";
import OverviewTab from "./tabs/OverviewTab";
import SeoTab from "./tabs/SeoTab";
import SecurityTab from "./tabs/SecurityTab";
import SitemapTab from "./tabs/SitemapTab";
import AccessibilityTab from "./tabs/AccessibilityTab";
import StructureTab from "./tabs/StructureTab";
import PerformanceTab from "./tabs/PerformanceTab";
import ContentTab from "./tabs/ContentTab";
import LinksTab from "./tabs/LinksTab";
import NetworkTab from "./tabs/NetworkTab";
import ComplianceTab from "./tabs/ComplianceTab";
import SocialTab from "./tabs/SocialTab";
import StackTab from "./tabs/StackTab";

type TabKey = ReportTabKey;

function computeHealthScore(r: ScanReport): number {
  let s = 100;
  const missingAlt = r.seo.image_alts?.missing_alt || 0;
  const cookieIssues = r.security.cookies?.issues?.length || 0;
  const exposedPaths = r.security.sensitive_paths_found?.length || 0;
  const brokenLinks = r.sitemap.broken_links?.length || 0;
  if (!r.seo.standard_meta?.title) s -= 10;
  if (!r.seo.standard_meta?.description) s -= 10;
  if (r.seo.headings?.missing_h1) s -= 8;
  if (r.seo.headings?.multiple_h1) s -= 5;
  if (missingAlt > 0) s -= Math.min(missingAlt * 2, 10);
  if (!r.seo.canonical) s -= 5;
  if (!r.seo.robots_txt?.presence) s -= 3;
  s -= (r.security.headers?.missing_headers?.length || 0) * 3;
  if (!r.security.ssl?.valid) s -= 15;
  if (r.security.ssl?.is_expired) s -= 10;
  if (cookieIssues > 0) s -= Math.min(cookieIssues * 2, 10);
  if (exposedPaths > 0) s -= exposedPaths * 5;
  if (brokenLinks > 0) s -= Math.min(brokenLinks * 2, 15);
  const a11y = r.accessibility?.main_page?.summary;
  if (a11y) s -= Math.min((a11y.errors || 0) * 3 + (a11y.warnings || 0), 15);
  const struct = r.structure?.main_page?.summary;
  if (struct) s -= Math.min((struct.total_issues || 0) * 2, 10);
  return Math.max(0, Math.min(100, s));
}

function collectWarnings(r: ScanReport): string[] {
  const w: string[] = [];
  const missingAlt = r.seo.image_alts?.missing_alt || 0;
  const missingHeaders = r.security.headers?.missing_headers?.length || 0;
  const exposedPaths = r.security.sensitive_paths_found?.length || 0;
  const cookieIssues = r.security.cookies?.issues?.length || 0;
  const brokenLinks = r.sitemap.broken_links?.length || 0;
  if (!r.seo.standard_meta?.title) w.push("Page title is missing");
  if (!r.seo.standard_meta?.description) w.push("Meta description is missing");
  if (r.seo.headings?.missing_h1) w.push("No H1 tag found");
  if (r.seo.headings?.multiple_h1) w.push("Multiple H1 tags");
  if (missingAlt > 0) w.push(`${missingAlt} image(s) missing alt`);
  if (!r.seo.canonical) w.push("No canonical URL");
  if (!r.seo.robots_txt?.presence) w.push("robots.txt not found");
  if (!r.security.ssl?.valid) w.push("SSL invalid/missing");
  if (r.security.ssl?.is_expired) w.push("SSL expired");
  if (missingHeaders > 0) w.push(`${missingHeaders} security header(s) missing`);
  if (exposedPaths > 0) w.push(`${exposedPaths} sensitive path(s) exposed`);
  if (cookieIssues > 0) w.push(`${cookieIssues} cookie issue(s)`);
  if (brokenLinks > 0) w.push(`${brokenLinks} broken link(s)`);
  const a11y = r.accessibility?.main_page;
  const missingAriaLabels = a11y?.aria?.missing_labels || 0;
  if (a11y?.language && !a11y.language.has_lang) w.push("Missing html lang attribute");
  if (a11y?.language && !a11y.language.has_charset) w.push("Missing charset declaration");
  if (missingAriaLabels > 0) w.push(`${missingAriaLabels} element(s) missing ARIA label`);
  const struct = r.structure?.main_page;
  const structIssues = struct?.summary?.total_issues || 0;
  if (structIssues > 0) w.push(`${structIssues} structure issue(s) found`);
  if (struct?.semantic && !struct.semantic.has_main) w.push("Missing <main> landmark");
  return w;
}

export default function ReportDashboard({
  scanId,
  activeTab,
}: {
  scanId: string;
  activeTab: TabKey;
}) {

  const [status, setStatus] = useState("Connecting to engine...");
  const [isCompleted, setIsCompleted] = useState(false);
  const [reportData, setReportData] = useState<ScanReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  // First, try to load any existing stored report so revisits don't wait on SSE.
  useEffect(() => {
    let cancelled = false;
    setError(null);
    getReport<ScanReport>(scanId)
      .then((data) => {
        if (cancelled) return;
        if ("status" in data && data.status === "In Progress") {
          setStatus(data.current_step || "In Progress");
          setIsCompleted(false);
        } else {
          setReportData(data as ScanReport);
          setIsCompleted(true);
        }
      })
      .catch(() => {
        if (!cancelled) setError("Failed to load report data");
      });
    return () => {
      cancelled = true;
    };
  }, [scanId]);

  const handleQuickScan = async (targetUrl: string) => {
    try {
      const data = await triggerScan(targetUrl);
      window.location.href = `/reports/${data.scan_id}/overview`;
    } catch (err) {
      console.error("Scan trigger failed", err);
    }
  };

  useEffect(() => {
    if (reportData) return;

    const es = openProgressStream(scanId);
    es.onmessage = (event) => {
      const s = event.data;
      setStatus(s);
      if (s.includes("Completed") || s === "Unknown") { setIsCompleted(true); es.close(); }
      else if (s.includes("Error")) { setError(s); es.close(); }
    };
    es.onerror = () => { setIsCompleted(true); es.close(); };
    return () => { es.close(); };
  }, [scanId, reportData]);

  useEffect(() => {
    if (reportData || !isCompleted || error) return;

    if (isCompleted && !error) {
      getReport<ScanReport>(scanId)
        .then(data => {
          if ("status" in data) {
            if (data.status === "In Progress") setIsCompleted(false);
          } else {
            setReportData(data as ScanReport);
          }
        })
        .catch(() => setError("Failed to load report data"));
    }
  }, [isCompleted, error, scanId, reportData]);

  if (error) {
    return (
      <div className="dashboard-layout">
        <div className="glass-panel error-panel"><h2>SCAN_FAILED</h2><p>{error}</p></div>
      </div>
    );
  }

  if (!isCompleted || !reportData) {
    return (
      <div className="dashboard-layout">
        <div className="loading-state">
          <div className="glass-panel loader-panel">
            <div className="pulse-ring"><span className="loader-x">X</span></div>
            <h2 style={{ fontFamily: 'var(--font-headline)', fontWeight: 700, fontSize: '1rem', textTransform: 'uppercase', letterSpacing: '-0.02em' }}>ANALYSIS_IN_PROGRESS</h2>
            <p className="status-text">{status}</p>
            <div className="progress-bar"><div className="progress-fill indeterminate"></div></div>
          </div>
        </div>
      </div>
    );
  }

  const healthScore = computeHealthScore(reportData);
  const warnings = collectWarnings(reportData);

  return (
    <div className="dashboard-layout">
      <div className="dashboard-main">
        <div className="scan-lines-overlay"></div>

        {/* Header */}
        <header className="dashboard-header">
          <div>
            <div className="header-module">TARGET_MODULE: {activeTab.toUpperCase()}_CORE</div>
            <h1 className="header-title">
              {reportData.target_url.replace(/^https?:\/\//, '').replace(/\/$/, '')}
            </h1>
          </div>
          <div className="header-right">
            <div className="header-score">{healthScore}%</div>
            <div className="header-score-label">HEALTH_SCORE</div>
          </div>
        </header>

        {/* Tab Content */}
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
          {activeTab === "performance" && <PerformanceTab performance={reportData.performance} sitemap={reportData.sitemap} />}
          {activeTab === "content" && <ContentTab seo={reportData.seo} sitemap={reportData.sitemap} structure={reportData.structure} />}
          {activeTab === "links" && <LinksTab sitemap={reportData.sitemap} seo={reportData.seo} />}
          {activeTab === "network" && <NetworkTab sitemap={reportData.sitemap} security={reportData.security} />}
          {activeTab === "compliance" && <ComplianceTab security={reportData.security} accessibility={reportData.accessibility} seo={reportData.seo} />}
          {activeTab === "social" && <SocialTab seo={reportData.seo} />}
          {activeTab === "stack" && <StackTab security={reportData.security} sitemap={reportData.sitemap} />}
        </div>
      </div>
    </div>
  );
}
