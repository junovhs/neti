// src/graph/rank/queries.rs
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use crate::graph::rank::graph::RepoGraph;
use crate::graph::rank::tags::{Tag, TagKind};

#[must_use]
pub fn get_neighbors(graph: &RepoGraph, anchor: &Path) -> Vec<PathBuf> {
    let mut result = HashSet::new();
    let anchor_path = anchor.to_path_buf();
    collect_dependents(&graph.defines, &graph.references, &anchor_path, &mut result);
    collect_dependencies(&graph.defines, &graph.references, &anchor_path, &mut result);
    result.into_iter().collect()
}

#[must_use]
pub fn get_ranked_files(graph: &RepoGraph) -> Vec<(PathBuf, f64)> {
    let mut ranked: Vec<_> = graph.ranks.iter().map(|(p, r)| (p.clone(), *r)).collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

#[must_use]
pub fn get_dependencies(graph: &RepoGraph, anchor: &Path) -> Vec<PathBuf> {
    let mut result = HashSet::new();
    collect_dependencies(&graph.defines, &graph.references, &anchor.to_path_buf(), &mut result);
    let mut deps: Vec<_> = result.into_iter().collect();
    deps.sort();
    deps
}

#[must_use]
pub fn get_dependents(graph: &RepoGraph, anchor: &Path) -> Vec<PathBuf> {
    let mut result = HashSet::new();
    collect_dependents(&graph.defines, &graph.references, &anchor.to_path_buf(), &mut result);
    let mut deps: Vec<_> = result.into_iter().collect();
    deps.sort();
    deps
}

#[must_use]
pub fn get_graph_tags(graph: &RepoGraph) -> Vec<Tag> {
    graph.tags.iter().filter(|t| t.kind == TagKind::Def).cloned().collect()
}

#[allow(clippy::implicit_hasher)]
pub fn collect_dependents(
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, HashSet<PathBuf>>,
    anchor: &PathBuf,
    result: &mut HashSet<PathBuf>,
) {
    for (symbol, def_files) in def_map {
        if def_files.contains(anchor) {
            add_refs_to_result(symbol, ref_map, anchor, result);
        }
    }
}

#[allow(clippy::implicit_hasher)]
pub fn collect_dependencies(
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, HashSet<PathBuf>>,
    anchor: &PathBuf,
    result: &mut HashSet<PathBuf>,
) {
    for (symbol, ref_files) in ref_map {
        if ref_files.contains(anchor) {
            add_defs_to_result(symbol, def_map, anchor, result);
        }
    }
}

fn add_refs_to_result(
    symbol: &str,
    ref_map: &HashMap<String, HashSet<PathBuf>>,
    anchor: &PathBuf,
    result: &mut HashSet<PathBuf>,
) {
    if let Some(refs) = ref_map.get(symbol) {
        for f in refs {
            if f != anchor {
                result.insert(f.clone());
            }
        }
    }
}

fn add_defs_to_result(
    symbol: &str,
    def_map: &HashMap<String, HashSet<PathBuf>>,
    anchor: &PathBuf,
    result: &mut HashSet<PathBuf>,
) {
    if let Some(defs) = def_map.get(symbol) {
        for f in defs {
            if f != anchor {
                result.insert(f.clone());
            }
        }
    }
}