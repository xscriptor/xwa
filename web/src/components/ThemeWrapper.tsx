"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { usePathname } from "next/navigation";
import ThemeProvider from "./ThemeProvider";
import AsideSymbolIcon from "@/components/icons/aside/AsideSymbolIcon";
import NavHamburgerIcon from "@/components/icons/navbar/NavHamburgerIcon";
import NavStatusIcon from "@/components/icons/navbar/NavStatusIcon";
import {
  getCurrentReportId,
  getCurrentReportSection,
  getPageAsideLinks,
  getReportSectionHref,
  REPORT_SECTION_NAV,
  type AsideNavLink,
} from "@/lib/navigation";

export default function ThemeWrapper({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const reportId = getCurrentReportId(pathname);
  const currentReportSection = getCurrentReportSection(pathname);
  const [dynamicAsideLinks, setDynamicAsideLinks] = useState<AsideNavLink[]>([]);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const navLinks = [
    { href: "/", label: "Scan", isActive: pathname === "/" },
    { href: "/reports", label: "Reports", isActive: pathname?.startsWith("/reports") },
    ...(reportId
      ? REPORT_SECTION_NAV.map((item) => ({
          href: getReportSectionHref(reportId, item.key),
          label: item.label,
          isActive: currentReportSection === item.key,
        }))
      : []),
  ];

  useEffect(() => {
    if (!reportId) return;

    const slugify = (txt: string) =>
      txt
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, "")
        .trim()
        .replace(/\s+/g, "-");

    const refresh = () => {
      const root = document.querySelector(".page-transition") as HTMLElement | null;
      if (!root) return;

      const headingNodes = Array.from(root.querySelectorAll(".tab-content h2, .tab-content h3"))
        .slice(0, 12) as HTMLElement[];

      const links = headingNodes
        .map<AsideNavLink | null>((node, index) => {
          const text = (node.textContent || "").trim();
          if (!text) return null;

          const trimmed = text.length > 44 ? `${text.slice(0, 44)}...` : text;
          if (!node.id) {
            node.id = `sec-${slugify(text) || "section"}-${index + 1}`;
          }
          node.style.scrollMarginTop = "5.5rem";

          return {
            label: trimmed,
            href: `#${node.id}`,
            icon: node.tagName === "H2" ? "view_agenda" : "subdirectory_arrow_right",
          };
        })
        .filter((l): l is AsideNavLink => l !== null);

      setDynamicAsideLinks(links);
    };

    const raf = window.requestAnimationFrame(refresh);
    const bodyObserver = new MutationObserver(() => refresh());
    bodyObserver.observe(document.body, { childList: true, subtree: true, characterData: true });

    return () => {
      window.cancelAnimationFrame(raf);
      bodyObserver.disconnect();
    };
  }, [pathname, reportId]);

  const asideLinks = reportId ? dynamicAsideLinks : getPageAsideLinks(pathname);

  return (
    <ThemeProvider>
      <div className="app-container">
        <nav className="navbar">
          <div className="nav-left">
            <div className="nav-brand">
              <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false">
                <path fill="currentColor" d="M3 5.5A2.5 2.5 0 0 1 5.5 3h13A2.5 2.5 0 0 1 21 5.5v13A2.5 2.5 0 0 1 18.5 21h-13A2.5 2.5 0 0 1 3 18.5zm2 0a.5.5 0 0 0-.5.5v12a.5.5 0 0 0 .5.5h13a.5.5 0 0 0 .5-.5v-12a.5.5 0 0 0-.5-.5zm2.2 2.2a1 1 0 0 1 1.4 0l2.6 2.6a1 1 0 0 1 0 1.4l-2.6 2.6a1 1 0 1 1-1.4-1.4L9.09 11 7.2 9.1a1 1 0 0 1 0-1.4M12 14a1 1 0 0 1 1-1h3a1 1 0 1 1 0 2h-3a1 1 0 0 1-1-1"/>
              </svg>
              XWA
            </div>
          </div>
          <div className="nav-center">
            <div className="nav-links">
              {navLinks.map((item) => (
                <Link key={`${item.href}-${item.label}`} href={item.href} className="nav-link" data-active={item.isActive}>
                  {item.label}
                </Link>
              ))}
            </div>
          </div>
          <div className="nav-right">
            <button
              type="button"
              className="nav-hamburger"
              aria-label="Toggle navigation menu"
              aria-expanded={mobileMenuOpen}
              onClick={() => setMobileMenuOpen((prev) => !prev)}
            >
              <NavHamburgerIcon open={mobileMenuOpen} />
            </button>
            <div style={{
              width: '2.25rem',
              height: '2.25rem',
              background: 'var(--surface-highest)',
              border: '1px solid rgba(57,255,20,0.2)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'var(--primary)',
            }}>
              <NavStatusIcon />
            </div>
          </div>
        </nav>

        <div className="mobile-nav-panel" data-open={mobileMenuOpen ? "true" : "false"}>
          <div className="mobile-nav-links" aria-label="Mobile navigation links">
            {navLinks.map((item) => (
              <Link
                key={`mobile-${item.href}-${item.label}`}
                href={item.href}
                className="nav-link"
                data-active={item.isActive}
                onClick={() => setMobileMenuOpen(false)}
              >
                {item.label}
              </Link>
            ))}
          </div>
        </div>

        <div className={`content-shell ${asideLinks.length > 0 ? "has-aside" : ""}`}>
          <main className="main-content">
            <div className="scan-lines-overlay"></div>
            <div key={pathname || "root"} className="page-transition">
              {children}
            </div>
          </main>
        </div>

        {asideLinks.length > 0 && (
          <aside className="page-aside">
            <div className="page-aside-header">
              <AsideSymbolIcon symbol="Sections_M" size={12} />
            </div>
            <nav className="page-aside-nav" aria-label="Page sections">
              {asideLinks.map((link) => (
                <Link
                  key={`${link.label}-${link.href}`}
                  href={link.href}
                  className="page-aside-link"
                  data-active={link.isActive ? "true" : "false"}
                  aria-current={link.isActive ? "page" : undefined}
                >
                  {link.icon && <AsideSymbolIcon symbol={link.icon} size={12} />}
                  {link.label}
                </Link>
              ))}
            </nav>
          </aside>
        )}

        <footer className="app-footer">
          <a href="https://github.com/FrancisFoRL" target="_blank" rel="noreferrer">Francis</a>
          <span className="footer-sep">|</span>
          <a href="https://github.com/xscriptor" target="_blank" rel="noreferrer">Xscriptor</a>
        </footer>
      </div>
    </ThemeProvider>
  );
}
