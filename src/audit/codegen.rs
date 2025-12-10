use super::parameterize::RefactorStrategy;
use std::fmt::Write;

/// Generates a consolidated refactoring plan from strategies and unit names.
///
/// # Errors
/// Returns error if code generation fails.
pub fn generate_consolidated_plan(
    strategies: &[RefactorStrategy],
    unit_names: &[String],
) -> Result<String, std::fmt::Error> {
    let mut buffer = String::new();

    let has_enum_strategy = strategies
        .iter()
        .any(|s| matches!(s, RefactorStrategy::ExtractEnum { .. }));
    let has_generic = strategies
        .iter()
        .any(|s| matches!(s, RefactorStrategy::GenericParameter { .. }));

    if has_generic || has_enum_strategy {
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
        writeln!(buffer, "pub fn query(&self, kind: {enum_name}) -> &'static str {{")?;
        writeln!(buffer, "    match (self, kind) {{")?;
        for variant in &variants {
            writeln!(buffer, "        (Self::Rust, {enum_name}::{variant}) => todo!(),")?;
        }
        writeln!(buffer, "        // #__SLOPCHOP_IGNORE__# other Lang variants")?;
        writeln!(buffer, "    }}")?;
        writeln!(buffer, "}}")?;
        writeln!(buffer)?;
        writeln!(buffer, "// Delete these functions: {}", unit_names.join(", "))?;
    }

    let manual_count = strategies
        .iter()
        .filter(|s| matches!(s, RefactorStrategy::ManualAttention))
        .count();

    if manual_count > 0 {
        writeln!(buffer)?;
        writeln!(buffer, "// NOTE: {manual_count} difference(s) may require manual review")?;
    }

    Ok(buffer)
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

    writeln!(buffer, "#__SLOPCHOP_FILE__# {original_path}")?;

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

    writeln!(buffer, "#__SLOPCHOP_END__#")?;

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