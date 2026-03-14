//! Heuristics to reject item-level doc comments mistaken for module docs.

pub fn looks_like_item_doc(comment: &str) -> bool {
    let trimmed = comment.trim().trim_end_matches('.');
    let lower = trimmed.to_lowercase();
    let first_word = lower.split_whitespace().next().unwrap_or("");

    if is_license_header(&lower) {
        return true;
    }
    if is_imperative_verb(first_word) {
        return true;
    }
    if starts_with_linking_verb(&lower) {
        return true;
    }
    if starts_with_state_prefix(&lower) {
        return true;
    }
    if starts_with_article_lowercase(trimmed) {
        return true;
    }
    if starts_with_quantity_prefix(&lower) {
        return true;
    }
    if references_internals(&lower) {
        return true;
    }
    if is_truncated(trimmed) {
        return true;
    }

    trimmed.split_whitespace().count() < 3
}

fn is_license_header(lower: &str) -> bool {
    const LICENSE_PREFIXES: &[&str] = &[
        "copyright",
        "spdx-license",
        "licensed under",
        "this file is part of",
        "permission is hereby granted",
        "redistribution and use",
        "this program is free software",
        "all rights reserved",
    ];
    LICENSE_PREFIXES
        .iter()
        .any(|p| lower.starts_with(p) || lower.contains(p))
}

const VERB_BASES: &[&str] = &[
    "add",
    "allow",
    "apply",
    "build",
    "calculate",
    "call",
    "check",
    "compile",
    "compute",
    "connect",
    "continue",
    "convert",
    "create",
    "decode",
    "delete",
    "determine",
    "disconnect",
    "emit",
    "encode",
    "ensure",
    "execute",
    "extract",
    "fetch",
    "fill",
    "find",
    "flush",
    "format",
    "generate",
    "get",
    "handle",
    "implement",
    "indicate",
    "initialize",
    "insert",
    "invoke",
    "load",
    "look",
    "map",
    "merge",
    "notify",
    "open",
    "parse",
    "perform",
    "process",
    "produce",
    "push",
    "put",
    "read",
    "receive",
    "register",
    "remove",
    "render",
    "represent",
    "request",
    "reset",
    "resolve",
    "return",
    "round",
    "run",
    "save",
    "scan",
    "send",
    "serialize",
    "set",
    "skip",
    "sort",
    "specify",
    "start",
    "stop",
    "store",
    "strip",
    "submit",
    "test",
    "toggle",
    "transform",
    "trigger",
    "update",
    "validate",
    "verify",
    "visit",
    "walk",
    "write",
    "yield",
];

fn is_imperative_verb(word: &str) -> bool {
    if VERB_BASES.contains(&word) {
        return true;
    }
    to_verb_base(word).is_some_and(|b| VERB_BASES.contains(&b.as_str()))
}

fn to_verb_base(word: &str) -> Option<String> {
    if let Some(s) = word.strip_suffix("ies") {
        return Some(format!("{s}y"));
    }
    if let Some(base) = word.strip_suffix('s') {
        if VERB_BASES.contains(&base) {
            return Some(base.to_string());
        }
        if let Some(es_base) = word.strip_suffix("es") {
            if VERB_BASES.contains(&es_base) {
                return Some(es_base.to_string());
            }
        }
        return Some(base.to_string());
    }
    None
}

fn starts_with_linking_verb(lower: &str) -> bool {
    let mut words = lower.split_whitespace();
    let first = words.next().unwrap_or("");
    let second = words.next().unwrap_or("");

    matches!(second, "is" | "are" | "was" | "were")
        && !matches!(first, "this" | "it" | "there" | "here")
}

fn starts_with_state_prefix(lower: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "whether ",
        "if ",
        "when ",
        "true ",
        "false ",
        "set to ",
        "used to ",
        "used for ",
        "called when",
        "called by",
        "called after",
        "called before",
    ];
    PREFIXES.iter().any(|p| lower.starts_with(p))
}

fn starts_with_article_lowercase(text: &str) -> bool {
    for prefix in &["a ", "an "] {
        if text.to_lowercase().starts_with(prefix) {
            let rest = &text[prefix.len()..];
            if rest.starts_with(|c: char| c.is_lowercase()) && !rest.is_empty() {
                return true;
            }
        }
    }
    false
}

fn starts_with_quantity_prefix(lower: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "maximum ",
        "minimum ",
        "default ",
        "number of ",
        "count of ",
        "size of ",
        "the number",
        "the count",
        "the size",
    ];
    PREFIXES.iter().any(|p| lower.starts_with(p))
}

fn references_internals(lower: &str) -> bool {
    const TERMS: &[&str] = &[
        "field",
        "parameter",
        "argument",
        "return value",
        "struct member",
        "enum variant",
        "callback",
        "handler",
    ];
    TERMS.iter().any(|term| lower.contains(term))
}

fn is_truncated(trimmed: &str) -> bool {
    trimmed.ends_with(':') || trimmed.ends_with('-') || trimmed.ends_with("e.g")
}
