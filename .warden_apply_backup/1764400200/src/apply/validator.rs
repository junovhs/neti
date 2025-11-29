// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
use regex::Regex;
use std::sync::LazyLock;

const SENSITIVE_PATHS: &[&str] = &[
    ".git/",
    ".env",
    ".ssh/",
    ".aws/",
    ".gnupg/",
    "id_rsa",
    "id_ed25519",
    "credentials",
    ".warden_apply_backup/",
];

/// Compiled regex patterns for detecting lazy/truncated AI output.
/// These are compile-time constant patterns; if any fail to compile,
/// it's a programmer error that will surface immediately at first use.
static LAZY_MARKERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        // // ...
        r"^\s*//\s*\.{3,}\s*$",
        // /* ... */
        r"^\s*/\*\s*\.{3,}\s*\*/\s*$",
        // // ... rest of code, // remaining code, etc.
        r"(?i)^\s*//.*(rest of|remaining|existing|implement|logic here).*$",
        // # ... (Python style)
        r"^\s*#\s*\.{3,}\s*$",
    ]
    .iter()
    .filter_map(|pattern| match Regex::new(pattern) {
        Ok(re) => Some(re),
        Err(e) => {
            eprintln!("Warning: Invalid lazy marker pattern '{pattern}': {e}");
            None
        }
    })
    .collect()
});

#[must_use]
pub fn validate(manifest: &Manifest, extracted: &ExtractedFiles) -> ApplyOutcome {
    let mut errors = Vec::new();
    check_path_safety(extracted, &mut errors);

    if !errors.is_empty() {
        let ai_message = messages::format_ai_rejection(&[], &errors);
        return ApplyOutcome::ValidationFailure {
            errors,
            missing: Vec::new(),
            ai_message,
        };
    }

    let missing = check_missing(manifest, extracted);
    let content_errors = check_content(extracted);

    if !missing.is_empty() || !content_errors.is_empty() {
        let ai_message = messages::format_ai_rejection(&missing, &content_errors);
        return ApplyOutcome::ValidationFailure {
            errors: content_errors,
            missing,
            ai_message,
        };
    }

    let written = extracted.keys().cloned().collect();
    let deleted = manifest
        .iter()
        .filter(|e| e.operation == Operation::Delete)
        .map(|e| e.path.clone())
        .collect();

    ApplyOutcome::Success {
        written,
        deleted,
        backed_up: true,
    }
}

fn check_path_safety(extracted: &ExtractedFiles, errors: &mut Vec<String>) {
    for path in extracted.keys() {
        validate_single_path(path, errors);
    }
}

fn validate_single_path(path: &str, errors: &mut Vec<String>) {
    if has_traversal(path) {
        errors.push(format!(
            "SECURITY: path contains directory traversal: {path}"
        ));
        return;
    }

    if is_absolute_path(path) {
        errors.push(format!("SECURITY: absolute path not allowed: {path}"));
        return;
    }

    if is_sensitive_path(path) {
        errors.push(format!("SECURITY: sensitive path blocked: {path}"));
        return;
    }

    if is_hidden_file(path) {
        errors.push(format!("SECURITY: hidden file not allowed: {path}"));
    }
}

fn has_traversal(path: &str) -> bool {
    path.contains("../") || path.starts_with("..")
}

fn is_absolute_path(path: &str) -> bool {
    if path.starts_with('/') {
        return true;
    }
    // Windows drive letter check (e.g., C:\)
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn is_sensitive_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    SENSITIVE_PATHS.iter().any(|s| lower.contains(s))
}

fn is_hidden_file(path: &str) -> bool {
    path.split('/')
        .filter(|s| !s.is_empty())
        .any(|seg| seg.starts_with('.') && seg != "." && seg != "..")
}

fn check_missing(manifest: &Manifest, extracted: &ExtractedFiles) -> Vec<String> {
    manifest
        .iter()
        .filter(|entry| entry.operation != Operation::Delete)
        .filter(|entry| !extracted.contains_key(&entry.path))
        .map(|entry| entry.path.clone())
        .collect()
}

fn check_content(extracted: &ExtractedFiles) -> Vec<String> {
    let mut errors = Vec::new();
    for (path, file) in extracted {
        check_single_file(path, &file.content, &mut errors);
    }
    errors
}

fn check_single_file(path: &str, content: &str, errors: &mut Vec<String>) {
    if content.trim().is_empty() {
        errors.push(format!("{path} is empty"));
        return;
    }
    check_lazy_truncation(path, content, errors);
}

fn check_lazy_truncation(path: &str, content: &str, errors: &mut Vec<String>) {
    for (line_num, line) in content.lines().enumerate() {
        for regex in LAZY_MARKERS.iter() {
            if regex.is_match(line) {
                errors.push(format!(
                    "{path}:{}: Detected lazy truncation marker: '{}'. Full file required.",
                    line_num + 1,
                    line.trim()
                ));
            }
        }
    }
}
