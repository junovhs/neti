//! Structural metrics calculation (LCOM4, CBO, SFOUT, AHF).
//! Renamed from v2/metrics.rs to avoid collision with root metrics.rs (complexity/nesting).

use super::scope::{Method, Scope};
use std::collections::{HashMap, HashSet};

pub struct ScopeMetrics;

impl ScopeMetrics {
    /// Calculates LCOM4 (Lack of Cohesion of Methods).
    #[must_use]
    pub fn calculate_lcom4(scope: &Scope) -> usize {
        let methods = scope.methods();
        if methods.is_empty() {
            return 0;
        }

        let method_names: Vec<&String> = methods.keys().collect();
        let adj = build_adjacency_graph(scope, &method_names);

        count_components(&method_names, &adj)
    }

    /// Calculates CBO (Coupling Between Objects).
    #[must_use]
    pub fn calculate_cbo(scope: &Scope) -> usize {
        let unique_deps: HashSet<_> = scope
            .methods()
            .values()
            .flat_map(|m| &m.external_calls)
            .collect();
        unique_deps.len()
    }

    /// Calculates the maximum SFOUT (Structural Fan-Out) among methods.
    #[must_use]
    pub fn calculate_max_sfout(scope: &Scope) -> usize {
        scope
            .methods()
            .values()
            .map(|m| m.external_calls.len())
            .max()
            .unwrap_or(0)
    }

    /// Calculates AHF (Attribute Hiding Factor).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_ahf(scope: &Scope) -> f64 {
        let fields = scope.fields();
        if fields.is_empty() {
            return 100.0;
        }

        let total_fields = fields.len() as f64;
        let private_fields = fields.values().filter(|f| !f.is_public).count() as f64;

        (private_fields / total_fields) * 100.0
    }
}

fn build_adjacency_graph<'a>(
    scope: &Scope,
    method_names: &[&'a String],
) -> HashMap<&'a String, Vec<&'a String>> {
    let mut adj = HashMap::new();
    for name in method_names {
        adj.insert(*name, Vec::new());
    }

    let methods = scope.methods();
    let pairs = method_name_pairs(method_names);

    for (name_a, name_b) in &pairs {
        let method_a = &methods[*name_a];
        let method_b = &methods[*name_b];

        if are_connected(method_a, method_b) {
            if let Some(vec_a) = adj.get_mut(*name_a) {
                vec_a.push(*name_b);
            }
            if let Some(vec_b) = adj.get_mut(*name_b) {
                vec_b.push(*name_a);
            }
        }
    }
    adj
}

fn method_name_pairs<'a>(names: &[&'a String]) -> Vec<(&'a String, &'a String)> {
    names
        .iter()
        .enumerate()
        .flat_map(|(i, a)| names[i + 1..].iter().map(move |b| (*a, *b)))
        .collect()
}

fn count_components<'a>(names: &[&'a String], adj: &HashMap<&'a String, Vec<&'a String>>) -> usize {
    let mut visited = HashSet::new();
    let mut components = 0;

    for name in names {
        if !visited.contains(name) {
            components += 1;
            traverse(name, adj, &mut visited);
        }
    }
    components
}

fn are_connected(a: &Method, b: &Method) -> bool {
    if !a.field_access.is_disjoint(&b.field_access) {
        return true;
    }
    if a.internal_calls.contains(&b.name) || b.internal_calls.contains(&a.name) {
        return true;
    }
    false
}

fn traverse<'a>(
    start: &'a String,
    adj: &HashMap<&'a String, Vec<&'a String>>,
    visited: &mut HashSet<&'a String>,
) {
    let mut stack = vec![start];
    visited.insert(start);

    while let Some(current) = stack.pop() {
        let Some(neighbors) = adj.get(current) else {
            continue;
        };

        stack.extend(neighbors.iter().filter(|n| visited.insert(n)).copied());
    }
}
