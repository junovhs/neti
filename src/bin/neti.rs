use clap::Parser;
use colored::Colorize;
use neti_core::cli::{self, Cli};
use neti_core::exit::NetiExit;

fn main() -> NetiExit {
    let cli = Cli::parse();

    let result = if let Some(cmd) = cli.command {
        cli::dispatch::execute(cmd)
    } else {
        use clap::CommandFactory;
        let _ = Cli::command().print_help();
        Ok(NetiExit::Success)
    };

    match result {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            NetiExit::Error
        }
    }
}
