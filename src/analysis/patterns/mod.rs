//! AST pattern detection for violations.

pub mod concurrency;
pub mod concurrency_lock;
pub mod concurrency_sync;
pub mod db_patterns;
pub mod idiomatic;
pub mod logic;
pub mod logic_helpers;
pub mod logic_proof;
pub mod performance;
pub mod performance_test_ctx;
pub mod resource;
pub mod security;
pub mod semantic;
pub mod state;

use crate::lang::Lang;
use crate::types::Violation;
use omni_ast::SemanticLanguage;
use std::path::Path;
use tree_sitter::Parser;

/// Runs all pattern detections on a file.
#[must_use]
pub fn detect_all(path: &Path, source: &str) -> Vec<Violation> {
    let Some(language) = semantic_language(path) else {
        return Vec::new();
    };

    if language != SemanticLanguage::Rust {
        let mut out = Vec::new();
        out.extend(performance::detect(source, None, path));
        out.extend(logic::detect(source, None, path));
        return out;
    }

    let Some(tree) = parse_source(source, Lang::Rust) else {
        return Vec::new();
    };
    let root = tree.root_node();

    let mut out = Vec::new();
    out.extend(state::detect(source, root));
    out.extend(concurrency::detect(source, root));
    out.extend(performance::detect(source, Some(root), path));
    out.extend(db_patterns::detect(source, root));
    out.extend(security::detect(source, root));
    out.extend(semantic::detect(source, root));
    out.extend(resource::detect(source, root));
    out.extend(idiomatic::detect(source, root));
    out.extend(logic::detect(source, Some(root), path));
    out
}

fn semantic_language(path: &Path) -> Option<SemanticLanguage> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(SemanticLanguage::from_ext)
}

fn parse_source(source: &str, lang: Lang) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser.set_language(&lang.grammar()).ok()?;
    parser.parse(source, None)
}

/// Helper to get a node from a capture by index.
#[must_use]
pub fn get_capture_node<'a>(
    m: &tree_sitter::QueryMatch<'_, 'a>,
    idx: Option<u32>,
) -> Option<tree_sitter::Node<'a>> {
    let i = idx?;
    for c in m.captures {
        if c.index == i {
            return Some(c.node);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::detect_all;
    use std::path::Path;

    #[test]
    fn detect_all_runs_shared_p06_for_python() {
        let source = r#"
for needle in needles:
    if needle in haystack:
        hits.append(needle)
"#;

        let violations = detect_all(Path::new("src/example.py"), source);
        assert!(violations.iter().any(|violation| violation.law == "P06"));
    }

    #[test]
    fn detect_all_runs_shared_p06_for_typescript() {
        let source = r#"
for (const needle of needles) {
    const found = haystack.find((value) => value === needle);
    consume(found);
}
"#;

        let violations = detect_all(Path::new("src/example.ts"), source);
        assert!(violations.iter().any(|violation| violation.law == "P06"));
    }

    #[test]
    fn detect_all_runs_shared_l02_for_python() {
        let source = r#"
if idx <= len(values):
    return values[idx]
"#;

        let violations = detect_all(Path::new("src/example.py"), source);
        assert!(violations.iter().any(|violation| violation.law == "L02"));
    }

    #[test]
    fn detect_all_runs_shared_l02_for_typescript() {
        let source = r#"
if (idx <= values.length) {
  return values[idx];
}
"#;

        let violations = detect_all(Path::new("src/example.ts"), source);
        assert!(violations.iter().any(|violation| violation.law == "L02"));
    }
}
