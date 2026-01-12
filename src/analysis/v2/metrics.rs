// src/analysis/v2/metrics.rs
//! Metrics calculation for V2 Scopes (LCOM4, CBO, SFOUT, AHF).

use super::scope::{Scope, Method};
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
        let adj = Self::build_adjacency_graph(scope, &method_names);

        Self::count_components(&method_names, &adj)
    }

    /// Calculates CBO (Coupling Between Objects).
    #[must_use]
    pub fn calculate_cbo(scope: &Scope) -> usize {
        let mut unique_deps = HashSet::new();
        for method in scope.methods().values() {
            for call in &method.external_calls {
                unique_deps.insert(call);
            }
        }
        unique_deps.len()
    }

    /// Calculates the maximum SFOUT (Structural Fan-Out) among methods.
    #[must_use]
    pub fn calculate_max_sfout(scope: &Scope) -> usize {
        scope.methods()
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

    fn build_adjacency_graph<'a>(
        scope: &Scope,
        method_names: &[&'a String],
    ) -> HashMap<&'a String, Vec<&'a String>> {
        let mut adj = HashMap::new();
        for name in method_names {
            adj.insert(*name, Vec::new());
        }

        let methods = scope.methods();
        for (i, name_a) in method_names.iter().enumerate() {
            let method_a = &methods[*name_a];

            for name_b in method_names.iter().skip(i + 1) {
                let method_b = &methods[*name_b];

                if Self::are_connected(method_a, method_b) {
                    if let Some(vec_a) = adj.get_mut(*name_a) {
                        vec_a.push(*name_b);
                    }
                    if let Some(vec_b) = adj.get_mut(*name_b) {
                        vec_b.push(*name_a);
                    }
                }
            }
        }
        adj
    }

    fn count_components<'a>(
        names: &[&'a String],
        adj: &HashMap<&'a String, Vec<&'a String>>,
    ) -> usize {
        let mut visited = HashSet::new();
        let mut components = 0;

        for name in names {
            if !visited.contains(name) {
                components += 1;
                Self::traverse(name, adj, &mut visited);
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

            for neighbor in neighbors {
                if visited.insert(neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }
}
