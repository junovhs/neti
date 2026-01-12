use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default = "default_auto_copy")]
    pub auto_copy: bool,
    #[serde(default)]
    pub auto_format: bool,
    #[serde(default)]
    pub system_bell: bool,
    #[serde(default = "default_backup_retention")]
    pub backup_retention: usize,
    #[serde(default = "default_progress_bars")]
    pub progress_bars: bool,
    #[serde(default)]
    pub require_plan: bool,
    #[serde(default)]
    pub write_fix_packet: bool,
    #[serde(default = "default_fix_packet_path")]
    pub fix_packet_path: String,
    #[serde(default)]
    pub auto_promote: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            auto_copy: default_auto_copy(),
            auto_format: false,
            system_bell: false,
            backup_retention: default_backup_retention(),
            progress_bars: default_progress_bars(),
            require_plan: false,
            write_fix_packet: false,
            fix_packet_path: default_fix_packet_path(),
            auto_promote: false,
        }
    }
}

fn default_auto_copy() -> bool { true }
fn default_progress_bars() -> bool { true }
fn default_backup_retention() -> usize { 5 }
fn default_fix_packet_path() -> String { "slopchop-fixme.txt".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    #[serde(default = "default_max_tokens")]
    pub max_file_tokens: usize,
    // RENAMED: was max_cyclomatic_complexity
    #[serde(default = "default_max_complexity", alias = "max_cyclomatic_complexity")]
    pub max_cognitive_complexity: usize,
    #[serde(default = "default_max_depth")]
    pub max_nesting_depth: usize,
    #[serde(default = "default_max_args")]
    pub max_function_args: usize,
    #[serde(default = "default_max_words")]
    pub max_function_words: usize,

    // Scan v2 Metrics
    #[serde(default = "default_lcom4")]
    pub max_lcom4: usize,
    #[serde(default = "default_ahf")]
    pub min_ahf: f64,
    #[serde(default = "default_cbo")]
    pub max_cbo: usize,
    #[serde(default = "default_sfout")]
    pub max_sfout: usize,

    #[serde(default)]
    pub ignore_naming_on: Vec<String>,
    #[serde(default = "default_ignore_tokens")]
    pub ignore_tokens_on: Vec<String>,
    #[serde(default)]
    pub safety: SafetyConfig,
    #[serde(default)]
    pub locality: super::locality::LocalityConfig,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            max_file_tokens: default_max_tokens(),
            max_cognitive_complexity: default_max_complexity(),
            max_nesting_depth: default_max_depth(),
            max_function_args: default_max_args(),
            max_function_words: default_max_words(),

            max_lcom4: default_lcom4(),
            min_ahf: default_ahf(),
            max_cbo: default_cbo(),
            max_sfout: default_sfout(),

            ignore_naming_on: Vec::new(),
            ignore_tokens_on: default_ignore_tokens(),
            safety: SafetyConfig::default(),
            locality: super::locality::LocalityConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    #[serde(default = "default_true")]
    pub require_safety_comment: bool,
    #[serde(default)]
    pub ban_unsafe: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self { require_safety_comment: true, ban_unsafe: false }
    }
}

const fn default_true() -> bool { true }
const fn default_max_tokens() -> usize { 2000 }
// UPDATED DEFAULT: 25 provides breathing room for complex reporting logic without being totally loose
const fn default_max_complexity() -> usize { 25 } 
const fn default_max_depth() -> usize { 3 }
const fn default_max_args() -> usize { 5 }
const fn default_max_words() -> usize { 10 } 

// V2 Defaults
const fn default_lcom4() -> usize { 1 }
const fn default_ahf() -> f64 { 60.0 }
const fn default_cbo() -> usize { 9 }
const fn default_sfout() -> usize { 7 }

fn default_ignore_tokens() -> Vec<String> {
    vec!["Cargo.lock".into(), "package-lock.json".into(), "README.md".into()]
}

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
            Self::List(l) => l,
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

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub include_patterns: Vec<regex::Regex>,
    pub exclude_patterns: Vec<regex::Regex>,
    pub code_only: bool,
    pub verbose: bool,
    pub rules: RuleConfig,
    pub preferences: Preferences,
    pub commands: HashMap<String, Vec<String>>,
}