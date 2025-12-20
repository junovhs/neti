// src/tui/dashboard/apply.rs
use super::state::DashboardApp;
use crate::apply::types::ApplyOutcome;
use crate::tui::runner;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;

pub fn handle_interactive_apply(
    app: &mut DashboardApp,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) {
    let Some(payload) = app.pending_payload.take() else {
        return;
    };

    if let Err(e) = runner::restore_terminal() {
        app.log(&format!("Failed to restore terminal: {e}"));
        return;
    }

    println!("\n🚀 SlopChop Interactive Mode\n");
    let repo_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let ctx = crate::apply::types::ApplyContext::new(app.config, repo_root);

    match crate::apply::process_input(&payload, &ctx) {
        Ok(outcome) => print_outcome(&outcome, app),
        Err(e) => {
            println!("\n💥 Error: {e}");
            app.log(&format!("💥 Apply failed: {e}"));
        }
    }

    println!("\nPress Enter to return to dashboard...");
    let _ = std::io::stdin().read_line(&mut String::new());

    if let Err(e) = runner::setup_terminal() {
        eprintln!("Failed to restart terminal: {e}");
        std::process::exit(1);
    }
    if let Err(e) = terminal.clear() {
        app.log(&format!("Failed to clear terminal: {e}"));
    }
}

fn print_outcome(outcome: &ApplyOutcome, app: &mut DashboardApp) {
    match outcome {
        ApplyOutcome::Success {
            written, staged, ..
        } => {
            if *staged {
                println!("\n✅ Staged {} file(s)", written.len());
                app.log(&format!("✅ Staged {} file(s)", written.len()));
            } else {
                println!("\n✅ Applied {} file(s)", written.len());
                app.log(&format!("✅ Applied {} file(s)", written.len()));
            }
        }
        ApplyOutcome::Promoted { written, deleted } => {
            println!(
                "\n✅ Promoted {} file(s), deleted {}",
                written.len(),
                deleted.len()
            );
            app.log(&format!("✅ Promoted {} file(s)", written.len()));
        }
        ApplyOutcome::StageReset => {
            println!("\n✅ Stage reset");
            app.log("✅ Stage reset");
        }
        ApplyOutcome::ValidationFailure { errors, .. } => {
            app.log(&format!("❌ Validation failed: {} error(s)", errors.len()));
        }
        ApplyOutcome::ParseError(e) => {
            app.log(&format!("⚠️ Parse error: {e}"));
        }
        ApplyOutcome::WriteError(e) => {
            app.log(&format!("💥 Write error: {e}"));
        }
    }
}
