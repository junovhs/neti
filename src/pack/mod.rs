// src/pack/mod.rs
pub mod focus;
pub mod formats;

use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;

use crate::analysis::RuleEngine;
use crate::clipboard;
use crate::config::{Config, GitMode};
use crate::discovery;
use crate::prompt::PromptGenerator;
use crate::tokens::Tokenizer;

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Xml,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct PackOptions {
    pub stdout: bool,
    pub copy: bool,
    pub verbose: bool,
    pub prompt: bool,
    pub format: OutputFormat,
    pub skeleton: bool,
    pub git_only: bool,
    pub no_git: bool,
    pub code_only: bool,
    pub target: Option<PathBuf>,
    pub focus: Vec<PathBuf>,
    pub depth: usize,
}

/// Internal struct to pass focus information to format functions.
pub struct FocusContext {
    pub foveal: HashSet<PathBuf>,
    pub peripheral: HashSet<PathBuf>,
}

/// Entry point for the pack command.
///
/// # Errors
/// Returns error if configuration, discovery, or output fails.
pub fn run(options: &PackOptions) -> Result<()> {
    let config = setup_config(options)?;
    print_start_message(options);

    let files = discovery::discover(&config)?;
    if options.verbose {
        eprintln!("📦 Discovered {} files...", files.len());
    }

    let content = generate_content(&files, options, &config)?;
    let token_count = Tokenizer::count(&content);

    output_result(&content, token_count, options)
}

fn print_start_message(options: &PackOptions) {
    if options.stdout || options.copy {
        return;
    }
    if !options.focus.is_empty() {
        let names: Vec<_> = options
            .focus
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        println!("🔬 Packing with focus: {}", names.join(", "));
    } else if let Some(t) = &options.target {
        println!("🧶 Knitting repository (Focus: {})...", t.display());
    } else {
        println!("🧶 Knitting repository...");
    }
}

fn setup_config(opts: &PackOptions) -> Result<Config> {
    let mut config = Config::new();
    config.verbose = opts.verbose;
    config.code_only = opts.code_only;
    config.git_mode = match (opts.git_only, opts.no_git) {
        (true, _) => GitMode::Yes,
        (_, true) => GitMode::No,
        _ => GitMode::Auto,
    };
    config.load_local_config();
    config.validate()?;
    Ok(config)
}

/// Generates the context content string from a list of files.
///
/// # Errors
/// Returns error if file reading fails.
pub fn generate_content(files: &[PathBuf], opts: &PackOptions, config: &Config) -> Result<String> {
    let mut ctx = String::with_capacity(100_000);

    let (focus_ctx, pack_files) = build_focus_context(files, opts);

    if opts.prompt {
        write_header(&mut ctx, config)?;
        inject_violations(&mut ctx, files, config)?;
    }

    pack_files_to_output(&pack_files, &mut ctx, opts, &focus_ctx)?;

    if opts.prompt {
        write_footer(&mut ctx, config)?;
    }

    Ok(ctx)
}

fn build_focus_context(files: &[PathBuf], opts: &PackOptions) -> (FocusContext, Vec<PathBuf>) {
    if opts.focus.is_empty() {
        let ctx = FocusContext {
            foveal: HashSet::new(),
            peripheral: HashSet::new(),
        };
        return (ctx, files.to_vec());
    }

    let (foveal, peripheral) = focus::compute_sets(files, &opts.focus, opts.depth);
    let combined: Vec<_> = foveal.iter().chain(peripheral.iter()).cloned().collect();
    let ctx = FocusContext { foveal, peripheral };
    (ctx, combined)
}

fn pack_files_to_output(
    files: &[PathBuf],
    ctx: &mut String,
    opts: &PackOptions,
    focus: &FocusContext,
) -> Result<()> {
    match opts.format {
        OutputFormat::Text => formats::pack_slopchop_focus(files, ctx, opts, focus),
        OutputFormat::Xml => formats::pack_xml_focus(files, ctx, opts, focus),
    }
}

fn inject_violations(ctx: &mut String, files: &[PathBuf], config: &Config) -> Result<()> {
    let engine = RuleEngine::new(config.clone());
    let report = engine.scan(files.to_vec());

    if !report.has_errors() {
        return Ok(());
    }

    writeln!(ctx, "{}", "═".repeat(67))?;
    writeln!(ctx, "⚠️  ACTIVE VIOLATIONS (PRIORITY FIX REQUIRED)")?;
    writeln!(ctx, "{}\n", "═".repeat(67))?;

    for file in report.files.iter().filter(|f| !f.is_clean()) {
        for v in &file.violations {
            writeln!(ctx, "FILE: {}", file.path.display())?;
            writeln!(ctx, "LAW:  {} | LINE: {} | {}", v.law, v.row + 1, v.message)?;
            writeln!(ctx, "{}", "─".repeat(40))?;
        }
    }
    writeln!(ctx)?;
    Ok(())
}

fn write_header(ctx: &mut String, config: &Config) -> Result<()> {
    let gen = PromptGenerator::new(config.rules.clone());
    writeln!(ctx, "{}", gen.wrap_header()?)?;
    writeln!(
        ctx,
        "\n{}\nBEGIN CODEBASE\n{}\n",
        "═".repeat(67),
        "═".repeat(67)
    )?;
    Ok(())
}

fn write_footer(ctx: &mut String, config: &Config) -> Result<()> {
    let gen = PromptGenerator::new(config.rules.clone());
    writeln!(
        ctx,
        "\n{}\nEND CODEBASE\n{}\n",
        "═".repeat(67),
        "═".repeat(67)
    )?;
    writeln!(ctx, "{}", gen.generate_reminder()?)?;
    Ok(())
}

fn output_result(content: &str, tokens: usize, opts: &PackOptions) -> Result<()> {
    let info = format!(
        "\n📊 Context Size: {} tokens",
        tokens.to_string().yellow().bold()
    );

    if opts.stdout {
        print!("{content}");
        eprintln!("{info}");
        return Ok(());
    }

    if opts.copy {
        let msg = clipboard::smart_copy(content)?;
        println!("{}", "✓ Copied to clipboard".green());
        println!("  ({msg})");
        println!("{info}");
        return Ok(());
    }

    write_to_file(content, &info)
}

fn write_to_file(content: &str, info: &str) -> Result<()> {
    let output_path = PathBuf::from("context.txt");
    fs::write(&output_path, content)?;
    println!("✅ Generated 'context.txt'");

    if let Ok(abs) = fs::canonicalize(&output_path) {
        if clipboard::copy_file_path(&abs).is_ok() {
            println!("{}", "📎 File path copied to clipboard".cyan());
        }
    }
    println!("{info}");
    Ok(())
}
