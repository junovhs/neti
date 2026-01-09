// src/analysis/v2/mod.rs
pub mod cognitive;
pub mod scope;
pub mod visitor;

use crate::config::Config;
use crate::lang::Lang;
use crate::types::{Violation, ViolationDetails};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::Parser;

pub struct ScanEngineV2 {
    #[allow(dead_code)]
    config: Config,
}

impl ScanEngineV2 {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the Scan v2 engine and returns violations mapped by file path.
    #[must_use]
    pub fn run(&self, files: &[PathBuf]) -> HashMap<PathBuf, Vec<Violation>> {
        let mut global_scopes = HashMap::new();
        let mut path_map = HashMap::new();

        for path in files {
            if let Some((scopes, p_str)) = Self::process_file(path) {
                for (name, scope) in scopes {
                    let key = format!("{p_str}::{name}");
                    global_scopes.insert(key, scope);
                    path_map.insert(p_str.clone(), path.clone());
                }
            }
        }

        Self::analyze_all_scopes(&global_scopes, &path_map)
    }

    fn process_file(path: &Path) -> Option<(HashMap<String, scope::Scope>, String)> {
        let source = std::fs::read_to_string(path).ok()?;
        let ext = path.extension()?.to_str()?;
        let lang = Lang::from_ext(ext)?;

        // Currently only Rust is supported
        if lang != Lang::Rust {
            return None;
        }

        let mut parser = Parser::new();
        parser.set_language(lang.grammar()).ok()?;

        let tree = parser.parse(&source, None)?;
        let visitor = visitor::AstVisitor::new(&source, lang);
        let file_scopes = visitor.extract_scopes(tree.root_node());

        Some((file_scopes, path.to_string_lossy().to_string()))
    }

    fn analyze_all_scopes(
        scopes: &HashMap<String, scope::Scope>,
        path_map: &HashMap<String, PathBuf>,
    ) -> HashMap<PathBuf, Vec<Violation>> {
        let mut results: HashMap<PathBuf, Vec<Violation>> = HashMap::new();

        for (full_name, scope) in scopes {
            let path_str = full_name.split("::").next().unwrap_or("");
            let Some(path) = path_map.get(path_str) else {
                continue;
            };

            let mut violations = Vec::new();

            let lcom4 = scope.calculate_lcom4();
            let cbo = scope.calculate_cbo();
            let sfout = scope.calculate_max_sfout();

            Self::check_scope_cohesion(scope, lcom4, &mut violations);
            Self::check_scope_encapsulation(scope, &mut violations);
            Self::check_scope_coupling(scope, cbo, sfout, &mut violations);

            if !violations.is_empty() {
                results.entry(path.clone()).or_default().extend(violations);
            }
        }

        results
    }

    fn check_scope_cohesion(scope: &scope::Scope, lcom4: usize, out: &mut Vec<Violation>) {
        // LCOM4 was designed for OOP classes with instance variables.
        // Enums are sum types — they don't have shared mutable state.
        // Skip LCOM4 entirely for enums.
        if scope.is_enum {
            return;
        }

        // LCOM4 requires state to measure cohesion against.
        // Per Hitz & Montazeri: "methods share instance variables"
        // If no method accesses any fields, cohesion is meaningless — skip.
        // This filters out stateless utility structs.
        let has_field_access = scope.methods.values().any(|m| !m.field_access.is_empty());
        if !has_field_access {
            return;
        }

        // Small types (< 4 methods) with low LCOM4 are usually fine DTOs.
        // Real god objects have many methods in disconnected groups.
        // Require at least 4 methods to flag LCOM4 violations.
        if scope.methods.len() < 4 {
            return;
        }

        if lcom4 > 1 {
            out.push(Violation::with_details(
                scope.row,
                format!("Class '{}' has low cohesion (LCOM4: {})", scope.name, lcom4),
                "LCOM4",
                ViolationDetails {
                    function_name: Some(scope.name.clone()),
                    analysis: vec![
                        format!("Connected components: {lcom4}"),
                        format!("Methods: {}", scope.methods.len()),
                        "Methods in this struct/class don't share fields or call each other."
                            .into(),
                        "This suggests it's doing multiple unrelated things.".into(),
                    ],
                    suggestion: Some(format!(
                        "Consider splitting '{}' into {} smaller, focused types.",
                        scope.name, lcom4
                    )),
                },
            ));
        }
    }

    fn check_scope_encapsulation(scope: &scope::Scope, out: &mut Vec<Violation>) {
        if scope.is_enum {
            return;
        }

        // Calculate AHF
        let ahf = scope.calculate_ahf();
        // Threshold: AHF < 60%
        if ahf < 60.0 {
            // Only report if there are actually fields
            if scope.fields.is_empty() {
                return;
            }

            // Create violation
             out.push(Violation::with_details(
                scope.row,
                format!("Class '{}' exposes too much state (AHF: {:.1}%)", scope.name, ahf),
                "AHF",
                ViolationDetails {
                    function_name: Some(scope.name.clone()),
                    analysis: vec![
                        format!("Attribute Hiding Factor (AHF) is {:.1}% (min 60%)", ahf),
                        format!("{} of {} fields are private.",
                            scope.fields.values().filter(|f| !f.is_public).count(),
                            scope.fields.len()
                        ),
                        "Low AHF means state is leaking, increasing coupling risk.".into(),
                    ],
                    suggestion: Some(
                        "Encapsulate fields: make them private and provide accessors if needed.".into(),
                    ),
                },
            ));
        }
    }

    fn check_scope_coupling(
        scope: &scope::Scope,
        cbo: usize,
        sfout: usize,
        out: &mut Vec<Violation>,
    ) {
        if cbo > 9 {
            out.push(Violation::with_details(
                scope.row,
                format!("Class '{}' is tightly coupled (CBO: {})", scope.name, cbo),
                "CBO",
                ViolationDetails {
                    function_name: Some(scope.name.clone()),
                    analysis: vec![
                        format!("External dependencies: {cbo}"),
                        "High coupling predicts defects and makes changes risky.".into(),
                    ],
                    suggestion: Some(
                        "Reduce dependencies on external modules. Consider facade pattern.".into(),
                    ),
                },
            ));
        }

        if sfout > 7 {
            out.push(Violation::with_details(
                scope.row,
                format!(
                    "Class '{}' has high fan-out (Max SFOUT: {})",
                    scope.name, sfout
                ),
                "SFOUT",
                ViolationDetails {
                    function_name: Some(scope.name.clone()),
                    analysis: vec![
                        format!("Max outgoing calls in one method: {sfout}"),
                        "High fan-out methods are bottlenecks — changes ripple everywhere.".into(),
                    ],
                    suggestion: Some(
                        "Delegate responsibilities to helper functions or modules.".into(),
                    ),
                },
            ));
        }
    }
}


