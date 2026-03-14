use super::tables::{
    contains_any, exported_api_needles, heap_needles, heap_type_needles, length_needles,
    locking_import_needles, locking_needles, lookup_import_needles, lookup_needles, loop_needles,
    mutation_needles, path_contains,
};
use super::{Concept, SemanticContext, SemanticLanguage};

pub(super) fn is_test_context(language: SemanticLanguage, context: &SemanticContext) -> bool {
    let path = context.path_hint.as_deref().unwrap_or_default();
    let source = context.source_text.as_str();
    let annotations = join_lower(&context.fingerprint.annotations);

    match language {
        SemanticLanguage::Rust => {
            path_contains(path, &["/tests/", "_test.rs", "tests/"])
                || source.contains("#[test]")
                || source.contains("#[cfg(test)]")
                || annotations.contains("test")
        }
        SemanticLanguage::Python => {
            path_contains(path, &["/tests/", "/test_", "_test.py"])
                || contains_any(source, &["def test_", "pytest", "unittest", "testcase"])
                || annotations.contains("pytest")
        }
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            path_contains(path, &[".test.", ".spec.", "/__tests__/"])
                || contains_any(source, &["describe(", "it(", "test(", "expect("])
        }
        SemanticLanguage::Go => {
            path_contains(path, &["_test.go", "/tests/"])
                || contains_any(source, &["func test", "testing.t", "t.run("])
        }
        SemanticLanguage::Cpp => {
            path_contains(path, &["/tests/", "_test.", "_spec."])
                || contains_any(source, &["test(", "test_f(", "expect_", "assert_"])
        }
        SemanticLanguage::Swift => {
            path_contains(path, &["tests/", "tests.swift"])
                || contains_any(source, &["xctestcase", "func test", "xctassert"])
        }
    }
}

pub(super) fn has_concept(
    language: SemanticLanguage,
    concept: Concept,
    context: &SemanticContext,
) -> bool {
    let source = context.source_text.as_str();
    let imports = join_lower(&context.fingerprint.imports);
    let params = join_lower(&context.fingerprint.param_types);
    let returns = join_lower(&context.fingerprint.return_types);
    let exports = !context.fingerprint.exports.is_empty();
    let role = context.badges.role.as_deref().unwrap_or_default();

    match concept {
        Concept::TestContext => is_test_context(language, context),
        Concept::HeapAllocation => {
            contains_any(source, heap_needles(language))
                || contains_any(source, heap_type_needles(language))
                || contains_any(&params, heap_type_needles(language))
                || contains_any(&returns, heap_type_needles(language))
        }
        Concept::Lookup => {
            contains_any(source, lookup_needles(language))
                || contains_any(&imports, lookup_import_needles(language))
        }
        Concept::Length => contains_any(source, length_needles(language)),
        Concept::Mutation => contains_any(source, mutation_needles(language)),
        Concept::Locking => {
            contains_any(source, locking_needles(language))
                || contains_any(&imports, locking_import_needles(language))
        }
        Concept::Loop => contains_any(source, loop_needles(language)),
        Concept::ExportedApi => {
            exports || !role.is_empty() || contains_any(source, exported_api_needles(language))
        }
    }
}

fn join_lower(values: &[String]) -> String {
    values.join(" ").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::logic_queries::{
        has_guarding_collection_check, has_length_boundary_risk, has_unguarded_collection_access,
        has_unwrapped_front_access,
    };
    use crate::harvester::SemanticFingerprint;
    use crate::taxonomy::SemanticBadges;

    #[test]
    fn rust_test_context_detects_inline_tests() {
        let context = SemanticContext::from_source("#[test]\nfn parses() { assert!(true); }");

        assert!(is_test_context(SemanticLanguage::Rust, &context));
        assert!(has_concept(
            SemanticLanguage::Rust,
            Concept::TestContext,
            &context
        ));
    }

    #[test]
    fn python_semantics_detect_lookup_and_loop() {
        let context = SemanticContext::from_source(
            "for needle in needles:\n    if needle in haystack:\n        hits.append(needle)\n",
        );

        assert!(has_concept(SemanticLanguage::Python, Concept::Lookup, &context));
        assert!(has_concept(SemanticLanguage::Python, Concept::Loop, &context));
        assert!(has_concept(SemanticLanguage::Python, Concept::Mutation, &context));
    }

    #[test]
    fn typescript_semantics_detect_heap_allocation() {
        let context = SemanticContext::from_source(
            "for (const label of labels) { out.push(label.toString()); }",
        );

        assert!(has_concept(
            SemanticLanguage::TypeScript,
            Concept::HeapAllocation,
            &context
        ));
        assert!(has_concept(SemanticLanguage::TypeScript, Concept::Loop, &context));
    }

    #[test]
    fn exported_api_uses_fingerprint_and_badges() {
        let context = SemanticContext {
            fingerprint: SemanticFingerprint {
                exports: vec![String::from("parse_config")],
                ..SemanticFingerprint::default()
            },
            badges: SemanticBadges {
                role: Some(String::from("Parses")),
                ..SemanticBadges::default()
            },
            source_text: String::new(),
            path_hint: None,
        };

        assert!(has_concept(
            SemanticLanguage::Rust,
            Concept::ExportedApi,
            &context
        ));
    }

    #[test]
    fn length_boundary_risk_matches_bad_operator_direction() {
        let context = SemanticContext::from_source("idx <= values.len()");

        assert!(has_length_boundary_risk(SemanticLanguage::Rust, &context));
        assert!(!has_length_boundary_risk(
            SemanticLanguage::Rust,
            &SemanticContext::from_source("idx >= values.len()")
        ));
    }

    #[test]
    fn unguarded_collection_access_detects_front_access() {
        assert!(has_unguarded_collection_access(
            SemanticLanguage::Rust,
            &SemanticContext::from_source("values[0]")
        ));
        assert!(has_unwrapped_front_access(
            SemanticLanguage::Rust,
            &SemanticContext::from_source("values.first().unwrap()")
        ));
        assert!(has_guarding_collection_check(
            SemanticLanguage::Rust,
            &SemanticContext::from_source("if !values.is_empty() { values[0] }")
        ));
    }
}
