"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { triggerScan } from "@/lib/api";
import ScanErrorIcon from "@/components/icons/scan/ScanErrorIcon";
import ScanRadarIcon from "@/components/icons/scan/ScanRadarIcon";
import "./ScanForm.css";

export default function ScanForm() {
  const [url, setUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    let targetUrl = url.trim();
    if (!targetUrl) return;

    if (!targetUrl.startsWith("http://") && !targetUrl.startsWith("https://")) {
      targetUrl = "https://" + targetUrl;
    }

    setLoading(true);
    try {
      const data = await triggerScan(targetUrl);
      if (data.scan_id) {
        router.push(`/reports/${data.scan_id}/overview`);
      }
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError(err.message || "Scan failed");
      } else {
        setError("Scan failed");
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <form className="scan-form" onSubmit={handleSubmit} id="scan-form">
      <div className="scan-bar">
        <span className="scan-bar-prefix">&gt;_</span>
        <input
          type="text"
          className="scan-input"
          placeholder="TARGET_URL://"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          disabled={loading}
          id="scan-url-input"
          autoComplete="off"
          spellCheck={false}
        />
        <button
          type="submit"
          className="scan-btn"
          disabled={loading || !url.trim()}
          id="scan-submit-btn"
        >
          {loading ? (
            <span className="scan-btn-loading">SCANNING...</span>
          ) : (
            <>
              <ScanRadarIcon />
              INITIATE_SCAN
            </>
          )}
        </button>
      </div>
      {error && (
        <div className="scan-error">
          <ScanErrorIcon />
          {error}
        </div>
      )}
    </form>
  );
}
