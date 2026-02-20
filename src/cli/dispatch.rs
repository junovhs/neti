//! Command dispatch logic extracted from binary to reduce main function size.

use super::{
    args::Commands,
    git_ops::{handle_abort, handle_branch, handle_promote},
    handlers::{handle_check, handle_scan},
};
use crate::exit::NetiExit;
use anyhow::{anyhow, Result};

/// Executes the parsed command.
///
/// # Errors
/// Returns error if the command handler fails.
pub fn execute(command: Commands) -> Result<NetiExit> {
    match command {
        Commands::Check { .. } | Commands::Scan { .. } | Commands::Mutate { .. } => {
            handle_analysis(command)
        }

        Commands::Branch { .. } | Commands::Promote { .. } | Commands::Abort => {
            handle_git_ops(&command)
        }

        Commands::Clean { .. } | Commands::Config => handle_core_ops(&command),
    }
}

fn handle_analysis(command: Commands) -> Result<NetiExit> {
    match command {
        Commands::Check { json } => handle_check(json),
        Commands::Scan {
            verbose,
            locality,
            json,
        } => {
            if locality {
                return super::locality::handle_locality();
            }
            handle_scan(verbose, false, json)
        }
        Commands::Mutate {
            workers,
            timeout,
            json,
            filter,
        } => super::mutate_handler::handle_mutate(workers, timeout, json, filter),
        _ => Err(anyhow!("Internal error: Invalid analysis command")),
    }
}

fn handle_git_ops(command: &Commands) -> Result<NetiExit> {
    match command {
        Commands::Branch { force } => handle_branch(*force),
        Commands::Promote { dry_run } => handle_promote(*dry_run),
        Commands::Abort => handle_abort(),
        _ => Err(anyhow!("Internal error: Invalid git command")),
    }
}

fn handle_core_ops(command: &Commands) -> Result<NetiExit> {
    match command {
        Commands::Clean { commit } => {
            crate::clean::run(*commit)?;
            Ok(NetiExit::Success)
        }
        Commands::Config => {
            super::config_ui::run_config_editor()?;
            Ok(NetiExit::Success)
        }
        _ => Err(anyhow!("Internal error: Invalid core command")),
    }
}
