// src/audit/similarity.rs
//! Similarity detection using locality-sensitive hashing and clustering.
//!
//! This module finds groups of structurally similar code units that may
//! represent duplication or opportunities for consolidation.

use super::similarity_core;
use super::types::{CodeUnit, SimilarityCluster};
use std::collections::HashMap;

/// Minimum size (in lines) for a unit to be considered for duplication.
const MIN_UNIT_SIZE: usize = 5;

/// Finds clusters of similar code units.
#[must_use]
pub fn find_clusters(units: &[CodeUnit]) -> Vec<SimilarityCluster> {
    let exact_groups = group_by_fingerprint(units);
    let near_groups = similarity_core::find_near_duplicates(units, &exact_groups, MIN_UNIT_SIZE);

    let mut clusters = Vec::new();

    collect_exact_clusters(&exact_groups, &mut clusters);
    collect_near_clusters(&near_groups, &mut clusters);

    clusters.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));

    clusters
}

fn group_by_fingerprint(units: &[CodeUnit]) -> HashMap<u64, Vec<CodeUnit>> {
    let mut groups: HashMap<u64, Vec<CodeUnit>> = HashMap::new();

    for unit in units {
        if unit.line_count() >= MIN_UNIT_SIZE {
            groups
                .entry(unit.fingerprint.hash)
                .or_default()
                .push(unit.clone());
        }
    }

    groups
}

fn collect_exact_clusters(
    exact_groups: &HashMap<u64, Vec<CodeUnit>>,
    clusters: &mut Vec<SimilarityCluster>,
) {
    for group in exact_groups.values() {
        if group.len() >= 2 && group.len() <= similarity_core::MAX_CLUSTER_SIZE {
            if let Some(cluster) = similarity_core::create_cluster(group.clone(), 1.0) {
                clusters.push(cluster);
            }
        }
    }
}

fn collect_near_clusters(
    near_groups: &[similarity_core::NearGroup],
    clusters: &mut Vec<SimilarityCluster>,
) {
    for group in near_groups {
        if group.units.len() <= similarity_core::MAX_CLUSTER_SIZE {
            if let Some(cluster) =
                similarity_core::create_cluster(group.units.clone(), group.avg_similarity)
            {
                clusters.push(cluster);
            }
        }
    }
}

/// Analyzes a cluster to produce a human-readable description.
#[must_use]
#[allow(clippy::indexing_slicing)] // Guarded: unique_files.len() == 1 check before files[0]
pub fn describe_cluster(cluster: &SimilarityCluster) -> String {
    let count = cluster.units.len();
    let kind = cluster.units.first().map_or("unit", |u| u.kind.label());
    let files: Vec<_> = cluster
        .units
        .iter()
        .map(|u| u.file.display().to_string())
        .collect();

    let unique_files: std::collections::HashSet<_> = files.iter().collect();

    if unique_files.len() == 1 {
        format!(
            "{count} similar {kind}s in {} (lines could be merged)",
            files[0]
        )
    } else {
        format!(
            "{count} similar {kind}s across {} files",
            unique_files.len()
        )
    }
}
