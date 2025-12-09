use super::Symbol;
use std::collections::{HashMap, HashSet};

/// Call graph for reachability analysis.
pub struct CallGraph {
    pub symbols: HashSet<Symbol>,
    pub calls: HashMap<Symbol, HashSet<Symbol>>,
    pub called_by: HashMap<Symbol, HashSet<Symbol>>,
    pub entry_points: HashSet<Symbol>,
    pub public_symbols: HashSet<Symbol>,
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
            self.process_callees(&sym, &reachable, &mut worklist);
        }

        reachable
    }

    fn process_callees(
        &self,
        sym: &Symbol,
        reachable: &HashSet<Symbol>,
        worklist: &mut Vec<Symbol>,
    ) {
        if let Some(callees) = self.calls.get(sym) {
            for callee in callees {
                if !reachable.contains(callee) {
                    worklist.push(callee.clone());
                }
            }
        }
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
