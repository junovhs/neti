// src/pack.rs
use crate::analysis::RuleEngine;
use crate::clipboard;
use crate::config::{Config, GitMode};
use crate::discovery;
use crate::prompt::PromptGenerator;
use crate::skeleton;
use crate::tokens::Tokenizer;
use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

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
}

/// Entry point for the pack command.
///
/// # Errors
/// Returns error if:
/// - Configuration loading fails
/// - File discovery fails
/// - Content generation fails
/// - Clipboard access fails (if --copy is used)
/// - File writing fails
pub fn run(options: &PackOptions) -> Result<()> {
    let config = setup_config(options)?;

    if !options.stdout && !options.copy {
        if let Some(t) = &options.target {
            println!("🧶 Knitting repository (Focus: {})...", t.display());
        } else {
            println!("🧶 Knitting repository...");
        }
    }

    let files = discovery::discover(&config)?;
    if options.verbose {
        eprintln!("📦 Packing {} files...", files.len());
    }

    let content = generate_content(&files, options, &config)?;
    let token_count = Tokenizer::count(&content);

    output_result(&content, token_count, options)
}

fn setup_config(opts: &PackOptions) -> Result<Config> {
    let mut config = Config::new();
    config.verbose = opts.verbose;
    config.code_only = opts.code_only;
    config.git_mode = if opts.git_only {
        GitMode::Yes
    } else if opts.no_git {
        GitMode::No
    } else {
        GitMode::Auto
    };
    config.load_local_config();
    config.validate()?;
    Ok(config)
}

/// Generates the context content string from a list of files.
/// Exposed for testing purposes.
///
/// # Errors
/// Returns error if file reading fails.
pub fn generate_content(files: &[PathBuf], opts: &PackOptions, config: &Config) -> Result<String> {
    let mut ctx = String::with_capacity(100_000);

    if opts.prompt {
        write_header(&mut ctx, config)?;
        inject_violations(&mut ctx, files, config)?;
    }

    write_body(files, &mut ctx, opts)?;

    if opts.prompt {
        write_footer(&mut ctx, config)?;
    }

    Ok(ctx)
}

fn inject_violations(ctx: &mut String, files: &[PathBuf], config: &Config) -> Result<()> {
    let engine = RuleEngine::new(config.clone());
    let report = engine.scan(files.to_vec());

    if !report.has_errors() {
        return Ok(());
    }

    writeln!(
        ctx,
        "═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(ctx, "⚠️  ACTIVE VIOLATIONS (PRIORITY FIX REQUIRED)")?;
    writeln!(
        ctx,
        "═══════════════════════════════════════════════════════════════════\n"
    )?;

    for file in report.files {
        if file.is_clean() {
            continue;
        }
        for v in file.violations {
            writeln!(ctx, "FILE: {}", file.path.display())?;
            writeln!(ctx, "LAW:  {}", v.law)?;
            writeln!(ctx, "LINE: {}", v.row + 1)?;
            writeln!(ctx, "ERR:  {}", v.message)?;
            writeln!(ctx, "{}", "─".repeat(40))?;
        }
    }
    writeln!(ctx)?;

    Ok(())
}

fn write_body(files: &[PathBuf], ctx: &mut String, opts: &PackOptions) -> Result<()> {
    match opts.format {
        OutputFormat::Text => pack_nabla(files, ctx, opts),
        OutputFormat::Xml => pack_xml(files, ctx, opts),
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

    let output_path = PathBuf::from("context.txt");
    fs::write(&output_path, content)?;
    println!("✅ Generated 'context.txt'");

    if let Ok(abs_path) = fs::canonicalize(&output_path) {
        if clipboard::copy_file_path(&abs_path).is_ok() {
            println!(
                "{}",
                "📎 File path copied to clipboard (paste as attachment)".cyan()
            );
        }
    }

    println!("{info}");
    Ok(())
}

fn should_skeletonize(path: &Path, opts: &PackOptions) -> bool {
    // If global skeleton flag is on, everything is skeletonized
    if opts.skeleton {
        return true;
    }

    // If a target is specified, everything EXCEPT the target is skeletonized
    if let Some(target) = &opts.target {
        // We do a loose match: if the path ends with the target string.
        // This allows "warden pack src/main.rs" to match "./src/main.rs"
        return !path.ends_with(target);
    }

    false
}

fn pack_nabla(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "∇∇∇ {p_str} ∇∇∇")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                if should_skeletonize(path, opts) {
                    out.push_str(&skeleton::clean(path, &content));
                } else {
                    out.push_str(&content);
                }
            }
            Err(e) => writeln!(out, "// <ERROR READING FILE: {e}>")?,
        }
        writeln!(out, "\n∆∆∆\n")?;
    }
    Ok(())
}

fn pack_xml(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    writeln!(out, "<documents>")?;
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "  <document path=\"{p_str}\"><![CDATA[")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                if should_skeletonize(path, opts) {
                    out.push_str(
                        &skeleton::clean(path, &content).replace("]]>", "]]]]><![CDATA[>"),
                    );
                } else {
                    out.push_str(&content.replace("]]>", "]]]]><![CDATA[>"));
                }
            }
            Err(e) => writeln!(out, "<!-- ERROR: {e} -->")?,
        }
        writeln!(out, "]]></document>")?;
    }
    writeln!(out, "</documents>")?;
    Ok(())
}