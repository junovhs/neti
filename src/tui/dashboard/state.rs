// src/tui/dashboard/state.rs
use crate::config::Config;
use crate::roadmap_v2::types::TaskStore;
use crate::tui::config::state::ConfigApp;
use crate::types::ScanReport;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Roadmap,
    Config,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatusFilter {
    All,
    Pending,
    Done,
}

impl TaskStatusFilter {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Pending,
            Self::Pending => Self::Done,
            Self::Done => Self::All,
        }
    }
}

pub struct DashboardApp<'a> {
    pub config: &'a mut Config,
    pub active_tab: Tab,
    pub scan_report: Option<ScanReport>,
    pub roadmap: Option<TaskStore>,
    pub config_editor: ConfigApp,
    pub last_scan: Option<Instant>,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub scroll: u16,
    pub roadmap_scroll: u16,
    pub roadmap_filter: TaskStatusFilter,
    /// Pending payload from clipboard watcher, waiting for user confirmation
    pub pending_payload: Option<String>,
}

impl<'a> DashboardApp<'a> {
    pub fn new(config: &'a mut Config) -> Self {
        Self {
            config,
            active_tab: Tab::Dashboard,
            scan_report: None,
            roadmap: None,
            config_editor: ConfigApp::new(),
            last_scan: None,
            logs: vec!["SlopChop Dashboard initialized".to_string()],
            should_quit: false,
            scroll: 0,
            roadmap_scroll: 0,
            roadmap_filter: TaskStatusFilter::All,
            pending_payload: None,
        }
    }

    pub fn log(&mut self, message: &str) {
        let timestamp = chrono_lite_timestamp();
        self.logs.push(format!("[{timestamp}] {message}"));
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn on_tick(&mut self) {
        if self.active_tab == Tab::Dashboard {
            if let Some(last) = self.last_scan {
                if last.elapsed() > Duration::from_secs(30) {
                    // Could auto-refresh here, but let's keep it manual for now
                }
            }
        }
        self.config_editor.check_message_expiry();
    }

    pub fn trigger_scan(&mut self) {
        self.last_scan = Some(Instant::now());
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Dashboard => Tab::Roadmap,
            Tab::Roadmap => Tab::Config,
            Tab::Config => Tab::Logs,
            Tab::Logs => Tab::Dashboard,
        };
    }

    pub fn previous_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Dashboard => Tab::Logs,
            Tab::Logs => Tab::Config,
            Tab::Config => Tab::Roadmap,
            Tab::Roadmap => Tab::Dashboard,
        };
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    #[must_use]
    pub fn has_pending_payload(&self) -> bool {
        self.pending_payload.is_some()
    }
}

/// Simple timestamp without pulling in chrono crate
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Just show HH:MM:SS (approximate, good enough for logs)
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{hours:02}:{mins:02}:{secs:02}")
}