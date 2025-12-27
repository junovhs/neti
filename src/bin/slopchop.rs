use clap::Parser;
use slopchop_core::cli::{self, args::{Cli, Commands}};
use slopchop_core::exit::SlopChopExit;
use std::process::{ExitCode, Termination};

fn main() -> ExitCode {
    let args = Cli::parse();

    let result = match args.command {
        Some(Commands::Prompt { copy }) => cli::handle_prompt(copy),
        Some(Commands::Check) => cli::handle_check(),
        Some(Commands::Fix) => cli::handle_fix(),
        Some(Commands::Scan { verbose }) => cli::handle_scan(verbose),
        Some(Commands::Apply {
            force,
            dry_run,
            stdin,
            check,
            file,
            reset,
            promote,
            sanitize,
            strict,
        }) => cli::handle_apply(&slopchop_core::cli::args::ApplyArgs {
            force,
            dry_run,
            stdin,
            check,
            file,
            reset,
            promote,
            sanitize,
            strict,
        }),
        Some(Commands::Clean { commit }) => slopchop_core::clean::run(commit).map(|()| SlopChopExit::Success),
        Some(Commands::Config) => slopchop_core::tui::run_config().map(|()| SlopChopExit::Success),
        Some(Commands::Dashboard) => cli::handle_dashboard(),
        Some(Commands::Audit {
            format,
            no_dead,
            no_dups,
            no_patterns,
            min_lines,
            max,
            verbose,
        }) => cli::handle_audit(&slopchop_core::cli::audit::AuditCliOptions {
            format: &format,
            no_dead,
            no_dups,
            no_patterns,
            min_lines,
            max,
            verbose,
        }).map(|()| SlopChopExit::Success),
        Some(Commands::Pack {
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
        }) => cli::handle_pack(slopchop_core::cli::handlers::PackArgs {
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
        }),
        Some(Commands::Trace { file, depth, budget }) => cli::handle_trace(&file, depth, budget),
        Some(Commands::Map { deps }) => cli::handle_map(deps),
        Some(Commands::Signatures { copy, stdout }) => cli::handle_signatures(slopchop_core::signatures::SignatureOptions { copy, stdout }),
        None => {
            if args.ui {
                cli::handle_dashboard()
            } else {
                use clap::CommandFactory;
                let _ = Cli::command().print_help();
                println!(); // new line
                Ok(SlopChopExit::Success)
            }
        },
    };

    match result {
        Ok(exit) => exit.report(),
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}