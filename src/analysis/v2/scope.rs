// src/analysis/v2/scope.rs
use std::collections::{HashMap, HashSet};

/// Represents a cohesion and coupling scope (Class, Struct+Impl, Enum).
#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub row: usize,
    pub is_enum: bool,
    pub fields: HashMap<String, FieldInfo>,
    pub methods: HashMap<String, Method>,
    pub derives: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub is_public: bool,
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
    /// Does the method mutate state? (&mut self)
    pub is_mutable: bool,
}

impl Scope {
    #[must_use]
    pub fn new(name: &str, row: usize) -> Self {
        Self {
            name: name.to_string(),
            row,
            is_enum: false,
            fields: HashMap::new(),
            methods: HashMap::new(),
            derives: HashSet::new(),
        }
    }

    #[must_use]
    pub fn new_enum(name: &str, row: usize) -> Self {
        Self {
            name: name.to_string(),
            row,
            is_enum: true,
            fields: HashMap::new(),
            methods: HashMap::new(),
            derives: HashSet::new(),
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
    /// Number of distinct external classes/scopes this scope depends on.
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
    /// SFOUT = number of outgoing calls from a single method.
    #[must_use]
    pub fn calculate_max_sfout(&self) -> usize {
        self.methods
            .values()
            .map(|m| m.external_calls.len())
            .max()
            .unwrap_or(0)
    }

    /// Calculates AHF (Attribute Hiding Factor).
    /// Percentage of fields that are private.
    /// AHF = (`sum(is_private)` / `total_fields`) * 100
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_ahf(&self) -> f64 {
        if self.fields.is_empty() {
            // If there are no fields, state leaking is impossible.
            return 100.0;
        }

        let total_fields = self.fields.len() as f64;
        let private_fields = self.fields.values().filter(|f| !f.is_public).count() as f64;

        (private_fields / total_fields) * 100.0
    }

    /// Calculates the sum of cognitive complexity of all methods.
    /// Used to distinguish "Data Structures" (complexity 0) from "Logic Objects".
    #[must_use]
    pub fn has_behavior(&self) -> bool {
        self.methods
            .values()
            .any(|m| m.cognitive_complexity > 0 || m.is_mutable)
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
        let mut scope = Scope::new("Cohesive", 1);
        scope.fields.insert(
            "x".into(),
            FieldInfo {
                name: "x".into(),
                is_public: false,
            },
        );

        scope.methods.insert(
            "get_x".into(),
            Method {
                name: "get_x".into(),
                field_access: HashSet::from(["x".into()]),
                internal_calls: HashSet::new(),
                external_calls: HashSet::new(),
                cognitive_complexity: 0,
                is_mutable: false,
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
                is_mutable: false,
            },
        );

        assert_eq!(scope.calculate_lcom4(), 1);
    }

    #[test]
    fn test_ahf_calculation() {
        let mut scope = Scope::new("TestAhf", 1);

        // 3 private fields, 1 public field
        // AHF = 3/4 = 75%
        scope.fields.insert(
            "p1".into(),
            FieldInfo {
                name: "p1".into(),
                is_public: false,
            },
        );
        scope.fields.insert(
            "p2".into(),
            FieldInfo {
                name: "p2".into(),
                is_public: false,
            },
        );
        scope.fields.insert(
            "p3".into(),
            FieldInfo {
                name: "p3".into(),
                is_public: false,
            },
        );
        scope.fields.insert(
            "pub1".into(),
            FieldInfo {
                name: "pub1".into(),
                is_public: true,
            },
        );

        assert!((scope.calculate_ahf() - 75.0).abs() < f64::EPSILON);
    }
}
