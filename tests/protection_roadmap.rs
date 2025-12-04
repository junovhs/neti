use slopchop_core::apply::types::{ApplyOutcome, FileContent, ManifestEntry, Operation};
use slopchop_core::apply::validator;
use std::collections::HashMap;

#[test]
fn test_roadmap_rewrite_is_blocked() {
    let manifest = vec![ManifestEntry {
        path: "ROADMAP.md".to_string(),
        operation: Operation::Update,
    }];

    let mut extracted = HashMap::new();
    extracted.insert(
        "ROADMAP.md".to_string(),
        FileContent {
            content: "# New Roadmap".to_string(),
            line_count: 1,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);

    match outcome {
        ApplyOutcome::ValidationFailure {
            errors, ai_message, ..
        } => {
            // It might be a "PROTECTED" error (if diff fails/file missing)
            // OR a "Roadmap rewrite converted" error (if diff succeeds).
            // Since this test runs in isolation without a real file on disk,
            // handle_roadmap_rewrite likely returns None (file not found),
            // falling back to the standard PROTECTED error.

            let has_protected = errors.iter().any(|e| e.contains("PROTECTED"));
            let has_converted = errors
                .iter()
                .any(|e| e.contains("Roadmap rewrite converted"));

            assert!(
                has_protected || has_converted,
                "Expected roadmap block, got errors: {errors:?}\nMessage: {ai_message}"
            );
        }
        _ => panic!("Should have failed validation"),
    }
}

#[test]
fn test_roadmap_rewrite_blocked_case_insensitive() {
    let manifest = vec![ManifestEntry {
        path: "roadmap.md".to_string(),
        operation: Operation::Update,
    }];

    let mut extracted = HashMap::new();
    extracted.insert(
        "roadmap.md".to_string(),
        FileContent {
            content: "# New Roadmap".to_string(),
            line_count: 1,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);

    if let ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors
            .iter()
            .any(|e| e.contains("PROTECTED") || e.contains("Roadmap rewrite converted")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_roadmap_error_suggests_command() {
    let manifest = vec![ManifestEntry {
        path: "ROADMAP.md".to_string(),
        operation: Operation::Update,
    }];

    let mut extracted = HashMap::new();
    extracted.insert(
        "ROADMAP.md".to_string(),
        FileContent {
            content: "# New Roadmap".to_string(),
            line_count: 1,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);

    if let ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        // The error message itself suggests using commands
        assert!(errors.iter().any(
            |e| e.contains("slopchop roadmap apply") || e.contains("Roadmap rewrite converted")
        ));
    } else {
        panic!("Should have failed validation");
    }
}

