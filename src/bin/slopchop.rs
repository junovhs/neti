// src/bin/slopchop.rs
use anyhow::Result;
use clap::Parser;
use slopchop_core::cli::{
    handle_apply, handle_audit, handle_check, handle_map, handle_pack, handle_scan,
    handle_signatures, Cli, Commands, PackArgs,
};
use slopchop_core::cli::args::ApplyArgs;
use slopchop_core::clean;
use slopchop_core::config::Config;
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
        None => {
            // Default behavior: show help or run scan
            handle_scan(false, false)
        }
        Some(cmd) => dispatch(cmd),
    }
}

fn dispatch(cmd: Commands) -> Result<SlopChopExit> {
    match cmd {
        Commands::Check => handle_check(),

        Commands::Scan { verbose, locality } => handle_scan(verbose, locality),

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

        Commands::Config => {
            let config = Config::load();
            println!("{}", toml::to_string_pretty(&config)?);
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
        } => handle_audit(format, no_dead, no_dups, no_patterns, min_lines, max, verbose),

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
    }
}
