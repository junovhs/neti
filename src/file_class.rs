// src/file_class.rs
//! File classification: distinguishes source code from config, assets, and data.
//!
//! Only source code files are subject to structural governance laws (token
//! limits, complexity, naming, etc.). Applying `LAW OF ATOMICITY` to a
//! generated JSON bundle or a minified stylesheet is a correctness bug,
//! not a false positive that can be filtered away.
//!
//! This module is the single source of truth for that distinction.

use std::path::Path;

/// Classification of a file for governance purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// Rust, Python, TypeScript/JavaScript.
    /// Subject to all structural governance laws.
    SourceCode,
    /// TOML, YAML, JSON, INI — structured config.
    /// Token limits and complexity do not apply.
    Config,
    /// HTML, CSS, SVG, bundled/minified artifacts.
    /// Token limits and complexity do not apply.
    Asset,
    /// Markdown, lock files, binary-ish data, anything else.
    Other,
}

impl FileKind {
    /// Returns `true` if structural laws apply (token limits, complexity, naming).
    #[must_use]
    pub fn is_governed(&self) -> bool {
        matches!(self, Self::SourceCode)
    }

    /// Returns `true` if secrets scanning may meaningfully apply.
    #[must_use]
    pub fn secrets_applicable(&self) -> bool {
        !matches!(self, Self::Other)
    }
}

/// Classifies a file path into a `FileKind`.
///
/// Decision order:
/// 1. Minified/bundled name pattern → `Asset`
/// 2. Extension lookup → specific kind
/// 3. Bare filename (Makefile, Dockerfile) → `Other`
#[must_use]
pub fn classify(path: &Path) -> FileKind {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if is_minified_artifact(&name) {
        return FileKind::Asset;
    }

    classify_by_ext(&ext)
}

/// Returns `true` for filenames that indicate a minified or bundled artifact.
fn is_minified_artifact(name: &str) -> bool {
    name.contains(".min.")
        || name.ends_with(".min")
        || name.ends_with(".bundle.js")
        || name.ends_with(".bundle.ts")
        || name.ends_with("-bundle.js")
}

fn classify_by_ext(ext: &str) -> FileKind {
    match ext {
        // Source code — governed by all structural laws
        "rs" | "py" | "ts" | "tsx" | "js" | "jsx" => FileKind::SourceCode,

        // Config — structured data, no complexity rules
        // JSON is tricky: generated schemas and lockfiles can be enormous.
        // Classify as Config so token limits do not apply.
        "toml" | "yaml" | "yml" | "ini" | "cfg" | "env" | "properties" | "json" | "jsonc" => {
            FileKind::Config
        }

        // Assets — presentation and styling, no governance
        "html" | "htm" | "xml" | "svg" | "css" | "scss" | "sass" | "less" => FileKind::Asset,

        // Everything else: docs, data, lock files, generated artifacts
        _ => FileKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn source_files_are_governed() {
        for ext in [
            "main.rs",
            "service.py",
            "component.ts",
            "App.tsx",
            "index.js",
            "util.jsx",
        ] {
            let kind = classify(Path::new(ext));
            assert_eq!(kind, FileKind::SourceCode, "{ext} should be SourceCode");
            assert!(kind.is_governed());
        }
    }

    #[test]
    fn html_is_asset_not_governed() {
        let kind = classify(Path::new("index.html"));
        assert_eq!(kind, FileKind::Asset);
        assert!(!kind.is_governed());
    }

    #[test]
    fn json_is_config_not_governed() {
        let kind = classify(Path::new("package.json"));
        assert_eq!(kind, FileKind::Config);
        assert!(!kind.is_governed());
    }

    #[test]
    fn toml_is_config() {
        assert_eq!(classify(Path::new("neti.toml")), FileKind::Config);
    }

    #[test]
    fn minified_js_is_asset() {
        assert_eq!(classify(Path::new("dist/app.min.js")), FileKind::Asset);
        assert_eq!(classify(Path::new("vendor/lib.bundle.js")), FileKind::Asset);
    }

    #[test]
    fn markdown_is_other() {
        assert_eq!(classify(Path::new("README.md")), FileKind::Other);
    }

    #[test]
    fn svg_is_asset() {
        assert_eq!(classify(Path::new("icon.svg")), FileKind::Asset);
    }

    #[test]
    fn lock_files_are_other() {
        assert_eq!(classify(Path::new("Cargo.lock")), FileKind::Other);
    }
}
