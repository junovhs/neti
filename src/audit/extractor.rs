// src/audit/extractor.rs
//! AST extraction logic for code units.
//!
//! Separates the mechanical process of walking the AST from the
//! semantic logic of fingerprinting.

use super::fingerprint::compute;
use super::types::Fingerprint;
use tree_sitter::Node;

/// Type alias for extracted unit data to reduce signature complexity.
pub type ExtractedUnit = (
    String,       // name
    &'static str, // kind
    usize,        // start
    usize,        // end
    Fingerprint,  // structural fingerprint
    Vec<String>,  // signature (e.g. enum variants)
);

/// Extracts fingerprinted units from a parsed file.
#[must_use]
pub fn extract_units(source: &str, tree: &tree_sitter::Tree) -> Vec<ExtractedUnit> {
    let mut units = Vec::new();
    extract_from_node(tree.root_node(), source.as_bytes(), &mut units);
    units
}

fn extract_from_node(node: Node, source: &[u8], units: &mut Vec<ExtractedUnit>) {
    if let Some(unit_kind) = match_unit_kind(node.kind()) {
        if let Some(name) = extract_name(node, source) {
            let fp = compute(node, source);
            let start = node.start_position().row + 1;
            let end = node.end_position().row + 1;
            let sig = extract_signature(node, source, unit_kind);
            units.push((name, unit_kind, start, end, fp, sig));
        }
    }
    for child in node.children(&mut node.walk()) {
        extract_from_node(child, source, units);
    }
}

fn extract_signature(node: Node, source: &[u8], kind: &str) -> Vec<String> {
    if kind != "enum" {
        return Vec::new();
    }
    let mut variants = Vec::new();
    // Scan children for variants (Rust: enum_variant, TS: property_identifier in enum_body)
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Rust style: enum_variant_list containing enum_variant
        if child.kind() == "enum_variant_list" || child.kind() == "enum_body" {
            extract_variants_from_list(child, source, &mut variants);
        }
    }
    variants
}

fn extract_variants_from_list(node: Node, source: &[u8], variants: &mut Vec<String>) {
    let mut cursor = node.walk();
    for inner in node.children(&mut cursor) {
        if inner.kind() == "enum_variant" || inner.kind() == "property_identifier" {
            if let Some(name) = extract_variant_name(inner, source) {
                variants.push(name);
            }
        }
    }
}

fn extract_variant_name(node: Node, source: &[u8]) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        name_node.utf8_text(source).ok().map(ToString::to_string)
    } else {
        node.utf8_text(source).ok().map(ToString::to_string)
    }
}

fn match_unit_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "function_item" | "function_definition" => Some("function"),
        "impl_item" => Some("impl"),
        "struct_item" | "struct_definition" => Some("struct"),
        "enum_item" | "enum_definition" | "enum_declaration" => Some("enum"),
        "trait_item" | "trait_definition" => Some("trait"),
        "mod_item" => Some("module"),
        _ => None,
    }
}

fn extract_name(node: Node, source: &[u8]) -> Option<String> {
    node.child_by_field_name("name")?.utf8_text(source).ok().map(String::from)
}