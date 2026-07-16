# Tengu UI Architecture

## Framework

Angular 19 standalone (no NgModules). The UI follows the Nothing Design System as defined in the Samurai skill references.

## Component Tree

```
AppComponent (shell)
├── Sidebar
│   ├── Brand (TENGU / XWA - MODULE)
│   ├── Nav links (AUDIT, HISTORY)
│   └── Footer
│       ├── Theme toggle
│       └── Social links (GitHub, repo, dev site)
└── Router outlet
    ├── AuditComponent
    │   ├── Audit form (URL, mode, category tabs)
    │   ├── Severity summary (error, warning, info, pass counts)
    │   ├── Result tabs (category filter)
    │   ├── Findings accordion list
    │   ├── HTML source viewer with line highlighting
    │   └── AuditExportActionsComponent
    └── HistoryComponent
        ├── Refresh button
        ├── Audit table (URL, status, findings, date, load link)
        └── Error display
```

## Design System Tokens

All tokens are defined as CSS custom properties in `styles.scss`:

- **Font stack**: Space Grotesk (body/headings), Space Mono (data/code), Doto (display)
- **Color scheme**: Dark mode default, light mode via `.theme-light` body class
- **Spacing scale**: 8px base -- 2xs (4px), xs (8px), sm (12px), md (16px), lg (24px), xl (32px), 2xl (48px), 3xl (64px), 4xl (96px)
- **Dot-grid motif**: Background pattern via CSS gradient overlays

## Key Patterns

### Standalone Components
Every component is `standalone: true`. No NgModules. Shared functionality (ThemeService) is injected directly.

### WebSocket Streaming
The audit component opens a WebSocket to `/api/audit/live` with query parameters for URL, mode, subdomains, and checks. Messages follow a simple text protocol:

| Prefix | Content |
|---|---|
| `[AUDIT]` | Status log message |
| `[PAGE]` | Discovered page URL |
| `[HTML]` | Full pretty-printed HTML source |
| `[done]` | Audit completed successfully |
| `[!]` | Error message |
| JSON object | A single finding |

### Change Detection
WebSocket callbacks call `ChangeDetectorRef.detectChanges()` manually to update the view outside Angular's zone.

### Client-side Export
Exports generate content in-memory, create a Blob, and trigger download via a temporary anchor element. No server-side file generation. See `export-patterns.md` in the Samurai skill references.

### Line Highlighting
Findings include a snippet of the offending HTML element. After pretty-printing, each element occupies its own line, and the frontend matches snippets to lines by tag name and key attribute values.

## Routes

| Path | Component | Description |
|---|---|---|
| `/audit` | AuditComponent | Run and view audits |
| `/history` | HistoryComponent | Browse past audits |
| `/` | (redirect) | Redirects to `/audit` |

## Theme Service

`ThemeService` uses Angular Signals for the dark/light state. The current theme is persisted to `localStorage` under key `tengu-theme`. Toggling adds/removes the `.theme-light` class on `<body>`.
