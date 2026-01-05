// src/analysis/sabotage.rs
use crate::lang::Lang;
use crate::stage::StageManager;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Query, QueryCursor};

/// Result of a sabotage attempt.
pub struct SabotageReport {
    pub mutated: bool,
    pub original_op: String,
    pub new_op: String,
    pub line: usize,
    pub file: PathBuf,
}

/// Mutates a file in the stage to introduce a logical fault.
///
/// # Errors
/// Returns error if file operations or parsing fails.
pub fn sabotage_file(path: &Path, stage: &mut StageManager) -> Result<SabotageReport> {
    if !stage.exists() {
        anyhow::bail!("Stage does not exist. Run 'slopchop apply' first or create a stage.");
    }

    // Ensure we are operating on the STAGED file
    let staged_path = stage.worktree().join(path);
    if !staged_path.exists() {
        anyhow::bail!("File not found in stage: {}", path.display());
    }

    let source = fs::read_to_string(&staged_path)
        .with_context(|| format!("Failed to read staged file: {}", staged_path.display()))?;

    let lang = Lang::from_ext(staged_path.extension().and_then(|s| s.to_str()).unwrap_or(""))
        .context("Unsupported language for sabotage")?;

    let (mutated_source, report) = mutate_logic(lang, &source, path.to_path_buf())?;

    if report.mutated {
        fs::write(&staged_path, mutated_source)
            .with_context(|| format!("Failed to write mutated file to stage: {}", staged_path.display()))?;
    }

    Ok(report)
}

fn mutate_logic(lang: Lang, source: &str, file: PathBuf) -> Result<(String, SabotageReport)> {
    let mut parser = Parser::new();
    parser.set_language(lang.grammar())?;
    let tree = parser.parse(source, None).context("Failed to parse file")?;
    let root = tree.root_node();

    // Query for binary operators (==, !=, <, >) and boolean literals
    let query_str = match lang {
        Lang::Rust | Lang::TypeScript | Lang::Python => r"
            (binary_expression operator: _ @op)
            (boolean_literal) @bool
        ",
    };

    // Note: This is a simplified query. Real impl needs specific lang checks or general ones.
    // Rust/TS/Py all share similar binary expr structure in tree-sitter often, but let's refine.
    
    // Better query for Rust/TS/JS
    let q = Query::new(lang.grammar(), query_str).or_else(|_| {
         // Fallback or specific queries if needed. For now assume basic structure matches.
         // Actually, let's just target binary_expression for now.
         Query::new(lang.grammar(), "(binary_expression operator: _ @op)")
    })?;

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&q, root, source.as_bytes());

    // We take the FIRST mutation opportunity for deterministic behavior (or could be random?)
    // Requirement said "Identify a mutable node". Let's pick the first one we can flip.

    for m in matches {
        for capture in m.captures {
            let range = capture.node.byte_range();
            let text = &source[range.clone()];

            if let Some(replacement) = flip_operator(text) {
                let mut new_source = source.to_string();
                new_source.replace_range(range, replacement);
                
                return Ok((new_source, SabotageReport {
                    mutated: true,
                    original_op: text.to_string(),
                    new_op: replacement.to_string(),
                    line: capture.node.start_position().row + 1,
                    file,
                }));
            }
        }
    }

    Ok((source.to_string(), SabotageReport {
        mutated: false,
        original_op: String::new(),
        new_op: String::new(),
        line: 0,
        file,
    }))
}

fn flip_operator(op: &str) -> Option<&'static str> {
    flip_math_op(op).or_else(|| flip_bool_op(op))
}

fn flip_math_op(op: &str) -> Option<&'static str> {
    match op {
        "==" => Some("!="),
        "!=" => Some("=="),
        "<" => Some(">="),
        ">" => Some("<="),
        "<=" => Some(">"),
        ">=" => Some("<"),
        _ => None,
    }
}

fn flip_bool_op(op: &str) -> Option<&'static str> {
    match op {
        "true" => Some("false"),
        "false" => Some("true"),
        _ => None,
    }
}
