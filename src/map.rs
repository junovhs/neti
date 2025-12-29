// src/map.rs
//! Repository map generation with optional dependency tracking.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::discovery;
use crate::graph::rank::RepoGraph;
use crate::tokens::Tokenizer;

struct FileStats {
    size_kb: f64,
    tokens: usize,
}

/// Generates a repository map with optional dependency tracking.
///
/// # Errors
/// Returns an error if file discovery fails.
pub fn generate(show_deps: bool) -> Result<String> {
    let config = Config::load();
    let files = discovery::discover(&config)?;
    let contents = read_all_files(&files);

    let graph = if show_deps {
        let file_vec: Vec<_> = contents.iter().map(|(p, c)| (p.clone(), c.clone())).collect();
        Some(RepoGraph::build(&file_vec))
    } else {
        None
    };

    let mut out = String::from("# Repository Map\n\n");
    let dirs = group_by_directory(&files);

    for (dir, dir_files) in &dirs {
        write_dir_section(&mut out, dir, dir_files, &contents, graph.as_ref());
    }

    Ok(out)
}

fn read_all_files(files: &[PathBuf]) -> HashMap<PathBuf, String> {
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect()
}

fn group_by_directory(files: &[PathBuf]) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut dirs: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        dirs.entry(dir).or_default().push(file.clone());
    }
    dirs
}

fn write_dir_section(
    out: &mut String,
    dir: &Path,
    files: &[PathBuf],
    contents: &HashMap<PathBuf, String>,
    graph: Option<&RepoGraph>,
) {
    let _ = writeln!(out, "{}/", dir.display().to_string().blue().bold());

    for (i, f) in files.iter().enumerate() {
        let is_last = i == files.len() - 1;
        write_file_entry(out, f, is_last, contents, graph);
    }
    let _ = writeln!(out);
}

fn write_file_entry(
    out: &mut String,
    file: &Path,
    is_last: bool,
    contents: &HashMap<PathBuf, String>,
    graph: Option<&RepoGraph>,
) {
    let connector = if is_last { "└── " } else { "├── " };
    let name = file.file_name().unwrap_or_default().to_string_lossy();
    let stats = get_file_stats(file, contents);
    let meta = format!("{:.1} KB  {} toks", stats.size_kb, stats.tokens).dimmed();

    let _ = writeln!(out, "  {connector} {name:<30} ({meta})");

    if let Some(g) = graph {
        render_dependencies(out, g, file, is_last);
    }
}

fn render_dependencies(out: &mut String, graph: &RepoGraph, file: &Path, parent_is_last: bool) {
    let deps = graph.neighbors(file);
    if deps.is_empty() {
        return;
    }

    let prefix = if parent_is_last { "    " } else { "│   " };

    for (i, dep) in deps.iter().enumerate() {
        let is_last_dep = i == deps.len() - 1;
        let connector = if is_last_dep { "└── " } else { "├── " };
        let line = format_dep_line(file, dep, prefix, connector);
        let _ = writeln!(out, "  {line}");
    }
}

fn format_dep_line(file: &Path, dep: &Path, prefix: &str, connector: &str) -> String {
    let dep_name = dep.to_string_lossy();
    let distance = measure_distance(file, dep);
    let dist_label = if distance > 4 {
        " [FAR]".red()
    } else {
        "".normal()
    };

    format!("{prefix}  {connector}{dep_name}{dist_label}")
}

fn measure_distance(a: &Path, b: &Path) -> usize {
    let a_comps: Vec<_> = a.components().collect();
    let b_comps: Vec<_> = b.components().collect();
    let common = a_comps
        .iter()
        .zip(b_comps.iter())
        .take_while(|(ac, bc)| ac == bc)
        .count();
    (a_comps.len() - common) + (b_comps.len() - common)
}

#[allow(clippy::cast_precision_loss)]
fn get_file_stats(path: &Path, contents: &HashMap<PathBuf, String>) -> FileStats {
    let content = contents.get(path).map_or("", String::as_str);
    FileStats {
        size_kb: content.len() as f64 / 1024.0,
        tokens: Tokenizer::count(content),
    }
}
