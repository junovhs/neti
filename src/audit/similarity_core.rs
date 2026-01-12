// src/audit/similarity_core.rs
//! Core logic for clustering similar code units.

use super::similarity_math;
use super::types::{CodeUnit, SimilarityCluster};
use super::union_find::UnionFind;
use std::collections::HashMap;
use std::hash::BuildHasher;

pub const MAX_CLUSTER_SIZE: usize = 30;

pub struct NearGroup {
    pub units: Vec<CodeUnit>,
    pub avg_similarity: f64,
}

#[must_use]
pub fn find_near_duplicates<S: BuildHasher>(
    units: &[CodeUnit],
    exact_groups: &HashMap<u64, Vec<CodeUnit>, S>,
    min_size: usize,
) -> Vec<NearGroup> {
    let candidates = filter_candidates(units, exact_groups, min_size);
    if candidates.len() < 2 {
        return Vec::new();
    }

    let mut uf = UnionFind::new(candidates.len());
    cluster_candidates(&candidates, &mut uf);

    extract_groups(&candidates, &mut uf)
}

fn filter_candidates<'a, S: BuildHasher>(
    units: &'a [CodeUnit],
    exact_groups: &HashMap<u64, Vec<CodeUnit>, S>,
    min_size: usize,
) -> Vec<&'a CodeUnit> {
    units
        .iter()
        .filter(|u| {
            u.line_count() >= min_size
                && exact_groups
                    .get(&u.fingerprint.hash)
                    .is_none_or(|g| g.len() < 2)
        })
        .collect()
}

// Indices i,j are guaranteed valid by loop bounds: i < len, j < len
#[allow(clippy::indexing_slicing)]
fn cluster_candidates(candidates: &[&CodeUnit], uf: &mut UnionFind) {
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            if similarity_math::are_units_similar(candidates[i], candidates[j]) {
                uf.union(i, j);
            }
        }
    }
}

fn extract_groups(candidates: &[&CodeUnit], uf: &mut UnionFind) -> Vec<NearGroup> {
    let mut map: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..candidates.len() {
        map.entry(uf.find(i)).or_default().push(i);
    }

    map.into_values()
        .filter(|idxs| idxs.len() >= 2)
        .map(|idxs| build_near_group(candidates, &idxs))
        .collect()
}

// Indices come from extract_groups which only stores valid indices
#[allow(clippy::indexing_slicing)]
fn build_near_group(candidates: &[&CodeUnit], idxs: &[usize]) -> NearGroup {
    let units: Vec<CodeUnit> = idxs.iter().map(|&i| candidates[i].clone()).collect();
    let avg_similarity = compute_avg_similarity(&units);
    NearGroup {
        units,
        avg_similarity,
    }
}

#[allow(clippy::cast_precision_loss)]
fn compute_avg_similarity(units: &[CodeUnit]) -> f64 {
    if units.len() < 2 {
        return 0.0;
    }

    let mut total = 0.0;
    let mut count = 0;

    for (i, u1) in units.iter().enumerate() {
        // sum_pairs handles the inner loop to reduce nesting/complexity
        let (sub_total, sub_count) = sum_pairs(u1, units, i + 1);
        total += sub_total;
        count += sub_count;
    }

    if count > 0 {
        total / count as f64
    } else {
        0.0
    }
}

fn sum_pairs(u1: &CodeUnit, units: &[CodeUnit], start_idx: usize) -> (f64, usize) {
    let mut total = 0.0;
    let mut count = 0;

    for u2 in units.iter().skip(start_idx) {
        let fp_sim = similarity_math::calculate_fingerprint_similarity(
            &u1.fingerprint,
            &u2.fingerprint,
        );
        total += similarity_math::calculate_unit_similarity(u1, u2, fp_sim);
        count += 1;
    }

    (total, count)
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
