use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;
use tree_sitter::{Language, Parser, Query, QueryCursor};

use super::queries::DefExtractor;
use crate::lang::Lang;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub name: String,
    pub kind: DefKind,
    pub line: usize,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Constant,
    Class,
    Interface,
    Type,
}

static KIND_MAP: LazyLock<HashMap<&'static str, DefKind>> = LazyLock::new(|| {
    HashMap::from([
        ("struct_item", DefKind::Struct),
        ("enum_item", DefKind::Enum),
        ("enum_declaration", DefKind::Enum),
        ("struct_declaration", DefKind::Struct),
        ("trait_item", DefKind::Trait),
        ("impl_item", DefKind::Impl),
        ("const_item", DefKind::Constant),
        ("static_item", DefKind::Constant),
        ("type_item", DefKind::Type),
        ("type_alias_declaration", DefKind::Type),
        ("typealias_declaration", DefKind::Type),
        ("class_definition", DefKind::Class),
        ("class_declaration", DefKind::Class),
        ("interface_declaration", DefKind::Interface),
        ("protocol_declaration", DefKind::Interface),
    ])
});

#[must_use]
pub fn extract(path: &Path, content: &str) -> Vec<Definition> {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return Vec::new();
    };
    let Some(lang) = Lang::from_ext(ext) else {
        return Vec::new();
    };

    let (grammar, query) = DefExtractor::get_config(lang);
    run_extraction(content, &grammar, &query)
}

fn run_extraction(source: &str, lang: &Language, query: &Query) -> Vec<Definition> {
    let Some(tree) = parse_source(source, lang) else {
        return Vec::new();
    };

    let lines: Vec<&str> = source.lines().collect();
    let mut cursor = QueryCursor::new();
    let name_idx = query.capture_index_for_name("name").unwrap_or(0);
    let sig_idx = query.capture_index_for_name("sig").unwrap_or(0);

    cursor
        .matches(query, tree.root_node(), source.as_bytes())
        .filter_map(|m| build_def(&m, name_idx, sig_idx, source, &lines))
        .collect()
}

fn parse_source(source: &str, lang: &Language) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser.set_language(lang).ok()?;
    parser.parse(source, None)
}

fn build_def(
    m: &tree_sitter::QueryMatch,
    name_idx: u32,
    sig_idx: u32,
    source: &str,
    lines: &[&str],
) -> Option<Definition> {
    let (name, sig) = find_captures(m, name_idx, sig_idx, source)?;
    let row = sig.start_position().row;

    Some(Definition {
        name: name.to_string(),
        kind: KIND_MAP
            .get(sig.kind())
            .copied()
            .unwrap_or(DefKind::Function),
        line: row + 1,
        signature: get_signature(lines, row),
    })
}

fn find_captures<'a>(
    m: &'a tree_sitter::QueryMatch,
    name_idx: u32,
    sig_idx: u32,
    source: &'a str,
) -> Option<(&'a str, tree_sitter::Node<'a>)> {
    let mut name = None;
    let mut sig = None;

    for c in m.captures {
        if c.index == name_idx {
            name = c.node.utf8_text(source.as_bytes()).ok();
        }
        if c.index == sig_idx {
            sig = Some(c.node);
        }
    }

    Some((name?, sig?))
}

fn get_signature(lines: &[&str], row: usize) -> String {
    lines
        .get(row)
        .map_or(String::new(), |l| l.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_defs() {
        let code = "pub struct User { name: String }\nfn helper() -> bool { true }";
        let defs = extract(Path::new("lib.rs"), code);
        assert!(defs.iter().any(|d| d.name == "User"));
        assert!(defs.iter().any(|d| d.name == "helper"));
    }

    #[test]
    fn test_python_defs() {
        let code = "class UserService:\n    pass\n\ndef helper():\n    return True";
        let defs = extract(Path::new("service.py"), code);
        assert!(defs.iter().any(|d| d.name == "UserService"));
        assert!(defs.iter().any(|d| d.name == "helper"));
    }

    #[test]
    fn test_swift_defs() {
        let code = "public struct User {}\npublic func greet(name: String) -> String { name }";
        let defs = extract(Path::new("User.swift"), code);
        assert!(defs.iter().any(|d| d.name == "User"));
        assert!(defs.iter().any(|d| d.name == "greet"));
    }
}
