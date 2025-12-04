// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
use crate::roadmap::{diff, Command, Roadmap};
use regex::Regex;
use std::fmt::Write;
use std::path::Path;
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
    ".slopchop_apply_backup/",
];

const ALLOWED_DOTFILES: &[&str] = &[".gitignore", ".slopchopignore", ".slopchop_intent"];

static LAZY_MARKERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"^\s*//\s*\.{3,}\s*$",
        r"^\s*/\*\s*\.{3,}\s*\*/\s*$",
        r"(?i)^\s*//.*(rest of|remaining|existing|implement|logic here).*$",
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

    for path in extracted.keys() {
        if path.eq_ignore_ascii_case("ROADMAP.md") {
            if let Some(outcome) = handle_roadmap_rewrite(path, &extracted[path].content) {
                return outcome;
            }
            errors.push(
                "PROTECTED: ROADMAP.md is managed programmatically. Use 'slopchop roadmap apply' commands instead of rewriting the file.".to_string(),
            );
        } else {
            validate_single_path(path, &mut errors);
        }
    }

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

    build_success_outcome(manifest, extracted)
}

fn build_success_outcome(manifest: &Manifest, extracted: &ExtractedFiles) -> ApplyOutcome {
    let written = extracted.keys().cloned().collect();
    let deleted = manifest
        .iter()
        .filter(|e| e.operation == Operation::Delete)
        .map(|e| e.path.clone())
        .collect();

    ApplyOutcome::Success {
        written,
        deleted,
        roadmap_results: Vec::new(),
        backed_up: true,
    }
}

fn handle_roadmap_rewrite(path: &str, incoming_content: &str) -> Option<ApplyOutcome> {
    let commands = diff_roadmap(path, incoming_content)?;

    if commands.is_empty() {
        return None;
    }

    let msg = build_roadmap_rejection_message(&commands);

    Some(ApplyOutcome::ValidationFailure {
        errors: vec!["Roadmap rewrite converted to commands".to_string()],
        missing: vec![],
        ai_message: msg,
    })
}

fn diff_roadmap(path: &str, incoming_content: &str) -> Option<Vec<Command>> {
    let local_path = Path::new(path);
    if !local_path.exists() {
        return None;
    }

    let current = Roadmap::from_file(local_path).ok()?;
    let incoming = Roadmap::parse(incoming_content);
    Some(diff::diff(&current, &incoming))
}

fn build_roadmap_rejection_message(commands: &[Command]) -> String {
    let mut msg = String::new();
    let _ = writeln!(
        msg,
        "The SlopChop Protocol blocked a direct rewrite of ROADMAP.md.\n"
    );
    let _ = writeln!(
        msg,
        "However, I inferred your intent. Please use these commands instead:\n"
    );
    let _ = writeln!(msg, "#__SLOPCHOP_FILE__# ROADMAP");
    let _ = writeln!(msg, "===ROADMAP===");

    for cmd in commands {
        let _ = writeln!(msg, "{cmd}");
    }

    let _ = writeln!(msg, "===END===");
    let _ = writeln!(msg, "#__SLOPCHOP_END__#");
    msg
}

fn validate_single_path(path: &str, errors: &mut Vec<String>) {
    if path.eq_ignore_ascii_case("ROADMAP.md") {
        return;
    }

    if let Some(err) = check_path_security(path) {
        errors.push(err);
    }
}

fn check_path_security(path: &str) -> Option<String> {
    if has_traversal(path) {
        return Some(format!(
            "SECURITY: path contains directory traversal: {path}"
        ));
    }
    if is_absolute_path(path) {
        return Some(format!("SECURITY: absolute path not allowed: {path}"));
    }
    if is_sensitive_path(path) {
        return Some(format!("SECURITY: sensitive path blocked: {path}"));
    }
    if is_hidden_file(path) {
        return Some(format!("SECURITY: hidden file not allowed: {path}"));
    }
    None
}

fn has_traversal(path: &str) -> bool {
    path.contains("../") || path.starts_with("..")
}

fn is_absolute_path(path: &str) -> bool {
    if path.starts_with('/') {
        return true;
    }
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn is_sensitive_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    SENSITIVE_PATHS.iter().any(|s| lower.contains(s))
}

fn is_hidden_file(path: &str) -> bool {
    if ALLOWED_DOTFILES.iter().any(|&f| path.ends_with(f)) {
        return false;
    }

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
        if line.contains("slopchop:ignore") {
            continue;
        }

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
