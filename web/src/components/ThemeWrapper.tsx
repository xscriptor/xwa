"use client";

import ThemeProvider, { ThemeToggle } from "./ThemeProvider";

export default function ThemeWrapper({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider>
      <div className="app-container">
        <nav className="navbar">
          <div className="nav-brand">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M2 12h4l3-9 5 18 3-9h5" />
            </svg>
            xwa
          </div>
          <div className="nav-right">
            <div className="nav-links">
              <a href="/" className="nav-link" data-active="true">Scan</a>
              <a href="/reports" className="nav-link">Reports</a>
            </div>
            <ThemeToggle />
          </div>
        </nav>

        <main className="main-content">
          {children}
        </main>
      </div>
    </ThemeProvider>
  );
}
