mod app;
mod crawler;
mod db;
mod export;
mod recon;
mod scanner;
mod tasks;
mod tui;

use app::{App, ExportMode, Tab};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use db::connection::DbPool;
use std::io;
use std::sync::Arc;
use tasks::BgEvent;

/// A drop guard that ensures the terminal is restored on panic or early return.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let pool = match db::connection::create_pool().await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Fatal: no database backend available: {}", e);
            return Err(io::Error::other(e.to_string()));
        }
    };

    let backend_name = match &pool {
        DbPool::Postgres(_) => "PostgreSQL",
        DbPool::Sqlite(_) => "SQLite",
    };
    log::info!("Storage backend: {}", backend_name);

    if let Err(e) = db::connection::init_db(&pool).await {
        log::error!("DB init failed: {}", e);
    }

    let mut client_builder = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Samurai-TUI/2.5.0")
        .pool_max_idle_per_host(5);

    // Proxy desde variable de entorno SAMURAI_PROXY
    if let Ok(proxy_url) = std::env::var("SAMURAI_PROXY") {
        if !proxy_url.is_empty() {
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => {
                    client_builder = client_builder.proxy(proxy);
                    log::info!("HTTP proxy configured: {}", proxy_url);
                }
                Err(e) => {
                    log::warn!("Invalid SAMURAI_PROXY '{}': {}", proxy_url, e);
                }
            }
        }
    }

    // Proxy DNS (SOCKS5 con resolución remota) desde SAMURAI_PROXY_DNS
    if let Ok(proxy_dns) = std::env::var("SAMURAI_PROXY_DNS") {
        if !proxy_dns.is_empty() {
            match reqwest::Proxy::all(&proxy_dns) {
                Ok(proxy) => {
                    client_builder = client_builder.proxy(proxy);
                    log::info!("DNS proxy configured: {}", proxy_dns);
                }
                Err(e) => {
                    log::warn!("Invalid SAMURAI_PROXY_DNS '{}': {}", proxy_dns, e);
                }
            }
        }
    }

    let http_client = Arc::new(client_builder.build().unwrap_or_default());

    let dns_resolver = Arc::new(
        hickory_resolver::TokioAsyncResolver::tokio_from_system_conf()
            .unwrap_or_else(|_| {
                hickory_resolver::TokioAsyncResolver::tokio(
                    hickory_resolver::config::ResolverConfig::default(),
                    hickory_resolver::config::ResolverOpts::default(),
                )
            }),
    );

    let mut app = App::new(pool, http_client, dns_resolver);
    app.load_history().await;

    // Ensure terminal is restored on exit
    let _guard = TerminalGuard;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app).await;

    // TerminalGuard restores terminal when dropped

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<BgEvent>();
    let mut background_task: Option<tokio::task::JoinHandle<()>> = None;

    loop {
        terminal.draw(|f| tui::ui::render(f, app))?;

        if app.should_quit {
            break;
        }

        // Drain any background events first
        while let Ok(event) = event_rx.try_recv() {
            handle_bg_event(app, event);
        }

        if !event::poll(std::time::Duration::from_millis(33))? {
            if let Some(ref task) = background_task {
                if task.is_finished() {
                    background_task = None;
                }
            }
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                handle_key_event(app, &event_tx, &mut background_task, key.code).await;
            }
            Event::Mouse(mouse) => {
                handle_mouse_event(app, mouse).await;
            }
            _ => {}
        }

        // Final drain after event handling
        while let Ok(event) = event_rx.try_recv() {
            handle_bg_event(app, event);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Key event handling
// ---------------------------------------------------------------------------

async fn handle_key_event(
    app: &mut App,
    event_tx: &tokio::sync::mpsc::UnboundedSender<BgEvent>,
    background_task: &mut Option<tokio::task::JoinHandle<()>>,
    key: KeyCode,
) {
    match key {
        KeyCode::Char('q') if !app.is_any_running() => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            if app.is_any_running() {
                app.cancel_all();
            }
        }
        KeyCode::Tab => {
            app.cycle_tab().await;
        }
        KeyCode::Enter if !app.is_any_running() => {
            start_task(app, event_tx, background_task).await;
        }
        KeyCode::Char(' ') if !app.is_any_running() && app.tab == Tab::Export => {
            app.export_mode = match app.export_mode {
                ExportMode::Raw => ExportMode::Encrypted,
                ExportMode::Encrypted => ExportMode::Raw,
            };
            app.set_status(&format!(
                "[{} MODE]",
                if matches!(app.export_mode, ExportMode::Raw) {
                    "RAW"
                } else {
                    "ENCRYPTED"
                }
            ));
        }
        KeyCode::Left if !app.is_any_running() => match app.tab {
            Tab::Scanner if app.scanner_profile > 0 => app.scanner_profile -= 1,
            Tab::Crawler if app.crawler_max_pages > 1 => app.crawler_max_pages -= 1,
            _ => {}
        },
        KeyCode::Right if !app.is_any_running() => match app.tab {
            Tab::Scanner
                if app.scanner_profile < app.scanner_profiles.len() - 1 =>
            {
                app.scanner_profile += 1
            }
            Tab::Crawler if app.crawler_max_pages < 20 => app.crawler_max_pages += 1,
            _ => {}
        },
        KeyCode::Up if app.tab == Tab::History => {
            app.navigate_history(-1).await;
        }
        KeyCode::Down if app.tab == Tab::History => {
            app.navigate_history(1).await;
        }
        KeyCode::Char(c) if !app.is_any_running() => {
            app.type_char(c);
        }
        KeyCode::Backspace if !app.is_any_running() => {
            app.backspace();
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Mouse event handling
// ---------------------------------------------------------------------------

async fn handle_mouse_event(app: &mut App, mouse: crossterm::event::MouseEvent) {
    // Only handle left-click down events
    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return;
    }

    let col = mouse.column;
    let row = mouse.row;

    // Tab clicks: top row (row 0), ~20 chars per tab segment
    if row == 0 {
        let tab_idx = (col / 20).min(4) as usize;
        let new_tab = Tab::from_index(tab_idx);
        if new_tab != app.tab {
            app.tab = new_tab;
            let _ = app.load_history_if_needed().await;
        }
    }
}

// ---------------------------------------------------------------------------
// Task launcher
// ---------------------------------------------------------------------------

async fn start_task(
    app: &mut App,
    event_tx: &tokio::sync::mpsc::UnboundedSender<BgEvent>,
    background_task: &mut Option<tokio::task::JoinHandle<()>>,
) {
    let tx = event_tx.clone();
    let pool = app.pool.clone();
    let http = app.http_client.clone();
    let dns = app.dns_resolver.clone();

    match app.tab {
        Tab::Scanner => {
            let target = app.scanner_target.clone();
            let profile = app.scanner_profiles[app.scanner_profile].to_string();
            if target.is_empty() {
                app.set_status("[ERROR] Target required");
                return;
            }
            app.scanner_running = true;
            app.scanner_done = false;
            app.scanner_open_ports.clear();
            app.scanner_logs.clear();
            app.set_status(&format!("[RUNNING] Scanning {} ({})", target, profile));
            *background_task = Some(tokio::spawn(async move {
                tasks::run_scanner(target, profile, pool, tx).await;
            }));
        }
        Tab::Recon => {
            let target = app.recon_target.clone();
            if target.is_empty() {
                app.set_status("[ERROR] Target required");
                return;
            }
            app.recon_running = true;
            app.recon_done = false;
            app.recon_logs.clear();
            app.recon_results = None;
            app.set_status(&format!("[RUNNING] Recon on {}", target));
            *background_task = Some(tokio::spawn(async move {
                tasks::run_recon(target, pool, http, dns, tx).await;
            }));
        }
        Tab::Crawler => {
            let target = app.crawler_target.clone();
            let max_pages = app.crawler_max_pages;
            if target.is_empty() {
                app.set_status("[ERROR] Target required");
                return;
            }
            app.crawler_running = true;
            app.crawler_done = false;
            app.crawler_pages.clear();
            app.crawler_vulns.clear();
            app.crawler_logs.clear();
            app.set_status(&format!("[CRAWLING] {} (max {} pages)", target, max_pages));
            *background_task = Some(tokio::spawn(async move {
                tasks::run_crawler(target, max_pages, pool, http, tx).await;
            }));
        }
        Tab::Export => {
            let mode = app.export_mode;
            let password = app.export_password.clone();
            let path_str = if app.export_path.is_empty() {
                let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                match mode {
                    ExportMode::Raw => format!("samurai-db-export-{}.json", date),
                    ExportMode::Encrypted => format!("samurai-db-export-{}.bin.enc", date),
                }
            } else {
                app.export_path.clone()
            };
            app.export_running = true;
            app.export_done = false;
            app.export_error = None;
            app.set_status(&format!("[EXPORTING] {}", path_str));
            *background_task = Some(tokio::spawn(async move {
                tasks::run_export(mode, password, path_str, pool, tx).await;
            }));
        }
        Tab::History => {
            if let Some(scan) = app.history_selected.clone() {
                let id = scan.id;
                if let Err(e) = db::operations::delete_scan(&app.pool, id).await {
                    app.set_status(&format!("[ERROR] Delete failed: {}", e));
                } else {
                    app.set_status(&format!("[DELETED] Scan #{}", id));
                    app.history_selected = None;
                    app.history_findings.clear();
                    app.history_links.clear();
                    app.load_history().await;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Background event handler
// ---------------------------------------------------------------------------

fn handle_bg_event(app: &mut App, event: BgEvent) {
    match event {
        BgEvent::ScannerLog(line) => {
            if app.scanner_logs.len() >= 2000 {
                app.scanner_logs.pop_front();
            }
            app.scanner_logs.push_back(line);
        }
        BgEvent::ScannerPort(port) => {
            app.scanner_open_ports.push(port);
        }
        BgEvent::ScannerDone(ports) => {
            app.scanner_open_ports = ports;
            app.scanner_running = false;
            app.scanner_done = true;
            let len = app.scanner_open_ports.len();
            app.set_status(&format!("[DONE] Scanner — {} open ports", len));
        }
        BgEvent::ReconLog(line) => {
            if app.recon_logs.len() >= 2000 {
                app.recon_logs.pop_front();
            }
            app.recon_logs.push_back(line);
        }
        BgEvent::ReconDone(results) => {
            app.recon_results = Some(results);
            app.recon_running = false;
            app.recon_done = true;
            app.set_status("[DONE] Reconnaissance completed");
        }
        BgEvent::CrawlerLog(line) => {
            if app.crawler_logs.len() >= 2000 {
                app.crawler_logs.pop_front();
            }
            app.crawler_logs.push_back(line);
        }
        BgEvent::CrawlerDone(pages, vulns) => {
            app.crawler_pages = pages;
            app.crawler_vulns = vulns;
            app.crawler_running = false;
            app.crawler_done = true;
            app.set_status(&format!(
                "[DONE] Crawler — {} pages, {} vulns",
                app.crawler_pages.len(),
                app.crawler_vulns.len()
            ));
        }
        BgEvent::ExportSummary(scans, findings, links) => {
            app.export_summary = Some((scans, findings, links));
        }
        BgEvent::ExportDone => {
            app.export_running = false;
            app.export_done = true;
            app.set_status("[DONE] Database exported");
        }
        BgEvent::Status(msg) => {
            app.set_status(&msg);
        }
    }
}
