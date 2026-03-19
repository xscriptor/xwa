"use client";

import { Suspense, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { deleteAllReports, deleteReport, listReports, ReportSummary } from "@/lib/api";
import { REPORT_SECTION_NAV, isReportTabKey } from "@/lib/navigation";
import "./reports.css";

function ReportsPageContent() {
  const [reports, setReports] = useState<ReportSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [bulkDeleting, setBulkDeleting] = useState(false);
  const searchParams = useSearchParams();
  const selectedTab = searchParams.get("tab");
  const selectedSection = selectedTab && isReportTabKey(selectedTab) ? selectedTab : "overview";

  useEffect(() => {
    listReports()
      .then(data => {
        setReports(data || []);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load reports", err);
        setLoading(false);
      });
  }, []);

  const handleDeleteOne = async (scanId: number) => {
    const confirmed = window.confirm(`Delete scan #${scanId} permanently? This cannot be undone.`);
    if (!confirmed) return;

    try {
      setBusyId(scanId);
      await deleteReport(scanId);
      setReports((prev) => prev.filter((r) => r.id !== scanId));
    } catch (err) {
      console.error("Failed to delete scan", err);
      window.alert("Failed to delete scan. Please try again.");
    } finally {
      setBusyId(null);
    }
  };

  const handleDeleteAll = async () => {
    const confirmed = window.confirm("Delete ALL scan history permanently? This cannot be undone.");
    if (!confirmed) return;

    try {
      setBulkDeleting(true);
      await deleteAllReports();
      setReports([]);
    } catch (err) {
      console.error("Failed to delete all scans", err);
      window.alert("Failed to delete all scans. Please try again.");
    } finally {
      setBulkDeleting(false);
    }
  };

  const currentTabLabel = REPORT_SECTION_NAV.find((item) => item.key === selectedSection)?.label;

  return (
    <div className="reports-page fade-in">
      <header className="page-header">
        <div id="cleanup-tools"></div>
        <h1 className="title">Scan History</h1>
        <p className="subtitle">Review your past website analyses</p>
        <div className="reports-toolbar">
          <button
            type="button"
            className="delete-all-btn"
            onClick={handleDeleteAll}
            disabled={bulkDeleting || reports.length === 0}
          >
            {bulkDeleting ? "Deleting all..." : "Delete All Scans"}
          </button>
        </div>
          {currentTabLabel && reports.length > 0 && (
          <p className="selected-section-note">
            Selected section: <strong>{currentTabLabel}</strong>. Open any report to jump directly to this tab.
          </p>
        )}
      </header>
      
      {loading ? (
        <div className="loader-container">
          <span className="loader-spin dark-loader"></span>
        </div>
      ) : reports.length === 0 ? (
        <div className="glass-panel empty-state">
          <p>No scans found. Start by running a new analysis.</p>
        </div>
      ) : (
          <div className="glass-panel table-container" id="history-table">
          <table className="reports-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Target URL</th>
                <th>Date</th>
                <th>Sitemap URLs</th>
                <th>Issues</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {reports.map((r) => (
                <tr key={r.id}>
                  <td>#{r.id}</td>
                  <td className="font-medium">{r.target_url}</td>
                  <td className="text-muted">{new Date(r.timestamp).toLocaleString()}</td>
                  <td>{r.urls_found}</td>
                  <td>
                    {r.broken_links_count > 0 ? (
                      <span className="badge danger">{r.broken_links_count} Broken Links</span>
                    ) : (
                      <span className="badge success">0 Broken</span>
                    )}
                    {!r.is_ssl_valid && (
                      <span className="badge warning">SSL Invalid</span>
                    )}
                  </td>
                  <td>
                    <div className="row-actions">
                      <a
                          href={`/reports/${r.id}/${selectedSection}`}
                        className="view-btn"
                      >
                        View Details
                      </a>
                      <button
                        type="button"
                        className="delete-btn"
                        onClick={() => handleDeleteOne(r.id)}
                        disabled={busyId === r.id || bulkDeleting}
                      >
                        {busyId === r.id ? "Deleting..." : "Delete"}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

export default function ReportsPage() {
  return (
    <Suspense fallback={<div className="reports-page fade-in"><div className="loader-container"><span className="loader-spin dark-loader"></span></div></div>}>
      <ReportsPageContent />
    </Suspense>
  );
}
