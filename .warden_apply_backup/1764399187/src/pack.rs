// src/pack.rs
use crate::clipboard;
use crate::config::{Config, GitMode};
use crate::enumerate::FileEnumerator;
use crate::filter::FileFilter;
use crate::heuristics::HeuristicFilter;
use crate::prompt::PromptGenerator;
use crate::rules::RuleEngine;
use crate::skeleton;
use crate::tokens::Tokenizer;
use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Xml,
}

#[allow(clippy::struct_excessive_bools)]
pub struct PackOptions {
    pub stdout: bool,
    pub copy: bool,
    pub verbose: bool,
    pub prompt: bool,
    pub format: OutputFormat,
    pub skeleton: bool,
    // Config overrides
    pub git_only: bool,
    pub no_git: bool,
    pub code_only: bool,
}

/// Entry point for the pack command.
///
/// # Errors
/// Returns error if file operations or clipboard access fails.
pub fn run(options: &PackOptions) -> Result<()> {
    let config = setup_config(options)?;

    if !options.stdout && !options.copy {
        println!("🧶 Knitting repository...");
    }

    let files = discover_files(&config, options.verbose)?;
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

fn discover_files(config: &Config, verbose: bool) -> Result<Vec<PathBuf>> {
    let raw = FileEnumerator::new(config.clone()).enumerate()?;
    let h_files = HeuristicFilter::new().filter(raw);
    let t_files = FileFilter::new(config)?.filter(h_files);

    if verbose {
        eprintln!("📦 Packing {} files...", t_files.len());
    }
    Ok(t_files)
}

fn generate_content(files: &[PathBuf], opts: &PackOptions, config: &Config) -> Result<String> {
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
        OutputFormat::Text => pack_nabla(files, ctx, opts.skeleton),
        OutputFormat::Xml => pack_xml(files, ctx, opts.skeleton),
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

    fs::write("context.txt", content)?;
    println!("✅ Generated 'context.txt'");
    println!("{info}");
    Ok(())
}

fn pack_nabla(files: &[PathBuf], out: &mut String, skeleton: bool) -> Result<()> {
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "∇∇∇ {p_str} ∇∇∇")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                if skeleton {
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

fn pack_xml(files: &[PathBuf], out: &mut String, skeleton: bool) -> Result<()> {
    writeln!(out, "<documents>")?;
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "  <document path=\"{p_str}\"><![CDATA[")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                if skeleton {
                    out.push_str(
                        &skeleton::clean(path, &content).replace("]]>", "]]]]><![CDATA[>"),
                    );
                } else {
                    out.push_str(&content.replace("]]>", "]]]]><![CDATA[>"));
                }
            }
            Err(e) => writeln!(out, "ERROR READING FILE: {e}")?,
        }
        writeln!(out, "]]></document>")?;
    }
    writeln!(out, "</documents>")?;
    Ok(())
}