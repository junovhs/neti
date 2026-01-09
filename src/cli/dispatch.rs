// src/cli/dispatch.rs
//! Command dispatch logic extracted from binary to reduce main function size.

use super::{
    args::{ApplyArgs, Commands},
    audit::AuditCliOptions,
    handlers::{
        handle_abort, handle_apply, handle_branch, handle_check, handle_map, handle_pack,
        handle_promote, handle_scan, PackArgs,
    },
};
use crate::exit::SlopChopExit;
use crate::signatures::SignatureOptions;
use anyhow::{anyhow, Result};

/// Executes the parsed command.
///
/// # Errors
/// Returns error if the command handler fails.
pub fn execute(command: Commands) -> Result<SlopChopExit> {
    match command {
        Commands::Check { .. }
        | Commands::Scan { .. }
        | Commands::Audit { .. }
        | Commands::Map { .. }
        | Commands::Signatures { .. }
        | Commands::Mutate { .. } => handle_analysis(command),

        Commands::Branch { .. } | Commands::Promote { .. } | Commands::Abort => {
            handle_git_ops(&command)
        }

        Commands::Apply { .. }
        | Commands::Clean { .. }
        | Commands::Pack { .. }
        | Commands::Config => handle_core_ops(command),
    }
}

fn handle_analysis(command: Commands) -> Result<SlopChopExit> {
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
        Commands::Audit {
            format,
            no_dead,
            no_dups,
            no_patterns,
            min_lines,
            max,
            verbose,
        } => {
            let opts = AuditCliOptions {
                format: &format,
                no_dead,
                no_dups,
                no_patterns,
                min_lines,
                max,
                verbose,
            };
            super::audit::handle(&opts)?;
            Ok(SlopChopExit::Success)
        }
        Commands::Map { deps } => handle_map(deps),
        Commands::Signatures { copy, stdout } => {
            let opts = SignatureOptions { copy, stdout };
            super::handlers::handle_signatures(opts)
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

fn handle_git_ops(command: &Commands) -> Result<SlopChopExit> {
    match command {
        Commands::Branch { force } => handle_branch(*force),
        Commands::Promote { dry_run } => handle_promote(*dry_run),
        Commands::Abort => handle_abort(),
        _ => Err(anyhow!("Internal error: Invalid git command")),
    }
}

fn handle_core_ops(command: Commands) -> Result<SlopChopExit> {
    match command {
        Commands::Apply {
            force,
            dry_run,
            stdin,
            check,
            file,
            promote,
            sanitize,
            strict,
        } => {
            let args = ApplyArgs {
                force,
                dry_run,
                stdin,
                check,
                file,
                promote,
                sanitize,
                strict,
                reset: false,
                sync: false,
            };
            handle_apply(&args)
        }
        Commands::Clean { commit } => {
            crate::clean::run(commit)?;
            Ok(SlopChopExit::Success)
        }
        Commands::Pack {
            stdout,
            copy,
            noprompt,
            format,
            skeleton,
            code_only,
            verbose,
            target,
            focus,
            depth,
        } => {
            let args = PackArgs {
                stdout,
                copy,
                noprompt,
                format,
                skeleton,
                code_only,
                verbose,
                target,
                focus,
                depth,
            };
            handle_pack(args)
        }
        Commands::Config => {
            super::config_ui::run_config_editor()?;
            Ok(SlopChopExit::Success)
        }
        _ => Err(anyhow!("Internal error: Invalid core command")),
    }
}
