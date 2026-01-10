// src/graph/rank/builder.rs
//! Graph construction logic: extraction and edge building.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::tags::{Tag, TagKind};
use crate::graph::defs;
use crate::graph::imports;

/// Container for the raw data needed to construct the graph.
pub struct GraphData {
    pub tags: Vec<Tag>,
    pub defines: HashMap<String, HashSet<PathBuf>>,
    pub references: HashMap<String, HashSet<PathBuf>>,
    pub edges: HashMap<PathBuf, HashMap<PathBuf, usize>>,
    pub all_files: HashSet<PathBuf>,
}

/// Extracts tags from all files and builds the initial edge set.
#[must_use]
pub fn build_data(files: &[(PathBuf, String)]) -> GraphData {
    let extracted = extract_all_tags(files);
    let edges = build_edges(&extracted.defines, &extracted.references);
    let all_files = collect_all_files(&edges);

    GraphData {
        tags: extracted.tags,
        defines: extracted.defines,
        references: extracted.references,
        edges,
        all_files,
    }
}

/// Rebuilds edges and file list from existing definitions and references.
/// Used when re-focusing the graph without re-parsing files.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn rebuild_topology(
    defines: &HashMap<String, HashSet<PathBuf>>,
    references: &HashMap<String, HashSet<PathBuf>>,
) -> (HashMap<PathBuf, HashMap<PathBuf, usize>>, HashSet<PathBuf>) {
    let edges = build_edges(defines, references);
    let all_files = collect_all_files(&edges);
    (edges, all_files)
}

struct ExtractedTags {
    tags: Vec<Tag>,
    defines: HashMap<String, HashSet<PathBuf>>,
    references: HashMap<String, HashSet<PathBuf>>,
}

fn extract_all_tags(files: &[(PathBuf, String)]) -> ExtractedTags {
    let mut tags = Vec::new();
    let mut defines: HashMap<String, HashSet<PathBuf>> = HashMap::new();
    let mut references: HashMap<String, HashSet<PathBuf>> = HashMap::new();

    for (path, content) in files {
        extract_defs(path, content, &mut tags, &mut defines);
        extract_refs(path, content, &mut tags, &mut references);
    }

    ExtractedTags {
        tags,
        defines,
        references,
    }
}

fn extract_defs(
    path: &Path,
    content: &str,
    tags: &mut Vec<Tag>,
    defines: &mut HashMap<String, HashSet<PathBuf>>,
) {
    for def in defs::extract(path, content) {
        defines
            .entry(def.name.clone())
            .or_default()
            .insert(path.to_path_buf());
        tags.push(Tag {
            file: path.to_path_buf(),
            name: def.name,
            kind: TagKind::Def,
            line: def.line,
            signature: Some(def.signature),
        });
    }
}

fn extract_refs(
    path: &Path,
    content: &str,
    tags: &mut Vec<Tag>,
    references: &mut HashMap<String, HashSet<PathBuf>>,
) {
    for ref_name in imports::extract(path, content) {
        let symbol = ref_name
            .split("::")
            .last()
            .unwrap_or(&ref_name)
            .to_string();
        
        references
            .entry(symbol.clone())
            .or_default()
            .insert(path.to_path_buf());
            
        tags.push(Tag {
            file: path.to_path_buf(),
            name: symbol,
            kind: TagKind::Ref,
            line: 0,
            signature: None,
        });
    }
}

fn build_edges(
    defines: &HashMap<String, HashSet<PathBuf>>,
    references: &HashMap<String, HashSet<PathBuf>>,
) -> HashMap<PathBuf, HashMap<PathBuf, usize>> {
    let mut edges: HashMap<PathBuf, HashMap<PathBuf, usize>> = HashMap::new();

    for symbol in defines.keys() {
        if references.contains_key(symbol) {
            add_symbol_edges(symbol, defines, references, &mut edges);
        }
    }

    edges
}

fn add_symbol_edges(
    symbol: &str,
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, HashSet<PathBuf>>,
    edges: &mut HashMap<PathBuf, HashMap<PathBuf, usize>>,
) {
    let Some(def_files) = def_map.get(symbol) else {
        return;
    };
    let Some(ref_files) = ref_map.get(symbol) else {
        return;
    };

    for ref_file in ref_files {
        for def_file in def_files {
            if ref_file != def_file {
                *edges
                    .entry(ref_file.clone())
                    .or_default()
                    .entry(def_file.clone())
                    .or_default() += 1;
            }
        }
    }
}

fn collect_all_files(edges: &HashMap<PathBuf, HashMap<PathBuf, usize>>) -> HashSet<PathBuf> {
    let mut files = HashSet::new();
    for (src, targets) in edges {
        files.insert(src.clone());
        files.extend(targets.keys().cloned());
    }
    files
}