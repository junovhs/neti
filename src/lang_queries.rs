// src/lang_queries.rs
pub const QUERIES: [[&str; 6]; 4] = [
    // Rust
    [
        "(function_item name: (identifier) @name)",
        r"
            (if_expression) @branch
            (match_arm) @branch
            (while_expression) @branch
            (for_expression) @branch
            (binary_expression) @branch
        ",
        r"
            (use_declaration argument: (_) @import)
            (mod_item name: (identifier) @mod)
        ",
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
        ",
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
        ",
        "(function_item body: (block) @body) (impl_item body: (declaration_list) @body)",
    ],
    // Python
    [
        "(function_definition name: (identifier) @name)",
        r"
            (if_statement) @branch
            (for_statement) @branch
            (while_statement) @branch
            (except_clause) @branch
            (boolean_operator) @branch
        ",
        r"
            (import_statement name: (dotted_name) @import)
            (aliased_import name: (dotted_name) @import)
            (import_from_statement module_name: (dotted_name) @import)
        ",
        r"
            (function_definition name: (identifier) @name) @sig
            (class_definition name: (identifier) @name) @sig
        ",
        r"
            (function_definition) @export
            (class_definition) @export
        ",
        "(function_definition body: (block) @body)",
    ],
    // TypeScript
    [
        r"
            (function_declaration name: (identifier) @name)
            (method_definition name: (property_identifier) @name)
            (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
        ",
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
        "#,
        r#"
            (import_statement source: (string) @import)
            (export_statement source: (string) @import)
            (call_expression
              function: (identifier) @func
              arguments: (arguments (string) @import)
              (#eq? @func "require"))
        "#,
        r"
            (function_declaration name: (identifier) @name) @sig
            (class_declaration name: (type_identifier) @name) @sig
            (interface_declaration name: (type_identifier) @name) @sig
            (type_alias_declaration name: (type_identifier) @name) @sig
        ",
        r"
            (export_statement) @export
        ",
        r"
            (function_declaration body: (statement_block) @body)
            (method_definition body: (statement_block) @body)
            (arrow_function body: (statement_block) @body)
        ",
    ],
    // Swift
    [
        // Naming
        "(function_declaration name: (simple_identifier) @name)",
        // Complexity
        r#"
            (if_statement) @branch
            (guard_statement) @branch
            (for_statement) @branch
            (while_statement) @branch
            (repeat_while_statement) @branch
            (catch_clause) @branch
            (switch_statement) @branch
            (ternary_expression) @branch
            (binary_expression) @branch
        "#,
        // Imports
        r"
            (import_declaration (identifier) @import)
            (import_declaration (scoped_identifier) @import)
        ",
        // Defs
        r"
            (function_declaration name: (simple_identifier) @name) @sig
            (class_declaration name: (type_identifier) @name) @sig
            (struct_declaration name: (type_identifier) @name) @sig
            (enum_declaration name: (type_identifier) @name) @sig
            (protocol_declaration name: (type_identifier) @name) @sig
            (typealias_declaration name: (type_identifier) @name) @sig
        ",
        // Exports
        r#"
            (function_declaration (modifiers) @vis) @export
            (class_declaration (modifiers) @vis) @export
            (struct_declaration (modifiers) @vis) @export
            (enum_declaration (modifiers) @vis) @export
            (protocol_declaration (modifiers) @vis) @export
        "#,
        // Skeleton
        r"
            (function_declaration body: (function_body) @body)
            (initializer_declaration body: (function_body) @body)
            (deinitializer_declaration body: (function_body) @body)
        ",
    ],
];
