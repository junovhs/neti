use super::codegen;
use super::diff;
use super::parameterize;
use super::types::{Opportunity, OpportunityKind};
use crate::config::Config;
use crate::lang::Lang;
use std::fs;
use tree_sitter::Parser;

pub fn enhance_opportunities(opportunities: &mut [Opportunity], limit: usize, _config: &Config) {
    let mut enhanced_count = 0;

    for opportunity in opportunities.iter_mut() {
        if enhanced_count >= limit {
            break;
        }

        if opportunity.kind != OpportunityKind::Duplication {
            continue;
        }

        if opportunity.impact.confidence < 0.8 {
            continue;
        }

        if let Some(plan) = generate_plan(opportunity) {
            opportunity.refactoring_plan = Some(plan);
            enhanced_count += 1;
        }
    }
}

fn generate_plan(opportunity: &Opportunity) -> Option<String> {
    if opportunity.units.len() < 2 {
        return None;
    }

    let unit_a = &opportunity.units[0];
    let unit_b = &opportunity.units[1];

    let src_a = fs::read_to_string(&unit_a.file).ok()?;
    let src_b = if unit_a.file == unit_b.file {
        src_a.clone()
    } else {
        fs::read_to_string(&unit_b.file).ok()?
    };

    let ext_a = unit_a.file.extension().and_then(|s| s.to_str())?;
    let lang_a = Lang::from_ext(ext_a)?;

    let mut parser = Parser::new();
    if parser.set_language(lang_a.grammar()).is_err() {
        return None;
    }

    let tree_a = parser.parse(&src_a, None)?;
    let tree_b = parser.parse(&src_b, None)?;

    let node_a = find_function_node(&tree_a, &src_a, unit_a.start_line, unit_a.end_line, &unit_a.name)?;
    let node_b = find_function_node(&tree_b, &src_b, unit_b.start_line, unit_b.end_line, &unit_b.name)?;

    let model = diff::diff_trees(node_a, src_a.as_bytes(), node_b, src_b.as_bytes())?;
    let strategies = parameterize::infer_strategies(&model);
    let unit_names: Vec<String> = opportunity.units.iter().map(|u| u.name.clone()).collect();

    match codegen::generate_consolidated_plan(&strategies, &unit_names) {
        Ok(plan) if !plan.is_empty() => Some(plan),
        _ => None,
    }
}

fn find_function_node<'a>(
    tree: &'a tree_sitter::Tree,
    source: &str,
    start_line: usize,
    end_line: usize,
    name: &str,
) -> Option<tree_sitter::Node<'a>> {
    let root = tree.root_node();
    find_node_recursive(root, source.as_bytes(), start_line, end_line, name)
}

fn find_node_recursive<'a>(
    node: tree_sitter::Node<'a>,
    source: &[u8],
    start_line: usize,
    end_line: usize,
    name: &str,
) -> Option<tree_sitter::Node<'a>> {
    let node_start = node.start_position().row + 1;
    let node_end = node.end_position().row + 1;

    if node.kind() == "function_item" && node_start == start_line && node_end == end_line {
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(node_name) = name_node.utf8_text(source) {
                if node_name == name {
                    return Some(node);
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_node_recursive(child, source, start_line, end_line, name) {
            return Some(found);
        }
    }

    None
}