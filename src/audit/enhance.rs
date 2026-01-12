// src/audit/enhance.rs
use super::codegen;
use super::diff;
use super::parameterize;
use super::types::{CodeUnit, Opportunity, OpportunityKind};
use crate::config::Config;
use crate::lang::Lang;
use std::fs;
use tree_sitter::Parser;

/// Enhances top opportunities with refactoring plans.
pub fn enhance_opportunities(opportunities: &mut [Opportunity], limit: usize, _config: &Config) {
    let mut enhanced_count = 0;

    for opportunity in opportunities.iter_mut() {
        if enhanced_count >= limit {
            break;
        }
        if try_enhance_opportunity(opportunity) {
            enhanced_count += 1;
        }
    }
}

fn try_enhance_opportunity(opportunity: &mut Opportunity) -> bool {
    if opportunity.kind != OpportunityKind::Duplication {
        return false;
    }
    if opportunity.impact.confidence < 0.8 {
        return false;
    }

    if let Some(plan) = build_refactoring_plan(opportunity) {
        opportunity.refactoring_plan = Some(plan);
        return true;
    }
    false
}

struct RefactorContext<'a> {
    lang: Lang,
    src_a: &'a str,
    src_b: &'a str,
    unit_a: &'a CodeUnit,
    unit_b: &'a CodeUnit,
}

// Indexing is safe: we check len() >= 2 before accessing [0] and [1]
#[allow(clippy::indexing_slicing)]
fn build_refactoring_plan(opportunity: &Opportunity) -> Option<String> {
    if opportunity.units.len() < 2 {
        return None;
    }

    let unit_a = &opportunity.units[0];
    let unit_b = &opportunity.units[1];

    let (src_a, src_b) = load_sources(&unit_a.file, &unit_b.file)?;
    let lang = Lang::from_ext(unit_a.file.extension()?.to_str()?)?;

    let ctx = RefactorContext {
        lang,
        src_a: &src_a,
        src_b: &src_b,
        unit_a,
        unit_b,
    };

    generate_plan_from_source(&ctx, &opportunity.units)
}

fn load_sources(path_a: &std::path::Path, path_b: &std::path::Path) -> Option<(String, String)> {
    let src_a = fs::read_to_string(path_a).ok()?;
    let src_b = if path_a == path_b {
        src_a.clone()
    } else {
        fs::read_to_string(path_b).ok()?
    };
    Some((src_a, src_b))
}

fn generate_plan_from_source(ctx: &RefactorContext, all_units: &[CodeUnit]) -> Option<String> {
    let model = calculate_diff(ctx)?;
    let strategies = parameterize::infer_strategies(&model);
    let names: Vec<String> = all_units.iter().map(|u| u.name.clone()).collect();

    // Pass the unit kind so codegen can generate appropriate suggestions
    let kind = ctx.unit_a.kind;
    codegen::generate_consolidated_plan_with_kind(&strategies, &names, kind).ok()
}

fn calculate_diff(ctx: &RefactorContext) -> Option<diff::DiffModel> {
    let mut parser = Parser::new();
    if parser.set_language(ctx.lang.grammar()).is_err() {
        return None;
    }

    let tree_a = parser.parse(ctx.src_a, None)?;
    let tree_b = parser.parse(ctx.src_b, None)?;

    let node_a = find_target_node(&tree_a, ctx.src_a, ctx.unit_a)?;
    let node_b = find_target_node(&tree_b, ctx.src_b, ctx.unit_b)?;

    diff::diff_trees(
        node_a,
        ctx.src_a.as_bytes(),
        node_b,
        ctx.src_b.as_bytes(),
    )
}

fn find_target_node<'a>(
    tree: &'a tree_sitter::Tree,
    source: &str,
    unit: &CodeUnit,
) -> Option<tree_sitter::Node<'a>> {
    find_node_recursive(
        tree.root_node(),
        source.as_bytes(),
        unit.start_line,
        unit.end_line,
        &unit.name,
    )
}

fn find_node_recursive<'a>(
    node: tree_sitter::Node<'a>,
    source: &[u8],
    start: usize,
    end: usize,
    name: &str,
) -> Option<tree_sitter::Node<'a>> {
    if is_matching_node(node, source, start, end, name) {
        return Some(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_node_recursive(child, source, start, end, name) {
            return Some(found);
        }
    }

    None
}

fn is_matching_node(
    node: tree_sitter::Node,
    source: &[u8],
    start: usize,
    end: usize,
    name: &str,
) -> bool {
    let node_start = node.start_position().row + 1;
    let node_end = node.end_position().row + 1;

    if node_start != start || node_end != end {
        return false;
    }

    // Check if the node contains the expected name
    node.utf8_text(source)
        .map(|text| text.contains(name))
        .unwrap_or(false)
}
