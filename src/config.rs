use crate::error::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct RuleConfig {
    #[serde(default = "default_max_tokens")]
    pub max_file_tokens: usize,
    #[serde(default = "default_max_complexity")]
    pub max_cyclomatic_complexity: usize,
    #[serde(default = "default_max_depth")]
    pub max_nesting_depth: usize,
    #[serde(default = "default_max_args")]
    pub max_function_args: usize,
    #[serde(default = "default_max_words")]
    pub max_function_words: usize,
    #[serde(default)]
    pub ignore_naming_on: Vec<String>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            max_file_tokens: default_max_tokens(),
            max_cyclomatic_complexity: default_max_complexity(),
            max_nesting_depth: default_max_depth(),
            max_function_args: default_max_args(),
            max_function_words: default_max_words(),
            ignore_naming_on: Vec::new(),
        }
    }
}

// DEFAULTS
const fn default_max_tokens() -> usize {
    2000
}
const fn default_max_complexity() -> usize {
    10
}
const fn default_max_depth() -> usize {
    4
}
const fn default_max_args() -> usize {
    5
}
const fn default_max_words() -> usize {
    3
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WardenToml {
    #[serde(default)]
    pub rules: RuleConfig,
    #[serde(default)]
    pub commands: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum GitMode {
    Auto,
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub git_mode: GitMode,
    pub include_patterns: Vec<Regex>,
    pub exclude_patterns: Vec<Regex>,
    pub code_only: bool,
    pub verbose: bool,
    pub rules: RuleConfig,
    pub commands: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self {
            git_mode: GitMode::Auto,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            code_only: false,
            verbose: false,
            rules: RuleConfig::default(),
            commands: HashMap::new(),
        }
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Currently returns `Ok`. Reserved for future validation logic that might
    /// return an error if configuration combinations are invalid.
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn load_local_config(&mut self) {
        self.load_ignore_file();
        self.load_toml_config();
    }

    fn load_ignore_file(&mut self) {
        let ignore_path = Path::new(".wardenignore");
        if let Ok(content) = fs::read_to_string(ignore_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Ok(re) = Regex::new(trimmed) {
                    self.exclude_patterns.push(re);
                }
            }
        }
    }

    fn load_toml_config(&mut self) {
        let toml_path = Path::new("warden.toml");
        if let Ok(content) = fs::read_to_string(toml_path) {
            if let Ok(parsed) = toml::from_str::<WardenToml>(&content) {
                self.rules = parsed.rules;
                self.commands = parsed.commands;
                if self.verbose {
                    println!("🔧 Loaded warden.toml configuration");
                }
            } else if self.verbose {
                eprintln!("⚠️ Failed to parse warden.toml");
            }
        }
    }
}

// PRUNE_DIRS and PATTERNS
pub const PRUNE_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    "gen",
    ".venv",
    "venv",
    ".tox",
    "__pycache__",
    "coverage",
    "vendor",
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lockb",
    "go.sum",
    "Gemfile.lock",
    "tests",
    "test",
    "spec",
    "docs",
    "examples",
    "fixtures",
];
pub const BIN_EXT_PATTERN: &str =
    r"(?i)\.(png|jpg|gif|svg|ico|webp|woff2?|ttf|pdf|mp4|zip|gz|tar|exe|dll|so|dylib|class|pyc)$";
pub const SECRET_PATTERN: &str =
    r"(?i)(^\.?env(\..*)?$|/\.?env(\..*)?$|(^|/)(id_rsa|id_ed25519|.*\.(pem|p12|key|pfx))$)";
pub const CODE_EXT_PATTERN: &str = r"(?i)\.(rs|go|py|js|jsx|ts|tsx|java|c|cpp|h|hpp|cs|php|rb|sh|sql|html|css|scss|json|toml|yaml|md)$";
pub const CODE_BARE_PATTERN: &str = r"(?i)(Makefile|Dockerfile|CMakeLists\.txt)$";
