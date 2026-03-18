"use client";

import { useState } from "react";

function DOMTreeNode({ node, depth = 0 }: { node: any; depth?: number }) {
  const [open, setOpen] = useState(depth < 2);
  if (!node) return null;

  const hasChildren = (node.children || []).length > 0;
  const hasIssues = (node.issues || []).length > 0;
  const attrs = node.attrs || {};

  // Build attribute string
  const attrParts: string[] = [];
  if (attrs.id) attrParts.push(`id="${attrs.id}"`);
  if (attrs.class) attrParts.push(`class="${attrs.class}"`);
  if (attrs.role) attrParts.push(`role="${attrs.role}"`);
  if (attrs.aria) {
    Object.entries(attrs.aria).forEach(([k, v]: [string, any]) => {
      attrParts.push(`${k}="${v}"`);
    });
  }

  return (
    <div className="struct-node" style={{ paddingLeft: `${depth * 0.8}rem` }}>
      <div
        className={`struct-label ${hasChildren ? "expandable" : ""} ${hasIssues ? "has-issues" : ""}`}
        onClick={() => hasChildren && setOpen(!open)}
      >
        {hasChildren && <span className={`tree-arrow ${open ? "open" : ""}`}>&#9654;</span>}
        <span className="struct-tag">&lt;{node.tag}</span>
        {attrParts.length > 0 && <span className="struct-attrs">{attrParts.join(' ')}</span>}
        <span className="struct-tag">&gt;</span>
        {node.text && <span className="struct-text">{node.text}</span>}
        {hasIssues && <span className="struct-issue-count">{node.issues.length}</span>}
      </div>

      {hasIssues && open && (
        <div className="struct-issues" style={{ paddingLeft: `${(depth + 1) * 0.8}rem` }}>
          {node.issues.map((issue: any, i: number) => (
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
          {node.children.map((child: any, i: number) => (
            <DOMTreeNode key={i} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

export default function StructureTab({ structure }: { structure: any }) {
  const data = structure?.main_page;

  if (!data) {
    return (
      <div className="details-card glass-panel">
        <h2>// STRUCTURE</h2>
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

  return (
    <div className="tab-sections">
      {/* Semantic Overview */}
      <div className="details-card glass-panel">
        <h2>// SEMANTIC_OVERVIEW</h2>
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
          ].map(([tag, present]: any) => (
            <div className="detail-item" key={tag}>
              <span className="label">{tag}:</span>
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
        </div>
      </div>

      {/* Issues */}
      {issues.length > 0 && (
        <div className="details-card glass-panel">
          <h2>// BAD_PRACTICES [{issues.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "350px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>TYPE</th><th>PATH</th><th>MESSAGE</th></tr></thead>
              <tbody>
                {issues.map((issue: any, i: number) => (
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

      {/* DOM Tree */}
      {data.tree && (
        <div className="details-card glass-panel">
          <h2>// DOM_TREE</h2>
          <div className="struct-tree">
            <DOMTreeNode node={data.tree} depth={0} />
          </div>
        </div>
      )}

      {/* All IDs */}
      {allIds.length > 0 && (
        <div className="details-card glass-panel">
          <h2>// ALL_IDS [{allIds.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "300px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>TAG</th><th>ID</th><th>TEXT</th></tr></thead>
              <tbody>
                {allIds.map((el: any, i: number) => (
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

      {/* ARIA Elements */}
      {allAria.length > 0 && (
        <div className="details-card glass-panel">
          <h2>// ARIA_ELEMENTS [{allAria.length}]</h2>
          <div className="table-responsive" style={{ maxHeight: "300px", overflowY: "auto" }}>
            <table className="link-table">
              <thead><tr><th>TAG</th><th>ARIA</th><th>TEXT</th></tr></thead>
              <tbody>
                {allAria.map((el: any, i: number) => (
                  <tr key={i}>
                    <td className="font-medium">&lt;{el.tag}&gt;{el.id ? `#${el.id}` : ""}</td>
                    <td className="url-cell text-accent">
                      {Object.entries(el.aria || {}).map(([k, v]: [string, any]) => `${k}="${v}"`).join(' ')}
                    </td>
                    <td className="text-muted">{el.text || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Clean Text */}
      <div className="details-card glass-panel">
        <h2>// CLEAN_TEXT words:{cleanText.total_words || 0} chars:{cleanText.total_chars || 0}</h2>
        <div className="details-content">
          {(cleanText.sections || []).length > 0 && (
            <div className="table-responsive" style={{ maxHeight: "350px", overflowY: "auto" }}>
              <table className="link-table">
                <thead><tr><th>TAG</th><th>ID</th><th>TEXT_CONTENT</th></tr></thead>
                <tbody>
                  {cleanText.sections.map((s: any, i: number) => (
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
    </div>
  );
}
