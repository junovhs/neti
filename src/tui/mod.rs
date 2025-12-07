// src/tui/mod.rs
pub mod config;
pub mod dashboard;
pub mod runner;
pub mod state;
pub mod view;
pub mod watcher;

use crate::config::Config;
use anyhow::Result;

/// Runs the TUI dashboard (the main entry point).
///
/// # Errors
/// Returns error if TUI execution fails or IO error occurs.
pub fn run(config: &mut Config) -> Result<()> {
    dashboard::run(config)
}

/// Runs the standalone configuration TUI.
///
/// Note: Config editing is also available as Tab 3 in the dashboard.
///
/// # Errors
/// Returns error if TUI setup or execution fails.
pub fn run_config() -> Result<()> {
    runner::setup_terminal()?;

    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;

    let mut app = config::state::ConfigApp::new();
    let res = app.run(&mut terminal);

    runner::restore_terminal()?;

    res
}
