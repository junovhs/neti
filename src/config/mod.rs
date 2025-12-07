// src/config/mod.rs
pub mod io;
pub mod types;

pub use self::types::{
    CommandEntry, Config, GitMode, Preferences, RuleConfig, SlopChopToml, Theme,
};
use anyhow::Result;

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates configuration.
    /// # Errors
    /// Returns Ok.
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn load_local_config(&mut self) {
        io::load_ignore_file(self);
        io::load_toml_config(self);
        io::apply_project_defaults(self);
    }

    pub fn process_ignore_line(&mut self, line: &str) {
        io::process_ignore_line(self, line);
    }

    pub fn parse_toml(&mut self, content: &str) {
        io::parse_toml(self, content);
    }
}

pub use crate::constants::{
    BIN_EXT_PATTERN, CODE_BARE_PATTERN, CODE_EXT_PATTERN, PRUNE_DIRS, SECRET_PATTERN,
};

/// Saves the current configuration to `slopchop.toml`.
/// # Errors
/// Returns error if file write fails or serialization fails.
#[allow(clippy::implicit_hasher)]
pub fn save_to_file(
    rules: &RuleConfig,
    prefs: &Preferences,
    commands: &std::collections::HashMap<String, Vec<String>>,
) -> Result<()> {
    io::save_to_file(rules, prefs, commands)
}
