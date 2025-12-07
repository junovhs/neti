// src/tui/dashboard/mod.rs
pub mod state;
pub mod ui;

use crate::analysis::RuleEngine;
use crate::config::Config;
use crate::discovery;
use crate::roadmap_v2::types::TaskStore;
use crate::tui::runner;
use crate::tui::watcher::{self, WatcherEvent};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use state::{DashboardApp, Tab};
use std::io;
use std::sync::mpsc;
use std::time::Duration;

/// Runs the TUI dashboard.
///
/// # Errors
/// Returns error if terminal setup fails or during execution.
pub fn run(config: &mut Config) -> Result<()> {
    runner::setup_terminal()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let result = run_app(&mut terminal, config);

    runner::restore_terminal()?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: &mut Config,
) -> Result<()> {
    let mut app = DashboardApp::new(config);

    // Initial scan
    run_scan(&mut app);
    load_roadmap(&mut app);

    // Set up watcher channel
    let (tx, rx) = mpsc::channel();
    watcher::spawn_watcher(tx);
    app.log("Clipboard watcher started");

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Check for watcher events (non-blocking)
        if let Ok(event) = rx.try_recv() {
            handle_watcher_event(&mut app, event);
        }

        // Poll for keyboard input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_input(key.code, key.modifiers, &mut app);
            }
        }

        // Periodic tick
        app.on_tick();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_input(code: KeyCode, modifiers: KeyModifiers, app: &mut DashboardApp) {
    if handle_global_navigation(code, modifiers, app) {
        return;
    }

    if handle_actions(code, modifiers, app) {
        return;
    }

    if app.active_tab == Tab::Config && handle_config_input(code, app) {
        return;
    }

    handle_scrolling(code, app);
}

fn handle_global_navigation(
    code: KeyCode,
    modifiers: KeyModifiers,
    app: &mut DashboardApp,
) -> bool {
    match (modifiers, code) {
        (_, KeyCode::Char('q')) => {
            app.quit();
            true
        }
        (_, KeyCode::Tab) => {
            app.next_tab();
            true
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            app.previous_tab();
            true
        }
        (_, KeyCode::Char('1')) => {
            app.active_tab = Tab::Dashboard;
            true
        }
        (_, KeyCode::Char('2')) => {
            app.active_tab = Tab::Roadmap;
            true
        }
        (_, KeyCode::Char('3')) => {
            app.active_tab = Tab::Config;
            true
        }
        (_, KeyCode::Char('4')) => {
            app.active_tab = Tab::Logs;
            true
        }
        _ => false,
    }
}

fn handle_actions(code: KeyCode, modifiers: KeyModifiers, app: &mut DashboardApp) -> bool {
    match (modifiers, code) {
        (_, KeyCode::Char('r')) => {
            refresh_app(app);
            true
        }
        (_, KeyCode::Char('f')) => {
            app.roadmap_filter = app.roadmap_filter.next();
            true
        }
        (KeyModifiers::CONTROL, KeyCode::Enter) | (_, KeyCode::Char('a')) => {
            if app.pending_payload.is_some() {
                handle_apply(app);
            }
            true
        }
        (_, KeyCode::Esc) => {
            if app.pending_payload.is_some() {
                app.pending_payload = None;
                app.log("Payload dismissed");
            }
            true
        }
        _ => false,
    }
}

fn handle_config_input(code: KeyCode, app: &mut DashboardApp) -> bool {
    match code {
        KeyCode::Left | KeyCode::Char('h') => {
            crate::tui::config::helpers::adjust_rule(&mut app.config_editor, false);
            true
        }
        KeyCode::Right | KeyCode::Char('l') => {
            crate::tui::config::helpers::adjust_rule(&mut app.config_editor, true);
            true
        }
        KeyCode::Char('s') => {
            app.config_editor.save();
            app.log("Config saved");
            true
        }
        _ => false,
    }
}

fn handle_scrolling(code: KeyCode, app: &mut DashboardApp) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => handle_scroll_down(app),
        KeyCode::Up | KeyCode::Char('k') => handle_scroll_up(app),
        _ => {}
    }
}

fn refresh_app(app: &mut DashboardApp) {
    run_scan(app);
    load_roadmap(app);
    app.log("Refreshed");
}

fn handle_watcher_event(app: &mut DashboardApp, event: WatcherEvent) {
    match event {
        WatcherEvent::PayloadDetected(content) => {
            let file_count = content.matches("#__SLOPCHOP_FILE__#").count();
            app.pending_payload = Some(content);
            app.log(&format!(
                "📋 Payload detected: {file_count} file(s). Press 'a' or Ctrl+Enter to apply, Esc to dismiss"
            ));
        }
    }
}

fn handle_apply(app: &mut DashboardApp) {
    let Some(payload) = app.pending_payload.take() else {
        return;
    };

    app.log("Applying payload...");

    // We can access app.config directly here, avoiding the borrow checker issue
    let ctx = crate::apply::types::ApplyContext::new(app.config);

    match crate::apply::process_input(&payload, &ctx) {
        Ok(outcome) => match &outcome {
            crate::apply::types::ApplyOutcome::Success { written, .. } => {
                app.log(&format!("✅ Applied {} file(s)", written.len()));
            }
            crate::apply::types::ApplyOutcome::ValidationFailure { errors, .. } => {
                app.log(&format!("❌ Validation failed: {} error(s)", errors.len()));
                for e in errors.iter().take(3) {
                    app.log(&format!("   - {e}"));
                }
            }
            crate::apply::types::ApplyOutcome::ParseError(e) => {
                app.log(&format!("⚠️ Parse error: {e}"));
            }
            crate::apply::types::ApplyOutcome::WriteError(e) => {
                app.log(&format!("💥 Write error: {e}"));
            }
        },
        Err(e) => {
            app.log(&format!("💥 Apply failed: {e}"));
        }
    }
}

fn handle_scroll_down(app: &mut DashboardApp) {
    match app.active_tab {
        Tab::Roadmap => app.roadmap_scroll = app.roadmap_scroll.saturating_add(1),
        Tab::Logs => app.scroll = app.scroll.saturating_add(1),
        Tab::Config => {
            app.config_editor.selected_field = (app.config_editor.selected_field + 1).min(11);
        }
        Tab::Dashboard => {}
    }
}

fn handle_scroll_up(app: &mut DashboardApp) {
    match app.active_tab {
        Tab::Roadmap => app.roadmap_scroll = app.roadmap_scroll.saturating_sub(1),
        Tab::Logs => app.scroll = app.scroll.saturating_sub(1),
        Tab::Config => {
            app.config_editor.selected_field = app.config_editor.selected_field.saturating_sub(1);
        }
        Tab::Dashboard => {}
    }
}

fn run_scan(app: &mut DashboardApp) {
    // Access app.config directly.
    let files = match discovery::discover(app.config) {
        Ok(f) => f,
        Err(e) => {
            app.log(&format!("Scan failed: {e}"));
            return;
        }
    };

    let engine = RuleEngine::new(app.config.clone());
    let report = engine.scan(files);
    app.scan_report = Some(report);
    app.trigger_scan();
}

fn load_roadmap(app: &mut DashboardApp) {
    match TaskStore::load(None) {
        Ok(store) => app.roadmap = Some(store),
        Err(_) => app.roadmap = None,
    }
}