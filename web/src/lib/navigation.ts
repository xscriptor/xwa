export type ReportTabKey =
  | "overview"
  | "performance"
  | "seo"
  | "accessibility"
  | "structure"
  | "content"
  | "links"
  | "sitemap"
  | "security"
  | "network"
  | "compliance"
  | "social"
  | "stack";

export interface ReportSectionNavItem {
  key: ReportTabKey;
  label: string;
  icon: string;
}

export interface AsideNavLink {
  label: string;
  href: string;
  icon?: string;
  isActive?: boolean;
}

export const REPORT_SECTION_NAV: ReportSectionNavItem[] = [
  { key: "overview", label: "Overview", icon: "dashboard" },
  { key: "performance", label: "Performance", icon: "speed" },
  { key: "seo", label: "SEO", icon: "search" },
  { key: "accessibility", label: "A11y", icon: "accessibility_new" },
  { key: "structure", label: "Structure", icon: "account_tree" },
  { key: "content", label: "Content", icon: "description" },
  { key: "links", label: "Links", icon: "link" },
  { key: "sitemap", label: "Sitemaps", icon: "lan" },
  { key: "security", label: "Security", icon: "security" },
  { key: "network", label: "Network", icon: "lan" },
  { key: "compliance", label: "Compliance", icon: "fact_check" },
  { key: "social", label: "Social", icon: "share" },
  { key: "stack", label: "Stack", icon: "layers" },
];

export function isReportTabKey(value: string): value is ReportTabKey {
  return REPORT_SECTION_NAV.some((item) => item.key === value);
}

export function getReportSectionHref(scanId: string, tab: ReportTabKey): string {
  return `/reports/${scanId}/${tab}`;
}

export function getCurrentReportSection(pathname: string | null): ReportTabKey | null {
  if (!pathname) return null;
  const match = pathname.match(/^\/reports\/([^/]+)(?:\/([^/]+))?$/);
  if (!match) return null;
  const section = match[2];
  if (!section) return "overview";
  return isReportTabKey(section) ? section : null;
}

export function getCurrentReportId(pathname: string | null): string | null {
  if (!pathname) return null;
  const match = pathname.match(/^\/reports\/([^/]+)(?:\/[^/]+)?$/);
  return match ? match[1] : null;
}

export function getPageAsideLinks(pathname: string | null): AsideNavLink[] {
  if (!pathname) return [];

  const reportId = getCurrentReportId(pathname);

  if (reportId) {
    return [];
  }

  if (pathname === "/reports") {
    return [
      { label: "History Table", href: "#history-table", icon: "table" },
      { label: "Cleanup Tools", href: "#cleanup-tools", icon: "delete_sweep" },
    ];
  }

  if (pathname === "/") {
    return [
      { label: "Scanner", href: "#scanner", icon: "radar" },
      { label: "Quick Start", href: "#quick-start", icon: "play_circle" },
      { label: "Reports", href: "/reports", icon: "history" },
    ];
  }

  return [];
}
