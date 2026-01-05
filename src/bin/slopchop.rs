// src/bin/slopchop.rs
use anyhow::Result;
use clap::Parser;
use slopchop_core::clean;
use slopchop_core::cli::args::ApplyArgs;
use slopchop_core::cli::audit::AuditCliOptions;
use slopchop_core::cli::{
    handle_apply, handle_check, handle_map, handle_pack, handle_scan, handle_signatures, Cli,
    Commands, PackArgs,
};
use slopchop_core::exit::SlopChopExit;
use slopchop_core::signatures::SignatureOptions;

fn main() {
    let exit_code = match run() {
        Ok(exit) => exit.code(),
        Err(e) => {
            eprintln!("Error: {e:?}");
            SlopChopExit::Error.code()
        }
    };
    std::process::exit(exit_code);
}

fn run() -> Result<SlopChopExit> {
    let cli = Cli::parse();

    match cli.command {
        None => handle_scan(false, false, false),
        Some(cmd) => dispatch(cmd),
    }
}

fn dispatch(cmd: Commands) -> Result<SlopChopExit> {
    match cmd {
        Commands::Check { json } => handle_check(json),

        Commands::Scan {
            verbose,
            locality,
            json,
        } => handle_scan(verbose, locality, json),

        Commands::Apply {
            force,
            dry_run,
            stdin,
            check,
            file,
            reset,
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
                reset,
                promote,
                sanitize,
                strict,
            };
            handle_apply(&args)
        }

        Commands::Clean { commit } => {
            clean::run(commit)?;
            Ok(SlopChopExit::Success)
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
            slopchop_core::cli::audit::handle(&opts)?;
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

        Commands::Map { deps } => handle_map(deps),

        Commands::Signatures { copy, stdout } => {
            let opts = SignatureOptions { copy, stdout };
            handle_signatures(opts)
        }

        Commands::Config => slopchop_core::cli::handlers::handle_config(),

        Commands::Sabotage { file } => slopchop_core::cli::handlers::handle_sabotage(&file),
    }
}
