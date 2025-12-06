// src/apply/validator.rs
use crate::apply::messages::format_ai_rejection;
use crate::apply::types::{ExtractedFiles, Manifest};
use crate::apply::ApplyOutcome;
use std::path::{Component, Path};

const PROTECTED_FILES: &[&str] = &[
    "ROADMAP.md", // We protect the roadmap structure but allow specific updates via commands
    ".slopchopignore",
    "slopchop.toml",
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
];

const BLOCKED_DIRS: &[&str] = &[
    ".git",
    ".env",
    ".ssh",
    ".aws",
    ".gnupg",
    "id_rsa",
    "credentials",
    ".slopchop_apply_backup",
];

#[must_use]
pub fn validate(manifest: &Manifest, extracted: &ExtractedFiles) -> ApplyOutcome {
    let mut errors = Vec::new();

    for entry in manifest {
        if let Err(e) = validate_path(&entry.path) {
            errors.push(e);
        }
        if is_protected(&entry.path) {
            errors.push(format!("Cannot overwrite protected file: {}", entry.path));
        }
    }

    for (path, content) in extracted {
        if !manifest.iter().any(|e| e.path == *path) {
            errors.push(format!("File extracted but not in manifest: {path}"));
        }
        if let Err(e) = validate_content(path, &content.content) {
            errors.push(e);
        }
    }

    if errors.is_empty() {
        ApplyOutcome::Success {
            written: vec![],
            deleted: vec![],
            roadmap_results: vec![],
            backed_up: false,
        }
    } else {
        let ai_message = format_ai_rejection(&[], &errors);
        ApplyOutcome::ValidationFailure {
            errors,
            missing: vec![],
            ai_message,
        }
    }
}

fn validate_path(path_str: &str) -> Result<(), String> {
    // 1. Manual check for Windows Absolute paths (Drive letter or UNC)
    // We do this manually because Path::is_absolute depends on the running OS.
    let is_drive = path_str.len() >= 2
        && path_str.chars().nth(1) == Some(':')
        && path_str
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic());

    if is_drive || path_str.starts_with('\\') || path_str.starts_with('/') {
        return Err(format!("Absolute paths not allowed: {path_str}"));
    }

    // 2. Standard Path checks
    let path = Path::new(path_str);
    if path.is_absolute() {
        return Err(format!("Absolute paths not allowed: {path_str}"));
    }

    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!("Path traversal not allowed: {path_str}"));
    }

    for component in path.components() {
        if let Component::Normal(os_str) = component {
            let s = os_str.to_string_lossy();
            if BLOCKED_DIRS.contains(&s.as_ref()) {
                return Err(format!("Access to sensitive directory blocked: {s}"));
            }
            if s.starts_with('.')
                && !s.eq(".gitignore")
                && !s.eq(".slopchopignore")
                && !s.eq(".github")
            {
                return Err(format!("Hidden files blocked: {s}"));
            }
        }
    }
    Ok(())
}

fn is_protected(path_str: &str) -> bool {
    // Exceptions for specific workflows (like updating README if needed) can be handled here,
    // but generally we protect config files.
    // NOTE: 'ROADMAP.md' is protected because it should be updated via commands, not raw overwrites.
    PROTECTED_FILES.iter().any(|&f| f.eq_ignore_ascii_case(path_str))
}

fn validate_content(path: &str, content: &str) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err(format!("File is empty: {path}"));
    }

    // Allow markdown fences in markdown files
    let is_markdown = Path::new(path).extension().is_some_and(|ext| {
        ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
    });

    if !is_markdown {
        // Use escape sequences to prevent self-rejection
        // \x60 = backtick, \x7E = tilde
        if content.contains("\x60\x60\x60") || content.contains("\x7E\x7E\x7E") {
            return Err(format!(
                "Markdown fences detected in {path}. Content must be raw code."
            ));
        }
    }

    if let Some(line) = detect_truncation(content) {
        return Err(format!(
            "Truncation detected in {path} at line {line}: AI gave up."
        ));
    }
    Ok(())
}

fn detect_truncation(content: &str) -> Option<usize> {
    // We add slopchop:ignore to these lines so the CURRENT version of the tool
    // doesn't flag this NEW version of the source code as truncated.
    let truncation_patterns = [
        "// ...",         // slopchop:ignore
        "/* ... */",      // slopchop:ignore
        "# ...",          // slopchop:ignore
        "// rest of",     // slopchop:ignore
        "// remaining",   // slopchop:ignore
        "# rest of",      // slopchop:ignore
        "# remaining",    // slopchop:ignore
        "<!-- ... -->",   // slopchop:ignore
    ];
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        for pattern in &truncation_patterns {
            if trimmed.contains(pattern) && !trimmed.contains("slopchop:ignore") {
                return Some(i + 1);
            }
        }
    }
    None
}