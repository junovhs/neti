// src/pack/mod.rs
pub mod docs;
pub mod focus;
pub mod formats;
pub mod xml_format;

use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;

use crate::analysis::RuleEngine;
use crate::clipboard;
use crate::config::Config;
use crate::discovery;
use crate::prompt::PromptGenerator;
use crate::tokens::Tokenizer;

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Xml,
    /// Markdown specification generated from doc comments
    Spec,
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
    pub code_only: bool,
    pub target: Option<PathBuf>,
    pub focus: Vec<PathBuf>,
    pub depth: usize,
}

pub struct FocusContext {
    pub foveal: HashSet<PathBuf>,
    pub peripheral: HashSet<PathBuf>,
}

/// Runs the pack command to generate context.
///
/// # Errors
/// Returns an error if configuration loading, directory detection, or file writing fails.
pub fn run(options: &PackOptions) -> Result<()> {
    run_with_progress(options, |_, _, _| {})
}

/// Runs the pack command with progress reporting.
pub fn run_with_progress<F>(options: &PackOptions, on_progress: F) -> Result<()>
where
    F: Fn(usize, usize, &str) + Sync,
{
    let config = setup_config(options)?;

    print_start_message(options, &config);

    on_progress(0, 0, "Discovering files...");
    let files = discovery::discover(&config)?;
    if options.verbose {
        eprintln!("📂 Discovered {} files in workspace...", files.len());
    }

    on_progress(0, files.len(), "Generating content...");
    let content = generate_content(&files, options, &config, &on_progress)?;

    on_progress(files.len(), files.len(), "Counting tokens...");
    let token_count = Tokenizer::count(&content);
    output_result(&content, token_count, options, &config)
}

fn print_start_message(options: &PackOptions, config: &Config) {
    if options.stdout { return; }
    if options.copy || config.preferences.auto_copy { return; }

    if options.focus.is_empty() {
        println!("{} 🧶 Knitting repository...", "[WORKSPACE MODE]".cyan());
    } else {
        let names: Vec<_> = options.focus.iter().map(|p| p.display().to_string()).collect();
        println!("{} 🎯 Packing focus: {}", "[WORKSPACE MODE]".cyan(), names.join(", "));
    }
}

fn setup_config(opts: &PackOptions) -> Result<Config> {
    let mut config = Config::load();
    config.verbose = opts.verbose;
    config.code_only = opts.code_only;
    config.validate()?;
    Ok(config)
}

/// Generates the packed context content.
///
/// # Errors
/// Returns an error if generating prompt headers or writing file blocks fails.
pub fn generate_content<F>(
    files: &[PathBuf],
    opts: &PackOptions,
    config: &Config,
    on_progress: &F,
) -> Result<String>
where
    F: Fn(usize, usize, &str) + Sync,
{
    let mut ctx = String::with_capacity(100_000);
    let (focus_ctx, pack_files) = build_focus_context(files, opts);

    if opts.prompt {
        write_header(&mut ctx, config)?;
        inject_violations(&mut ctx, files, config)?;
    }

    pack_files_to_output(&pack_files, &mut ctx, opts, &focus_ctx, on_progress)?;

    if opts.prompt {
        write_footer(&mut ctx, config)?;
    }

    Ok(ctx)
}

fn build_focus_context(files: &[PathBuf], opts: &PackOptions) -> (FocusContext, Vec<PathBuf>) {
    if opts.focus.is_empty() {
        let ctx = FocusContext { foveal: HashSet::new(), peripheral: HashSet::new() };
        return (ctx, files.to_vec());
    }

    let (foveal, peripheral) = focus::compute_sets(files, &opts.focus, opts.depth);
    let combined: Vec<_> = foveal.iter().chain(peripheral.iter()).cloned().collect();
    (FocusContext { foveal, peripheral }, combined)
}

fn pack_files_to_output<F>(
    files: &[PathBuf],
    ctx: &mut String,
    opts: &PackOptions,
    focus: &FocusContext,
    on_progress: &F,
) -> Result<()>
where
    F: Fn(usize, usize, &str) + Sync,
{
    match opts.format {
        OutputFormat::Text => formats::pack_slopchop_focus(files, ctx, opts, focus, on_progress),
        OutputFormat::Xml => xml_format::pack_xml_focus(files, ctx, opts, focus, on_progress),
        OutputFormat::Spec => {
            let spec = formats::pack_spec(files)?;
            ctx.push_str(&spec);
            Ok(())
        },
    }
}

fn inject_violations(ctx: &mut String, files: &[PathBuf], config: &Config) -> Result<()> {
    let engine = RuleEngine::new(config.clone());
    let report = engine.scan(files);
    if !report.has_errors() { return Ok(()); }

    writeln!(ctx, "{}\n⚠ ACTIVE VIOLATIONS\n{}\n", "=".repeat(67), "=".repeat(67))?;
    for file in report.files.iter().filter(|f| !f.is_clean()) {
        for v in &file.violations {
            writeln!(ctx, "FILE: {} | LAW: {} | LINE: {} | {}", file.path.display(), v.law, v.row + 1, v.message)?;
        }
    }
    writeln!(ctx)?;
    Ok(())
}

fn write_header(ctx: &mut String, config: &Config) -> Result<()> {
    let gen = PromptGenerator::new(config.rules.clone());
    writeln!(ctx, "{}\nBEGIN CODEBASE\n", gen.wrap_header()?)?;
    Ok(())
}

fn write_footer(ctx: &mut String, config: &Config) -> Result<()> {
    let gen = PromptGenerator::new(config.rules.clone());
    writeln!(ctx, "\nEND CODEBASE\n{}", gen.generate_reminder()?)?;
    Ok(())
}

fn output_result(content: &str, tokens: usize, opts: &PackOptions, config: &Config) -> Result<()> {
    let info = format!("\n📊 Context Size: {} tokens", tokens.to_string().yellow().bold());

    if opts.stdout {
        print!("{content}");
        eprintln!("{info}");
        return Ok(());
    }

    if opts.copy || config.preferences.auto_copy {
        match clipboard::smart_copy(content) {
            Ok(msg) => {
                println!("{} ({msg}){info}", "📋 Copied to clipboard".green());
                return Ok(());
            }
            Err(e) => {
                eprintln!("{} Failed to copy to clipboard: {e}", "⚠".yellow());
                eprintln!("Falling back to file output...");
            }
        }
    }

    let output_path = PathBuf::from("context.txt");
    fs::write(&output_path, content)?;
    println!("✅ Generated 'context.txt'{info}");
    Ok(())
}