// src/mutate/mutations.rs
//! Mutation types and application logic.
//!
//! Defines what mutations are possible and how to apply/revert them.

use std::path::PathBuf;

/// A single mutation point discovered in the codebase.
#[derive(Debug, Clone)]
pub struct MutationPoint {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub original: String,
    pub mutated: String,
    pub kind: MutationKind,
}

/// Categories of mutations we can apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationKind {
    /// Comparison operators: == != < > <= >=
    Comparison,
    /// Logical operators: && ||
    Logical,
    /// Boolean literals: true false
    Boolean,
    /// Arithmetic operators: + - * /
    Arithmetic,
    /// Return value mutations
    ReturnValue,
}

impl MutationKind {
    #[must_use]
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Comparison => "CMP",
            Self::Logical => "LOG",
            Self::Boolean => "BOOL",
            Self::Arithmetic => "MATH",
            Self::ReturnValue => "RET",
        }
    }
}

/// Attempts to create a mutation for an operator or literal.
#[must_use]
pub fn get_mutation(text: &str) -> Option<(&'static str, MutationKind)> {
    get_comparison(text)
        .or_else(|| get_logical(text))
        .or_else(|| get_boolean(text))
        .or_else(|| get_arithmetic(text))
}

fn get_comparison(op: &str) -> Option<(&'static str, MutationKind)> {
    let mutated = match op {
        "==" => "!=",
        "!=" => "==",
        "<" => ">=",
        ">" => "<=",
        "<=" => ">",
        ">=" => "<",
        _ => return None,
    };
    Some((mutated, MutationKind::Comparison))
}

fn get_logical(op: &str) -> Option<(&'static str, MutationKind)> {
    let mutated = match op {
        "&&" => "||",
        "||" => "&&",
        "and" => "or", // Python
        "or" => "and", // Python
        _ => return None,
    };
    Some((mutated, MutationKind::Logical))
}

fn get_boolean(op: &str) -> Option<(&'static str, MutationKind)> {
    let mutated = match op {
        "true" => "false",
        "false" => "true",
        "True" => "False", // Python
        "False" => "True", // Python
        _ => return None,
    };
    Some((mutated, MutationKind::Boolean))
}

fn get_arithmetic(op: &str) -> Option<(&'static str, MutationKind)> {
    let mutated = match op {
        "+" => "-",
        "-" => "+",
        "*" => "/",
        "/" => "*",
        _ => return None,
    };
    Some((mutated, MutationKind::Arithmetic))
}

/// Applies a mutation to source code, returning the mutated version.
#[must_use]
pub fn apply_mutation(source: &str, point: &MutationPoint) -> String {
    let mut result = source.to_string();
    result.replace_range(point.byte_start..point.byte_end, &point.mutated);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_mutations() {
        assert_eq!(get_mutation("=="), Some(("!=", MutationKind::Comparison)));
        assert_eq!(get_mutation("<"), Some((">=", MutationKind::Comparison)));
    }

    #[test]
    fn test_logical_mutations() {
        assert_eq!(get_mutation("&&"), Some(("||", MutationKind::Logical)));
        assert_eq!(get_mutation("and"), Some(("or", MutationKind::Logical)));
    }

    #[test]
    fn test_boolean_mutations() {
        assert_eq!(get_mutation("true"), Some(("false", MutationKind::Boolean)));
        assert_eq!(get_mutation("True"), Some(("False", MutationKind::Boolean)));
    }

    #[test]
    fn test_no_mutation() {
        assert_eq!(get_mutation("foo"), None);
        assert_eq!(get_mutation("let"), None);
    }
}
