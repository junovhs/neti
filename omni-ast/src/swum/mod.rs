//! SWUM (Software Word Usage Model) for identifier expansion.
//! Converts function/file names into readable sentences.

mod splitter;
mod verb_patterns;

pub use splitter::split_identifier;
pub use verb_patterns::expand_verb_pattern;

use std::collections::HashMap;

/// Minimum theme length to produce a proper sentence in summarize_exports.
const MIN_THEME_LEN: usize = 3;

/// Expand a file stem or function name into a readable description.
#[must_use]
pub fn expand_identifier(name: &str) -> String {
    let words = split_identifier(name);

    if words.is_empty() {
        return format!("Implements {name} functionality.");
    }

    let first = words.first().map_or("", String::as_str);
    let rest: String = words
        .iter()
        .skip(1)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");

    expand_verb_pattern(first, &rest)
}

/// Summarize a set of exports into a role-statement by voting on the dominant verb.
#[must_use]
pub fn summarize_exports(exports: &[String]) -> String {
    let first = match exports {
        [] => return String::new(),
        [only] => return expand_single_export(only),
        [first, ..] => first,
    };

    let mut verb_counts: HashMap<String, usize> = HashMap::new();
    let mut verb_themes: HashMap<String, Vec<String>> = HashMap::new();

    for export in exports {
        let words = split_identifier(export);
        let Some(verb) = words.first().cloned() else {
            continue;
        };
        let theme: String = words.get(1..).unwrap_or(&[]).join(" ");
        *verb_counts.entry(verb.clone()).or_insert(0) += 1;
        verb_themes.entry(verb).or_default().push(theme);
    }

    if verb_counts.is_empty() {
        return expand_single_export(first);
    }

    let dominant_verb = verb_counts
        .iter()
        .max_by(|(va, ca), (vb, cb)| {
            ca.cmp(cb)
                .then_with(|| verb_priority(vb).cmp(&verb_priority(va)))
        })
        .map(|(v, _)| v.clone())
        .unwrap_or_default();

    let best_theme = verb_themes
        .get(&dominant_verb)
        .and_then(|themes| {
            themes
                .iter()
                .filter(|t| !t.is_empty())
                .max_by_key(|t| t.len())
        })
        .map(String::as_str)
        .unwrap_or("");

    if best_theme.len() < MIN_THEME_LEN || looks_garbled(best_theme) {
        return expand_single_export(first);
    }

    expand_verb_pattern(&dominant_verb, best_theme)
}

fn expand_single_export(name: &str) -> String {
    let words = split_identifier(name);
    if words.is_empty() {
        return format!("Implements {name} functionality.");
    }
    let first = words.first().map_or("", String::as_str);
    let rest: String = words
        .iter()
        .skip(1)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    if rest.len() < MIN_THEME_LEN && looks_garbled(&rest) {
        return format!("Implements {name} functionality.");
    }
    expand_verb_pattern(first, &rest)
}

fn looks_garbled(theme: &str) -> bool {
    let theme = theme.trim();
    if theme.is_empty() {
        return true;
    }
    let mut words = theme.split_whitespace();
    let Some(first) = words.next() else {
        return true;
    };
    if words.next().is_none() && first.len() <= 2 && first.chars().all(|c| c.is_lowercase()) {
        return !matches!(first, "db" | "ui" | "io" | "id" | "os" | "fs" | "vm" | "ip");
    }
    false
}

fn verb_priority(verb: &str) -> u8 {
    match verb {
        "parse" | "extract" | "decode" => 0,
        "validate" | "check" | "verify" => 1,
        "render" | "format" | "display" | "print" => 2,
        "infer" | "classify" => 3,
        "get" | "find" | "search" | "lookup" | "query" => 4,
        _ => 5,
    }
}
