// src/tui/dashboard/mod.rs
pub mod state;
pub mod ui;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use state::DashboardApp;
use std::path::Path;
use std::time::Duration;
use crate::apply::{self, types::ApplyContext};
use crate::roadmap::{cmd_handlers, Roadmap, TaskStatus};
use crate::tui::runner::{spawn_checks, CheckEvent};
use crate::tui::watcher::{spawn_watcher, WatcherEvent};

pub fn run<B: ratatui::backend::Backend>(terminal: &mut ratatui::Terminal<B>) -> Result<()> {
    let mut app = DashboardApp::new()?;
    spawn_watcher(app.watch_tx.clone());

    while app.running {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        handle_event(&mut app)?;
        process_worker_messages(&mut app);
        process_watcher_messages(&mut app);
    }
    Ok(())
}

fn handle_event(app: &mut DashboardApp) -> Result<()> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            process_key(app, key.code);
        }
    }
    app.config.check_message_expiry();
    Ok(())
}

fn process_worker_messages(app: &mut DashboardApp) {
    while let Ok(msg) = app.check_rx.try_recv() {
        match msg {
            CheckEvent::Log(line) => {
                app.check_logs.push(line);
            }
            CheckEvent::Finished(_) => {
                app.check_running = false;
                app.check_logs.push("--- Finished ---".into());
            }
        }
    }
}

fn process_watcher_messages(app: &mut DashboardApp) {
    while let Ok(msg) = app.watch_rx.try_recv() {
        match msg {
            WatcherEvent::PayloadDetected(content) => {
                app.pending_payload = Some(content);
                app.show_popup = true;
                app.log_system("Payload detected in clipboard.");
            }
        }
    }
}

fn process_key(app: &mut DashboardApp, key: KeyCode) {
    if app.show_popup {
        handle_popup_input(app, key);
        return;
    }

    if handle_system(app, key) {
        return;
    }
    if handle_navigation(app, key) {
        return;
    }
    route_tab_input(app, key);
}

fn handle_popup_input(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(content) = app.pending_payload.clone() {
                apply_payload(app, &content);
            }
            app.show_popup = false;
            app.pending_payload = None;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.show_popup = false;
            app.pending_payload = None;
            app.log_system("Payload discarded.");
        }
        _ => {}
    }
}

fn apply_payload(app: &mut DashboardApp, content: &str) {
    app.log_system("Applying payload...");
    let mut config = crate::config::Config::new();
    config.load_local_config();
    
    let ctx = ApplyContext { config: &config, force: true, dry_run: false };

    match apply::process_input(content, &ctx) {
        Ok(outcome) => {
            app.log_system(format!("Apply complete: {outcome:?}"));
            // Reload roadmap as it might have changed
            app.roadmap = Roadmap::from_file(Path::new("ROADMAP.md")).ok();
            app.refresh_flat_roadmap();
            
            app.switch_tab(state::Tab::Checks);
            app.check_running = true;
            app.check_logs.clear();
            app.check_logs.push("Running verification...".into());
            spawn_checks(app.check_tx.clone());
        }
        Err(e) => app.log_system(format!("Apply failed: {e}")),
    }
}

fn handle_system(app: &mut DashboardApp, key: KeyCode) -> bool {
    if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
        app.running = false;
        return true;
    }
    false
}

fn handle_navigation(app: &mut DashboardApp, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('1') => app.switch_tab(state::Tab::Roadmap),
        KeyCode::Char('2') => app.switch_tab(state::Tab::Checks),
        KeyCode::Char('3') => app.switch_tab(state::Tab::Context),
        KeyCode::Char('4') => app.switch_tab(state::Tab::Config),
        KeyCode::Char('5') => app.switch_tab(state::Tab::Logs),
        _ => return false,
    }
    true
}

fn route_tab_input(app: &mut DashboardApp, key: KeyCode) {
    match app.active_tab {
        state::Tab::Config => app.config.handle_input(key),
        state::Tab::Checks => handle_checks_input(app, key),
        state::Tab::Roadmap => handle_roadmap_input(app, key),
        _ => {}
    }
}

fn handle_checks_input(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char('r') if !app.check_running => {
            app.check_running = true;
            app.check_logs.clear();
            app.check_logs.push("Starting checks...".into());
            spawn_checks(app.check_tx.clone());
        }
        KeyCode::Char('c') => {
            app.check_logs.clear();
            app.log_system("Logs cleared.");
        }
        _ => handle_scroll(app, key),
    }
}

fn handle_roadmap_input(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char(' ') | KeyCode::Enter => toggle_roadmap_task(app),
        _ => handle_scroll(app, key),
    }
}

fn handle_scroll(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
        _ => {}
    }
}

fn toggle_roadmap_task(app: &mut DashboardApp) {
    let Some(item) = app.flat_roadmap.get(app.selected_task) else { return; };
    if item.is_header { return; }

    let path = item.id.clone();
    let new_status = if item.status == TaskStatus::Pending {
        TaskStatus::Complete
    } else {
        TaskStatus::Pending
    };

    if update_roadmap_task(app, &path, new_status) {
        app.log_system(format!("Updated task: {path}"));
        app.refresh_flat_roadmap();
    } else {
        app.log_system(format!("Failed to update {path}"));
    }
}

fn update_roadmap_task(app: &mut DashboardApp, path: &str, status: TaskStatus) -> bool {
    let Some(ref mut roadmap) = app.roadmap else { return false; };
    
    let res = match status {
        TaskStatus::Complete => cmd_handlers::handle_check(roadmap, path),
        TaskStatus::Pending => cmd_handlers::handle_uncheck(roadmap, path),
    };

    if matches!(res, crate::roadmap::ApplyResult::Success(_)) {
        let _ = roadmap.save(Path::new("ROADMAP.md"));
        
        // CRITICAL FIX: The UI depends on `sections` structure, but commands modify `raw` string.
        // We must re-parse `raw` to update `sections` so the UI reflects the change.
        *roadmap = Roadmap::parse(&roadmap.raw);
        
        return true;
    }
    false
}