// src/graph/rank/pagerank.rs
//! `PageRank` algorithm implementation for file ranking.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const DAMPING: f64 = 0.85;
const ITERATIONS: usize = 20;

/// Computes `PageRank` scores for files in a graph.
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::implicit_hasher)]
pub fn compute(
    edges: &HashMap<PathBuf, HashMap<PathBuf, usize>>,
    all_files: &HashSet<PathBuf>,
    anchor: Option<&PathBuf>,
) -> HashMap<PathBuf, f64> {
    if all_files.is_empty() {
        return HashMap::new();
    }

    let n = all_files.len() as f64;
    let mut ranks = initialize_ranks(all_files, n);
    let personalization = build_personalization(all_files, anchor, n);

    for _ in 0..ITERATIONS {
        ranks = iterate_once(&ranks, edges, all_files, &personalization, n);
    }

    ranks
}

fn initialize_ranks(files: &HashSet<PathBuf>, n: f64) -> HashMap<PathBuf, f64> {
    files.iter().map(|f| (f.clone(), 1.0 / n)).collect()
}

fn build_personalization(
    files: &HashSet<PathBuf>,
    anchor: Option<&PathBuf>,
    n: f64,
) -> HashMap<PathBuf, f64> {
    match anchor {
        Some(a) if files.contains(a) => [(a.clone(), 1.0)].into_iter().collect(),
        _ => files.iter().map(|f| (f.clone(), 1.0 / n)).collect(),
    }
}

fn iterate_once(
    ranks: &HashMap<PathBuf, f64>,
    edges: &HashMap<PathBuf, HashMap<PathBuf, usize>>,
    all_files: &HashSet<PathBuf>,
    personalization: &HashMap<PathBuf, f64>,
    n: f64,
) -> HashMap<PathBuf, f64> {
    let default_pers = 1.0 / n;
    let mut new_ranks: HashMap<PathBuf, f64> = HashMap::new();

    for file in all_files {
        let incoming = compute_incoming_rank(file, ranks, edges);
        let pers = personalization.get(file).unwrap_or(&default_pers);
        new_ranks.insert(file.clone(), (1.0 - DAMPING) * pers + DAMPING * incoming);
    }

    normalize(&mut new_ranks);
    new_ranks
}

#[allow(clippy::cast_precision_loss)]
fn compute_incoming_rank(
    target: &PathBuf,
    ranks: &HashMap<PathBuf, f64>,
    edges: &HashMap<PathBuf, HashMap<PathBuf, usize>>,
) -> f64 {
    let mut rank = 0.0;
    let default_rank = 0.0;

    for (source, targets) in edges {
        let Some(&weight) = targets.get(target) else {
            continue;
        };

        let total_out: usize = targets.values().sum();
        if total_out == 0 {
            continue;
        }

        let source_rank = ranks.get(source).unwrap_or(&default_rank);
        rank += source_rank * (weight as f64 / total_out as f64);
    }

    rank
}

fn normalize(ranks: &mut HashMap<PathBuf, f64>) {
    let total: f64 = ranks.values().sum();
    if total > 0.0 {
        for rank in ranks.values_mut() {
            *rank /= total;
        }
    }
}