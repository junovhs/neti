// src/signatures.rs
use crate::config::Config;
use crate::discovery;
use crate::lang::Lang;
use crate::prompt::PromptGenerator;
use crate::skeleton;
use crate::tokens::Tokenizer;
use anyhow::Result;
use colored::Colorize;
use rayon::prelude::*;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[derive(Debug, Clone, Copy)]
pub struct SignatureOptions {
    pub copy: bool,
    pub stdout: bool,
}

/// Generates a type-surface signature map of the codebase.
///
/// # Errors
/// Returns error if file discovery or reading fails.
pub fn run(opts: &SignatureOptions) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();

    println!("{}", "🔍 Scanning type surface...".cyan());

    let files = discovery::discover(&config)?;
    let signatures: Vec<String> = files
        .par_iter()
        .filter_map(|p| process_file(p))
        .collect();

    let output = format_output(&signatures, &config.rules)?;
    let tokens = Tokenizer::count(&output);

    println!(
        "✨ Extracted {} signatures (~{} tokens)",
        signatures.len(),
        tokens
    );

    if opts.stdout {
        println!("{output}");
    } else {
        let msg = crate::clipboard::smart_copy(&output)?;
        println!("{}", "✓ Copied to clipboard".green());
        println!("  ({msg})");
    }

    Ok(())
}

fn process_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let ext = path.extension().and_then(|s| s.to_str())?;
    let lang = Lang::from_ext(ext)?;

    let filtered_content = extract_exports(lang, &content)?;
    let clean_content = skeleton::clean(path, &filtered_content);

    if clean_content.trim().is_empty() {
        return None;
    }

    let rel_path = path.to_string_lossy();
    Some(format!(
        "// ══════════════════════════════════════════════════\n// {rel_path}\n// ══════════════════════════════════════════════════\n{clean_content}\n"
    ))
}

fn extract_exports(lang: Lang, content: &str) -> Option<String> {
    if lang == Lang::Python {
        return Some(content.to_string());
    }

    let grammar = lang.grammar();
    let query_str = lang.q_exports();
    let query = compile_query(grammar, query_str)?;

    let mut parser = Parser::new();
    if parser.set_language(grammar).is_err() {
        return None;
    }

    let tree = parser.parse(content, None)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let mut ranges = Vec::new();
    for m in matches {
        for capture in m.captures {
            ranges.push(capture.node.byte_range());
        }
    }

    if ranges.is_empty() {
        return None;
    }

    Some(merge_and_extract(content, ranges))
}

fn merge_and_extract(source: &str, mut ranges: Vec<std::ops::Range<usize>>) -> String {
    if ranges.is_empty() {
        return String::new();
    }

    ranges.sort_by_key(|r| r.start);

    let mut merged = Vec::new();
    let mut current = ranges[0].clone();

    for next in ranges.into_iter().skip(1) {
        if next.start <= current.end {
            current.end = std::cmp::max(current.end, next.end);
        } else {
            merged.push(current);
            current = next;
        }
    }
    merged.push(current);

    let mut result = String::new();
    for range in merged {
        result.push_str(&source[range]);
        result.push('\n');
    }

    result
}

fn format_output(signatures: &[String], rules: &crate::config::RuleConfig) -> Result<String> {
    let mut out = String::new();
    let gen = PromptGenerator::new(rules.clone());

    // 1. System Prompt (Header)
    writeln!(out, "{}", gen.wrap_header()?)?;

    // 2. Mode Instruction
    writeln!(out, "\n// >>> CONTEXT: TYPE MAP (ARCHITECT MODE) <<<")?;
    writeln!(out, "// 1. Analyze this map to locate relevant files.")?;
    writeln!(
        out,
        "// 2. Request specific implementation details using: slopchop pack --focus src/foo.rs"
    )?;
    writeln!(
        out,
        "// ══════════════════════════════════════════════════\n"
    )?;

    // 3. Signatures content
    for sig in signatures {
        out.push_str(sig);
        out.push('\n');
    }

    // 4. Footer Reminder
    writeln!(out, "\n{}", gen.generate_reminder()?)?;

    Ok(out)
}

fn compile_query(lang: Language, pattern: &str) -> Option<Query> {
    Query::new(lang, pattern).ok()
}