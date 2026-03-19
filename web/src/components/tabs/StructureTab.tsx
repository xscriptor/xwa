"use client";

import { useMemo, useState } from "react";
import type { StructureData, StructurePage } from "@/lib/types";

function DOMTreeNode({ node, depth = 0 }: { node: Record<string, unknown>; depth?: number }) {
  const [open, setOpen] = useState(depth < 2);
  if (!node) return null;

  const children = (node.children as Record<string, unknown>[] | undefined) || [];
  const issues = (node.issues as Array<{ type: string; message: string }> | undefined) || [];
  const attrs = (node.attrs as Record<string, unknown> | undefined) || {};
  const hasChildren = children.length > 0;
  const hasIssues = issues.length > 0;

  // Build attribute string
  const attrParts: string[] = [];
  if (attrs.id) attrParts.push(`id="${String(attrs.id)}"`);
  if (attrs.class) attrParts.push(`class="${String(attrs.class)}"`);
  if (attrs.role) attrParts.push(`role="${String(attrs.role)}"`);
  if (attrs.aria) {
    Object.entries((attrs.aria as Record<string, string>) || {}).forEach(([k, v]) => {
      attrParts.push(`${k}="${String(v)}"`);
    });
  }

  return (
    <div className="struct-node" style={{ paddingLeft: `${depth * 0.8}rem` }}>
      <div
        className={`struct-label ${hasChildren ? "expandable" : ""} ${hasIssues ? "has-issues" : ""}`}
        onClick={() => hasChildren && setOpen(!open)}
      >
        {hasChildren && <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>}
        <span className="struct-tag">&lt;{String(node.tag || "node")}</span>
        {attrParts.length > 0 && <span className="struct-attrs">{attrParts.join(' ')}</span>}
        <span className="struct-tag">&gt;</span>
        {typeof node.text === "string" && node.text.length > 0 && (
          <span className="struct-text">{node.text}</span>
        )}
        {hasIssues && <span className="struct-issue-count">{issues.length}</span>}
      </div>

      {hasIssues && open && (
        <div className="struct-issues" style={{ paddingLeft: `${(depth + 1) * 0.8}rem` }}>
          {issues.map((issue, i) => (
            <div key={i} className="struct-issue-item">
              <span className={`status-badge ${issue.type === 'missing_alt' || issue.type === 'missing_label' ? 'badge-4xx' : 'badge-3xx'}`}>
                {issue.type.toUpperCase()}
              </span>
              <span>{issue.message}</span>
            </div>
          ))}
        </div>
      )}

      {open && hasChildren && (
        <div className="struct-children">
          {children.map((child, i) => (
            <DOMTreeNode key={i} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

function StructurePageCard({ page, defaultOpen = false }: { page: StructurePage; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  const issueCount = page.summary?.total_issues || 0;

  const issueBuckets = useMemo(() => {
    const buckets: Record<string, number> = {};
    (page.issues || []).forEach((i) => {
      const k = i.type || "unknown";
      buckets[k] = (buckets[k] || 0) + 1;
    });
    return Object.entries(buckets).sort((a, b) => b[1] - a[1]);
  }, [page.issues]);

  const maxBucket = issueBuckets[0]?.[1] || 1;

  return (
    <div className="details-card glass-panel">
      <div className="a11y-page-header" onClick={() => setOpen(!open)} style={{ cursor: "pointer" }}>
        <div className="a11y-page-title">
          <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>
          <span className="url-cell" style={{ fontSize: "0.85rem" }}>{page.url || "Main page"}</span>
        </div>
        <div className="a11y-page-badges">
          <span className={`status-badge ${issueCount > 0 ? "badge-3xx" : "badge-2xx"}`}>ISS:{issueCount}</span>
          <span className="status-badge badge-2xx">IDS:{page.summary?.total_ids || 0}</span>
          <span className="status-badge badge-2xx">ARIA:{page.summary?.total_aria || 0}</span>
        </div>
      </div>

      {open && (
        <div className="details-content">
          <div className="detail-item"><span className="label">Semantic Ratio:</span><span className="value">{page.semantic?.semantic_ratio || 0}%</span></div>
          <div className="detail-item"><span className="label">Div Count:</span><span className="value">{page.semantic?.total_divs || 0}</span></div>
          <div className="detail-item"><span className="label">Semantic Tags:</span><span className="value">{page.semantic?.total_semantic_tags || 0}</span></div>
          <div className="detail-item"><span className="label">Words:</span><span className="value">{page.clean_text?.total_words || 0}</span></div>

          <div className="detail-item vertical mt-4">
            <span className="label">LANDMARKS</span>
            <div className="status-badges-row">
              <span className={`status-badge ${page.semantic?.has_main ? "badge-2xx" : "badge-4xx"}`}>MAIN</span>
              <span className={`status-badge ${page.semantic?.has_nav ? "badge-2xx" : "badge-4xx"}`}>NAV</span>
              <span className={`status-badge ${page.semantic?.has_header ? "badge-2xx" : "badge-4xx"}`}>HEADER</span>
              <span className={`status-badge ${page.semantic?.has_footer ? "badge-2xx" : "badge-4xx"}`}>FOOTER</span>
            </div>
          </div>

          {issueBuckets.length > 0 && (
            <div className="detail-item vertical mt-4">
              <span className="label">ISSUE_DISTRIBUTION</span>
              <div className="struct-issue-bars">
                {issueBuckets.map(([name, count]) => (
                  <div className="struct-issue-bar-row" key={name}>
                    <span className="struct-issue-bar-label">{name}</span>
                    <div className="struct-issue-bar-track">
                      <div className="struct-issue-bar-fill" style={{ width: `${(count / maxBucket) * 100}%` }} />
                    </div>
                    <span className="struct-issue-bar-count">{count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {(page.issues || []).length > 0 && (
            <div className="table-responsive mt-4" style={{ maxHeight: "220px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TYPE</th><th>PATH</th><th>MESSAGE</th></tr></thead>
                <tbody>
                  {(page.issues || []).map((issue, i) => (
                    <tr key={i}>
                      <td><span className="status-badge badge-3xx">{issue.type}</span></td>
                      <td className="url-cell">{issue.path || "-"}</td>
                      <td className="url-cell">{issue.message}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {(page.ids || []).length > 0 && (
            <div className="table-responsive mt-4" style={{ maxHeight: "180px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TAG</th><th>ID</th><th>TEXT</th></tr></thead>
                <tbody>
                  {(page.ids || []).map((idNode, i) => (
                    <tr key={i}>
                      <td>{idNode.tag || "-"}</td>
                      <td className="text-accent">{idNode.id || "-"}</td>
                      <td className="url-cell">{idNode.text || "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {(page.aria_elements || []).length > 0 && (
            <div className="table-responsive mt-4" style={{ maxHeight: "180px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TAG</th><th>ARIA</th><th>TEXT</th></tr></thead>
                <tbody>
                  {(page.aria_elements || []).map((ariaNode, i) => (
                    <tr key={i}>
                      <td>{ariaNode.tag || "-"}</td>
                      <td className="url-cell text-accent">{Object.entries(ariaNode.aria || {}).map(([k, v]) => `${k}=${v}`).join(" ") || "-"}</td>
                      <td className="url-cell">{ariaNode.text || "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {(page.clean_text?.sections || []).length > 0 && (
            <div className="table-responsive mt-4" style={{ maxHeight: "180px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TAG</th><th>ID</th><th>CONTENT</th></tr></thead>
                <tbody>
                  {(page.clean_text?.sections || []).map((s, i) => (
                    <tr key={i}>
                      <td>{s.tag || "-"}</td>
                      <td className="text-accent">{s.id || "-"}</td>
                      <td className="url-cell">{s.text || "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {page.tree && (
            <div className="detail-item vertical mt-4">
              <span className="label">DOM_TREE</span>
              <div className="struct-tree" style={{ maxHeight: "260px", overflowY: "auto", border: "1px solid rgba(57,255,20,0.12)", padding: "0.5rem" }}>
                <DOMTreeNode node={page.tree as Record<string, unknown>} depth={0} />
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default function StructureTab({ structure }: { structure: StructureData }) {
  const data = structure?.main_page;
  const perUrl = structure?.per_url || [];

  if (!data) {
    return (
      <div className="details-card glass-panel">
        <h2>STRUCTURE</h2>
        <span className="text-muted">No structure data available</span>
      </div>
    );
  }

  const semantic = data.semantic || {};
  const cleanText = data.clean_text || {};
  const allIds = data.ids || [];
  const allAria = data.aria_elements || [];
  const issues = data.issues || [];
  const summary = data.summary || {};
  const viewportNodes = [
    ...(data.ids || []).slice(0, 10).map((idNode) => idNode.id || idNode.tag || "id"),
    ...(data.aria_elements || []).slice(0, 8).map((ariaNode) => ariaNode.tag || "aria"),
  ].slice(0, 20);

  return (
    <div className="tab-sections">
      <div className="protocol-hero-grid">
        <div className="details-card glass-panel protocol-hero-main">
          <h2>NODE_VIEWPORT</h2>
          <p className="mono-subline">Canvas-like projection of semantic and accessible nodes in the DOM graph.</p>
          <div className="node-viewport-grid">
            {viewportNodes.length > 0 ? viewportNodes.map((node, idx) => (
              <span key={`${node}-${idx}`} className="node-cell">{node}</span>
            )) : (
              <span className="node-cell">no_nodes</span>
            )}
          </div>
        </div>

        <div className="details-card glass-panel protocol-hero-rail">
          <h3>DASHBOARD_HIERARCHY</h3>
          <div className="mini-stat-grid">
            <div className="mini-stat"><span className="label">semantic ratio</span><span className="value">{semantic.semantic_ratio || 0}%</span></div>
            <div className="mini-stat"><span className="label">issues</span><span className="value">{issues.length}</span></div>
            <div className="mini-stat"><span className="label">ids</span><span className="value">{summary.total_ids || 0}</span></div>
            <div className="mini-stat"><span className="label">aria</span><span className="value">{summary.total_aria || 0}</span></div>
          </div>
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>SEMANTIC_OVERVIEW</h2>
        <div className="crawl-stats-row">
          <div className="crawl-stat">
            <span className={`crawl-stat-value ${issues.length > 0 ? "text-warning" : "text-success"}`}>{issues.length}</span>
            <span className="crawl-stat-label">issues</span>
          </div>
          <div className="crawl-stat">
            <span className="crawl-stat-value text-accent">{summary.total_ids || 0}</span>
            <span className="crawl-stat-label">ids</span>
          </div>
          <div className="crawl-stat">
            <span className="crawl-stat-value text-accent">{summary.total_aria || 0}</span>
            <span className="crawl-stat-label">aria</span>
          </div>
          <div className="crawl-stat">
            <span className="crawl-stat-value text-accent">{semantic.semantic_ratio || 0}%</span>
            <span className="crawl-stat-label">semantic</span>
          </div>
        </div>
        <div className="details-content">
          {[
            ["<main>", semantic.has_main],
            ["<nav>", semantic.has_nav],
            ["<header>", semantic.has_header],
            ["<footer>", semantic.has_footer],
          ].map(([tag, present]) => (
            <div className="detail-item" key={String(tag)}>
              <span className="label">{String(tag)}:</span>
              <span className={`value ${present ? "text-success" : "text-danger"}`}>
                {present ? "PRESENT" : "MISSING"}
              </span>
            </div>
          ))}
          <div className="detail-item">
            <span className="label">div_count:</span>
            <span className="value">{semantic.total_divs || 0}</span>
          </div>
          <div className="detail-item">
            <span className="label">semantic_tags:</span>
            <span className="value">{semantic.total_semantic_tags || 0}</span>
          </div>

          <div className="detail-item vertical mt-4">
            <span className="label">LANDMARK_COVERAGE</span>
            <div className="status-badges-row">
              <span className={`status-badge ${semantic.has_main ? "badge-2xx" : "badge-4xx"}`}>MAIN</span>
              <span className={`status-badge ${semantic.has_nav ? "badge-2xx" : "badge-4xx"}`}>NAV</span>
              <span className={`status-badge ${semantic.has_header ? "badge-2xx" : "badge-4xx"}`}>HEADER</span>
              <span className={`status-badge ${semantic.has_footer ? "badge-2xx" : "badge-4xx"}`}>FOOTER</span>
            </div>
          </div>
        </div>
      </div>

      {issues.length > 0 && (
        <div className="details-card glass-panel">
          <h2>BAD_PRACTICES [{issues.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "350px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>TYPE</th><th>PATH</th><th>MESSAGE</th></tr></thead>
              <tbody>
                {issues.map((issue, i) => (
                  <tr key={i} className="row-error">
                    <td>
                      <span className={`status-badge ${issue.type.includes('missing') ? 'badge-4xx' : 'badge-3xx'}`}>
                        {issue.type.toUpperCase().replace('_', ' ')}
                      </span>
                    </td>
                    <td className="url-cell">{issue.path}</td>
                    <td className="text-danger">{issue.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {data.tree && (
        <div className="details-card glass-panel">
          <h2>DOM_TREE</h2>
          <div className="struct-tree">
            <DOMTreeNode node={data.tree as Record<string, unknown>} depth={0} />
          </div>
        </div>
      )}

      {allIds.length > 0 && (
        <div className="details-card glass-panel">
          <h2>ALL_IDS [{allIds.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "300px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>TAG</th><th>ID</th><th>TEXT</th></tr></thead>
              <tbody>
                {allIds.map((el, i) => (
                  <tr key={i}>
                    <td className="font-medium">&lt;{el.tag}&gt;</td>
                    <td className="text-accent">{el.id}</td>
                    <td className="text-muted">{el.text || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {allAria.length > 0 && (
        <div className="details-card glass-panel">
          <h2>ARIA_ELEMENTS [{allAria.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "300px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>TAG</th><th>ARIA</th><th>TEXT</th></tr></thead>
              <tbody>
                {allAria.map((el, i) => (
                  <tr key={i}>
                    <td className="font-medium">&lt;{el.tag}&gt;{el.id ? `#${el.id}` : ""}</td>
                    <td className="url-cell text-accent">
                      {Object.entries(el.aria || {}).map(([k, v]) => `${k}="${v}"`).join(' ')}
                    </td>
                    <td className="text-muted">{el.text || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <div className="details-card glass-panel">
        <h2>CLEAN_TEXT words:{cleanText.total_words || 0} chars:{cleanText.total_chars || 0}</h2>
        <div className="details-content">
          {(cleanText.sections || []).length > 0 && (
            <div className="table-responsive" style={{ maxHeight: "350px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TAG</th><th>ID</th><th>TEXT_CONTENT</th></tr></thead>
                <tbody>
                  {(cleanText.sections || []).map((s, i) => (
                    <tr key={i}>
                      <td className="font-medium">&lt;{s.tag}&gt;</td>
                      <td className="text-accent">{s.id || "-"}</td>
                      <td className="url-cell">{s.text}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>

      <div className="details-card glass-panel">
        <h2>STRUCTURE_PER_LINK [{perUrl.length}]</h2>
        {perUrl.length > 0 ? (
          <>
            <span className="text-muted">Expande cada URL para ver toda la información estructural discriminada por enlace.</span>
            <div style={{ marginTop: "1rem" }}>
              {perUrl.map((page, i) => (
                <StructurePageCard key={`${page.url || "page"}-${i}`} page={page} defaultOpen={i === 0} />
              ))}
            </div>
          </>
        ) : (
          <span className="text-muted">No hay análisis estructural por enlace en este reporte.</span>
        )}
      </div>
    </div>
  );
}
