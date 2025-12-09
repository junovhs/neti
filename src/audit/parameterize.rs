use super::diff::{DiffModel, Hole};

/// Strategy for handling a specific difference (hole).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RefactorStrategy {
    /// Extract hole values to a new Enum.
    ExtractEnum {
        name: String,
        variants: Vec<String>,
    },
    /// Pass hole values as a generic arguments/closures.
    GenericParameter {
        name: String,
        // TODO: Type bounds
    },
    /// Pass hole values as a simple function argument.
    FunctionArgument {
        name: String,
        ty: String, // e.g. "String", "bool", "usize"
    },
    /// Complex/Unknown difference - fallback to "Manual" or explanation only.
    ManualAttention,
}

/// Infers strategies for all holes in the diff model.
#[must_use]
pub fn infer_strategies(model: &DiffModel) -> Vec<RefactorStrategy> {
    model.holes.iter().map(analyze_hole).collect()
}

fn analyze_hole(hole: &Hole) -> RefactorStrategy {
    match hole.kind.as_str() {
        "string_literal" => infer_enum_or_arg(hole, "str"),
        "integer_literal" => infer_enum_or_arg(hole, "int"),
        "boolean_literal" => RefactorStrategy::FunctionArgument {
            name: "flag".to_string(),
            ty: "bool".to_string(),
        },
        "identifier" | "field_identifier" => RefactorStrategy::GenericParameter {
            name: "param".to_string(),
        },
        _ => RefactorStrategy::ManualAttention,
    }
}

fn infer_enum_or_arg(hole: &Hole, base_type: &str) -> RefactorStrategy {
    // Heuristic: If there are few variants (e.g. 2-3) and they look like "modes" or "options",
    // suggest an Enum. If they are many or look like arbitrary data, suggest an argument.
    
    // For now, default to Enum if <= 3 variants, otherwise argument.
    if hole.variants.len() <= 3 {
        let enum_name = format!("{}Options", base_type.to_uppercase()); // Placeholder name
        RefactorStrategy::ExtractEnum {
            name: enum_name,
            variants: hole.variants.clone(),
        }
    } else {
        RefactorStrategy::FunctionArgument {
            name: "value".to_string(),
            ty: match base_type {
                "int" => "usize".to_string(),
                _ => "String".to_string(), // Simplified type inference
            },
        }
    }
}
