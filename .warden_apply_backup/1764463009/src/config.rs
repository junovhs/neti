// src/config.rs
pub use crate::constants::{
    BIN_EXT_PATTERN, CODE_BARE_PATTERN, CODE_EXT_PATTERN, PRUNE_DIRS, SECRET_PATTERN,
};
use crate::error::Result;
use crate::project::{self, ProjectType};
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
    5
}
const fn default_max_depth() -> usize {
    2
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

    /// Validates configuration.
    /// # Errors
    /// Currently always returns Ok.
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn load_local_config(&mut self) {
        self.load_ignore_file();
        self.load_toml_config();
        self.apply_project_defaults();
    }

    fn apply_project_defaults(&mut self) {
        if self.commands.contains_key("check") {
            return;
        }
        let defaults = project_defaults(ProjectType::detect());
        for (k, v) in defaults {
            self.commands.entry(k).or_insert(v);
        }
    }

    fn load_ignore_file(&mut self) {
        let Ok(content) = fs::read_to_string(".wardenignore") else {
            return;
        };
        for line in content.lines() {
            self.process_ignore_line(line);
        }
    }

    fn process_ignore_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        if let Ok(re) = Regex::new(trimmed) {
            self.exclude_patterns.push(re);
        }
    }

    fn load_toml_config(&mut self) {
        if !Path::new("warden.toml").exists() {
            return;
        }
        let Ok(content) = fs::read_to_string("warden.toml") else {
            return;
        };
        self.parse_toml(&content);
    }

    fn parse_toml(&mut self, content: &str) {
        let Ok(parsed) = toml::from_str::<WardenToml>(content) else {
            return;
        };
        self.rules = parsed.rules;
        self.commands = parsed.commands;
    }
}

fn project_defaults(project: ProjectType) -> HashMap<String, String> {
    let mut m = HashMap::new();
    match project {
        ProjectType::Rust => {
            m.insert(
                "check".into(),
                "cargo clippy --all-targets -- -D warnings -D clippy::pedantic".into(),
            );
            m.insert("fix".into(), "cargo fmt".into());
        }
        ProjectType::Node => {
            let npx = project::npx_cmd();
            m.insert("check".into(), format!("{npx} @biomejs/biome check src/"));
            m.insert(
                "fix".into(),
                format!("{npx} @biomejs/biome check --write src/"),
            );
        }
        ProjectType::Python => {
            m.insert("check".into(), "ruff check .".into());
            m.insert("fix".into(), "ruff check --fix .".into());
        }
        ProjectType::Go => {
            m.insert("check".into(), "go vet ./...".into());
            m.insert("fix".into(), "go fmt ./...".into());
        }
        ProjectType::Unknown => {}
    }
    m
}