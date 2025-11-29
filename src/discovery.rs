// src/discovery.rs
use crate::config::{
    Config, GitMode, BIN_EXT_PATTERN, CODE_BARE_PATTERN, CODE_EXT_PATTERN, SECRET_PATTERN,
};
use crate::constants::should_prune;
use crate::error::{Result, WardenError};
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
    let final_files = filter_config(heuristic_files, config)?;
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
        return Err(WardenError::NotInGitRepo);
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
        return Err(WardenError::Other(format!(
            "git ls-files failed: {}",
            out.status
        )));
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
        .filter(|p| !contains_pruned_component(p))
        .collect()
}

fn contains_pruned_component(path: &Path) -> bool {
    path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .any(should_prune)
}

// --- Heuristics ---

const MIN_TEXT_ENTROPY: f64 = 3.5;
const MAX_TEXT_ENTROPY: f64 = 5.5;
const BUILD_MARKERS: &[&str] = &[
    "find_package",
    "add_executable",
    "target_link_libraries",
    "cmake_minimum_required",
    "project(",
    "add-apt-repository",
    "conanfile.py",
    "dependency",
    "require",
    "include",
    "import",
];

static CODE_EXT_RE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(CODE_EXT_PATTERN).ok());
static CODE_BARE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(CODE_BARE_PATTERN).ok());

fn filter_heuristics(files: Vec<PathBuf>) -> Vec<PathBuf> {
    files.into_iter().filter(|p| keep_heuristic(p)).collect()
}

fn keep_heuristic(path: &Path) -> bool {
    let s = path.to_string_lossy();
    if is_known_code(&s) {
        return true;
    }

    let Ok(entropy) = calculate_entropy(path) else {
        return false;
    };
    if (MIN_TEXT_ENTROPY..=MAX_TEXT_ENTROPY).contains(&entropy) {
        return true;
    }
    has_build_markers(path)
}

fn is_known_code(path_str: &str) -> bool {
    let ext = CODE_EXT_RE.as_ref().is_some_and(|r| r.is_match(path_str));
    let bare = CODE_BARE_RE.as_ref().is_some_and(|r| r.is_match(path_str));
    ext || bare
}

fn has_build_markers(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let lower = content.to_lowercase();
    BUILD_MARKERS.iter().any(|m| lower.contains(m))
}

#[allow(clippy::cast_precision_loss)]
fn calculate_entropy(path: &Path) -> std::io::Result<f64> {
    let bytes = fs::read(path)?;
    if bytes.is_empty() {
        return Ok(0.0);
    }
    let mut freq = HashMap::new();
    for &b in &bytes {
        *freq.entry(b).or_insert(0) += 1;
    }
    let len = bytes.len() as f64;
    Ok(freq
        .values()
        .fold(0.0, |acc, &n| acc - (f64::from(n) / len) * (f64::from(n) / len).log2()))
}

// --- Config Filter ---

struct FilterContext<'a> {
    config: &'a Config,
    bin_re: Regex,
    secret_re: Regex,
    code_re: Option<Regex>,
    bare_re: Option<Regex>,
}

fn filter_config(files: Vec<PathBuf>, config: &Config) -> Result<Vec<PathBuf>> {
    let ctx = FilterContext {
        config,
        bin_re: Regex::new(BIN_EXT_PATTERN)?,
        secret_re: Regex::new(SECRET_PATTERN)?,
        code_re: if config.code_only {
            Some(Regex::new(CODE_EXT_PATTERN)?)
        } else {
            None
        },
        bare_re: if config.code_only {
            Some(Regex::new(CODE_BARE_PATTERN)?)
        } else {
            None
        },
    };

    Ok(files
        .into_iter()
        .filter(|p| should_keep_config(p, &ctx))
        .collect())
}

fn should_keep_config(path: &Path, ctx: &FilterContext) -> bool {
    let s = path.to_string_lossy().replace('\\', "/");

    if ctx.secret_re.is_match(&s)
        || ctx.bin_re.is_match(&s)
        || ctx.config.exclude_patterns.iter().any(|p| p.is_match(&s))
    {
        return false;
    }

    if ctx.config.code_only {
        let is_code = ctx.code_re.as_ref().is_some_and(|r| r.is_match(&s))
            || ctx.bare_re.as_ref().is_some_and(|r| r.is_match(&s));
        if !is_code {
            return false;
        }
    }

    ctx.config.include_patterns.is_empty()
        || ctx.config.include_patterns.iter().any(|p| p.is_match(&s))
}