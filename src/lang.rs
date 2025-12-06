use tree_sitter::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Rust,
    Python,
    TypeScript,
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
    pub fn grammar(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::language(),
            Self::Python => tree_sitter_python::language(),
            Self::TypeScript => tree_sitter_typescript::language_typescript(),
        }
    }

    #[must_use]
    pub fn skeleton_replacement(&self) -> &'static str {
        match self {
            Self::Rust | Self::TypeScript => "{ ... }",
            Self::Python => "...",
        }
    }

    // --- QUERIES ---

    #[must_use]
    pub fn q_naming(&self) -> &'static str {
        match self {
            Self::Rust => "(function_item name: (identifier) @name)",
            Self::Python => "(function_definition name: (identifier) @name)",
            Self::TypeScript => r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
        }
    }

    #[must_use]
    pub fn q_complexity(&self) -> &'static str {
        match self {
            Self::Rust => r#"
                (if_expression) @branch
                (match_arm) @branch
                (while_expression) @branch
                (for_expression) @branch
                (binary_expression operator: ["&&" "||"]) @branch
            "#,
            Self::Python => r"
                (if_statement) @branch
                (for_statement) @branch
                (while_statement) @branch
                (except_clause) @branch
                (boolean_operator) @branch
            ",
            Self::TypeScript => r#"
                (if_statement) @branch
                (for_statement) @branch
                (for_in_statement) @branch
                (while_statement) @branch
                (do_statement) @branch
                (switch_case) @branch
                (catch_clause) @branch
                (ternary_expression) @branch
                (binary_expression operator: ["&&" "||" "??"]) @branch
            "#,
        }
    }

    #[must_use]
    pub fn q_banned(&self) -> Option<&'static str> {
        match self {
            Self::Rust => Some(r"(call_expression function: (field_expression field: (field_identifier) @method)) @call"),
            _ => None,
        }
    }

    #[must_use]
    pub fn q_imports(&self) -> &'static str {
        match self {
            Self::Rust => r"
                (use_declaration argument: (_) @import)
                (mod_item name: (identifier) @mod)
            ",
            Self::Python => r"
                (import_statement name: (dotted_name) @import)
                (aliased_import name: (dotted_name) @import)
                (import_from_statement module_name: (dotted_name) @import)
            ",
            Self::TypeScript => r#"
                (import_statement source: (string) @import)
                (export_statement source: (string) @import)
                (call_expression
                  function: (identifier) @func
                  arguments: (arguments (string) @import)
                  (#eq? @func "require"))
            "#,
        }
    }

    #[must_use]
    pub fn q_defs(&self) -> &'static str {
        match self {
            Self::Rust => r"
                (function_item name: (identifier) @name) @sig
                (struct_item name: (type_identifier) @name) @sig
                (enum_item name: (type_identifier) @name) @sig
                (trait_item name: (type_identifier) @name) @sig
                (impl_item type: (type_identifier) @name) @sig
                (const_item name: (identifier) @name) @sig
                (static_item name: (identifier) @name) @sig
                (type_item name: (type_identifier) @name) @sig
            ",
            Self::Python => r"
                (function_definition name: (identifier) @name) @sig
                (class_definition name: (identifier) @name) @sig
            ",
            Self::TypeScript => r"
                (function_declaration name: (identifier) @name) @sig
                (class_declaration name: (type_identifier) @name) @sig
                (interface_declaration name: (type_identifier) @name) @sig
                (type_alias_declaration name: (type_identifier) @name) @sig
            ",
        }
    }

    /// Queries for identifying public/exported items.
    #[must_use]
    pub fn q_exports(&self) -> &'static str {
        match self {
            Self::Rust => r"
                (function_item (visibility_modifier)) @export
                (struct_item (visibility_modifier)) @export
                (enum_item (visibility_modifier)) @export
                (trait_item (visibility_modifier)) @export
                (const_item (visibility_modifier)) @export
                (static_item (visibility_modifier)) @export
                (type_item (visibility_modifier)) @export
                (impl_item) @export
                (mod_item (visibility_modifier)) @export
            ",
            Self::TypeScript => r"
                (export_statement) @export
            ",
            Self::Python => r"
                (function_definition) @export
                (class_definition) @export
            ",
        }
    }

    #[must_use]
    pub fn q_skeleton(&self) -> &'static str {
        match self {
            Self::Rust => "(function_item body: (block) @body) (impl_item body: (declaration_list) @body)",
            Self::Python => "(function_definition body: (block) @body)",
            Self::TypeScript => r"
                (function_declaration body: (statement_block) @body)
                (method_definition body: (statement_block) @body)
                (arrow_function body: (statement_block) @body)
            ",
        }
    }
}