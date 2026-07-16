use std::collections::VecDeque;
use std::sync::Arc;
use crate::db::connection::DbPool;
use crate::scanner::engine::PortInfo;
use crate::recon::ReconResults;
use crate::crawler::{DiscoveredPage, Vulnerability};
use crate::db::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Scanner,
    Recon,
    Crawler,
    History,
    Export,
}

impl Tab {
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Scanner => "Scanner",
            Tab::Recon => "Recon",
            Tab::Crawler => "Crawler",
            Tab::History => "History",
            Tab::Export => "Export",
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Tab::Scanner,
            1 => Tab::Recon,
            2 => Tab::Crawler,
            3 => Tab::History,
            _ => Tab::Export,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Scanner => 0,
            Tab::Recon => 1,
            Tab::Crawler => 2,
            Tab::History => 3,
            Tab::Export => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportMode {
    Raw,
    Encrypted,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

pub struct App {
    pub pool: DbPool,
    pub http_client: Arc<reqwest::Client>,
    pub dns_resolver: Arc<hickory_resolver::TokioAsyncResolver>,
    pub tab: Tab,
    pub should_quit: bool,

    pub scanner_target: String,
    pub scanner_profile: usize,
    pub scanner_profiles: Vec<&'static str>,
    pub scanner_logs: VecDeque<String>,
    pub scanner_running: bool,
    pub scanner_open_ports: Vec<PortInfo>,
    pub scanner_done: bool,

    pub recon_target: String,
    pub recon_logs: VecDeque<String>,
    pub recon_running: bool,
    pub recon_results: Option<ReconResults>,
    pub recon_done: bool,

    pub crawler_target: String,
    pub crawler_max_pages: usize,
    pub crawler_logs: VecDeque<String>,
    pub crawler_running: bool,
    pub crawler_pages: Vec<DiscoveredPage>,
    pub crawler_vulns: Vec<Vulnerability>,
    pub crawler_done: bool,

    pub history_scans: Vec<Scan>,
    pub history_selected: Option<Scan>,
    pub history_findings: Vec<Finding>,
    pub history_links: Vec<DiscoveredLink>,

    pub export_mode: ExportMode,
    pub export_password: String,
    pub export_path: String,
    pub export_running: bool,
    pub export_done: bool,
    pub export_error: Option<String>,
    pub export_summary: Option<(usize, usize, usize)>,

    pub status_message: String,
}

impl App {
    pub fn new(
        pool: DbPool,
        http_client: Arc<reqwest::Client>,
        dns_resolver: Arc<hickory_resolver::TokioAsyncResolver>,
    ) -> Self {
        App {
            pool,
            http_client,
            dns_resolver,
            tab: Tab::Scanner,
            should_quit: false,

            scanner_target: String::new(),
            scanner_profile: 0,
            scanner_profiles: vec!["quick", "balanced", "deep", "udp"],
            scanner_logs: VecDeque::with_capacity(2000),
            scanner_running: false,
            scanner_open_ports: Vec::new(),
            scanner_done: false,

            recon_target: String::new(),
            recon_logs: VecDeque::with_capacity(2000),
            recon_running: false,
            recon_results: None,
            recon_done: false,

            crawler_target: String::new(),
            crawler_max_pages: 5,
            crawler_logs: VecDeque::with_capacity(2000),
            crawler_running: false,
            crawler_pages: Vec::new(),
            crawler_vulns: Vec::new(),
            crawler_done: false,

            history_scans: Vec::new(),
            history_selected: None,
            history_findings: Vec::new(),
            history_links: Vec::new(),

            export_mode: ExportMode::Raw,
            export_password: String::new(),
            export_path: String::new(),
            export_running: false,
            export_done: false,
            export_error: None,
            export_summary: None,

            status_message: String::from("[READY] Tab to switch · Enter to run · q to quit · 🖱️ click tabs"),
        }
    }

    // -----------------------------------------------------------------------
    // Status helpers
    // -----------------------------------------------------------------------

    pub fn is_any_running(&self) -> bool {
        self.scanner_running || self.recon_running || self.crawler_running || self.export_running
    }

    pub fn cancel_all(&mut self) {
        self.scanner_running = false;
        self.recon_running = false;
        self.crawler_running = false;
        self.export_running = false;
        self.set_status("[CANCELLED]");
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
    }

    pub fn tab_name(&self) -> &'static str {
        self.tab.name()
    }

    // -----------------------------------------------------------------------
    // Tab navigation
    // -----------------------------------------------------------------------

    pub async fn cycle_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Scanner => Tab::Recon,
            Tab::Recon => Tab::Crawler,
            Tab::Crawler => Tab::History,
            Tab::History => Tab::Export,
            Tab::Export => Tab::Scanner,
        };
        self.set_status(&format!("[{}]", self.tab_name()));
        self.load_history_if_needed().await;
    }

    pub async fn load_history_if_needed(&mut self) -> bool {
        if self.tab == Tab::History {
            self.load_history().await;
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // History navigation
    // -----------------------------------------------------------------------

    pub async fn navigate_history(&mut self, delta: i32) {
        if self.history_scans.is_empty() {
            return;
        }
        let current = self
            .history_selected
            .as_ref()
            .and_then(|s| self.history_scans.iter().position(|hs| hs.id == s.id));

        let new_idx = match current {
            Some(idx) => {
                let next = idx as i32 + delta;
                if next < 0 {
                    0
                } else if next as usize >= self.history_scans.len() {
                    self.history_scans.len() - 1
                } else {
                    next as usize
                }
            }
            None => {
                if delta > 0 {
                    0
                } else {
                    self.history_scans.len() - 1
                }
            }
        };

        let scan = self.history_scans[new_idx].clone();
        self.history_selected = Some(scan.clone());
        self.load_history_details(&scan).await;
    }

    /// Load findings and links for a given scan.
    pub async fn load_history_details(&mut self, scan: &Scan) {
        if let Ok(findings) =
            crate::db::operations::get_findings_for_scan(&self.pool, scan.id).await
        {
            self.history_findings = findings;
        } else {
            self.history_findings.clear();
        }
        if let Ok(links) = crate::db::operations::get_links_for_scan(&self.pool, scan.id).await {
            self.history_links = links;
        } else {
            self.history_links.clear();
        }
    }

    pub async fn load_history(&mut self) {
        match crate::db::operations::get_all_scans(&self.pool).await {
            Ok(scans) => {
                self.history_scans = scans;
                self.set_status(&format!("[{} scans in database]", self.history_scans.len()));
            }
            Err(e) => {
                self.set_status(&format!("[ERROR] DB: {}", e));
            }
        }
        // Clear details if no scans or selection is gone
        if self.history_scans.is_empty() {
            self.history_selected = None;
            self.history_findings.clear();
            self.history_links.clear();
        }
    }

    // -----------------------------------------------------------------------
    // Input fields
    // -----------------------------------------------------------------------

    pub fn type_char(&mut self, c: char) {
        match self.tab {
            Tab::Scanner => self.scanner_target.push(c),
            Tab::Recon => self.recon_target.push(c),
            Tab::Crawler => self.crawler_target.push(c),
            Tab::Export if self.export_mode == ExportMode::Encrypted => self.export_password.push(c),
            Tab::Export => self.export_path.push(c),
            _ => {}
        }
    }

    pub fn backspace(&mut self) {
        match self.tab {
            Tab::Scanner => {
                self.scanner_target.pop();
            }
            Tab::Recon => {
                self.recon_target.pop();
            }
            Tab::Crawler => {
                self.crawler_target.pop();
            }
            Tab::Export if self.export_mode == ExportMode::Encrypted => {
                self.export_password.pop();
            }
            Tab::Export => {
                self.export_path.pop();
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Push log (deprecated — prefer BgEvent path, kept for compatibility)
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    pub fn push_log(&mut self, msg: &str, level: LogLevel) {
        let prefix = match level {
            LogLevel::Info => "",
            LogLevel::Success => "[+] ",
            LogLevel::Warning => "[!] ",
            LogLevel::Error => "[ERROR] ",
        };
        let entry = format!("{}{}", prefix, msg);

        let target = match self.tab {
            Tab::Scanner => &mut self.scanner_logs,
            Tab::Recon => &mut self.recon_logs,
            Tab::Crawler => &mut self.crawler_logs,
            _ => return,
        };

        if target.len() >= 2000 {
            target.pop_front();
        }
        target.push_back(entry);
    }

    // -----------------------------------------------------------------------
    // Severity helpers for the UI
    // -----------------------------------------------------------------------

    /// Count findings by severity.
    pub fn finding_severity_counts(&self) -> [(String, usize); 5] {
        let mut critical = 0usize;
        let mut high = 0usize;
        let mut medium = 0usize;
        let mut low = 0usize;
        let mut info = 0usize;

        for f in &self.history_findings {
            match f.severity.to_lowercase().as_str() {
                "critical" => critical += 1,
                "high" => high += 1,
                "medium" => medium += 1,
                "low" => low += 1,
                _ => info += 1,
            }
        }

        [
            ("CRITICAL".into(), critical),
            ("HIGH".into(), high),
            ("MEDIUM".into(), medium),
            ("LOW".into(), low),
            ("INFO".into(), info),
        ]
    }
}
