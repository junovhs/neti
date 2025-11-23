// warden:ignore
use crate::config::{CODE_BARE_PATTERN, CODE_EXT_PATTERN};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

// --- Configuration Constants for Heuristics ---
const MIN_TEXT_ENTROPY: f64 = 3.5;
const MAX_TEXT_ENTROPY: f64 = 5.5;

const BUILD_SYSTEM_PAMPS: &[&str] = &[
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
    "version",
    "dependencies",
];

// Pre-compiled regexes for known code files
static CODE_EXT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(CODE_EXT_PATTERN).unwrap());
static CODE_BARE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(CODE_BARE_PATTERN).unwrap());

pub struct HeuristicFilter;

impl HeuristicFilter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn filter(&self, files: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        files
            .into_iter()
            .filter(|path| Self::should_keep(path))
            .collect()
    }

    fn should_keep(path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        if CODE_EXT_RE.is_match(&path_str) || CODE_BARE_RE.is_match(&path_str) {
            return true;
        }

        if let Ok(entropy) = calculate_entropy(path) {
            if !(MIN_TEXT_ENTROPY..=MAX_TEXT_ENTROPY).contains(&entropy) {
                return false;
            }
        } else {
            return false;
        }

        if let Ok(content) = fs::read_to_string(path) {
            let lower_content = content.to_lowercase();
            for pamp in BUILD_SYSTEM_PAMPS {
                if lower_content.contains(pamp) {
                    return true;
                }
            }
        }

        true
    }
}

impl Default for HeuristicFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn calculate_entropy(path: &Path) -> std::io::Result<f64> {
    let bytes = fs::read(path)?;
    if bytes.is_empty() {
        return Ok(0.0);
    }

    let mut freq_map = HashMap::new();
    for &byte in &bytes {
        *freq_map.entry(byte).or_insert(0) += 1;
    }

    // Suppress cast precision loss for 64-bit length; entropy approximation is fine.
    #[allow(clippy::cast_precision_loss)]
    let len = bytes.len() as f64;

    let entropy = freq_map.values().fold(0.0, |acc, &count| {
        let probability = f64::from(count) / len;
        acc - probability * probability.log2()
    });

    Ok(entropy)
}
