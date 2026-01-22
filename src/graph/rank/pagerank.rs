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

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf { PathBuf::from(s) }
    fn f(paths: &[&str]) -> HashSet<PathBuf> { paths.iter().map(|s| p(s)).collect() }
    
    fn assert_approx_eq(a: f64, b: f64, desc: &str) {
        let diff = (a - b).abs();
        assert!(diff < 0.001, "{desc}: {a} != {b} (diff {diff})");
    }

    type GraphSetup = Box<dyn Fn() -> (HashMap<PathBuf, HashMap<PathBuf, usize>>, HashSet<PathBuf>, Option<PathBuf>)>;
    type ResultCheck = Box<dyn Fn(HashMap<PathBuf, f64>)>;

    #[test]
    fn test_compute_logic() {
        // Test cases defined as closures to construct complex data types
        let mut cases: Vec<(GraphSetup, ResultCheck, &str)> = Vec::new();

        // 1. Empty Graph
        cases.push((
            Box::new(|| (HashMap::new(), HashSet::new(), None)),
            Box::new(|res| assert!(res.is_empty())),
            "Empty graph"
        ));

        // 2. Single Node
        cases.push((
            Box::new(|| (HashMap::new(), f(&["a.rs"]), None)),
            Box::new(|res| {
                assert_eq!(res.len(), 1);
                assert_approx_eq(*res.get(&p("a.rs")).unwrap(), 1.0, "Single node rank");
            }),
            "Single node"
        ));

        // 3. Directed Edge (a -> b)
        cases.push((
            Box::new(|| {
                let mut edges = HashMap::new();
                edges.insert(p("a.rs"), [(p("b.rs"), 1)].into_iter().collect());
                (edges, f(&["a.rs", "b.rs"]), None)
            }),
            Box::new(|res| {
                let r_a = *res.get(&p("a.rs")).unwrap();
                let r_b = *res.get(&p("b.rs")).unwrap();
                assert!(r_b > r_a, "Target rank > source rank");
                assert_approx_eq(r_a + r_b, 1.0, "Sum to 1.0");
            }),
            "Directed edge"
        ));

        // 4. Cycle (a -> b -> c -> a)
        cases.push((
            Box::new(|| {
                let mut edges = HashMap::new();
                edges.insert(p("a.rs"), [(p("b.rs"), 1)].into_iter().collect());
                edges.insert(p("b.rs"), [(p("c.rs"), 1)].into_iter().collect());
                edges.insert(p("c.rs"), [(p("a.rs"), 1)].into_iter().collect());
                (edges, f(&["a.rs", "b.rs", "c.rs"]), None)
            }),
            Box::new(|res| {
                let expected = 1.0/3.0;
                for val in res.values() {
                    assert_approx_eq(*val, expected, "Cycle equal rank");
                }
            }),
            "Cycle"
        ));

        // 5. Anchor (Personalization)
        cases.push((
            Box::new(|| {
                let mut edges = HashMap::new();
                edges.insert(p("a.rs"), [(p("b.rs"), 1)].into_iter().collect());
                (edges, f(&["a.rs", "b.rs"]), Some(p("a.rs")))
            }),
            Box::new(|res| {
                let r_a = *res.get(&p("a.rs")).unwrap();
                // Without anchor, a would be low. With anchor, it's boosted.
                // In directed edge a->b, steady state is low for a. 
                // Anchor ensures probability resets to a.
                assert!(r_a > 0.3, "Anchor boosts rank significantly (got {r_a})"); 
            }),
            "Anchor boost"
        ));

        for (setup, check, _desc) in cases {
            let (edges, all_files, anchor_opt) = setup();
            let anchor = anchor_opt.as_ref();
            let result = compute(&edges, &all_files, anchor);
            check(result);
            // println!("Passed: {}", desc);
        }
    }

    #[test]
    fn test_helpers_table() {
        // initialize_ranks
        let files_set = f(&["a", "b"]);
        let ranks = initialize_ranks(&files_set, 2.0);
        assert_approx_eq(*ranks.get(&p("a")).unwrap(), 0.5, "init rank");

        // normalize
        let mut r = HashMap::new();
        r.insert(p("a"), 1.0);
        r.insert(p("b"), 3.0);
        normalize(&mut r);
        assert_approx_eq(*r.get(&p("a")).unwrap(), 0.25, "normalize a");
        assert_approx_eq(*r.get(&p("b")).unwrap(), 0.75, "normalize b");

        // normalize empty/zero
        let mut r_zero = HashMap::new();
        r_zero.insert(p("x"), 0.0);
        normalize(&mut r_zero);
        assert_approx_eq(*r_zero.get(&p("x")).unwrap(), 0.0, "normalize zero");
    }
}