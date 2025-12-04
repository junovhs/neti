// src/wizard.rs
use crate::project::{self, ProjectType, Strictness};
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;

/// Runs the interactive configuration wizard.
///
/// # Errors
/// Returns error if IO fails or file writing fails.
pub fn run() -> Result<()> {
    println!("{}", "🧙 SlopChop Configuration Wizard".bold().cyan());
    println!("{}", "─────────────────────────────────────".dimmed());

    if Path::new("slopchop.toml").exists() {
        println!("{}", "⚠️  slopchop.toml already exists.".yellow());
        if !confirm("Overwrite it?")? {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    let project_type = prompt_project_type()?;
    let strictness = prompt_strictness()?;

    println!();
    println!("Generating configuration for:");
    println!("  Type:       {}", format!("{project_type:?}").green());
    println!("  Strictness: {}", format!("{strictness:?}").green());

    let content = project::generate_toml(project_type, strictness);
    std::fs::write("slopchop.toml", content)?;

    println!();
    println!(
        "{}",
        "✅ Configuration created successfully!".green().bold()
    );
    println!("Run {} to analyze your project.", "slopchop".yellow());

    Ok(())
}

fn prompt_project_type() -> Result<ProjectType> {
    let detected = ProjectType::detect();
    println!();
    println!("Detected Project Type: {}", format!("{detected:?}").cyan());

    if confirm("Is this correct?")? {
        return Ok(detected);
    }

    println!();
    println!("Select Project Type:");
    println!("1. Rust");
    println!("2. Node/TypeScript");
    println!("3. Python");
    println!("4. Go");

    loop {
        print!("Enter selection [1-4]: ");
        io::stdout().flush()?;

        let input = read_line()?;
        match input.trim() {
            "1" => return Ok(ProjectType::Rust),
            "2" => return Ok(ProjectType::Node),
            "3" => return Ok(ProjectType::Python),
            "4" => return Ok(ProjectType::Go),
            _ => println!("{}", "Invalid selection.".red()),
        }
    }
}

fn prompt_strictness() -> Result<Strictness> {
    println!();
    println!("Select Strictness Level:");
    println!(
        "{}",
        "1. Strict   (Greenfield) - 1500 tokens, Low Complexity".green()
    );
    println!(
        "{}",
        "2. Standard (Recommended)- 2000 tokens, Medium Complexity".cyan()
    );
    println!(
        "{}",
        "3. Relaxed  (Legacy)     - 3000 tokens, High Complexity".yellow()
    );

    loop {
        print!("Enter selection [1-3] (default: 2): ");
        io::stdout().flush()?;

        let input = read_line()?;
        if input.trim().is_empty() {
            return Ok(Strictness::Standard);
        }

        match input.trim() {
            "1" => return Ok(Strictness::Strict),
            "2" => return Ok(Strictness::Standard),
            "3" => return Ok(Strictness::Relaxed),
            _ => println!("{}", "Invalid selection.".red()),
        }
    }
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let input = read_line()?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}
