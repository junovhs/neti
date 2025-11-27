// src/bin/knit.rs
use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use warden_core::clipboard;
use warden_core::config::{Config, GitMode};
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;
use warden_core::prompt::PromptGenerator;
use warden_core::tokens::Tokenizer;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Xml,
}

#[derive(Parser)]
#[command(name = "knit")]
#[command(about = "Stitches atomic files into a single context file.")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[arg(long, short)]
    stdout: bool,
    #[arg(long, short)]
    copy: bool, // <--- NEW FLAG
    #[arg(long, short)]
    verbose: bool,
    #[arg(long)]
    git_only: bool,
    #[arg(long)]
    no_git: bool,
    #[arg(long)]
    code_only: bool,
    #[arg(long, short)]
    prompt: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = setup_config(&cli)?;

    if !cli.stdout && !cli.copy {
        println!("🧶 Knitting repository...");
    }

    let files = discover_files(&config, cli.verbose)?;
    let content = generate_content(&files, &cli, &config)?;
    let token_count = Tokenizer::count(&content);

    output_result(&content, token_count, &cli)
}

fn setup_config(cli: &Cli) -> Result<Config> {
    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;
    config.git_mode = if cli.git_only {
        GitMode::Yes
    } else if cli.no_git {
        GitMode::No
    } else {
        GitMode::Auto
    };
    config.load_local_config();
    config.validate()?;
    Ok(config)
}

fn discover_files(config: &Config, verbose: bool) -> Result<Vec<PathBuf>> {
    let raw = FileEnumerator::new(config.clone()).enumerate()?;
    let h_files = HeuristicFilter::new().filter(raw);
    let t_files = FileFilter::new(config)?.filter(h_files);

    if verbose {
        eprintln!("📦 Packing {} files...", t_files.len());
    }
    Ok(t_files)
}

fn generate_content(files: &[PathBuf], cli: &Cli, config: &Config) -> Result<String> {
    let mut ctx = String::with_capacity(100_000);

    if cli.prompt {
        write_header(&mut ctx, config)?;
    }

    write_body(files, &mut ctx, &cli.format)?;

    if cli.prompt {
        write_footer(&mut ctx, config)?;
    }

    Ok(ctx)
}

fn write_body(files: &[PathBuf], ctx: &mut String, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => pack_text(files, ctx),
        OutputFormat::Xml => pack_xml(files, ctx),
    }
}

fn write_header(ctx: &mut String, config: &Config) -> Result<()> {
    let gen = PromptGenerator::new(config.rules.clone());
    writeln!(ctx, "{}", gen.wrap_header()?)?;
    writeln!(ctx, "\n═══════════════════════════════════════════════════════════════════\nBEGIN CODEBASE\n═══════════════════════════════════════════════════════════════════\n")?;
    Ok(())
}

fn write_footer(ctx: &mut String, config: &Config) -> Result<()> {
    let gen = PromptGenerator::new(config.rules.clone());
    writeln!(ctx, "\n═══════════════════════════════════════════════════════════════════\nEND CODEBASE\n═══════════════════════════════════════════════════════════════════\n")?;
    writeln!(ctx, "{}", gen.generate_reminder()?)?;
    Ok(())
}

fn output_result(content: &str, tokens: usize, cli: &Cli) -> Result<()> {
    let info = format!(
        "\n📊 Context Size: {} tokens",
        tokens.to_string().yellow().bold()
    );

    if cli.stdout {
        print!("{content}");
        eprintln!("{info}");
        return Ok(());
    }

    if cli.copy {
        let msg = clipboard::smart_copy(content)?;
        println!("{}", "✓ Copied to clipboard".green());
        println!("  ({msg})"); // Shows "Copied as file attachment..." if large
        println!("{info}");
        return Ok(());
    }

    // Default: Write to file
    fs::write("context.txt", content)?;
    println!("✅ Generated 'context.txt'");
    println!("{info}");
    Ok(())
}

fn pack_text(files: &[PathBuf], out: &mut String) -> Result<()> {
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "<file path=\"{p_str}\">")?;
        match fs::read_to_string(path) {
            Ok(c) => out.push_str(&c),
            Err(e) => writeln!(out, "<ERROR READING FILE: {e}>")?,
        }
        writeln!(out, "</file>\n")?;
    }
    Ok(())
}

fn pack_xml(files: &[PathBuf], out: &mut String) -> Result<()> {
    writeln!(out, "<documents>")?;
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "  <document path=\"{p_str}\"><![CDATA[")?;
        match fs::read_to_string(path) {
            Ok(c) => out.push_str(&c.replace("]]>", "]]]]><![CDATA[>")),
            Err(e) => writeln!(out, "ERROR READING FILE: {e}")?,
        }
        writeln!(out, "]]></document>")?;
    }
    writeln!(out, "</documents>")?;
    Ok(())
}
