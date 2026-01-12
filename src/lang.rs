use tree_sitter::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Rust,
    Python,
    TypeScript,
}

#[derive(Debug, Clone, Copy)]
pub enum QueryKind {
    Naming,
    Complexity,
    Imports,
    Defs,
    Exports,
    Skeleton,
}

impl Lang {
    #[must_use]
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "ts" | "tsx" | "js" | "jsx" => Some(Self::TypeScript),
            _ => None,
        }
    }

    #[must_use]
    pub fn grammar(self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::language(),
            Self::Python => tree_sitter_python::language(),
            Self::TypeScript => tree_sitter_typescript::language_typescript(),
        }
    }

    // Indexing is safe: lang_idx and query_idx are bounded by enum variant count
    // which matches the QUERIES array dimensions exactly
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn query(self, kind: QueryKind) -> &'static str {
        let lang_idx = self as usize;
        let query_idx = kind as usize;
        QUERIES[lang_idx][query_idx]
    }

    // Helpers for compatibility with existing modules
    #[must_use]
    pub fn q_naming(self) -> &'static str {
        self.query(QueryKind::Naming)
    }
    #[must_use]
    pub fn q_complexity(self) -> &'static str {
        self.query(QueryKind::Complexity)
    }
    #[must_use]
    pub fn q_imports(self) -> &'static str {
        self.query(QueryKind::Imports)
    }
    #[must_use]
    pub fn q_defs(self) -> &'static str {
        self.query(QueryKind::Defs)
    }
    #[must_use]
    pub fn q_exports(self) -> &'static str {
        self.query(QueryKind::Exports)
    }
    #[must_use]
    pub fn q_skeleton(self) -> &'static str {
        self.query(QueryKind::Skeleton)
    }
    #[must_use]
    pub fn skeleton_replacement(self) -> &'static str {
        if self == Self::Python {
            "\n    ..."
        } else {
            "{ ... }"
        }
    }
}

// [Rust, Python, TypeScript] x [Naming, Complexity, Imports, Defs, Exports, Skeleton]
const QUERIES: [[&str; 6]; 3] = [
    // Rust
    [
        "(function_item name: (identifier) @name)", // Naming
        r"
            (if_expression) @branch
            (match_arm) @branch
            (while_expression) @branch
            (for_expression) @branch
            (binary_expression) @branch
        ", // Complexity
        r"
            (use_declaration argument: (_) @import)
            (mod_item name: (identifier) @mod)
        ", // Imports
        r"
            (function_item name: (identifier) @name) @sig
            (struct_item name: (type_identifier) @name) @sig
            (enum_item name: (type_identifier) @name) @sig
            (trait_item name: (type_identifier) @name) @sig
            (impl_item type: (type_identifier) @name) @sig
            (const_item name: (identifier) @name) @sig
            (static_item name: (identifier) @name) @sig
            (type_item name: (type_identifier) @name) @sig
            (mod_item name: (identifier) @name) @sig
        ", // Defs
        r"
            (function_item (visibility_modifier)) @export
            (struct_item (visibility_modifier)) @export
            (enum_item (visibility_modifier)) @export
            (trait_item (visibility_modifier)) @export
            (const_item (visibility_modifier)) @export
            (static_item (visibility_modifier)) @export
            (type_item (visibility_modifier)) @export
            (impl_item) @export
            (mod_item (visibility_modifier)) @export
        ", // Exports
        "(function_item body: (block) @body) (impl_item body: (declaration_list) @body)", // Skeleton
    ],
    // Python
    [
        "(function_definition name: (identifier) @name)", // Naming
        r"
            (if_statement) @branch
            (for_statement) @branch
            (while_statement) @branch
            (except_clause) @branch
            (boolean_operator) @branch
        ", // Complexity
        r"
            (import_statement name: (dotted_name) @import)
            (aliased_import name: (dotted_name) @import)
            (import_from_statement module_name: (dotted_name) @import)
        ", // Imports
        r"
            (function_definition name: (identifier) @name) @sig
            (class_definition name: (identifier) @name) @sig
        ", // Defs
        r"
            (function_definition) @export
            (class_definition) @export
        ", // Exports
        "(function_definition body: (block) @body)",      // Skeleton
    ],
    // TypeScript
    [
        r"
            (function_declaration name: (identifier) @name)
            (method_definition name: (property_identifier) @name)
            (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
        ", // Naming
        r#"
            (if_statement) @branch
            (for_statement) @branch
            (for_in_statement) @branch
            (while_statement) @branch
            (do_statement) @branch
            (switch_case) @branch
            (catch_clause) @branch
            (ternary_expression) @branch
            (binary_expression operator: ["&&" "||" "??"]) @branch
        "#, // Complexity
        r#"
            (import_statement source: (string) @import)
            (export_statement source: (string) @import)
            (call_expression
              function: (identifier) @func
              arguments: (arguments (string) @import)
              (#eq? @func "require"))
        "#, // Imports
        r"
            (function_declaration name: (identifier) @name) @sig
            (class_declaration name: (type_identifier) @name) @sig
            (interface_declaration name: (type_identifier) @name) @sig
            (type_alias_declaration name: (type_identifier) @name) @sig
        ", // Defs
        r"
            (export_statement) @export
        ", // Exports
        r"
            (function_declaration body: (statement_block) @body)
            (method_definition body: (statement_block) @body)
            (arrow_function body: (statement_block) @body)
        ", // Skeleton
    ],
];
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_ext() {
        assert_eq!(Lang::from_ext("rs"), Some(Lang::Rust));
    }
}
