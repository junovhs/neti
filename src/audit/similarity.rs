// src/audit/similarity.rs
//! Similarity detection using locality-sensitive hashing and clustering.
//!
//! This module finds groups of structurally similar code units that may
//! represent duplication or opportunities for consolidation.

use super::fingerprint;
use super::types::{CodeUnit, SimilarityCluster};
use std::collections::HashMap;

/// Minimum similarity threshold for considering units as duplicates.
const SIMILARITY_THRESHOLD: f64 = 0.85;

/// Minimum size (in lines) for a unit to be considered for duplication.
const MIN_UNIT_SIZE: usize = 5;

/// Finds clusters of similar code units.
#[must_use]
pub fn find_clusters(units: &[CodeUnit]) -> Vec<SimilarityCluster> {
    let exact_groups = group_by_fingerprint(units);
    let near_groups = find_near_duplicates(units, &exact_groups);

    let mut clusters = Vec::new();

    for group in exact_groups.into_values() {
        if group.len() >= 2 {
            if let Some(cluster) = create_cluster(group, 1.0) {
                clusters.push(cluster);
            }
        }
    }

    for group in near_groups {
        if let Some(cluster) = create_cluster(group.units, group.avg_similarity) {
            clusters.push(cluster);
        }
    }

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

struct NearGroup {
    units: Vec<CodeUnit>,
    avg_similarity: f64,
}

fn find_near_duplicates(
    units: &[CodeUnit],
    exact_groups: &HashMap<u64, Vec<CodeUnit>>,
) -> Vec<NearGroup> {
    let singleton_units: Vec<&CodeUnit> = units
        .iter()
        .filter(|u| {
            u.line_count() >= MIN_UNIT_SIZE
                && exact_groups
                    .get(&u.fingerprint.hash)
                    .is_none_or(|g| g.len() < 2)
        })
        .collect();

    if singleton_units.len() < 2 {
        return Vec::new();
    }

    let mut uf = UnionFind::new(singleton_units.len());

    for i in 0..singleton_units.len() {
        for j in (i + 1)..singleton_units.len() {
            let sim = fingerprint::similarity(
                &singleton_units[i].fingerprint,
                &singleton_units[j].fingerprint,
            );

            let struct_sim = structural_similarity(singleton_units[i], singleton_units[j]);
            let combined_sim = f64::midpoint(sim, struct_sim);

            if combined_sim >= SIMILARITY_THRESHOLD {
                uf.union(i, j);
            }
        }
    }

    let mut cluster_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..singleton_units.len() {
        let root = uf.find(i);
        cluster_map.entry(root).or_default().push(i);
    }

    cluster_map
        .into_values()
        .filter(|indices| indices.len() >= 2)
        .map(|indices| {
            let units: Vec<CodeUnit> = indices
                .iter()
                .map(|&i| singleton_units[i].clone())
                .collect();
            let avg_similarity = compute_avg_similarity(&units);
            NearGroup {
                units,
                avg_similarity,
            }
        })
        .collect()
}

#[allow(clippy::cast_precision_loss)]
fn structural_similarity(a: &CodeUnit, b: &CodeUnit) -> f64 {
    if a.kind != b.kind {
        return 0.0;
    }

    let line_a = a.line_count() as f64;
    let line_b = b.line_count() as f64;
    let line_sim = 1.0 - (line_a - line_b).abs() / line_a.max(line_b).max(1.0);

    let tok_a = a.tokens as f64;
    let tok_b = b.tokens as f64;
    let tok_sim = 1.0 - (tok_a - tok_b).abs() / tok_a.max(tok_b).max(1.0);

    let fp_sim = fingerprint::similarity(&a.fingerprint, &b.fingerprint);

    line_sim * 0.2 + tok_sim * 0.3 + fp_sim * 0.5
}

fn compute_avg_similarity(units: &[CodeUnit]) -> f64 {
    if units.len() < 2 {
        return 0.0;
    }

    let mut total = 0.0;
    let mut count = 0;

    for i in 0..units.len() {
        for j in (i + 1)..units.len() {
            total += structural_similarity(&units[i], &units[j]);
            count += 1;
        }
    }

    if count > 0 {
        total / f64::from(count)
    } else {
        0.0
    }
}

fn create_cluster(units: Vec<CodeUnit>, similarity: f64) -> Option<SimilarityCluster> {
    if units.len() < 2 {
        return None;
    }

    let total_lines: usize = units.iter().map(CodeUnit::line_count).sum();
    let avg_size = total_lines / units.len();
    let potential_savings = avg_size * (units.len() - 1);

    Some(SimilarityCluster {
        units,
        similarity,
        potential_savings,
    })
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);

        if rx == ry {
            return;
        }

        match self.rank[rx].cmp(&self.rank[ry]) {
            std::cmp::Ordering::Less => self.parent[rx] = ry,
            std::cmp::Ordering::Greater => self.parent[ry] = rx,
            std::cmp::Ordering::Equal => {
                self.parent[ry] = rx;
                self.rank[rx] += 1;
            }
        }
    }
}

/// Analyzes a cluster to produce a human-readable description.
#[must_use]
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
