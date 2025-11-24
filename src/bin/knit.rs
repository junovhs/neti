use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::fmt::Write;
use std::fs;
use std::path::Path;

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
    verbose: bool,
    #[arg(long)]
    git_only: bool,
    #[arg(long)]
    no_git: bool,
    #[arg(long)]
    code_only: bool,
    /// Wrap output with Warden Protocol prompt
    #[arg(long, short)]
    prompt: bool,
    /// Output format (Text for standard, Xml for Claude/LLMs)
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;

    if cli.git_only {
        config.git_mode = GitMode::Yes;
    } else if cli.no_git {
        config.git_mode = GitMode::No;
    }

    config.load_local_config();
    config.validate()?;

    if !cli.stdout {
        println!("🧶 Knitting repository...");
    }

    let enumerator = FileEnumerator::new(config.clone());
    let raw_files = enumerator.enumerate()?;

    let heuristic_filter = HeuristicFilter::new();
    let heuristics_files = heuristic_filter.filter(raw_files);

    let filter = FileFilter::new(config.clone())?;
    let target_files = filter.filter(heuristics_files);

    if cli.verbose {
        eprintln!("📦 Packing {} files...", target_files.len());
    }

    let mut full_context = String::with_capacity(100_000);

    // Add prompt header if requested
    if cli.prompt {
        let generator = PromptGenerator::new(config.rules.clone());
        writeln!(full_context, "{}", generator.wrap_header())?;
        writeln!(full_context)?;
        writeln!(
            full_context,
            "═══════════════════════════════════════════════════════════════════"
        )?;
        writeln!(full_context, "BEGIN CODEBASE")?;
        writeln!(
            full_context,
            "═══════════════════════════════════════════════════════════════════"
        )?;
        writeln!(full_context)?;
    }

    match cli.format {
        OutputFormat::Text => pack_text(&target_files, &mut full_context)?,
        OutputFormat::Xml => pack_xml(&target_files, &mut full_context)?,
    }

    // Add prompt footer if requested
    if cli.prompt {
        let generator = PromptGenerator::new(config.rules);
        writeln!(full_context)?;
        writeln!(
            full_context,
            "═══════════════════════════════════════════════════════════════════"
        )?;
        writeln!(full_context, "END CODEBASE")?;
        writeln!(
            full_context,
            "═══════════════════════════════════════════════════════════════════"
        )?;
        writeln!(full_context, "{}", generator.generate_reminder())?;
    }

    // Count tokens
    let token_count = Tokenizer::count(&full_context);

    if cli.stdout {
        print!("{full_context}");
        eprintln!(
            "\n📊 Context Size: {} tokens",
            token_count.to_string().yellow().bold()
        );
    } else {
        fs::write("context.txt", &full_context)?;
        println!("✅ Generated 'context.txt'");
        println!(
            "📊 Context Size: {} tokens",
            token_count.to_string().yellow().bold()
        );
    }

    Ok(())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn pack_text(files: &[std::path::PathBuf], out: &mut String) -> Result<()> {
    for path in files {
        let path_str = normalize_path(path);
        // XML-style file wrapper: clean boundaries, AI-friendly
        writeln!(out, "<file path=\"{path_str}\">")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                out.push_str(&content);
            }
            Err(e) => {
                writeln!(out, "<ERROR READING FILE: {e}>")?;
            }
        }

        writeln!(out, "</file>")?;
        out.push('\n');
    }
    Ok(())
}

fn pack_xml(files: &[std::path::PathBuf], out: &mut String) -> Result<()> {
    writeln!(out, "<documents>")?;
    for path in files {
        let path_str = normalize_path(path);
        writeln!(out, "  <document path=\"{path_str}\"><![CDATA[")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                let safe_content = content.replace("]]>", "]]]]><![CDATA[>");
                out.push_str(&safe_content);
            }
            Err(e) => {
                writeln!(out, "ERROR READING FILE: {e}")?;
            }
        }

        writeln!(out, "]]></document>")?;
    }
    writeln!(out, "</documents>")?;
    Ok(())
}
