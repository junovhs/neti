// src/config/io.rs
use super::types::{CommandEntry, Config, Preferences, RuleConfig, SlopChopToml};
use crate::error::Result;
use crate::project::{self, ProjectType};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn load_ignore_file(config: &mut Config) {
    let Ok(content) = fs::read_to_string(".slopchopignore") else {
        return;
    };
    for line in content.lines() {
        process_ignore_line(config, line);
    }
}

pub fn process_ignore_line(config: &mut Config, line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return;
    }
    if let Ok(re) = Regex::new(trimmed) {
        config.exclude_patterns.push(re);
    }
}

pub fn load_toml_config(config: &mut Config) {
    if !Path::new("slopchop.toml").exists() {
        return;
    }
    let Ok(content) = fs::read_to_string("slopchop.toml") else {
        return;
    };
    parse_toml(config, &content);
}

pub fn parse_toml(config: &mut Config, content: &str) {
    let Ok(parsed) = toml::from_str::<SlopChopToml>(content) else {
        return;
    };
    config.rules = parsed.rules;
    config.preferences = parsed.preferences;
    config.commands = parsed
        .commands
        .into_iter()
        .map(|(k, v)| (k, v.into_vec()))
        .collect();
}

pub fn apply_project_defaults(config: &mut Config) {
    if config.commands.contains_key("check") {
        return;
    }
    let defaults = project_defaults(ProjectType::detect());
    for (k, v) in defaults {
        config.commands.entry(k).or_insert(v);
    }
}

/// Saves the configuration to the file system.
///
/// # Errors
/// Returns an error if the config cannot be serialized or written to disk.
#[allow(clippy::implicit_hasher)]
pub fn save_to_file(
    rules: &RuleConfig,
    prefs: &Preferences,
    commands: &HashMap<String, Vec<String>>,
) -> Result<()> {
    let cmd_entries: HashMap<String, CommandEntry> = commands
        .iter()
        .map(|(k, v)| (k.clone(), CommandEntry::List(v.clone())))
        .collect();

    let toml_struct = SlopChopToml {
        rules: rules.clone(),
        preferences: prefs.clone(),
        commands: cmd_entries,
    };

    let content = toml::to_string_pretty(&toml_struct).map_err(|e| {
        crate::error::SlopChopError::Other(format!("Failed to serialize config: {e}"))
    })?;

    fs::write("slopchop.toml", content)?;
    Ok(())
}

fn project_defaults(project: ProjectType) -> HashMap<String, Vec<String>> {
    let mut m = HashMap::new();
    match project {
        ProjectType::Rust => {
            m.insert(
                "check".into(),
                vec![
                    "cargo clippy --all-targets -- -D warnings -W clippy::pedantic".into(),
                    "cargo test".into(),
                ],
            );
            m.insert("fix".into(), vec!["cargo fmt".into()]);
        }
        ProjectType::Node => {
            let npx = project::npx_cmd();
            m.insert(
                "check".into(),
                vec![format!("{npx} @biomejs/biome check src/")],
            );
            m.insert(
                "fix".into(),
                vec![format!("{npx} @biomejs/biome check --write src/")],
            );
        }
        ProjectType::Python => {
            m.insert("check".into(), vec!["ruff check .".into()]);
            m.insert("fix".into(), vec!["ruff check --fix .".into()]);
        }
        ProjectType::Go => {
            m.insert("check".into(), vec!["go vet ./...".into()]);
            m.insert("fix".into(), vec!["go fmt ./...".into()]);
        }
        ProjectType::Unknown => {}
    }
    m
}
