// src/signatures/mod.rs
//! Holographic signature map generator.
//! Uses dependency graph for topological ordering and `PageRank` for importance.

mod docs;
mod ordering;

use crate::config::Config;
use crate::discovery;
use crate::graph::rank::RepoGraph;
use crate::lang::Lang;
use crate::prompt::PromptGenerator;
use crate::skeleton;
use crate::tokens::Tokenizer;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug, Clone, Copy)]
pub struct SignatureOptions {
    pub copy: bool,
    pub stdout: bool,
}

/// Generates a graph-aware type-surface signature map.
///
/// # Errors
/// Returns error if file discovery or reading fails.
pub fn run(opts: &SignatureOptions) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();

    println!("{}", "?? Scanning type surface...".cyan());

    let files = discovery::discover(&config)?;
    let contents = read_all_files(&files);
    let graph = RepoGraph::build(&to_tuples(&contents));

    let ordered = ordering::topological_order(&graph, &files);
    let ranks = build_rank_map(&graph);

    let signatures: Vec<String> = ordered
        .iter()
        .filter_map(|p| process_file(p, ranks.get(p)))
        .collect();

    let output = format_output(&signatures, &config.rules)?;
    let tokens = Tokenizer::count(&output);

    println!(
        "? Extracted {} signatures (~{} tokens)",
        signatures.len(),
        tokens
    );

    if opts.stdout {
        println!("{output}");
    } else {
        let msg = crate::clipboard::smart_copy(&output)?;
        println!("{}", "� Copied to clipboard".green());
        println!("  ({msg})");
    }

    Ok(())
}

fn read_all_files(files: &[PathBuf]) -> HashMap<PathBuf, String> {
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect()
}

fn to_tuples(map: &HashMap<PathBuf, String>) -> Vec<(PathBuf, String)> {
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

fn build_rank_map(graph: &RepoGraph) -> HashMap<PathBuf, f64> {
    graph.ranked_files().into_iter().collect()
}

fn process_file(path: &Path, rank: Option<&f64>) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let ext = path.extension().and_then(|s| s.to_str())?;
    let lang = Lang::from_ext(ext)?;

    let filtered = extract_exports(lang, &content)?;
    let clean = skeleton::clean(path, &filtered);

    if clean.trim().is_empty() {
        return None;
    }

    let rel_path = path.to_string_lossy();
    let tier = rank_tier(rank);

    Some(format!(
        "// ��������������������������������������������������\n// {rel_path}{tier}\n// ��������������������������������������������������\n{clean}\n"
    ))
}

fn rank_tier(rank: Option<&f64>) -> String {
    let Some(&r) = rank else {
        return String::new();
    };

    let label = if r >= 0.05 {
        "CORE"
    } else if r >= 0.02 {
        "HIGH"
    } else if r >= 0.01 {
        "MID"
    } else {
        "LOW"
    };

    format!("  [{label}]")
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

    let mut ranges: Vec<Range<usize>> = Vec::new();
    for m in matches {
        for capture in m.captures {
            ranges.push(capture.node.byte_range());
        }
    }

    if ranges.is_empty() {
        return None;
    }

    let ranges = docs::expand_ranges_for_docs(content, ranges);
    Some(merge_and_extract(content, ranges))
}

fn merge_and_extract(source: &str, mut ranges: Vec<Range<usize>>) -> String {
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

    writeln!(out, "{}", gen.wrap_header()?)?;
    writeln!(out, "\n// >>> CONTEXT: TYPE MAP (ARCHITECT MODE) <<<")?;
    writeln!(out, "// Files ordered: Base Dependencies  Top-Level Consumers")?;
    writeln!(out, "// Tier Key: [CORE] = high PageRank, [LOW] = leaf node")?;
    writeln!(out, "// Request implementation: slopchop pack --focus src/foo.rs")?;
    writeln!(out, "// ��������������������������������������������������\n")?;

    for sig in signatures {
        out.push_str(sig);
        out.push('\n');
    }

    writeln!(out, "\n{}", gen.generate_reminder()?)?;
    Ok(out)
}

fn compile_query(lang: tree_sitter::Language, pattern: &str) -> Option<Query> {
    Query::new(lang, pattern).ok()
}