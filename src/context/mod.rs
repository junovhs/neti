// src/context/mod.rs
//! Context generation for AI consumption.
//! Provides a "retina" view: structure + errors, no code.

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;

use crate::analysis::RuleEngine;
use crate::config::Config;
use crate::discovery;
use crate::graph::rank::RepoGraph;
use crate::tokens::Tokenizer;

/// Options for context generation.
#[derive(Default)]
pub struct ContextOptions {
    pub verbose: bool,
}

/// Generates the AI context map.
///
/// # Errors
/// Returns error if discovery or analysis fails.
pub fn run(_opts: &ContextOptions) -> Result<String> {
    let config = load_config();
    let files = discovery::discover(&config)?;
    let contents = read_all_files(&files);

    let file_vec: Vec<_> = contents
        .iter()
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();

    let graph = RepoGraph::build(&file_vec);

    let mut out = String::with_capacity(50_000);

    write_header(&mut out);
    write_errors(&mut out, &files, &config);
    write_structure(&mut out, &files, &contents, &graph);
    write_request_protocol(&mut out);

    Ok(out)
}

fn load_config() -> Config {
    let mut config = Config::new();
    config.load_local_config();
    config
}

fn read_all_files(files: &[PathBuf]) -> HashMap<PathBuf, String> {
    files
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect()
}

fn write_header(out: &mut String) {
    let _ = writeln!(out, "{}", "═".repeat(70));
    let _ = writeln!(out, "🔭 SLOPCHOP CONTEXT MAP");
    let _ = writeln!(out, "{}", "═".repeat(70));
    let _ = writeln!(out);
}

fn write_errors(out: &mut String, files: &[PathBuf], config: &Config) {
    let engine = RuleEngine::new(config.clone());
    let report = engine.scan(files.to_vec());

    let _ = writeln!(out, "## Active Violations\n");

    if !report.has_errors() {
        let _ = writeln!(out, "{}\n", "✅ No violations detected.".green());
        return;
    }

    for file in &report.files {
        for v in &file.violations {
            let _ = writeln!(
                out,
                "{}:{} [{}] {}",
                file.path.display().to_string().yellow(),
                (v.row + 1).to_string().cyan(),
                v.law.to_string().red(),
                v.message
            );
        }
    }
    let _ = writeln!(out);
}

#[allow(clippy::cast_precision_loss)]
fn write_structure(
    out: &mut String,
    files: &[PathBuf],
    contents: &HashMap<PathBuf, String>,
    graph: &RepoGraph,
) {
    let _ = writeln!(out, "## Repository Structure\n");

    let mut file_metrics: Vec<FileMetrics> = files
        .iter()
        .map(|path| build_file_metrics(path, contents, graph))
        .collect();

    file_metrics.sort_by(|a, b| b.dependents.len().cmp(&a.dependents.len()));

    for fm in &file_metrics {
        write_file_entry(out, fm);
    }

    let _ = writeln!(out);
}

fn build_file_metrics(
    path: &PathBuf,
    contents: &HashMap<PathBuf, String>,
    graph: &RepoGraph,
) -> FileMetrics {
    let content = contents.get(path).map_or("", String::as_str);
    FileMetrics {
        path: path.clone(),
        tokens: Tokenizer::count(content),
        dependencies: graph.dependencies(path),
        dependents: graph.dependents(path),
    }
}

fn write_file_entry(out: &mut String, fm: &FileMetrics) {
    let path_str = fm.path.display().to_string();
    let hub_marker = format_hub_marker(fm.dependents.len());

    let _ = writeln!(
        out,
        "{:<45} ({} toks){}",
        path_str.blue(),
        fm.tokens.to_string().yellow(),
        hub_marker
    );

    write_edge_list(out, "← uses:", &fm.dependencies);
    write_edge_list(out, "→ used by:", &fm.dependents);
}

fn format_hub_marker(dependent_count: usize) -> String {
    if dependent_count >= 5 {
        format!(" [HUB: {dependent_count} dependents]")
            .red()
            .to_string()
    } else {
        String::new()
    }
}

fn write_edge_list(out: &mut String, label: &str, edges: &[PathBuf]) {
    if edges.is_empty() {
        return;
    }
    let _ = write!(out, "  {} ", label.dimmed());
    for (i, dep) in edges.iter().enumerate() {
        if i > 0 {
            let _ = write!(out, ", ");
        }
        let _ = write!(out, "{}", dep.display().to_string().dimmed());
    }
    let _ = writeln!(out);
}

fn write_request_protocol(out: &mut String) {
    let _ = writeln!(out, "{}", "═".repeat(70));
    let _ = writeln!(out, "## Request Context\n");
    let _ = writeln!(out, "To get code for specific files:");
    let _ = writeln!(
        out,
        "  slopchop pack --focus <file1> [file2...] [--depth N]\n"
    );
    let _ = writeln!(out, "Examples:");
    let _ = writeln!(out, "  slopchop pack --focus src/graph/resolver.rs");
    let _ = writeln!(out, "  slopchop pack --focus src/lib.rs --depth 2");
    let _ = writeln!(out, "{}", "═".repeat(70));
}

struct FileMetrics {
    path: PathBuf,
    tokens: usize,
    dependencies: Vec<PathBuf>,
    dependents: Vec<PathBuf>,
}
