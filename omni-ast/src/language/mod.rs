pub mod cpp;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust;

use crate::doc_extractor;
use crate::types::DepKind;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::Path;

pub fn extract_import_strings(
    file: &Path,
    content: &str,
    rust_crate_name: Option<&str>,
) -> Vec<String> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "rs" => rust::extract_modules(content, rust_crate_name),
        "py" => python::extract_import_strings(content),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => javascript::extract_import_strings(content),
        "go" => go::extract_import_strings(content),
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            cpp::extract_import_strings(content)
        }
        _ => Vec::new(),
    }
}

pub fn resolve_imports(
    file: &Path,
    content: &str,
    known_paths: &HashSet<&str>,
    rust_crate_name: Option<&str>,
    go_module_name: Option<&str>,
) -> Vec<(String, DepKind)> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    let source_path = file.to_string_lossy();
    match ext {
        "py" => python::extract_imports(content, &source_path, known_paths),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => {
            javascript::extract_imports(content, &source_path, known_paths)
        }
        "go" => go::extract_imports(content, &go_module_name.map(str::to_owned), known_paths),
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            cpp::extract_imports(content, &source_path, known_paths)
        }
        "rs" => rust::extract_modules(content, rust_crate_name)
            .into_iter()
            .filter_map(|module| {
                let candidate = format!("{module}.rs");
                known_paths
                    .contains(candidate.as_str())
                    .then_some((candidate, DepKind::Import))
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub fn extract_doc_comment_for_file(file: &Path, content: &str) -> Option<String> {
    doc_extractor::extract_doc_comment_for_file(file, content)
}

pub fn resolve_semantic_exports(file: &Path, content: &str) -> BTreeSet<String> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            cpp::extract_exports(content, file)
        }
        _ => BTreeSet::new(),
    }
}

pub fn resolve_primary_symbol(file: &Path, content: &str) -> Option<String> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            cpp::primary_symbol(content, file)
        }
        _ => None,
    }
}

pub fn has_rust_inline_tests(content: &str) -> bool {
    rust::has_inline_tests(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DepKind;
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn resolves_python_relative_imports() {
        let known: HashSet<&str> = ["pkg/foo.py", "pkg/bar.py", "pkg/__init__.py"]
            .into_iter()
            .collect();
        let file = Path::new("pkg/main.py");
        let content = "from . import foo\nfrom .bar import baz\n";

        let deps = resolve_imports(file, content, &known, None, None);

        assert!(deps.contains(&(String::from("pkg/foo.py"), DepKind::Import)));
        assert!(deps.contains(&(String::from("pkg/bar.py"), DepKind::Import)));
    }

    #[test]
    fn extracts_import_strings_from_multiple_languages() {
        let rust = extract_import_strings(
            Path::new("lib.rs"),
            "use crate::config::Config;\nmod tests;\n",
            None,
        );
        let python = extract_import_strings(
            Path::new("main.py"),
            "import os\nfrom pkg.sub import thing\n",
            None,
        );
        let js = extract_import_strings(
            Path::new("app.ts"),
            "import x from \"react\";\nconst y = require(\"zod\");\n",
            None,
        );

        assert!(rust.contains(&String::from("config")));
        assert!(rust.contains(&String::from("tests")));
        assert!(python.contains(&String::from("os")));
        assert!(python.contains(&String::from("pkg.sub")));
        assert!(js.contains(&String::from("react")));
        assert!(js.contains(&String::from("zod")));
    }

    #[test]
    fn resolves_js_monorepo_bare_imports() {
        let known: HashSet<&str> = [
            "packages/zod/src/v4/core.ts",
            "packages/app/src/index.ts",
            "packages/app/src/local.ts",
        ]
        .into_iter()
        .collect();
        let file = Path::new("packages/app/src/index.ts");
        let content = "import { z } from \"zod/v4/core\";\nimport local from \"./local\";\n";

        let deps = resolve_imports(file, content, &known, None, None);

        assert!(deps.contains(&(String::from("packages/zod/src/v4/core.ts"), DepKind::Import)));
        assert!(deps.contains(&(String::from("packages/app/src/local.ts"), DepKind::Import)));
    }

    #[test]
    fn ignores_unresolvable_js_imports() {
        let known: HashSet<&str> = ["packages/app/src/index.ts"].into_iter().collect();
        let file = Path::new("packages/app/src/index.ts");
        let content = "import x from \"missing-pkg/foo\";\n";

        let deps = resolve_imports(file, content, &known, None, None);

        assert!(deps.is_empty());
    }

    #[test]
    fn extracts_cpp_primary_symbol_and_doc() {
        let file = Path::new("include/widget.hpp");
        let content = "#pragma once\n\n/// Widget manager for UI composition.\nclass WidgetManager {\npublic:\n    void Render();\n};\n";

        let doc = extract_doc_comment_for_file(file, content);
        let symbol = resolve_primary_symbol(file, content);
        let exports = resolve_semantic_exports(file, content);

        assert_eq!(doc.as_deref(), Some("Widget manager for UI composition."));
        assert_eq!(symbol.as_deref(), Some("WidgetManager"));
        assert!(exports.contains("WidgetManager"));
    }
}
