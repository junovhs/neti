// src/audit/patterns.rs
//! Pattern detection for repeated code idioms.
//!
//! This module identifies common patterns that appear multiple times across
//! the codebase, such as:
//! - Process spawning patterns (spawn → pipe → wait)
//! - Error handling patterns
//! - Builder patterns
//! - Similar struct/enum definitions
//!
//! The algorithm:
//! 1. Define pattern templates as tree-sitter queries
//! 2. Match patterns against all files
//! 3. Group similar matches
//! 4. Report patterns with 3+ occurrences

use super::types::{PatternLocation, RepeatedPattern};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::{Query, QueryCursor};

/// A pattern template to search for.
pub struct PatternTemplate {
    /// Human-readable name.
    pub name: &'static str,
    /// Description of what this pattern represents.
    pub description: &'static str,
    /// Tree-sitter query pattern (Rust).
    pub rust_query: &'static str,
    /// Tree-sitter query pattern (Python), if applicable.
    pub python_query: Option<&'static str>,
    /// Minimum occurrences to report.
    pub min_occurrences: usize,
}

/// Built-in patterns to detect.
pub const PATTERNS: &[PatternTemplate] = &[
    PatternTemplate {
        name: "process_spawn",
        description: "Process spawning with stdin pipe (spawn → pipe → wait)",
        rust_query: r#"
            (call_expression
                function: (scoped_identifier
                    path: (identifier) @_cmd (#eq? @_cmd "Command")
                    name: (identifier) @_new (#eq? @_new "new"))
            ) @spawn
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "option_map_chain",
        description: "Option chaining pattern (map/and_then/ok_or)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#match? @method "^(map|and_then|ok_or|unwrap_or)$"))
            ) @chain
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "error_context",
        description: "Error context wrapping pattern (.context())",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "context"))
            ) @context
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "string_format",
        description: "String formatting with format!",
        rust_query: r#"
            (macro_invocation
                macro: (identifier) @name
                (#match? @name "^(format|println|eprintln|write|writeln)$")
            ) @format
        "#,
        python_query: None,
        min_occurrences: 10,
    },
    PatternTemplate {
        name: "impl_default",
        description: "Default trait implementation pattern",
        rust_query: r#"
            (impl_item
                trait: (type_identifier) @trait
                (#eq? @trait "Default")
            ) @impl
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "match_result",
        description: "Match on Result pattern",
        rust_query: r#"
            (match_expression
                value: (_)
                body: (match_block
                    (match_arm
                        pattern: (tuple_struct_pattern
                            type: (identifier) @variant
                            (#match? @variant "^(Ok|Err)$")))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "vec_collect",
        description: "Iterator collect into Vec pattern",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "collect"))
            ) @collect
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "closure_move",
        description: "Move closure pattern",
        rust_query: r#"
            (closure_expression
                "move"
            ) @closure
        "#,
        python_query: None,
        min_occurrences: 3,
    },
];

/// A detected pattern occurrence.
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub matched_text: String,
}

/// Detects patterns in a file.
pub fn detect_in_file(
    source: &str,
    file: &Path,
    tree: &tree_sitter::Tree,
    lang: tree_sitter::Language,
) -> Vec<PatternMatch> {
    let mut matches = Vec::new();
    let source_bytes = source.as_bytes();

    for template in PATTERNS {
        let query_str = template.rust_query;

        // Compile query (skip if invalid)
        let query = match Query::new(lang, query_str) {
            Ok(q) => q,
            Err(_) => continue,
        };

        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, tree.root_node(), source_bytes);

        for qm in query_matches {
            // Get the main capture (usually the last one)
            if let Some(capture) = qm.captures.last() {
                let node = capture.node;
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;

                let matched_text = node
                    .utf8_text(source_bytes)
                    .unwrap_or("")
                    .chars()
                    .take(100)
                    .collect::<String>();

                matches.push(PatternMatch {
                    pattern_name: template.name.to_string(),
                    file: file.to_path_buf(),
                    start_line,
                    end_line,
                    matched_text,
                });
            }
        }
    }

    matches
}

/// Aggregates pattern matches across files into repeated patterns.
pub fn aggregate(matches: Vec<PatternMatch>) -> Vec<RepeatedPattern> {
    // Group by pattern name
    let mut groups: HashMap<String, Vec<PatternMatch>> = HashMap::new();

    for m in matches {
        groups.entry(m.pattern_name.clone()).or_default().push(m);
    }

    // Convert to RepeatedPattern, filtering by minimum occurrences
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

        // Calculate potential savings (rough estimate)
        // If we could extract a helper, we'd save ~(n-1) * avg_size
        let avg_size: usize = group_matches
            .iter()
            .map(|m| m.end_line - m.start_line + 1)
            .sum::<usize>()
            / group_matches.len().max(1);

        let potential_savings = avg_size * (group_matches.len() - 1);

        // Build signature from first match
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

    // Sort by occurrences (more = higher priority)
    patterns.sort_by(|a, b| b.locations.len().cmp(&a.locations.len()));

    patterns
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

/// Detects a custom pattern in a file.
pub fn detect_custom(
    pattern: &CustomPattern,
    source: &str,
    file: &Path,
    tree: &tree_sitter::Tree,
    lang: tree_sitter::Language,
) -> Vec<PatternMatch> {
    let query = match Query::new(lang, &pattern.query) {
        Ok(q) => q,
        Err(_) => return Vec::new(),
    };

    let mut matches = Vec::new();
    let source_bytes = source.as_bytes();
    let mut cursor = QueryCursor::new();

    for qm in cursor.matches(&query, tree.root_node(), source_bytes) {
        if let Some(capture) = qm.captures.last() {
            let node = capture.node;
            matches.push(PatternMatch {
                pattern_name: pattern.name.clone(),
                file: file.to_path_buf(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                matched_text: node
                    .utf8_text(source_bytes)
                    .unwrap_or("")
                    .chars()
                    .take(100)
                    .collect(),
            });
        }
    }

    matches
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
            "Extract a helper function in {} to consolidate {} occurrences",
            files.iter().next().unwrap_or(&String::new()),
            count
        )
    } else {
        format!(
            "Consider creating a shared utility module for {} occurrences across {} files",
            count,
            files.len()
        )
    }
}
