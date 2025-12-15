// src/audit/similarity_core.rs
//! Core logic for structural similarity comparison and clustering.

use super::fingerprint;
use super::types::{CodeUnit, SimilarityCluster};
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Minimum similarity threshold for considering units as duplicates.
pub const SIMILARITY_THRESHOLD: f64 = 0.92;

/// Higher threshold for trivial functions (no control flow).
pub const TRIVIAL_SIMILARITY_THRESHOLD: f64 = 0.97;

/// Minimum fingerprint similarity before considering structural similarity.
pub const MIN_FINGERPRINT_SIMILARITY: f64 = 0.6;

/// Maximum cluster size. Clusters larger than this are likely noise.
pub const MAX_CLUSTER_SIZE: usize = 30;

pub struct NearGroup {
    pub units: Vec<CodeUnit>,
    pub avg_similarity: f64,
}

/// Returns true if the fingerprint has non-trivial control flow.
fn has_cfg_complexity(unit: &CodeUnit) -> bool {
    let fp = &unit.fingerprint;
    fp.branch_count > 0 || fp.loop_count > 0 || fp.exit_count > 0
}

/// Gets the appropriate similarity threshold based on complexity.
fn get_threshold(a: &CodeUnit, b: &CodeUnit) -> f64 {
    if has_cfg_complexity(a) && has_cfg_complexity(b) {
        SIMILARITY_THRESHOLD
    } else {
        TRIVIAL_SIMILARITY_THRESHOLD
    }
}

#[must_use]
pub fn find_near_duplicates<S: BuildHasher>(
    units: &[CodeUnit],
    exact_groups: &HashMap<u64, Vec<CodeUnit>, S>,
    min_size: usize,
) -> Vec<NearGroup> {
    let singleton_units: Vec<&CodeUnit> = units
        .iter()
        .filter(|u| {
            u.line_count() >= min_size
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
            let a = singleton_units[i];
            let b = singleton_units[j];

            if a.kind != b.kind {
                continue;
            }

            let fp_sim = fingerprint::similarity(&a.fingerprint, &b.fingerprint);

            if fp_sim < MIN_FINGERPRINT_SIMILARITY {
                continue;
            }

            let struct_sim = structural_similarity(a, b);
            let combined_sim = f64::midpoint(fp_sim, struct_sim);

            let threshold = get_threshold(a, b);
            if combined_sim >= threshold {
                uf.union(i, j);
            }
        }
    }

    merge_and_extract(&singleton_units, &mut uf)
}

fn merge_and_extract(singleton_units: &[&CodeUnit], uf: &mut UnionFind) -> Vec<NearGroup> {
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

    // Weight fingerprint more heavily - it captures actual structure
    line_sim * 0.1 + tok_sim * 0.2 + fp_sim * 0.7
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

#[must_use]
pub fn create_cluster(units: Vec<CodeUnit>, similarity: f64) -> Option<SimilarityCluster> {
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