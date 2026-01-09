// src/apply/validator.rs
use crate::apply::messages::format_ai_rejection;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, ManifestEntry, Operation};
use crate::lang::Lang;
use tree_sitter::Parser;
use std::path::{Component, Path};

const PROTECTED_FILES: &[&str] = &[
    ".slopchopignore",
    "slopchop.toml",
    "build.rs",
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
];

const BLOCKED_DIRS: &[&str] = &[
    ".git",
    ".cargo",
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
                let verb = if entry.operation == Operation::New {
                    "create"
                } else {
                    "update"
                };
                errors.push(format!(
                    "Manifest says {verb} '{}', but no file block found.",
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
    if path_str.as_bytes().contains(&0) {
        return Err(format!("Path contains illegal null character: {path_str}"));
    }
    check_absolute_path(path_str)?;
    let path = Path::new(path_str);
    check_path_components(path, path_str)
}

fn check_absolute_path(path_str: &str) -> Result<(), String> {
    if is_absolute_os(path_str) {
        return Err(format!("Absolute paths not allowed: {path_str}"));
    }
    Ok(())
}

fn is_absolute_os(path_str: &str) -> bool {
    let is_drive = path_str.len() >= 2
        && path_str.chars().nth(1) == Some(':')
        && path_str
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic());
    is_drive
        || path_str.starts_with('\\')
        || path_str.starts_with('/')
        || Path::new(path_str).is_absolute()
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
    matches!(s, ".gitignore" | ".slopchopignore" | ".github")
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
    check_ast_syntax(path, content)?;
    Ok(())
}

fn check_ast_syntax(path: &str, content: &str) -> Result<(), String> {
    let Some(ext) = Path::new(path).extension().and_then(|s| s.to_str()) else {
        return Ok(());
    };

    let Some(lang) = Lang::from_ext(ext) else {
        return Ok(());
    };

    let mut parser = Parser::new();
    if parser.set_language(lang.grammar()).is_err() {
        return Ok(()); // Grammar not available, skip
    }

    let Some(tree) = parser.parse(content, None) else {
        return Ok(());
    };

    if has_syntax_errors(tree.root_node()) {
        return Err(format!("Syntax error detected in {path}. Please fix it before applying."));
    }

    Ok(())
}

fn has_syntax_errors(node: tree_sitter::Node) -> bool {
    if node.is_error() || node.is_missing() {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if has_syntax_errors(child) {
            return true;
        }
    }
    false
}

fn check_markdown_fences(path: &str, content: &str) -> Result<(), String> {
    if is_markdown_file(path) {
        return Ok(());
    }
    if content.contains("\x60\x60\x60") || content.contains("\x7E\x7E\x7E") {
        return Err(format!("Markdown fences detected in {path}. Content must be raw code. Run with --sanitize to strip them automatically."));
    }
    Ok(())
}

fn is_markdown_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
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
        "// ...",             // slopchop:ignore
        "/* ... */",          // slopchop:ignore
        "# ...",              // slopchop:ignore
        "// rest of",         // slopchop:ignore
        "// remaining",       // slopchop:ignore
        "// TODO: implement", // slopchop:ignore
        "// implementation",  // slopchop:ignore
        "pass  #",            // slopchop:ignore
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
