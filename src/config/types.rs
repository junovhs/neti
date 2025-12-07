// src/config/types.rs
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Theme {
    Nasa,
    #[default]
    Cyberpunk,
    Corporate,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_auto_copy")]
    pub auto_copy: bool,
    #[serde(default)]
    pub auto_format: bool,
    #[serde(default)]
    pub auto_commit: bool,
    #[serde(default = "default_commit_prefix")]
    pub commit_prefix: String,
    #[serde(default)]
    pub allow_dirty_git: bool,
    #[serde(default)]
    pub system_bell: bool,
    #[serde(default = "default_backup_retention")]
    pub backup_retention: usize,
    #[serde(default = "default_progress_bars")]
    pub progress_bars: bool,
    #[serde(default)]
    pub require_plan: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            auto_copy: true,
            auto_format: false,
            auto_commit: false,
            commit_prefix: default_commit_prefix(),
            allow_dirty_git: false,
            system_bell: false,
            backup_retention: default_backup_retention(),
            progress_bars: true,
            require_plan: false,
        }
    }
}

fn default_auto_copy() -> bool {
    true
}
fn default_progress_bars() -> bool {
    true
}
fn default_backup_retention() -> usize {
    5
}
fn default_commit_prefix() -> String {
    "AI: ".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default = "default_ignore_tokens")]
    pub ignore_tokens_on: Vec<String>,
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
            ignore_tokens_on: default_ignore_tokens(),
        }
    }
}

const fn default_max_tokens() -> usize {
    2000
}
const fn default_max_complexity() -> usize {
    8
}
const fn default_max_depth() -> usize {
    3
}
const fn default_max_args() -> usize {
    5
}
const fn default_max_words() -> usize {
    5
}
fn default_ignore_tokens() -> Vec<String> {
    vec!["README.md".to_string(), "lock".to_string()]
}

/// Helper enum to deserialize commands as either a single string or a list of strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandEntry {
    Single(String),
    List(Vec<String>),
}

impl CommandEntry {
    #[must_use]
    pub fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s],
            Self::List(v) => v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlopChopToml {
    #[serde(default)]
    pub rules: RuleConfig,
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub commands: HashMap<String, CommandEntry>,
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
    pub preferences: Preferences,
    pub commands: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            git_mode: GitMode::Auto,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            code_only: false,
            verbose: false,
            rules: RuleConfig::default(),
            preferences: Preferences::default(),
            commands: HashMap::new(),
        }
    }
}