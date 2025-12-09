use super::parameterize::RefactorStrategy;
use std::fmt::Write;

/// Generates the refactored code (protocol block) for a given strategy.
///
/// # Errors
/// Returns error if code generation fails (e.g. formatting).
pub fn generate_refactor(strategy: &RefactorStrategy, _original_path: &str) -> Result<String, std::fmt::Error> {
    let mut buffer = String::new();

    match strategy {
        RefactorStrategy::ExtractEnum { name, variants } => {
            // 1. Generate the Enum definition
            writeln!(buffer, "#__SLOPCHOP_FILE__#")?;
            // We assume a new file or appending to existing?
            // For now, let's suggest a new types file or nearby.
            // But to be safe, let's output it as a standalone block first.
            writeln!(buffer, "[NEW] src/types.rs")?; // Placeholder
            writeln!(buffer, "pub enum {name} {{")?;
            for variant in variants {
                // Heuristic: specific variant naming/normalization might be needed here.
                // For raw strings, we might need a mapping.
                // Assuming variants are valid identifiers for now.
                writeln!(buffer, "    {variant},")?;
            }
            writeln!(buffer, "}}")?;
            writeln!(buffer, "#__SLOPCHOP_END__#")?;
        }
        RefactorStrategy::GenericParameter { name } => {
            // Explain the generic refactor
            writeln!(buffer, "Refactor using generic parameter: {name}")?;
        }
        RefactorStrategy::FunctionArgument { name, ty } => {
             // Explain argument refactor
             writeln!(buffer, "Refactor by adding argument '{name}: {ty}'")?;
        }
        RefactorStrategy::ManualAttention => {
            writeln!(buffer, "Manual attention required. No code generated.")?;
        }
    }

    Ok(buffer)
}
