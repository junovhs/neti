// src/graph/locality/tests/part2.rs
//! Integration tests for locality analysis — part 2.
//!
//! Covers: upward dependency categorization, cycle detection,
//! lib.rs exemption, vertical routing exemption.

#[allow(clippy::indexing_slicing)]
#[allow(clippy::useless_vec)]
#[allow(clippy::uninlined_format_args)]
mod integration2 {
    use super::super::super::analysis::violations::{categorize_violation, ViolationKind};
    use super::super::super::types::{LocalityEdge, NodeIdentity};
    use super::super::super::validator::{validate_graph, ValidatorConfig};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_upward_dep_categorization_with_manual_layers() {
        let edge = LocalityEdge {
            from: PathBuf::from("src/core/types.rs"),
            to: PathBuf::from("src/cli/mod.rs"),
            distance: 4,
            target_skew: 0.0,
            target_identity: NodeIdentity::Standard,
        };

        let mut layers = HashMap::new();
        layers.insert(PathBuf::from("src/core/types.rs"), 0);
        layers.insert(PathBuf::from("src/cli/mod.rs"), 2);

        let couplings = HashMap::new();

        let kind = categorize_violation(&edge, &couplings, &layers);

        assert_eq!(
            kind,
            ViolationKind::UpwardDep,
            "Edge from L0 to L2 should be UpwardDep, got {kind:?}"
        );
    }

    #[test]
    fn test_cycle_detection_three_node() {
        let edges = vec![
            (Path::new("src/a.rs"), Path::new("src/b.rs")),
            (Path::new("src/b.rs"), Path::new("src/c.rs")),
            (Path::new("src/c.rs"), Path::new("src/a.rs")),
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

    #[test]
    fn test_lib_rs_exempt_from_all_rules() {
        let edges = vec![(
            Path::new("src/lib.rs"),
            Path::new("src/deep/nested/internal/private.rs"),
        )];

        let config = ValidatorConfig {
            max_distance: 1,
            ..Default::default()
        };

        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

        assert!(
            report.failed().is_empty(),
            "lib.rs should be exempt from all rules, got {} failures",
            report.failed().len()
        );
    }

    #[test]
    fn test_vertical_routing_same_module() {
        let edges = vec![
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
            max_distance: 1,
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
