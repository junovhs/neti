// src/audit/similarity_core.rs
//! Core logic for structural similarity comparison and clustering.

use super::types::{CodeUnit, CodeUnitKind, Fingerprint, SimilarityCluster};
use std::collections::{HashMap, HashSet};
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

/// Computes similarity between two fingerprints.
/// Moved from fingerprint.rs to reduce size.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    if a.hash == b.hash {
        return 1.0;
    }
    if a.cfg_hash == b.cfg_hash {
        return 0.85 + (structural_similarity_fp(a, b) * 0.15);
    }
    if cfg_metrics_match(a, b) {
        return 0.95;
    }
    cfg_similarity(a, b) * 0.6 + structural_similarity_fp(a, b) * 0.4
}

fn cfg_metrics_match(a: &Fingerprint, b: &Fingerprint) -> bool {
    a.branch_count == b.branch_count
        && a.loop_count == b.loop_count
        && a.exit_count == b.exit_count
}

#[allow(clippy::cast_precision_loss)]
fn cfg_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let branch = metric_sim(a.branch_count, b.branch_count);
    let loops = metric_sim(a.loop_count, b.loop_count);
    let exits = metric_sim(a.exit_count, b.exit_count);
    branch * 0.5 + loops * 0.3 + exits * 0.2
}

#[allow(clippy::cast_precision_loss)]
fn structural_similarity_fp(a: &Fingerprint, b: &Fingerprint) -> f64 {
    metric_sim(a.depth, b.depth) * 0.3 + metric_sim(a.node_count, b.node_count) * 0.7
}

#[allow(clippy::cast_precision_loss)]
fn metric_sim(a: usize, b: usize) -> f64 {
    let max = a.max(b) as f64;
    if max == 0.0 {
        1.0
    } else {
        1.0 - (a as f64 - b as f64).abs() / max
    }
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

/// Semantic gate: Enums must share significant variants to be clustered.
#[allow(clippy::cast_precision_loss)]
fn passes_enum_semantic_gate(a: &CodeUnit, b: &CodeUnit) -> bool {
    if a.kind != CodeUnitKind::Enum || b.kind != CodeUnitKind::Enum {
        return true;
    }

    let set_a: HashSet<_> = a.signature.iter().map(|s| s.to_lowercase()).collect();
    let set_b: HashSet<_> = b.signature.iter().map(|s| s.to_lowercase()).collect();

    let intersection_count = set_a.intersection(&set_b).count();
    let min_size = set_a.len().min(set_b.len());

    match min_size {
        0 => false,
        1 | 2 => intersection_count == min_size, // Strict 100% overlap for tiny enums
        3 => intersection_count >= 2,            // 2/3 for small enums
        _ => (intersection_count as f64 / min_size as f64) >= 0.5,
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
            // Fix: remove unnecessary '&' as singleton_units[x] is already &CodeUnit
            if are_units_similar(singleton_units[i], singleton_units[j]) {
                uf.union(i, j);
            }
        }
    }

    merge_and_extract(&singleton_units, &mut uf)
}

fn are_units_similar(a: &CodeUnit, b: &CodeUnit) -> bool {
    if a.kind != b.kind {
        return false;
    }

    if !passes_enum_semantic_gate(a, b) {
        return false;
    }

    let fp_sim = calculate_similarity(&a.fingerprint, &b.fingerprint);
    if fp_sim < MIN_FINGERPRINT_SIMILARITY {
        return false;
    }

    let struct_sim = structural_similarity(a, b);
    let combined_sim = f64::midpoint(fp_sim, struct_sim);
    let threshold = get_threshold(a, b);

    combined_sim >= threshold
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

    // Reuse the fingerprint-based similarity
    let fp_sim = calculate_similarity(&a.fingerprint, &b.fingerprint);

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