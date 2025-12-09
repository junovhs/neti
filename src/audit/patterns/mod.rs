//! Pattern detection for repeated code idioms.

pub mod detect;
pub mod registry;

use crate::audit::types::{PatternLocation, RepeatedPattern};
use std::collections::HashMap;
use std::path::PathBuf;

pub use detect::{detect_custom, detect_in_file};
pub use registry::PATTERNS;

/// A pattern template to search for.
pub struct PatternTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub rust_query: &'static str,
    pub python_query: Option<&'static str>,
    pub min_occurrences: usize,
}

/// A detected pattern occurrence.
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub matched_text: String,
}

/// Custom pattern builder for user-defined patterns.
pub struct CustomPattern {
    pub name: String,
    pub description: String,
    pub query: String,
    pub min_occurrences: usize,
}

impl CustomPattern {
    /// Creates a new custom pattern.
    #[must_use]
    pub fn new(name: &str, query: &str) -> Self {
        Self {
            name: name.to_string(),
            description: format!("Custom pattern: {name}"),
            query: query.to_string(),
            min_occurrences: 2,
        }
    }

    /// Sets the minimum occurrences threshold.
    #[must_use]
    pub fn min_occurrences(mut self, n: usize) -> Self {
        self.min_occurrences = n;
        self
    }

    /// Sets the description.
    #[must_use]
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
}

/// Aggregates pattern matches across files into repeated patterns.
#[must_use]
pub fn aggregate(matches: Vec<PatternMatch>) -> Vec<RepeatedPattern> {
    let mut groups: HashMap<String, Vec<PatternMatch>> = HashMap::new();

    for m in matches {
        groups.entry(m.pattern_name.clone()).or_default().push(m);
    }

    let mut patterns = Vec::new();

    for (name, group_matches) in groups {
        let template = PATTERNS.iter().find(|t| t.name == name);
        let min_occurrences = template.map_or(3, |t| t.min_occurrences);

        if group_matches.len() < min_occurrences {
            continue;
        }

        let description = template.map_or_else(|| name.clone(), |t| t.description.to_string());

        let locations: Vec<PatternLocation> = group_matches
            .iter()
            .map(|m| PatternLocation {
                file: m.file.clone(),
                start_line: m.start_line,
                end_line: m.end_line,
            })
            .collect();

        let avg_size: usize = group_matches
            .iter()
            .map(|m| m.end_line - m.start_line + 1)
            .sum::<usize>()
            / group_matches.len().max(1);

        let potential_savings = avg_size * (group_matches.len() - 1);

        let signature = group_matches
            .first()
            .map_or_else(String::new, |m| m.matched_text.clone());

        patterns.push(RepeatedPattern {
            description,
            locations,
            signature,
            potential_savings,
        });
    }

    patterns.sort_by(|a, b| b.locations.len().cmp(&a.locations.len()));

    patterns
}

/// Provides recommendations for extracting repeated patterns.
#[must_use]
pub fn recommend_extraction(pattern: &RepeatedPattern) -> String {
    let count = pattern.locations.len();
    let files: std::collections::HashSet<_> = pattern
        .locations
        .iter()
        .map(|l| l.file.display().to_string())
        .collect();

    if files.len() == 1 {
        format!(
            "Extract a helper function in {} to consolidate {count} occurrences",
            files.iter().next().unwrap_or(&String::new()),
        )
    } else {
        format!(
            "Consider creating a shared utility module for {count} occurrences across {} files",
            files.len()
        )
    }
}
