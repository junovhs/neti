// src/apply/validator.rs
use crate::apply::messages::format_ai_rejection;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, ManifestEntry, Operation};
use std::path::{Component, Path};

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
    ".slopchop",
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

    validate_extracted_files(extracted, manifest, &mut errors);

    if errors.is_empty() {
        ApplyOutcome::Success {
            written: vec![],
            deleted: vec![],
            backed_up: false,
            staged: false,
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

fn validate_extracted_files(
    extracted: &ExtractedFiles,
    manifest: &Manifest,
    errors: &mut Vec<String>,
) {
    for (path, content) in extracted {
        if !manifest.iter().any(|e| e.path == *path) {
            errors.push(format!("File extracted but not in manifest: {path}"));
        }
        if let Err(e) = validate_content(path, &content.content) {
            errors.push(e);
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
    check_absolute_path(path_str)?;
    let path = Path::new(path_str);
    check_path_components(path, path_str)
}

fn check_absolute_path(path_str: &str) -> Result<(), String> {
    let is_drive = path_str.len() >= 2
        && path_str.chars().nth(1) == Some(':')
        && path_str
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic());

    if is_drive || path_str.starts_with('\\') || path_str.starts_with('/') {
        return Err(format!("Absolute paths not allowed: {path_str}"));
    }

    let path = Path::new(path_str);
    if path.is_absolute() {
        return Err(format!("Absolute paths not allowed: {path_str}"));
    }
    Ok(())
}

fn check_path_components(path: &Path, original_str: &str) -> Result<(), String> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!("Path traversal not allowed: {original_str}"));
    }

    for component in path.components() {
        validate_component(component)?;
    }
    Ok(())
}

fn validate_component(component: Component) -> Result<(), String> {
    if let Component::Normal(os_str) = component {
        let s = os_str.to_string_lossy();
        if BLOCKED_DIRS.contains(&s.as_ref()) {
            return Err(format!("Access to sensitive directory blocked: {s}"));
        }
        if s.starts_with('.') && !is_allowed_dotfile(&s) {
            return Err(format!("Hidden files blocked: {s}"));
        }
    }
    Ok(())
}

fn is_allowed_dotfile(s: &str) -> bool {
    s.eq(".gitignore") || s.eq(".slopchopignore") || s.eq(".github")
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

    check_markdown_fences(path, content)?;
    check_truncation(path, content)?;
    Ok(())
}

fn check_markdown_fences(path: &str, content: &str) -> Result<(), String> {
    let is_markdown = Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"));

    if is_markdown {
        return Ok(());
    }

    if content.contains("\x60\x60\x60") || content.contains("\x7E\x7E\x7E") {
        return Err(format!(
            "Markdown fences detected in {path}. Content must be raw code."
        ));
    }
    Ok(())
}

fn check_truncation(path: &str, content: &str) -> Result<(), String> {
    if let Some(line) = find_truncation_line(content) {
        return Err(format!(
            "Truncation detected in {path} at line {line}: AI gave up."
        ));
    }
    Ok(())
}

fn find_truncation_line(content: &str) -> Option<usize> {
    let patterns = [
        "// ...",
        "/* ... */",
        "# ...",
        "// rest of",
        "// remaining", // slopchop:ignore
        "// TODO: implement",
        "// implementation",
        "pass  #", // slopchop:ignore
    ];

    for (i, line) in content.lines().enumerate() {
        if line.contains("slopchop:ignore") {
            continue;
        }
        let lower = line.to_lowercase();
        if patterns.iter().any(|p| lower.contains(&p.to_lowercase())) {
            return Some(i + 1);
        }
    }
    None
}
