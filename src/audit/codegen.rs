// src/audit/codegen.rs
//! Code generation for refactoring suggestions.

use super::parameterize::RefactorStrategy;
use super::types::CodeUnitKind;
use std::fmt::Write;

const SIGIL: &str = "XSC7XSC";

/// Generates a consolidated refactoring plan from strategies and unit names.
///
/// # Errors
/// Returns error if code generation fails.
pub fn generate_consolidated_plan(
    strategies: &[RefactorStrategy],
    unit_names: &[String],
) -> Result<String, std::fmt::Error> {
    generate_consolidated_plan_with_kind(strategies, unit_names, CodeUnitKind::Function)
}

/// Generates a consolidated refactoring plan, aware of the unit kind.
///
/// # Errors
/// Returns error if code generation fails.
pub fn generate_consolidated_plan_with_kind(
    strategies: &[RefactorStrategy],
    unit_names: &[String],
    kind: CodeUnitKind,
) -> Result<String, std::fmt::Error> {
    let mut buffer = String::new();

    // Handle enums specially - they shouldn't suggest function consolidation
    if kind == CodeUnitKind::Enum {
        return generate_enum_suggestion(&mut buffer, unit_names);
    }

    let has_enum_strategy = strategies
        .iter()
        .any(|s| matches!(s, RefactorStrategy::ExtractEnum { .. }));
    let has_generic = strategies
        .iter()
        .any(|s| matches!(s, RefactorStrategy::GenericParameter { .. }));

    if has_generic || has_enum_strategy {
        generate_function_consolidation(&mut buffer, unit_names)?;
    }

    append_manual_notes(&mut buffer, strategies)?;

    Ok(buffer)
}

fn generate_enum_suggestion(
    buffer: &mut String,
    unit_names: &[String],
) -> Result<String, std::fmt::Error> {
    writeln!(buffer, "// PATTERN DETECTED: Similar enum definitions")?;
    writeln!(buffer, "//")?;
    writeln!(
        buffer,
        "// These {} enums have similar structure.",
        unit_names.len()
    )?;
    writeln!(buffer, "// Consider:")?;
    writeln!(buffer, "// 1. If they represent the same concept, consolidate into one")?;
    writeln!(buffer, "// 2. If they share variants, extract a common base enum")?;
    writeln!(buffer, "// 3. If intentionally separate, add doc comments explaining why")?;
    writeln!(buffer)?;
    writeln!(buffer, "// Enums: {}", unit_names.join(", "))?;
    Ok(buffer.clone())
}

fn generate_function_consolidation(
    buffer: &mut String,
    unit_names: &[String],
) -> Result<(), std::fmt::Error> {
    let enum_name = infer_enum_name(unit_names);
    let variants: Vec<String> = unit_names.iter().map(|n| to_variant_name(n)).collect();

    writeln!(buffer, "// REFACTORING SUGGESTION")?;
    writeln!(buffer, "//")?;
    writeln!(
        buffer,
        "// These {} functions are structurally identical.",
        unit_names.len()
    )?;
    writeln!(
        buffer,
        "// Consolidate into one function with an enum parameter:"
    )?;
    writeln!(buffer)?;
    writeln!(buffer, "#[derive(Debug, Clone, Copy)]")?;
    writeln!(buffer, "pub enum {enum_name} {{")?;
    for variant in &variants {
        writeln!(buffer, "    {variant},")?;
    }
    writeln!(buffer, "}}")?;
    writeln!(buffer)?;
    writeln!(buffer, "// Unified function signature:")?;
    writeln!(
        buffer,
        "pub fn unified(kind: {enum_name}) -> /* return type */ {{"
    )?;
    writeln!(buffer, "    match kind {{")?;
    for variant in &variants {
        writeln!(buffer, "        {enum_name}::{variant} => todo!(),")?;
    }
    writeln!(buffer, "    }}")?;
    writeln!(buffer, "}}")?;
    writeln!(buffer)?;
    writeln!(buffer, "// Delete these functions: {}", unit_names.join(", "))?;
    Ok(())
}

fn append_manual_notes(
    buffer: &mut String,
    strategies: &[RefactorStrategy],
) -> Result<(), std::fmt::Error> {
    let manual_count = strategies
        .iter()
        .filter(|s| matches!(s, RefactorStrategy::ManualAttention))
        .count();

    if manual_count > 0 {
        writeln!(buffer)?;
        writeln!(
            buffer,
            "// NOTE: {manual_count} difference(s) may require manual review"
        )?;
    }
    Ok(())
}

/// Generates refactoring suggestion for a single strategy.
///
/// # Errors
/// Returns error if code generation fails.
pub fn generate_refactor(
    strategy: &RefactorStrategy,
    original_path: &str,
) -> Result<String, std::fmt::Error> {
    let mut buffer = String::new();

    writeln!(buffer, "{SIGIL} FILE {SIGIL} {original_path}")?;

    match strategy {
        RefactorStrategy::ExtractEnum { name, variants } => {
            writeln!(buffer, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
            writeln!(buffer, "pub enum {name} {{")?;
            for variant in variants {
                writeln!(buffer, "    {variant},")?;
            }
            writeln!(buffer, "}}")?;
        }
        RefactorStrategy::GenericParameter { name } => {
            writeln!(buffer, "// Parameterize: {name}")?;
        }
        RefactorStrategy::FunctionArgument { name, ty } => {
            writeln!(buffer, "// Add parameter: {name}: {ty}")?;
        }
        RefactorStrategy::ManualAttention => {
            writeln!(buffer, "// Manual attention needed")?;
        }
    }

    writeln!(buffer, "{SIGIL} END {SIGIL}")?;

    Ok(buffer)
}

fn infer_enum_name(unit_names: &[String]) -> String {
    if unit_names.iter().all(|n| n.starts_with("q_")) {
        return "QueryKind".to_string();
    }
    if unit_names.iter().all(|n| n.starts_with("test_")) {
        return "TestCase".to_string();
    }
    if unit_names.iter().all(|n| n.ends_with("_commands")) {
        return "CommandSet".to_string();
    }
    "Kind".to_string()
}

fn to_variant_name(name: &str) -> String {
    let stripped = name
        .strip_prefix("q_")
        .or_else(|| name.strip_prefix("test_"))
        .or_else(|| name.strip_suffix("_commands"))
        .unwrap_or(name);

    stripped
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}