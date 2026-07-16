use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};
use crate::app::{App, ExportMode, Tab};
use crate::tui::theme;
use std::sync::LazyLock;

static TAB_TITLES: LazyLock<[&str; 5]> = LazyLock::new(|| {
    [" Scanner ", " Recon ", " Crawler ", " History ", " Export "]
});

pub fn render(f: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    render_tabs(f, app, main_layout[0]);
    render_content(f, app, main_layout[1]);
    render_status_bar(f, app, main_layout[2]);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs: Vec<Line> = TAB_TITLES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let tab_idx = Tab::from_index(i);
            if app.tab == tab_idx {
                Line::from(vec![
                    Span::styled("[", Style::default().fg(theme::BORDER_VISIBLE)),
                    Span::styled(
                        *t,
                        Style::default()
                            .fg(theme::TEXT_DISPLAY)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("]", Style::default().fg(theme::BORDER_VISIBLE)),
                ])
            } else {
                Line::from(Span::styled(*t, Style::default().fg(theme::TEXT_DISABLED)))
            }
        })
        .collect();

    f.render_widget(
        Tabs::new(tabs)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme::BORDER)),
            )
            .select(app.tab.index())
            .highlight_style(Style::default().fg(theme::INTERACTIVE)),
        area,
    );
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    match app.tab {
        Tab::Scanner => render_scanner(f, app, area),
        Tab::Recon => render_recon(f, app, area),
        Tab::Crawler => render_crawler(f, app, area),
        Tab::History => render_history(f, app, area),
        Tab::Export => render_export(f, app, area),
    }
}

fn render_log_entries(
    logs: &std::collections::VecDeque<String>,
    limit: usize,
) -> (Vec<ListItem<'_>>, usize) {
    let count = logs.len();
    let items: Vec<ListItem> = logs
        .iter()
        .rev()
        .take(limit)
        .map(|l| {
            let style = if l.starts_with("[ERROR]") {
                Style::default().fg(theme::ACCENT)
            } else if l.starts_with("[+]")
                || l.starts_with("[PORT]")
                || l.starts_with("[HDR]")
            {
                Style::default().fg(theme::SUCCESS)
            } else if l.starts_with("[!]")
                || l.starts_with("[DNS]")
                || l.starts_with("[SUB]")
                || l.starts_with("[API]")
                || l.starts_with("[TECH]")
                || l.starts_with("[CRAWL]")
                || l.starts_with("[PAGE]")
            {
                Style::default().fg(theme::WARNING)
            } else if l.contains("/tcp") && l.contains("open") {
                Style::default().fg(theme::GOLD)
            } else if l.starts_with("[CMD]") || l.starts_with("[stderr]") {
                Style::default().fg(theme::TEXT_DISABLED)
            } else {
                Style::default().fg(theme::TEXT_SECONDARY)
            };
            ListItem::new(Span::styled(l.as_str(), style))
        })
        .collect();
    (items, count)
}

// ---------------------------------------------------------------------------
// Scanner tab
// ---------------------------------------------------------------------------

fn render_scanner(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(6), Constraint::Min(0)]).split(area);

    let status = if app.scanner_running {
        ("[RUNNING]", theme::WARNING)
    } else if app.scanner_done {
        ("[DONE] Press ENTER to run again", theme::SUCCESS)
    } else {
        ("[READY] Press ENTER to start scan", theme::INTERACTIVE)
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                format!(" Target: {}▊", app.scanner_target),
                Style::default().fg(theme::TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                format!(
                    " Profile: {}  (← → to change)",
                    app.scanner_profiles[app.scanner_profile]
                ),
                Style::default().fg(theme::TEXT_SECONDARY),
            )),
            Line::from(Span::styled(status.0, Style::default().fg(status.1))),
        ])
        .block(
            Block::bordered()
                .title(Span::styled(
                    " Nmap Scanner ",
                    Style::default().fg(theme::INTERACTIVE),
                ))
                .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
        )
        .style(Style::default().bg(theme::SURFACE)),
        chunks[0],
    );

    let output_chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

    // Left: logs
    let (items, _) = render_log_entries(&app.scanner_logs, 100);
    f.render_widget(
        List::new(items)
            .block(
                Block::bordered()
                    .title(Span::styled(" Log ", Style::default().fg(theme::TEXT_DISPLAY)))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK)),
        output_chunks[0],
    );

    // Right: open ports
    let port_items: Vec<ListItem> = if app.scanner_open_ports.is_empty() {
        vec![ListItem::new(Span::styled(
            "(no open ports)",
            Style::default().fg(theme::TEXT_DISABLED),
        ))]
    } else {
        app.scanner_open_ports
            .iter()
            .map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled(&p.port, Style::default().fg(theme::GOLD)),
                    Span::styled("  ", Style::default()),
                    Span::styled(&p.service, Style::default().fg(theme::TEXT_PRIMARY)),
                ]))
            })
            .collect()
    };

    f.render_widget(
        List::new(port_items)
            .block(
                Block::bordered()
                    .title(Span::styled(
                        format!(" Open Ports ({}) ", app.scanner_open_ports.len()),
                        Style::default().fg(theme::GOLD),
                    ))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK)),
        output_chunks[1],
    );
}

// ---------------------------------------------------------------------------
// Recon tab
// ---------------------------------------------------------------------------

fn render_recon(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).split(area);

    let status = if app.recon_running {
        ("[RUNNING] DNS → Subdomains → APIs → Headers → Tech", theme::WARNING)
    } else if app.recon_done {
        ("[DONE] Press ENTER to run again", theme::SUCCESS)
    } else {
        ("[READY] Press ENTER to start reconnaissance", theme::INTERACTIVE)
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                format!(" Target: {}▊", app.recon_target),
                Style::default().fg(theme::TEXT_PRIMARY),
            )),
            Line::from(Span::styled(status.0, Style::default().fg(status.1))),
        ])
        .block(
            Block::bordered()
                .title(Span::styled(
                    " Web Reconnaissance ",
                    Style::default().fg(theme::INTERACTIVE),
                ))
                .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
        )
        .style(Style::default().bg(theme::SURFACE)),
        chunks[0],
    );

    let output_chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

    // Left: logs
    let (items, _) = render_log_entries(&app.recon_logs, 100);
    f.render_widget(
        List::new(items)
            .block(
                Block::bordered()
                    .title(Span::styled(" Log ", Style::default().fg(theme::TEXT_DISPLAY)))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK)),
        output_chunks[0],
    );

    // Right: results
    let result_lines: Vec<Line> = if let Some(ref r) = app.recon_results {
        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            format!(" Target: {}", r.target),
            Style::default().fg(theme::TEXT_DISPLAY),
        )));
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            format!(" DNS Records:    {}", r.dns_records.len()),
            Style::default().fg(theme::SUCCESS),
        )));
        lines.push(Line::from(Span::styled(
            format!(" Subdomains:     {}", r.subdomains.len()),
            Style::default().fg(theme::WARNING),
        )));
        lines.push(Line::from(Span::styled(
            format!(" API Endpoints:  {}", r.api_endpoints.len()),
            Style::default().fg(theme::INTERACTIVE),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                " Headers:        {} total, {} missing",
                r.security_headers.len(),
                r.security_headers.iter().filter(|h| !h.present).count()
            ),
            Style::default().fg(theme::TEXT_PRIMARY),
        )));
        lines.push(Line::from(Span::styled(
            format!(" Technologies:   {}", r.technologies.len()),
            Style::default().fg(theme::GOLD),
        )));
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            " DNS Records:",
            Style::default()
                .fg(theme::TEXT_DISPLAY)
                .add_modifier(Modifier::BOLD),
        )));
        for d in r.dns_records.iter().take(5) {
            lines.push(Line::from(Span::styled(
                format!("  {} {} → {}", d.record_type, d.name, d.value),
                Style::default().fg(theme::TEXT_SECONDARY),
            )));
        }
        if r.dns_records.len() > 5 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", r.dns_records.len() - 5),
                Style::default().fg(theme::TEXT_DISABLED),
            )));
        }
        lines.push(Line::from(Span::styled(
            " Subdomains:",
            Style::default()
                .fg(theme::TEXT_DISPLAY)
                .add_modifier(Modifier::BOLD),
        )));
        for s in r.subdomains.iter().take(5) {
            lines.push(Line::from(Span::styled(
                format!("  {} [{}]", s.name, s.resolved_ips.join(", ")),
                Style::default().fg(theme::TEXT_SECONDARY),
            )));
        }
        if r.subdomains.len() > 5 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", r.subdomains.len() - 5),
                Style::default().fg(theme::TEXT_DISABLED),
            )));
        }
        lines
    } else {
        vec![Line::from(Span::styled(
            " Results will appear here after reconnaissance completes.",
            Style::default().fg(theme::TEXT_DISABLED),
        ))]
    };

    f.render_widget(
        Paragraph::new(result_lines)
            .block(
                Block::bordered()
                    .title(Span::styled(" Results ", Style::default().fg(theme::TEXT_DISPLAY)))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK))
            .wrap(Wrap { trim: false }),
        output_chunks[1],
    );
}

// ---------------------------------------------------------------------------
// Crawler tab
// ---------------------------------------------------------------------------

fn render_crawler(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(6), Constraint::Min(0)]).split(area);

    let status = if app.crawler_running {
        ("[CRAWLING] Discovering pages...", theme::WARNING)
    } else if app.crawler_done {
        ("[DONE] Scan complete", theme::SUCCESS)
    } else {
        ("[READY] Press ENTER to start vulnerability scan", theme::INTERACTIVE)
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                format!(" Target: {}▊", app.crawler_target),
                Style::default().fg(theme::TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                format!(" Max Pages: {}  (← → to adjust)", app.crawler_max_pages),
                Style::default().fg(theme::TEXT_SECONDARY),
            )),
            Line::from(Span::styled(status.0, Style::default().fg(status.1))),
        ])
        .block(
            Block::bordered()
                .title(Span::styled(
                    " Vulnerability Crawler (DAST) ",
                    Style::default().fg(theme::ACCENT),
                ))
                .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
        )
        .style(Style::default().bg(theme::SURFACE)),
        chunks[0],
    );

    let output_chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

    // Left: logs
    let (items, _) = render_log_entries(&app.crawler_logs, 100);
    f.render_widget(
        List::new(items)
            .block(
                Block::bordered()
                    .title(Span::styled(" Log ", Style::default().fg(theme::TEXT_DISPLAY)))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK)),
        output_chunks[0],
    );

    // Right: discovered pages
    let page_items: Vec<ListItem> = if app.crawler_pages.is_empty() {
        vec![ListItem::new(Span::styled(
            "(no pages discovered)",
            Style::default().fg(theme::TEXT_DISABLED),
        ))]
    } else {
        app.crawler_pages
            .iter()
            .map(|p| {
                let color = if p.status_code >= 200 && p.status_code < 300 {
                    theme::SUCCESS
                } else if p.status_code >= 300 && p.status_code < 400 {
                    theme::WARNING
                } else if p.status_code >= 400 {
                    theme::ACCENT
                } else {
                    theme::TEXT_SECONDARY
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("HTTP {} ", p.status_code),
                        Style::default().fg(color),
                    ),
                    Span::styled(&p.url, Style::default().fg(theme::TEXT_PRIMARY)),
                ]))
            })
            .collect()
    };

    f.render_widget(
        List::new(page_items)
            .block(
                Block::bordered()
                    .title(Span::styled(
                        format!(
                            " Discovered Pages ({}) ",
                            app.crawler_pages.len()
                        ),
                        Style::default().fg(theme::TEXT_DISPLAY),
                    ))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK)),
        output_chunks[1],
    );
}

// ---------------------------------------------------------------------------
// History tab — MEJORADO
// ---------------------------------------------------------------------------

fn render_history(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    // Left: scan list
    let scan_items: Vec<ListItem> = app
        .history_scans
        .iter()
        .map(|s| {
            let color = match s.status.as_str() {
                "COMPLETED" => theme::SUCCESS,
                "ERROR" => theme::ACCENT,
                _ => theme::WARNING,
            };
            let is_selected = app
                .history_selected
                .as_ref()
                .map(|sel| sel.id == s.id)
                .unwrap_or(false);
            let prefix = if is_selected { "▸" } else { " " };

            // Show type badge
            let type_label = match s.scan_type.as_str() {
                t if t.starts_with("port_scan") => "[SCAN]",
                "web_recon" => "[RECON]",
                "crawler" => "[CRAWL]",
                _ => &s.scan_type,
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme::INTERACTIVE)),
                Span::styled(
                    format!("#{} ", s.id),
                    Style::default().fg(theme::INTERACTIVE),
                ),
                Span::styled(
                    format!("{} ", type_label),
                    Style::default().fg(theme::TEXT_DISABLED),
                ),
                Span::styled(&s.domain_target, Style::default().fg(theme::TEXT_PRIMARY)),
                Span::styled(
                    format!(" [{}]", s.status),
                    Style::default().fg(color),
                ),
            ]))
        })
        .collect();

    f.render_widget(
        List::new(scan_items)
            .block(
                Block::bordered()
                    .title(Span::styled(
                        format!(" Scans ({}) ", app.history_scans.len()),
                        Style::default().fg(theme::INTERACTIVE),
                    ))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::SURFACE))
            .highlight_style(Style::default().bg(theme::SURFACE_RAISED)),
        chunks[0],
    );

    // Right: detail panel — MEJORADO
    let detail = if let Some(scan) = &app.history_selected {
        let mut lines = vec![
            Line::from(Span::styled(
                format!(" Scan #{}", scan.id),
                Style::default()
                    .fg(theme::TEXT_DISPLAY)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled(" Target: ", Style::default().fg(theme::TEXT_DISABLED)),
                Span::styled(
                    &scan.domain_target,
                    Style::default().fg(theme::TEXT_PRIMARY),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Type:   ", Style::default().fg(theme::TEXT_DISABLED)),
                Span::styled(&scan.scan_type, Style::default().fg(theme::TEXT_SECONDARY)),
            ]),
            Line::from(vec![
                Span::styled(" Status: ", Style::default().fg(theme::TEXT_DISABLED)),
                Span::styled(
                    &scan.status,
                    Style::default().fg(match scan.status.as_str() {
                        "COMPLETED" => theme::SUCCESS,
                        _ => theme::WARNING,
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Date:   ", Style::default().fg(theme::TEXT_DISABLED)),
                Span::styled(
                    scan.created_at
                        .as_deref()
                        .unwrap_or("unknown"),
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ]),
            Line::from(Span::styled("", Style::default())),
        ];

        // --- Severity summary bar ---
        let severities = app.finding_severity_counts();
        let has_findings = severities.iter().any(|(_, count)| *count > 0);
        if has_findings {
            lines.push(Line::from(Span::styled(
                " Vulnerability Summary:",
                Style::default()
                    .fg(theme::TEXT_DISPLAY)
                    .add_modifier(Modifier::BOLD),
            )));

            let mut summary_parts: Vec<Span> = Vec::new();
            for (label, count) in &severities {
                if *count == 0 {
                    continue;
                }
                let color = match label.as_str() {
                    "CRITICAL" => theme::ACCENT,
                    "HIGH" => Color::Rgb(255, 85, 85),
                    "MEDIUM" => theme::WARNING,
                    "LOW" => Color::Rgb(153, 204, 255),
                    _ => theme::TEXT_SECONDARY,
                };
                if !summary_parts.is_empty() {
                    summary_parts.push(Span::styled(" │ ", Style::default().fg(theme::TEXT_DISABLED)));
                }
                summary_parts.push(Span::styled(
                    format!("{} {}", count, label),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
            }
            lines.push(Line::from(summary_parts));
            lines.push(Line::from(Span::styled("", Style::default())));
        }

        // --- Findings list ---
        if !app.history_findings.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" Findings ({}):", app.history_findings.len()),
                Style::default()
                    .fg(theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            )));

            for finding in &app.history_findings {
                let sev_color = match finding.severity.to_lowercase().as_str() {
                    "critical" => theme::ACCENT,
                    "high" => Color::Rgb(255, 85, 85),
                    "medium" => theme::WARNING,
                    "low" => Color::Rgb(153, 204, 255),
                    _ => theme::SUCCESS,
                };

                // Truncate description
                let desc = if finding.description.len() > 100 {
                    format!("{}...", &finding.description[..97])
                } else {
                    finding.description.clone()
                };

                // CVSS score if available
                let cvss_str = finding
                    .cvss_score
                    .as_ref()
                    .map(|s| format!(" (CVSS:{})", s))
                    .unwrap_or_default();

                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" [{:>8}] ", finding.severity.to_uppercase()),
                        Style::default().fg(sev_color).bg(theme::SURFACE_RAISED),
                    ),
                    Span::styled(
                        format!(" {}", finding.finding_type),
                        Style::default().fg(theme::INTERACTIVE),
                    ),
                    Span::styled(cvss_str, Style::default().fg(theme::GOLD)),
                ]));
                lines.push(Line::from(Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(theme::TEXT_SECONDARY),
                )));

                // Show poc_payload if available and short enough
                if let Some(ref payload) = finding.poc_payload {
                    if payload.len() < 200 {
                        lines.push(Line::from(Span::styled(
                            format!("  └─ {}", payload),
                            Style::default().fg(theme::TEXT_DISABLED),
                        )));
                    }
                }
            }
            lines.push(Line::from(Span::styled("", Style::default())));
        } else {
            lines.push(Line::from(Span::styled(
                " No findings for this scan.",
                Style::default().fg(theme::TEXT_DISABLED),
            )));
            lines.push(Line::from(Span::styled("", Style::default())));
        }

        // --- Discovered links ---
        if !app.history_links.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" Discovered Links ({}):", app.history_links.len()),
                Style::default()
                    .fg(theme::WARNING)
                    .add_modifier(Modifier::BOLD),
            )));
            for link in &app.history_links {
                let status_color = match link.status_code {
                    Some(200..=299) => theme::SUCCESS,
                    Some(300..=399) => theme::WARNING,
                    Some(400..=599) => theme::ACCENT,
                    _ => theme::TEXT_SECONDARY,
                };
                let status_str = link
                    .status_code
                    .map(|c| format!("HTTP {}", c))
                    .unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::styled(status_str, Style::default().fg(status_color)),
                    Span::styled(" ", Style::default()),
                    Span::styled(&link.url, Style::default().fg(theme::TEXT_PRIMARY)),
                    Span::styled(
                        link.content_type
                            .as_ref()
                            .map(|ct| format!(" [{}]", ct))
                            .unwrap_or_default(),
                        Style::default().fg(theme::TEXT_DISABLED),
                    ),
                ]));
            }
            lines.push(Line::from(Span::styled("", Style::default())));
        }

        lines.push(Line::from(Span::styled(
            " ENTER — Delete scan  |  ↑↓ — Navigate  |  🖱️ Click to select",
            Style::default().fg(theme::TEXT_DISABLED),
        )));

        lines
    } else {
        vec![
            Line::from(Span::styled(
                " Select a scan to view details",
                Style::default().fg(theme::TEXT_DISABLED),
            )),
            Line::from(Span::styled(
                " (↑↓ · click to select · DEL to delete)",
                Style::default().fg(theme::TEXT_DISABLED),
            )),
        ]
    };

    f.render_widget(
        Paragraph::new(detail)
            .block(
                Block::bordered()
                    .title(Span::styled(" Details ", Style::default().fg(theme::TEXT_DISPLAY)))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK))
            .wrap(Wrap { trim: false }),
        chunks[1],
    );
}

// ---------------------------------------------------------------------------
// Export tab
// ---------------------------------------------------------------------------

fn render_export(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).split(area);

    let mode_text = match app.export_mode {
        ExportMode::Raw => "RAW JSON",
        ExportMode::Encrypted => "ENCRYPTED (AES-256-GCM)",
    };

    let mut form_lines = vec![
        Line::from(Span::styled(
            format!(" Mode: {}  (SPACE to toggle)", mode_text),
            Style::default().fg(theme::TEXT_PRIMARY),
        )),
        Line::from(Span::styled(
            format!(" Path: {}▊", app.export_path),
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ];

    if app.export_mode == ExportMode::Encrypted {
        let masked = "•".repeat(app.export_password.len());
        form_lines.push(Line::from(Span::styled(
            format!(" Password: {}▊", masked),
            Style::default().fg(theme::WARNING),
        )));
    }

    f.render_widget(
        Paragraph::new(form_lines)
            .block(
                Block::bordered()
                    .title(Span::styled(
                        " Database Export ",
                        Style::default().fg(theme::GOLD),
                    ))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::SURFACE)),
        chunks[0],
    );

    let mut info = vec![
        Line::from(Span::styled(
            " ENTER — Execute export  |  SPACE — Toggle mode  |  Type path/password",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
        Line::from(Span::styled("", Style::default())),
    ];

    if let Some((scans, findings, links)) = app.export_summary {
        info.push(Line::from(Span::styled(
            format!(
                " ✓ EXPORTED: {} scans · {} findings · {} links",
                scans, findings, links
            ),
            Style::default().fg(theme::SUCCESS),
        )));
    }
    if let Some(ref err) = app.export_error {
        info.push(Line::from(Span::styled(
            format!(" ✗ ERROR: {}", err),
            Style::default().fg(theme::ACCENT),
        )));
    }
    if app.export_running {
        info.push(Line::from(Span::styled(
            " ⏳ Exporting...",
            Style::default().fg(theme::WARNING),
        )));
    }

    f.render_widget(
        Paragraph::new(info)
            .block(
                Block::bordered()
                    .title(Span::styled(
                        " Status ",
                        Style::default().fg(theme::TEXT_DISPLAY),
                    ))
                    .border_style(Style::default().fg(theme::BORDER_VISIBLE)),
            )
            .style(Style::default().bg(theme::BLACK)),
        chunks[1],
    );
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            &app.status_message,
            Style::default().fg(theme::TEXT_SECONDARY),
        )))
        .style(Style::default().bg(theme::SURFACE)),
        area,
    );
}
