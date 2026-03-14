//! Stage 1 semantic signal harvesting.

use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;
use tree_sitter::{Language, Parser};

#[path = "harvester_tree.rs"]
mod tree;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticFingerprint {
    pub imports: Vec<String>,
    pub annotations: Vec<String>,
    pub param_types: Vec<String>,
    pub return_types: Vec<String>,
    pub strings: Vec<String>,
    pub comment_nouns: Vec<String>,
    pub exports: Vec<String>,
}

impl SemanticFingerprint {
    #[must_use]
    pub fn with_exports(mut self, exports: Vec<String>) -> Self {
        self.exports = exports;
        self
    }
}

#[derive(Default)]
pub(crate) struct Collector {
    pub(crate) imports: BTreeSet<String>,
    pub(crate) annotations: BTreeSet<String>,
    pub(crate) param_types: BTreeSet<String>,
    pub(crate) return_types: BTreeSet<String>,
    pub(crate) strings: BTreeSet<String>,
    pub(crate) comments: Vec<String>,
}

#[must_use]
pub fn harvest(_file: &Path, content: &str, ext: &str) -> SemanticFingerprint {
    let Some(language) = language_for_ext(ext) else {
        return SemanticFingerprint::default();
    };

    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        return SemanticFingerprint::default();
    }
    let Some(tree) = parser.parse(content, None) else {
        return SemanticFingerprint::default();
    };

    let mut collector = Collector::default();
    tree::walk(tree.root_node(), content, &mut collector, ext, false);

    let mut fingerprint = SemanticFingerprint {
        imports: sorted(collector.imports),
        annotations: sorted(collector.annotations),
        param_types: sorted(collector.param_types),
        return_types: sorted(collector.return_types),
        strings: filter_strings(collector.strings),
        comment_nouns: top_comment_nouns(&collector.comments),
        exports: Vec::new(),
    };

    if ext == "go" {
        fingerprint.annotations = expand_go_tags(&fingerprint.annotations);
    }

    fingerprint
}

fn language_for_ext(ext: &str) -> Option<Language> {
    match ext {
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        "ts" | "js" | "mjs" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" | "jsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            Some(tree_sitter_cpp::LANGUAGE.into())
        }
        _ => None,
    }
}

pub(crate) fn normalize_string(raw: &str) -> Option<String> {
    let trimmed = raw
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'');
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(crate) fn normalize_type_text(raw: &str) -> String {
    raw.replace(['\n', '\t'], " ")
        .trim()
        .trim_matches(',')
        .trim()
        .to_string()
}

fn filter_strings(values: BTreeSet<String>) -> Vec<String> {
    let url_re = Regex::new(r"https?://").ok();
    let registry_re = Regex::new(r"^(HKEY_|HKLM|HKCU)").ok();
    let path_re = Regex::new(r"(^~/|^/etc/|^[A-Za-z]:\\|^%APPDATA%|/api/|/v\d+/|/repos/)").ok();
    let sql_re = Regex::new(r"(?i)\b(SELECT|INSERT|UPDATE|DELETE|CREATE TABLE)\b").ok();
    let auth_re = Regex::new(r"(?i)(Bearer |Authorization:|client_secret)").ok();

    values
        .into_iter()
        .filter(|value| {
            url_re.as_ref().is_some_and(|re| re.is_match(value))
                || registry_re.as_ref().is_some_and(|re| re.is_match(value))
                || path_re.as_ref().is_some_and(|re| re.is_match(value))
                || sql_re.as_ref().is_some_and(|re| re.is_match(value))
                || auth_re.as_ref().is_some_and(|re| re.is_match(value))
                || value.len() > 10
        })
        .collect()
}

fn top_comment_nouns(comments: &[String]) -> Vec<String> {
    let stopwords: HashSet<&str> = [
        "a", "an", "and", "are", "as", "at", "by", "for", "from", "in", "into", "is", "it", "of",
        "on", "or", "that", "the", "this", "to", "with",
    ]
    .into_iter()
    .collect();

    let mut counts: HashMap<String, usize> = HashMap::new();
    let words = comments.iter().flat_map(|comment| {
        comment
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|word| word.trim().to_lowercase())
            .collect::<Vec<String>>()
    });
    for word in words.filter(|word| word.len() > 2) {
        if stopwords.contains(word.as_str()) || word.ends_with("ing") || word.ends_with("ed") {
            continue;
        }
        *counts.entry(word).or_insert(0) += 1;
    }

    let mut items: Vec<(String, usize)> = counts.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.into_iter().take(5).map(|(word, _)| word).collect()
}

fn expand_go_tags(tags: &[String]) -> Vec<String> {
    let expanded: BTreeSet<String> = tags
        .iter()
        .flat_map(|tag| {
            std::iter::once(tag.clone()).chain(tag.split_whitespace().filter_map(|part| {
                part.split_once(':')
                    .map(|(name, _)| name.trim_matches('`').to_owned())
            }))
        })
        .collect();
    expanded.into_iter().collect()
}

fn sorted(set: BTreeSet<String>) -> Vec<String> {
    set.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn harvests_rust_semantic_fingerprint() {
        let content = r#"
use std::fs;
#[derive(Clone, Debug)]
pub fn parse_config(input: String) -> Result<String, std::io::Error> {
    format!("https://example.com/{input}")
}
"#;

        let fingerprint = harvest(Path::new("lib.rs"), content, "rs");

        assert!(fingerprint.imports.iter().any(|i| i.contains("std::fs")));
        assert!(fingerprint.annotations.contains(&String::from("Clone")));
        assert!(fingerprint.annotations.contains(&String::from("Debug")));
        assert!(fingerprint.param_types.iter().any(|t| t.contains("String")));
        assert!(fingerprint
            .return_types
            .iter()
            .any(|t| t.contains("Result<String")));
        assert!(fingerprint
            .strings
            .iter()
            .any(|s| s.contains("https://example.com")));
    }

    #[test]
    fn unsupported_extension_returns_empty_fingerprint() {
        let fingerprint = harvest(Path::new("README.md"), "# title", "md");

        assert_eq!(fingerprint, SemanticFingerprint::default());
    }
}
