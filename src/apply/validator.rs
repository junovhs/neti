// src/apply/validator.rs
use crate::apply::messages::format_ai_rejection;
use crate::apply::types::{ExtractedFiles, Manifest, ManifestEntry, Operation};
use crate::apply::ApplyOutcome;
use std::path::{Component, Path};

// V1 ROADMAP.md is dead. tasks.toml is handled by roadmap_v2 module.
const PROTECTED_FILES: &[&str] = &[
    ".slopchopignore",
    "slopchop.toml",
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
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
        check_manifest_consistency(entry, extracted, &mut errors);
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

fn check_manifest_consistency(
    entry: &ManifestEntry,
    extracted: &ExtractedFiles,
    errors: &mut Vec<String>,
) {
    match entry.operation {
        Operation::New | Operation::Update => {
            if !extracted.contains_key(&entry.path) {
                errors.push(format!(
                    "Manifest says {} '{}', but no file block found.",
                    if entry.operation == Operation::New {
                        "create"
                    } else {
                        "update"
                    },
                    entry.path
                ));
            }
        }
        Operation::Delete => {
            if extracted.contains_key(&entry.path) {
                errors.push(format!(
                    "Manifest says delete '{}', but file block provided.",
                    entry.path
                ));
            }
        }
    }
}

fn validate_path(path_str: &str) -> Result<(), String> {
    // 1. Manual check for Windows Absolute paths (Drive letter or UNC)
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
    PROTECTED_FILES
        .iter()
        .any(|&f| f.eq_ignore_ascii_case(path_str))
}

fn validate_content(path: &str, content: &str) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err(format!("File is empty: {path}"));
    }

    // Allow markdown fences in markdown files
    let is_markdown = Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"));

    if !is_markdown {
        // Use escape sequences to prevent self-rejection
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
    let truncation_patterns = [
        "// ...",             // slopchop:ignore
        "/* ... */",          // slopchop:ignore
        "# ...",              // slopchop:ignore
        "// rest of",         // slopchop:ignore
        "// remaining",       // slopchop:ignore
        "// TODO: implement", // slopchop:ignore
        "// implementation",  // slopchop:ignore
        "pass  #",            // slopchop:ignore (Python placeholder)
    ];

    for (line_num, line) in content.lines().enumerate() {
        // Skip lines with ignore marker
        if line.contains("slopchop:ignore") {
            continue;
        }

        let lower = line.to_lowercase();
        for pattern in &truncation_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                return Some(line_num + 1);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_blocked() {
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("foo/../bar").is_err());
    }

    #[test]
    fn test_absolute_path_blocked() {
        assert!(validate_path("/etc/passwd").is_err());
        assert!(validate_path("C:\\Windows\\System32").is_err());
    }

    #[test]
    fn test_sensitive_dirs_blocked() {
        assert!(validate_path(".git/config").is_err());
        assert!(validate_path(".ssh/id_rsa").is_err());
        assert!(validate_path(".env").is_err());
    }

    #[test]
    fn test_valid_paths_allowed() {
        assert!(validate_path("src/main.rs").is_ok());
        assert!(validate_path("tests/unit_test.rs").is_ok());
        assert!(validate_path(".gitignore").is_ok());
        assert!(validate_path(".github/workflows/ci.yml").is_ok());
    }

    #[test]
    fn test_protected_files() {
        assert!(is_protected("slopchop.toml"));
        assert!(is_protected("Cargo.lock"));
        assert!(!is_protected("src/main.rs"));
        assert!(!is_protected("ROADMAP.md"));
    }
}