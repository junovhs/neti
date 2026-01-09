// src/analysis/v2/scope.rs
use std::collections::{HashMap, HashSet};

/// Represents a cohesion scope (Class, Struct+Impl).
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
    /// Other methods in the same scope called by this method
    pub method_calls: HashSet<String>,
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
    /// LCOM4 = Number of connected components in the method dependency graph.
    #[must_use]
    pub fn calculate_lcom4(&self) -> usize {
        if self.methods.is_empty() {
            return 0;
        }

        let method_names: Vec<&String> = self.methods.keys().collect();
        let adj = self.build_adjacency_graph(&method_names);

        Self::count_components(&method_names, &adj)
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
                    // Safe unwrap: keys initialized above
                    // Dereference *name_b to go from &&String to &String
                    adj.get_mut(*name_a).unwrap().push(*name_b);
                    adj.get_mut(*name_b).unwrap().push(*name_a);
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
        if a.method_calls.contains(&b.name) {
            return true;
        }
        if b.method_calls.contains(&a.name) {
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
            // Flatten nesting by using guard clause
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
                method_calls: HashSet::new(),
            },
        );

        scope.methods.insert(
            "set_x".into(),
            Method {
                name: "set_x".into(),
                field_access: HashSet::from(["x".into()]),
                method_calls: HashSet::new(),
            },
        );

        assert_eq!(scope.calculate_lcom4(), 1);
    }

    #[test]
    fn test_lcom4_split() {
        let mut scope = Scope::new("Split");
        scope.fields.insert("x".into());
        scope.fields.insert("y".into());

        scope.methods.insert(
            "m1".into(),
            Method {
                name: "m1".into(),
                field_access: HashSet::from(["x".into()]),
                method_calls: HashSet::new(),
            },
        );

        scope.methods.insert(
            "m2".into(),
            Method {
                name: "m2".into(),
                field_access: HashSet::from(["y".into()]),
                method_calls: HashSet::new(),
            },
        );

        assert_eq!(scope.calculate_lcom4(), 2);
    }
}