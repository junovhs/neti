// src/discovery.rs
use crate::config::{
    Config, GitMode, BIN_EXT_PATTERN, CODE_BARE_PATTERN, CODE_EXT_PATTERN, SECRET_PATTERN,
};
use crate::constants::should_prune;
use anyhow::{bail, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use walkdir::WalkDir;

/// Runs the full file discovery pipeline: Enumerate -> Heuristics -> Filter.
///
/// # Errors
/// Returns error if git commands fail or regexes are invalid.
pub fn discover(config: &Config) -> Result<Vec<PathBuf>> {
    let raw_files = enumerate_files(config)?;
    let heuristic_files = filter_heuristics(raw_files);
    let final_files = filter_config(heuristic_files, config);
    Ok(final_files)
}

// --- Enumeration ---

fn enumerate_files(config: &Config) -> Result<Vec<PathBuf>> {
    match config.git_mode {
        GitMode::Yes => enumerate_git_required(),
        GitMode::No => Ok(walk_filesystem(config.verbose)),
        GitMode::Auto => Ok(enumerate_auto(config.verbose)),
    }
}

fn enumerate_git_required() -> Result<Vec<PathBuf>> {
    if !in_git_repo() {
        bail!("Not inside a Git repository. Use --no-git to scan without git.");
    }
    git_ls_files().map(filter_pruned)
}

fn enumerate_auto(verbose: bool) -> Vec<PathBuf> {
    if in_git_repo() {
        git_ls_files().map_or_else(|_| walk_filesystem(verbose), filter_pruned)
    } else {
        walk_filesystem(verbose)
    }
}

fn walk_filesystem(verbose: bool) -> Vec<PathBuf> {
    let walker = WalkDir::new(".")
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_prune(&e.file_name().to_string_lossy()));

    let (paths, error_count) = accumulate_walker(walker);
    if error_count > 0 && verbose {
        eprintln!("WARN: Encountered {error_count} errors during file walk");
    }
    paths
}

fn accumulate_walker<I>(walker: I) -> (Vec<PathBuf>, usize)
where
    I: Iterator<Item = walkdir::Result<walkdir::DirEntry>>,
{
    let mut paths = Vec::new();
    let mut errors = 0;
    for item in walker {
        match item {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    let p = entry.path().strip_prefix(".").unwrap_or(entry.path());
                    paths.push(p.to_path_buf());
                }
            }
            Err(_) => errors += 1,
        }
    }
    (paths, errors)
}

fn in_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_ls_files() -> Result<Vec<PathBuf>> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "-c", "-o", "--exclude-standard", "."])
        .output()?;

    if !out.status.success() {
        bail!("git ls-files failed: {}", out.status);
    }

    let paths = out
        .stdout
        .split(|&b| b == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| PathBuf::from(String::from_utf8_lossy(chunk).as_ref()))
        .collect();

    Ok(paths)
}

fn filter_pruned(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|p| {
            !p.components()
                .any(|c| should_prune(&c.as_os_str().to_string_lossy()))
        })
        .collect()
}

// --- Heuristics ---

static BIN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(BIN_EXT_PATTERN).unwrap_or_else(|_| panic!("Invalid Regex")));
static SECRET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(SECRET_PATTERN).unwrap_or_else(|_| panic!("Invalid Regex")));
static CODE_EXT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(CODE_EXT_PATTERN).unwrap_or_else(|_| panic!("Invalid Regex")));
static CODE_BARE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(CODE_BARE_PATTERN).unwrap_or_else(|_| panic!("Invalid Regex")));

fn filter_heuristics(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().filter(is_code_like).collect()
}

fn is_code_like(path: &PathBuf) -> bool {
    let filename = path.file_name().map_or("", |f| f.to_str().unwrap_or(""));

    if BIN_RE.is_match(filename) {
        return false;
    }
    if SECRET_RE.is_match(filename) {
        return false;
    }
    if CODE_EXT_RE.is_match(filename) {
        return true;
    }
    if CODE_BARE_RE.is_match(filename) {
        return true;
    }

    // Fall back to checking file content for shebang
    if let Ok(content) = fs::read_to_string(path) {
        if content.starts_with("#!") {
            return true;
        }
    }

    false
}

// --- Config Filtering ---

fn filter_config(mut paths: Vec<PathBuf>, config: &Config) -> Vec<PathBuf> {
    if !config.include_patterns.is_empty() {
        paths.retain(|p| {
            let s = p.to_string_lossy();
            config.include_patterns.iter().any(|re| re.is_match(&s))
        });
    }

    if !config.exclude_patterns.is_empty() {
        paths.retain(|p| {
            let s = p.to_string_lossy();
            !config.exclude_patterns.iter().any(|re| re.is_match(&s))
        });
    }

    paths
}

/// Groups files by their parent directory.
#[must_use]
pub fn group_by_directory(files: &[PathBuf]) -> HashMap<PathBuf, Vec<PathBuf>> {
    let mut groups: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for file in files {
        let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        groups.entry(dir).or_default().push(file.clone());
    }

    groups
}