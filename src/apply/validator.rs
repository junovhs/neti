// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
use crate::roadmap::{diff, Roadmap, Command, MovePosition};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;
use std::fmt::Write;

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

const ALLOWED_DOTFILES: &[&str] = &[
    ".gitignore",
    ".wardenignore",
    ".warden_intent",
];

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
    
    // Check path safety and handle ROADMAP.md specifically
    for path in extracted.keys() {
         if path.eq_ignore_ascii_case("ROADMAP.md") {
             if let Some(outcome) = handle_roadmap_rewrite(path, &extracted[path].content) {
                 return outcome;
             }
             // If we couldn't infer commands or load the file, default to standard block
             errors.push(
                "PROTECTED: ROADMAP.md is managed programmatically. Use 'warden roadmap apply' commands instead of rewriting the file.".to_string(),
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

/// Attempts to diff the incoming ROADMAP.md with the existing one // warden:ignore
/// and return a specific rejection message with proposed commands.
fn handle_roadmap_rewrite(path: &str, incoming_content: &str) -> Option<ApplyOutcome> {
    let local_path = Path::new(path);
    if !local_path.exists() {
        return None;
    }

    let Ok(current) = Roadmap::from_file(local_path) else {
        return None;
    };
    
    let incoming = Roadmap::parse(incoming_content);
    let commands = diff::diff(&current, &incoming);
    
    if commands.is_empty() {
        return None;
    }

    let mut msg = String::new();
    let _ = writeln!(msg, "The Warden Protocol blocked a direct rewrite of ROADMAP.md.\n");
    let _ = writeln!(msg, "However, I inferred your intent. Please use these commands instead:\n");
    let _ = writeln!(msg, "∇∇∇ ROADMAP ∇∇∇");
    let _ = writeln!(msg, "===ROADMAP===");
    
    for cmd in commands {
        match cmd {
            Command::Check { path } => { let _ = writeln!(msg, "CHECK {path}"); },
            Command::Uncheck { path } => { let _ = writeln!(msg, "UNCHECK {path}"); },
            Command::Update { path, text } => { let _ = writeln!(msg, "UPDATE {path} \"{text}\""); },
            Command::Add { parent, text, .. } => { let _ = writeln!(msg, "ADD {parent} \"{text}\""); },
            Command::Delete { path } => { let _ = writeln!(msg, "DELETE {path}"); },
            Command::AddSection { heading } => { let _ = writeln!(msg, "SECTION \"{heading}\""); },
            Command::Move { path, position } => {
                match position {
                    MovePosition::After(t) => { let _ = writeln!(msg, "MOVE {path} AFTER {t}"); },
                    MovePosition::Before(t) => { let _ = writeln!(msg, "MOVE {path} BEFORE {t}"); },
                    MovePosition::EndOfSection(s) => { let _ = writeln!(msg, "MOVE {path} TO {s}"); },
                }
            },
            _ => {}
        }
    }
    
    let _ = writeln!(msg, "===END===");
    let _ = writeln!(msg, "∆∆∆");

    Some(ApplyOutcome::ValidationFailure {
        errors: vec!["Roadmap rewrite converted to commands".to_string()],
        missing: vec![],
        ai_message: msg,
    })
}

fn validate_single_path(path: &str, errors: &mut Vec<String>) {
    if path.eq_ignore_ascii_case("ROADMAP.md") {
        // Handled in main loop
        return;
    }

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
        if line.contains("warden:ignore") {
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