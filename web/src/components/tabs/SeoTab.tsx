"use client";

export default function SeoTab({ seo }: { seo: any }) {
  const structuredData = seo.structured_data || {};
  const linkTags = seo.link_tags || [];
  const headingDetails = seo.headings?.details || [];

  return (
    <div className="tab-sections">
      {/* Standard Meta */}
      <div className="details-card glass-panel">
        <h2>// STANDARD_META</h2>
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
          ].map(([label, value, warn]: any) => (
            <div className="detail-item" key={label}>
              <span className="label">{label}:</span>
              <span className={`value ${!value && warn ? "text-warning" : ""}`}>
                {value || (warn ? "MISSING" : <span className="text-muted">not_set</span>)}
              </span>
            </div>
          ))}
          <div className="detail-item">
            <span className="label">Keywords:</span>
            <span className="value">
              {seo.standard_meta?.keywords
                ? <div className="tag-chips">{seo.standard_meta.keywords.split(",").map((k: string, i: number) => (
                    <span key={i} className="tag-chip">{k.trim()}</span>
                  ))}</div>
                : <span className="text-muted">not_set</span>
              }
            </span>
          </div>
        </div>
      </div>

      {/* Social Meta - OpenGraph */}
      <div className="details-card glass-panel">
        <h2>// OPENGRAPH [{Object.keys(seo.social_meta?.og || {}).length}]</h2>
        <div className="details-content">
          {Object.keys(seo.social_meta?.og || {}).length > 0 ? (
            <div className="table-responsive">
              <table className="link-table">
                <thead><tr><th>Property</th><th>Value</th></tr></thead>
                <tbody>
                  {Object.entries(seo.social_meta.og).map(([key, val]: [string, any]) => (
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

      {/* Social Meta - Twitter */}
      <div className="details-card glass-panel">
        <h2>// TWITTER_CARDS [{Object.keys(seo.social_meta?.twitter || {}).length}]</h2>
        <div className="details-content">
          {Object.keys(seo.social_meta?.twitter || {}).length > 0 ? (
            <div className="table-responsive">
              <table className="link-table">
                <thead><tr><th>Property</th><th>Value</th></tr></thead>
                <tbody>
                  {Object.entries(seo.social_meta.twitter).map(([key, val]: [string, any]) => (
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

      {/* Structured Data */}
      <div className="details-card glass-panel">
        <h2>// STRUCTURED_DATA</h2>
        <div className="details-content">
          <div className="detail-item vertical">
            <span className="label">JSON-LD [{(structuredData.json_ld || []).length}]</span>
            {(structuredData.json_ld || []).length > 0 ? (
              <div className="table-responsive">
                <table className="link-table">
                  <thead><tr><th>@type</th><th>Name</th><th>Keys</th></tr></thead>
                  <tbody>
                    {structuredData.json_ld.map((item: any, i: number) => (
                      <tr key={i}>
                        <td className="font-medium text-accent">{item.type}</td>
                        <td className="url-cell">{item.name || "-"}</td>
                        <td className="text-muted">{(item.raw_keys || []).join(', ')}</td>
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
                {structuredData.microdata_types.map((t: string, i: number) => <li key={i}>{t}</li>)}
              </ul>
            ) : <span className="text-muted">no microdata</span>}
          </div>
        </div>
      </div>

      {/* Headings Detail */}
      <div className="details-card glass-panel">
        <h2>// HEADINGS h1:{seo.headings?.counts?.h1 || 0} h2:{seo.headings?.counts?.h2 || 0} h3:{seo.headings?.counts?.h3 || 0}</h2>
        <div className="details-content">
          {seo.headings?.missing_h1 && <div className="alert-box alert-danger">No H1 tag found on the page</div>}
          {seo.headings?.multiple_h1 && <div className="alert-box alert-warning">Multiple H1 tags detected ({seo.headings?.counts?.h1})</div>}
          {headingDetails.length > 0 && (
            <div className="table-responsive" style={{ maxHeight: "250px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TAG</th><th>TEXT</th><th>ID</th></tr></thead>
                <tbody>
                  {headingDetails.map((h: any, i: number) => (
                    <tr key={i}>
                      <td className="font-medium">{h.tag}</td>
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

      {/* Link Tags */}
      {linkTags.length > 0 && (
        <div className="details-card glass-panel">
          <h2>// LINK_TAGS [{linkTags.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "250px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>REL</th><th>HREF</th><th>TYPE</th><th>LANG</th></tr></thead>
              <tbody>
                {linkTags.map((lt: any, i: number) => (
                  <tr key={i}>
                    <td className="font-medium">{lt.rel}</td>
                    <td className="url-cell">{lt.href}</td>
                    <td className="text-muted">{lt.type || "-"}</td>
                    <td className="text-muted">{lt.hreflang || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Robots.txt */}
      <div className="details-card glass-panel">
        <h2>// ROBOTS_TXT</h2>
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
              <span className="label">Sitemaps [{seo.robots_txt.sitemaps.length}]</span>
              <ul className="value-list">
                {seo.robots_txt.sitemaps.map((s: string, i: number) => <li key={i}>{s}</li>)}
              </ul>
            </div>
          )}
          {(seo.robots_txt?.directives || []).length > 0 && (
            <div className="detail-item vertical mt-4">
              <span className="label">Directives [{seo.robots_txt.directives.length}]</span>
              <div className="struct-code-block">
                {seo.robots_txt.directives.map((d: string, i: number) => (
                  <div key={i} className="struct-code-line">{d}</div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Text Ratio */}
      <div className="details-card glass-panel">
        <h2>// TEXT_RATIO</h2>
        <div className="details-content">
          {[
            ["Ratio", `${seo.text_ratio?.text_to_html_ratio || 0}%`],
            ["Words", seo.text_ratio?.word_count || 0],
            ["HTML_Size", `${((seo.text_ratio?.html_size_bytes || 0) / 1024).toFixed(1)} KB`],
            ["Text_Size", `${((seo.text_ratio?.text_size_bytes || 0) / 1024).toFixed(1)} KB`],
          ].map(([label, value]: any) => (
            <div className="detail-item" key={label}>
              <span className="label">{label}:</span>
              <span className="value">{value}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
