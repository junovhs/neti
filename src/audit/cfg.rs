// src/audit/cfg.rs
//! Control Flow Graph node classification and normalization.
//!
//! Separates CFG semantics from fingerprint computation for clarity.

/// Control flow node types that affect program execution path.
pub const CFG_NODES: &[&str] = &[
    "if_expression",
    "else_clause",
    "match_expression",
    "match_arm",
    "for_expression",
    "while_expression",
    "loop_expression",
    "return_expression",
    "break_expression",
    "continue_expression",
    "try_expression",
    "block",
    "closure_expression",
];

/// Nodes that introduce branching (decision points).
pub const BRANCH_NODES: &[&str] = &["if_expression", "match_arm", "try_expression"];

/// Nodes that introduce iteration.
pub const LOOP_NODES: &[&str] = &["for_expression", "while_expression", "loop_expression"];

/// Nodes that exit current control flow (return, break, continue).
pub const EXIT_NODES: &[&str] = &["return_expression", "break_expression", "continue_expression"];

/// Tokens with structural significance (operators, keywords).
pub const STRUCTURAL_TOKENS: &[&str] = &[
    "+", "-", "*", "/", "%", "&&", "||", "!", "==", "!=", "<", ">", "<=", ">=",
    "&", "|", "^", "<<", ">>", "+=", "-=", "*=", "/=", "if", "else", "match",
    "while", "for", "loop", "return", "break", "continue", "let", "mut", "const",
    "fn", "struct", "enum", "impl", "trait", "pub", "async", "await", "move",
    "=>", "->", "?", "true", "false", "None", "Some", "Ok", "Err",
];

/// AST node kinds that represent identifiers/literals (normalized away).
pub const IDENTIFIER_KINDS: &[&str] = &[
    "identifier",
    "type_identifier",
    "field_identifier",
    "scoped_identifier",
    "scoped_type_identifier",
    "string_literal",
    "raw_string_literal",
    "char_literal",
    "integer_literal",
    "float_literal",
];

/// Normalized CFG node categories for Level 4 equivalence detection.
pub struct CfgCategory;

impl CfgCategory {
    pub const BRANCH: &str = "BRANCH";
    pub const BRANCH_ARM: &str = "BRANCH_ARM";
    pub const LOOP: &str = "LOOP";
    pub const RETURN: &str = "RETURN";
    pub const LOOP_EXIT: &str = "LOOP_EXIT";
    pub const BLOCK: &str = "BLOCK";
    pub const CLOSURE: &str = "CLOSURE";
    pub const TRY: &str = "TRY";
}

/// Normalizes a CFG node kind to its semantic category.
/// Enables detecting equivalent control flow with different syntax.
#[must_use]
pub fn normalize(kind: &str) -> &str {
    // Lookup table approach avoids match complexity
    if is_branch_node(kind) {
        return CfgCategory::BRANCH;
    }
    if is_branch_arm(kind) {
        return CfgCategory::BRANCH_ARM;
    }
    if is_loop_node(kind) {
        return CfgCategory::LOOP;
    }
    categorize_exit_or_special(kind)
}

fn is_branch_node(kind: &str) -> bool {
    kind == "if_expression" || kind == "match_expression"
}

fn is_branch_arm(kind: &str) -> bool {
    kind == "else_clause" || kind == "match_arm"
}

fn is_loop_node(kind: &str) -> bool {
    kind == "for_expression" || kind == "while_expression" || kind == "loop_expression"
}

fn categorize_exit_or_special(kind: &str) -> &str {
    match kind {
        "return_expression" => CfgCategory::RETURN,
        "break_expression" | "continue_expression" => CfgCategory::LOOP_EXIT,
        "block" => CfgCategory::BLOCK,
        "closure_expression" => CfgCategory::CLOSURE,
        "try_expression" => CfgCategory::TRY,
        _ => kind,
    }
}