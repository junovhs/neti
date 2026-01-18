// src/graph/locality/tests.rs
//! Integration tests for locality analysis.
//!
//! These tests verify the full validation pipeline, not just individual functions.
//! Designed to catch regressions and survive mutation testing.

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // Safe in tests with prior assertions
#[allow(clippy::useless_vec)] // Vec is clearer for test data
#[allow(clippy::uninlined_format_args)] // Clearer in assertion messages
mod integration {
    use super::super::analysis::violations::{categorize_violation, ViolationKind};
    use super::super::coupling::compute_coupling;
    use super::super::layers::infer_layers;
    use super::super::validator::{validate_graph, ValidatorConfig};
    use std::path::{Path, PathBuf};

    /// Helper to run validation and extract failed edges' violation kinds.
    fn run_and_categorize(
        edges: &[(&Path, &Path)],
        config: &ValidatorConfig,
    ) -> Vec<ViolationKind> {
        let iter = || edges.iter().map(|(a, b)| (*a, *b));
        let report = validate_graph(iter(), config);
        let couplings = compute_coupling(iter());
        let layers = infer_layers(iter());

        report
            .failed()
            .iter()
            .map(|e| categorize_violation(e, &couplings, &layers))
            .collect()
    }

    // ========================================================================
    // TEST 1: Encapsulation Breach
    // Importing src/foo/internal.rs instead of src/foo/mod.rs should fail.
    // Must use distance > max_distance to force the violation path.
    // ========================================================================
    #[test]
    fn test_encapsulation_breach_detects_internal_import() {
        let edges = vec![
            // Deep nesting to exceed distance threshold
            // This violates encapsulation: importing internal file directly
            (
                Path::new("src/cli/deep/handlers.rs"),
                Path::new("src/apply/nested/internal.rs"),
            ),
        ];

        let config = ValidatorConfig {
            max_distance: 2, // Tight threshold forces distance failure
            l1_threshold: 1, // Very tight L1
            ..Default::default()
        };

        let violations = run_and_categorize(&edges, &config);

        assert_eq!(violations.len(), 1, "Should detect exactly one violation");
        assert_eq!(
            violations[0],
            ViolationKind::EncapsulationBreach,
            "Importing internal file should be EncapsulationBreach, not {:?}",
            violations[0]
        );
    }

    // ========================================================================
    // TEST 2: Encapsulation OK for mod.rs
    // Importing src/foo/mod.rs is fine - that's the public API.
    // ========================================================================
    #[test]
    fn test_encapsulation_allows_mod_rs_import() {
        let edges = vec![
            // mod.rs is the public API, should pass
            (
                Path::new("src/cli/handlers.rs"),
                Path::new("src/apply/mod.rs"),
            ),
        ];

        let config = ValidatorConfig {
            max_distance: 10,
            ..Default::default()
        };

        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

        // Either passes (exempt or within distance), but NOT a failure
        assert!(
            report.failed().is_empty(),
            "Importing mod.rs should not be a violation, got {} failures",
            report.failed().len()
        );
    }

    // ========================================================================
    // TEST 3: Distance Threshold Boundary
    // At max_distance: PASS. At max_distance+1: FAIL.
    // ========================================================================
    #[test]
    fn test_distance_boundary_condition() {
        // Distance 4 = src(1) + tui(2) + view.rs(3) to src(1) + apply(2) + types.rs(3)
        // LCA = src, so distance = (3-1) + (3-1) = 4
        let edge_at_threshold = vec![(
            Path::new("src/tui/view.rs"),
            Path::new("src/apply/types.rs"),
        )];

        // Distance 6 = deeper nesting
        let edge_over_threshold = vec![(
            Path::new("src/tui/widgets/sidebar.rs"),
            Path::new("src/apply/patch/context.rs"),
        )];

        let config = ValidatorConfig {
            max_distance: 4,
            l1_threshold: 2,
            ..Default::default()
        };

        let report_at = validate_graph(edge_at_threshold.iter().map(|(a, b)| (*a, *b)), &config);
        let report_over =
            validate_graph(edge_over_threshold.iter().map(|(a, b)| (*a, *b)), &config);

        assert!(
            report_at.failed().is_empty(),
            "Edge at max_distance ({}) should pass",
            config.max_distance
        );
        assert!(
            !report_over.failed().is_empty(),
            "Edge over max_distance should fail"
        );
    }

    // ========================================================================
    // TEST 4: Hub Exemption
    // Importing from a StableHub should pass regardless of distance.
    // ========================================================================
    #[test]
    fn test_hub_exemption_ignores_distance() {
        // Create a hub by having many files depend on it
        let edges = vec![
            // 5 incoming edges makes it a hub (afferent >= 3)
            (
                Path::new("src/a/file1.rs"),
                Path::new("src/shared/types.rs"),
            ),
            (
                Path::new("src/b/file2.rs"),
                Path::new("src/shared/types.rs"),
            ),
            (
                Path::new("src/c/file3.rs"),
                Path::new("src/shared/types.rs"),
            ),
            (
                Path::new("src/d/file4.rs"),
                Path::new("src/shared/types.rs"),
            ),
            // This one is far away but types.rs is a hub
            (
                Path::new("src/deep/nested/far/away.rs"),
                Path::new("src/shared/types.rs"),
            ),
        ];

        let config = ValidatorConfig {
            max_distance: 2, // Very tight threshold
            ..Default::default()
        };

        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

        // The far away import should still pass because target is a hub
        // Only edges to non-hubs at distance > 2 would fail
        let failed_to_hub: Vec<_> = report
            .failed()
            .iter()
            .filter(|e| e.to.to_string_lossy().contains("types.rs"))
            .collect();

        assert!(
            failed_to_hub.is_empty(),
            "Imports to hub should be exempt from distance, but {} failed",
            failed_to_hub.len()
        );
    }

    // ========================================================================
    // TEST 5: Upward Dependency Detection (via categorize_violation)
    // With auto-inferred layers, upward deps are structurally impossible.
    // But categorize_violation accepts a layer map, so we test it directly.
    // Must use mod.rs target to avoid EncapsulationBreach short-circuit.
    // ========================================================================
    #[test]
    fn test_upward_dep_categorization_with_manual_layers() {
        use super::super::types::{LocalityEdge, NodeIdentity};
        use std::collections::HashMap;

        // Create a failed edge - target is mod.rs (public API, not internal)
        let edge = LocalityEdge {
            from: PathBuf::from("src/core/types.rs"),
            to: PathBuf::from("src/cli/mod.rs"), // mod.rs passes internal_import check
            distance: 4,
            target_skew: 0.0,
            target_identity: NodeIdentity::Standard,
        };

        // Mock layers: types at L0, cli at L2
        // This simulates "types should be low-level but imports high-level cli"
        let mut layers = HashMap::new();
        layers.insert(PathBuf::from("src/core/types.rs"), 0);
        layers.insert(PathBuf::from("src/cli/mod.rs"), 2);

        let couplings = HashMap::new(); // Empty - no hub detection

        let kind = categorize_violation(&edge, &couplings, &layers);

        assert_eq!(
            kind,
            ViolationKind::UpwardDep,
            "Edge from L0 to L2 should be UpwardDep, got {kind:?}"
        );
    }

    // ========================================================================
    // TEST 6: Three-Node Cycle Detection
    // A -> B -> C -> A should be detected as a cycle.
    // ========================================================================
    #[test]
    fn test_cycle_detection_three_node() {
        let edges = vec![
            (Path::new("src/a.rs"), Path::new("src/b.rs")),
            (Path::new("src/b.rs"), Path::new("src/c.rs")),
            (Path::new("src/c.rs"), Path::new("src/a.rs")), // Closes the cycle
        ];

        let config = ValidatorConfig::default();
        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

        assert!(
            !report.cycles().is_empty(),
            "Should detect the A->B->C->A cycle"
        );
        assert!(
            report.cycles()[0].len() >= 3,
            "Cycle should involve at least 3 nodes, got {}",
            report.cycles()[0].len()
        );
    }

    // ========================================================================
    // TEST 7: Structural Exemption - lib.rs
    // lib.rs can import anything without violation.
    // ========================================================================
    #[test]
    fn test_lib_rs_exempt_from_all_rules() {
        let edges = vec![
            // lib.rs importing something far away
            (
                Path::new("src/lib.rs"),
                Path::new("src/deep/nested/internal/private.rs"),
            ),
        ];

        let config = ValidatorConfig {
            max_distance: 1, // Extremely restrictive
            ..Default::default()
        };

        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

        assert!(
            report.failed().is_empty(),
            "lib.rs should be exempt from all rules, got {} failures",
            report.failed().len()
        );
    }

    // ========================================================================
    // TEST 8: Vertical Routing Exemption
    // Files in the same module subtree should be exempt.
    // ========================================================================
    #[test]
    fn test_vertical_routing_same_module() {
        let edges = vec![
            // Both in src/apply/* - same module subtree
            (
                Path::new("src/apply/writer.rs"),
                Path::new("src/apply/types.rs"),
            ),
            (
                Path::new("src/apply/deep/nested.rs"),
                Path::new("src/apply/types.rs"),
            ),
        ];

        let config = ValidatorConfig {
            max_distance: 1, // Would fail if not exempt
            ..Default::default()
        };

        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

        assert!(
            report.failed().is_empty(),
            "Same-module imports should be exempt (vertical routing), got {} failures",
            report.failed().len()
        );
    }
}
