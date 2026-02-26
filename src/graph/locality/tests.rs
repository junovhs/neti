// src/graph/locality/tests.rs
//! Integration tests for locality analysis — part 1.
//!
//! Covers: encapsulation breach, encapsulation OK, distance boundary, hub exemption.

#[cfg(test)]
mod part2;

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::useless_vec)]
#[allow(clippy::uninlined_format_args)]
mod integration {
    use super::super::analysis::violations::{categorize_violation, ViolationKind};
    use super::super::coupling::compute_coupling;
    use super::super::layers::infer_layers;
    use super::super::validator::{validate_graph, ValidatorConfig};
    use std::path::Path;

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

    #[test]
    fn test_encapsulation_breach_detects_internal_import() {
        let edges = vec![(
            Path::new("src/cli/deep/handlers.rs"),
            Path::new("src/apply/nested/internal.rs"),
        )];

        let config = ValidatorConfig {
            max_distance: 2,
            l1_threshold: 1,
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

    #[test]
    fn test_encapsulation_allows_mod_rs_import() {
        let edges = vec![(
            Path::new("src/cli/handlers.rs"),
            Path::new("src/apply/mod.rs"),
        )];

        let config = ValidatorConfig {
            max_distance: 10,
            ..Default::default()
        };

        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

        assert!(
            report.failed().is_empty(),
            "Importing mod.rs should not be a violation, got {} failures",
            report.failed().len()
        );
    }

    #[test]
    fn test_distance_boundary_condition() {
        let edge_at_threshold = vec![(
            Path::new("src/tui/view.rs"),
            Path::new("src/apply/types.rs"),
        )];
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
            "Edge at max_distance should pass"
        );
        assert!(
            !report_over.failed().is_empty(),
            "Edge over max_distance should fail"
        );
    }

    #[test]
    fn test_hub_exemption_ignores_distance() {
        let edges = vec![
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
            (
                Path::new("src/deep/nested/far/away.rs"),
                Path::new("src/shared/types.rs"),
            ),
        ];

        let config = ValidatorConfig {
            max_distance: 2,
            ..Default::default()
        };

        let report = validate_graph(edges.iter().map(|(a, b)| (*a, *b)), &config);

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
}
