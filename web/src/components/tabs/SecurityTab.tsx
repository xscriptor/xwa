"use client";

export default function SecurityTab({ security: sec }: { security: any }) {
  return (
    <div className="tab-sections">
      <div className="details-card glass-panel">
        <h2>// SSL_TLS_CERT</h2>
        <div className="details-content">
          {[
            ["Status", sec.ssl?.valid ? "VALID" : "INVALID", sec.ssl?.valid ? "text-success" : "text-danger"],
            ["Issuer", sec.ssl?.issuer || "Unknown", ""],
            ["Subject", sec.ssl?.subject || "N/A", ""],
            ["Expires", sec.ssl?.expires_on || "N/A", ""],
            ["Days_Left", sec.ssl?.days_remaining ?? "N/A", (sec.ssl?.days_remaining || 0) < 30 ? "text-warning" : "text-success"],
            ["Expired", sec.ssl?.is_expired ? "YES" : "NO", sec.ssl?.is_expired ? "text-danger" : "text-success"],
          ].map(([label, value, cls]: any) => (
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

      <div className="details-card glass-panel">
        <h2>// SECURITY_HEADERS</h2>
        <div className="details-content">
          <div className="detail-item vertical">
            <span className="label text-success">present [{Object.keys(sec.headers?.headers_present || {}).length}]</span>
            {Object.keys(sec.headers?.headers_present || {}).length > 0 ? (
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>Header</th><th>Value</th></tr></thead>
                  <tbody>
                    {Object.entries(sec.headers.headers_present).map(([k, v]: [string, any]) => (
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
                ? sec.headers.missing_headers.map((h: string) => <li key={h}>{h}</li>)
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
                    {Object.entries(sec.headers.leaked_server_info).map(([k, v]: [string, any]) => (
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
        <h2>// COOKIES [{sec.cookies?.total || 0}]</h2>
        <div className="details-content">
          {(sec.cookies?.cookies || []).length > 0 ? (
            <>
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>Name</th><th>Secure</th><th>HttpOnly</th><th>SameSite</th></tr></thead>
                  <tbody>
                    {sec.cookies.cookies.map((c: any, i: number) => (
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
                    {sec.cookies.issues.map((issue: string, i: number) => <li key={i}>{issue}</li>)}
                  </ul>
                </div>
              )}
            </>
          ) : <span className="text-muted">no cookies</span>}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>// SENSITIVE_PATHS</h2>
        <div className="details-content">
          {(sec.sensitive_paths_found || []).length > 0 ? (
            <div className="detail-item vertical">
              <span className="label text-danger">exposed [{sec.sensitive_paths_found.length}]</span>
              <ul className="value-list error-list">
                {sec.sensitive_paths_found.map((p: string) => <li key={p}>{p}</li>)}
              </ul>
            </div>
          ) : (
            <div className="alert-box alert-success">All paths returned 404. No exposures.</div>
          )}
        </div>
      </div>
    </div>
  );
}
