// src/analysis/v2/scope.rs
use std::collections::{HashMap, HashSet};

/// Represents a cohesion and coupling scope (Class, Struct+Impl).
#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub fields: HashSet<String>,
    pub methods: HashMap<String, Method>,
}

/// Represents a method within a scope.
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    /// Fields accessed by this method
    pub field_access: HashSet<String>,
    /// Other methods in the same scope called by this method (Cohesion)
    pub internal_calls: HashSet<String>,
    /// Calls to things outside this scope (Coupling/SFOUT)
    pub external_calls: HashSet<String>,
    /// Human-understandability score
    pub cognitive_complexity: usize,
}

impl Scope {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: HashSet::new(),
            methods: HashMap::new(),
        }
    }

    /// Calculates LCOM4 (Lack of Cohesion of Methods).
    #[must_use]
    pub fn calculate_lcom4(&self) -> usize {
        if self.methods.is_empty() {
            return 0;
        }

        let method_names: Vec<&String> = self.methods.keys().collect();
        let adj = self.build_adjacency_graph(&method_names);

        Self::count_components(&method_names, &adj)
    }

    /// Calculates CBO (Coupling Between Objects).
    #[must_use]
    pub fn calculate_cbo(&self) -> usize {
        let mut unique_deps = HashSet::new();
        for method in self.methods.values() {
            for call in &method.external_calls {
                unique_deps.insert(call);
            }
        }
        unique_deps.len()
    }

    /// Calculates the maximum SFOUT (Structural Fan-Out) among methods.
    #[must_use]
    pub fn calculate_max_sfout(&self) -> usize {
        self.methods
            .values()
            .map(|m| m.external_calls.len())
            .max()
            .unwrap_or(0)
    }

    fn build_adjacency_graph<'a>(
        &self,
        method_names: &[&'a String],
    ) -> HashMap<&'a String, Vec<&'a String>> {
        let mut adj = HashMap::new();
        for name in method_names {
            adj.insert(*name, Vec::new());
        }

        for (i, name_a) in method_names.iter().enumerate() {
            let method_a = &self.methods[*name_a];

            for name_b in method_names.iter().skip(i + 1) {
                let method_b = &self.methods[*name_b];

                if Self::are_connected(method_a, method_b) {
                    adj.get_mut(*name_a).expect("Key exists").push(*name_b);
                    adj.get_mut(*name_b).expect("Key exists").push(*name_a);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcom4_cohesive() {
        let mut scope = Scope::new("Cohesive");
        scope.fields.insert("x".into());

        scope.methods.insert(
            "get_x".into(),
            Method {
                name: "get_x".into(),
                field_access: HashSet::from(["x".into()]),
                internal_calls: HashSet::new(),
                external_calls: HashSet::new(),
                cognitive_complexity: 0,
            },
        );

        scope.methods.insert(
            "set_x".into(),
            Method {
                name: "set_x".into(),
                field_access: HashSet::from(["x".into()]),
                internal_calls: HashSet::new(),
                external_calls: HashSet::new(),
                cognitive_complexity: 0,
            },
        );

        assert_eq!(scope.calculate_lcom4(), 1);
    }
}