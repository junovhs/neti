// src/audit/dead_code.rs
//! Dead code detection using call graph reachability analysis.
//!
//! This module identifies code that is:
//! 1. Not reachable from any entry point (main, lib exports, tests)
//! 2. Defined but never referenced anywhere
//! 3. Only referenced by other dead code

use super::types::{CodeUnit, DeadCode, DeadCodeReason};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A node in the call graph.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol {
    /// The defining file.
    pub file: PathBuf,
    /// The symbol name (fully qualified where possible).
    pub name: String,
}

/// Call graph for reachability analysis.
pub struct CallGraph {
    symbols: HashSet<Symbol>,
    calls: HashMap<Symbol, HashSet<Symbol>>,
    called_by: HashMap<Symbol, HashSet<Symbol>>,
    entry_points: HashSet<Symbol>,
    public_symbols: HashSet<Symbol>,
}

impl CallGraph {
    /// Creates a new empty call graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            symbols: HashSet::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            entry_points: HashSet::new(),
            public_symbols: HashSet::new(),
        }
    }

    /// Adds a symbol definition.
    pub fn add_symbol(&mut self, symbol: Symbol, is_public: bool, is_entry: bool) {
        self.symbols.insert(symbol.clone());

        if is_public {
            self.public_symbols.insert(symbol.clone());
        }

        if is_entry {
            self.entry_points.insert(symbol);
        }
    }

    /// Adds an edge: `from` calls/references `to`.
    pub fn add_edge(&mut self, from: Symbol, to: Symbol) {
        self.calls
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.called_by.entry(to).or_default().insert(from);
    }

    /// Computes reachable symbols from entry points.
    #[must_use]
    pub fn compute_reachable(&self) -> HashSet<Symbol> {
        let mut reachable = HashSet::new();
        let mut worklist: Vec<Symbol> = self.entry_points.iter().cloned().collect();

        worklist.extend(self.public_symbols.iter().cloned());

        while let Some(sym) = worklist.pop() {
            if reachable.contains(&sym) {
                continue;
            }

            reachable.insert(sym.clone());

            if let Some(callees) = self.calls.get(&sym) {
                for callee in callees {
                    if !reachable.contains(callee) {
                        worklist.push(callee.clone());
                    }
                }
            }
        }

        reachable
    }

    /// Finds unreachable symbols.
    #[must_use]
    pub fn find_unreachable(&self) -> Vec<(Symbol, DeadCodeReason)> {
        let reachable = self.compute_reachable();

        self.symbols
            .iter()
            .filter(|sym| !reachable.contains(*sym))
            .map(|sym| {
                let reason = classify_dead_reason(sym, &self.called_by, &reachable);
                (sym.clone(), reason)
            })
            .collect()
    }

    /// Returns the number of symbols in the graph.
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Returns the number of edges in the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.calls.values().map(HashSet::len).sum()
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn classify_dead_reason(
    sym: &Symbol,
    called_by: &HashMap<Symbol, HashSet<Symbol>>,
    reachable: &HashSet<Symbol>,
) -> DeadCodeReason {
    let Some(callers) = called_by.get(sym) else {
        return DeadCodeReason::Unused;
    };

    if callers.is_empty() {
        return DeadCodeReason::Unused;
    }

    let has_live_caller = callers.iter().any(|c| reachable.contains(c));
    if has_live_caller {
        DeadCodeReason::Unreachable
    } else {
        DeadCodeReason::OnlyDeadCallers
    }
}

/// Detects dead code from a list of code units and their references.
#[must_use]
pub fn detect(
    units: &[CodeUnit],
    references: &[(PathBuf, String, String)],
    entry_points: &[String],
) -> Vec<DeadCode> {
    let mut graph = CallGraph::new();

    for unit in units {
        let symbol = Symbol {
            file: unit.file.clone(),
            name: unit.name.clone(),
        };

        let is_entry = is_entry_point(&unit.name, &unit.file, entry_points);
        let is_public = is_likely_public(&unit.name);

        graph.add_symbol(symbol, is_public, is_entry);
    }

    for (file, from_name, to_name) in references {
        let from = Symbol {
            file: file.clone(),
            name: from_name.clone(),
        };
        let to = Symbol {
            file: file.clone(),
            name: to_name.clone(),
        };

        graph.add_edge(from, to);
    }

    let unreachable = graph.find_unreachable();

    let unit_map: HashMap<(&PathBuf, &str), &CodeUnit> = units
        .iter()
        .map(|u| ((&u.file, u.name.as_str()), u))
        .collect();

    unreachable
        .into_iter()
        .filter_map(|(sym, reason)| {
            let unit = unit_map.get(&(&sym.file, sym.name.as_str()))?;
            Some(DeadCode {
                unit: (*unit).clone(),
                reason,
            })
        })
        .collect()
}

fn is_entry_point(name: &str, file: &Path, explicit_entries: &[String]) -> bool {
    if explicit_entries.contains(&name.to_string()) {
        return true;
    }

    if name == "main" {
        return true;
    }

    if name.starts_with("test_") || name.contains("_test") {
        return true;
    }

    let path_str = file.to_string_lossy();
    if path_str.contains("/bin/") || path_str.contains("/examples/") {
        return true;
    }

    false
}

fn is_likely_public(name: &str) -> bool {
    !name.starts_with('_')
}

/// Analyzes imports and references to build edge list.
#[must_use]
pub fn extract_references(
    source: &str,
    _file: &Path,
    tree: &tree_sitter::Tree,
) -> Vec<(String, String)> {
    let mut refs = Vec::new();
    let source_bytes = source.as_bytes();

    extract_refs_from_node(tree.root_node(), source_bytes, &mut refs, None);

    refs
}

fn extract_refs_from_node(
    node: tree_sitter::Node,
    source: &[u8],
    refs: &mut Vec<(String, String)>,
    current_fn: Option<&str>,
) {
    let kind = node.kind();

    let new_fn = if kind == "function_item" || kind == "function_definition" {
        node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
    } else {
        None
    };

    let fn_name = new_fn.or(current_fn);

    if kind == "call_expression" {
        if let Some(caller) = fn_name {
            if let Some(callee) = extract_call_target(node, source) {
                refs.push((caller.to_string(), callee));
            }
        }
    }

    if kind == "identifier" || kind == "field_identifier" {
        if let Some(caller) = fn_name {
            if let Ok(name) = node.utf8_text(source) {
                if !is_common_identifier(name) {
                    refs.push((caller.to_string(), name.to_string()));
                }
            }
        }
    }

    for child in node.children(&mut node.walk()) {
        extract_refs_from_node(child, source, refs, fn_name);
    }
}

fn extract_call_target(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    let target = node.child_by_field_name("function")?;

    match target.kind() {
        "identifier" | "scoped_identifier" => target.utf8_text(source).ok().map(String::from),
        "field_expression" => target
            .child_by_field_name("field")
            .and_then(|n| n.utf8_text(source).ok())
            .map(String::from),
        _ => None,
    }
}

fn is_common_identifier(name: &str) -> bool {
    matches!(
        name,
        "self" | "Self" | "super" | "crate" | "true" | "false" | "None" | "Some" | "Ok" | "Err"
    )
}

