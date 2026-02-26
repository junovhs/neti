// src/graph/tsconfig.rs
//! Parser for tsconfig.json / jsconfig.json path mappings.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Resolved path mapping configuration from tsconfig.
#[derive(Debug, Default)]
pub struct TsConfig {
    pub base_url: Option<PathBuf>,
    pub paths: HashMap<String, Vec<PathBuf>>,
}

#[derive(Deserialize)]
struct RawConfig {
    #[serde(rename = "compilerOptions")]
    compiler_options: Option<CompilerOptions>,
}

#[derive(Deserialize)]
struct CompilerOptions {
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    paths: Option<HashMap<String, Vec<String>>>,
}

impl TsConfig {
    /// Attempt to load tsconfig.json or jsconfig.json from project root.
    #[must_use]
    pub fn load(root: &Path) -> Option<Self> {
        let candidates = ["tsconfig.json", "jsconfig.json"];
        candidates.iter().find_map(|name| Self::parse_file(&root.join(name), root))
    }

    fn parse_file(path: &Path, root: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Self::parse_content(&content, root)
    }

    fn parse_content(content: &str, root: &Path) -> Option<Self> {
        let clean = strip_json_comments(content);
        let raw: RawConfig = serde_json::from_str(&clean).ok()?;
        let opts = raw.compiler_options?;

        let base_url = opts.base_url.map(|b| root.join(&b));
        let base_for_paths = base_url.as_deref().unwrap_or(root);

        let paths = opts.paths.map_or_else(HashMap::new, |p| {
            p.into_iter()
                .map(|(pattern, targets)| {
                    let resolved = targets.into_iter().map(|t| base_for_paths.join(&t)).collect();
                    (pattern, resolved)
                })
                .collect()
        });

        Some(Self { base_url, paths })
    }

    /// Resolve an import using path aliases or baseUrl.
    #[must_use]
    pub fn resolve(&self, import: &str) -> Option<PathBuf> {
        self.resolve_alias(import).or_else(|| self.resolve_base_url(import))
    }

    fn resolve_alias(&self, import: &str) -> Option<PathBuf> {
        self.paths
            .iter()
            .find_map(|(pattern, targets)| try_resolve_pattern(pattern, targets, import))
    }

    fn resolve_base_url(&self, import: &str) -> Option<PathBuf> {
        find_ts_file(&self.base_url.as_ref()?.join(import))
    }
}

fn try_resolve_pattern(pattern: &str, targets: &[PathBuf], import: &str) -> Option<PathBuf> {
    let matched = match_pattern(pattern, import)?;
    targets.iter().find_map(|t| expand_and_find(t, matched))
}

fn match_pattern<'a>(pattern: &str, import: &'a str) -> Option<&'a str> {
    match pattern.strip_suffix('*') {
        Some(prefix) => import.strip_prefix(prefix),
        None if pattern == import => Some(""),
        None => None,
    }
}

fn expand_and_find(target: &Path, matched: &str) -> Option<PathBuf> {
    let target_str = target.to_string_lossy();
    let resolved = if target_str.contains('*') {
        PathBuf::from(target_str.replace('*', matched))
    } else {
        target.to_path_buf()
    };
    find_ts_file(&resolved)
}

fn find_ts_file(path: &Path) -> Option<PathBuf> {
    if path.is_file() { return Some(path.to_path_buf()); }

    for ext in &["ts", "tsx", "js", "jsx", "json", "d.ts"] {
        let with_ext = path.with_extension(ext);
        if with_ext.is_file() { return Some(with_ext); }
    }

    find_ts_index(path)
}

fn find_ts_index(path: &Path) -> Option<PathBuf> {
    if !path.is_dir() { return None; }

    for ext in &["ts", "tsx", "js", "jsx"] {
        let index = path.join(format!("index.{ext}"));
        if index.is_file() { return Some(index); }
    }
    None
}

/// Strip single-line (//) and multi-line (/* */) comments from JSON.
fn strip_json_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if in_string {
            result.push(c);
            in_string = handle_string_char(c, &mut chars, &mut result);
            continue;
        }

        match c {
            '"' => { in_string = true; result.push(c); }
            '/' => handle_slash(&mut chars, &mut result),
            _ => result.push(c),
        }
    }
    result
}

fn handle_string_char(c: char, chars: &mut std::iter::Peekable<std::str::Chars>, result: &mut String) -> bool {
    if c == '\\' {
        if let Some(&next) = chars.peek() {
            result.push(next);
            chars.next();
        }
        return true;
    }
    c != '"'
}

fn handle_slash(chars: &mut std::iter::Peekable<std::str::Chars>, result: &mut String) {
    match chars.peek() {
        Some(&'/') => skip_line_comment(chars, result),
        Some(&'*') => skip_block_comment(chars),
        _ => result.push('/'),
    }
}

fn skip_line_comment(chars: &mut std::iter::Peekable<std::str::Chars>, result: &mut String) {
    for ch in chars.by_ref() {
        if ch == '\n' { result.push('\n'); break; }
    }
}

fn skip_block_comment(chars: &mut std::iter::Peekable<std::str::Chars>) {
    chars.next(); // consume '*'
    while let Some(ch) = chars.next() {
        if ch == '*' && chars.peek() == Some(&'/') {
            chars.next();
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_comments() {
        let input = r#"{ // comment
            "baseUrl": "." /* inline */ }"#;
        let clean = strip_json_comments(input);
        assert!(!clean.contains("//"));
        assert!(!clean.contains("/*"));
        assert!(clean.contains("baseUrl"));
    }

    #[test]
    fn test_match_pattern() {
        assert_eq!(match_pattern("@/*", "@/components/Button"), Some("components/Button"));
        assert_eq!(match_pattern("@/*", "react"), None);
        assert_eq!(match_pattern("utils", "utils"), Some(""));
    }
}
