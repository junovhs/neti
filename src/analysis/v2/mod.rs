// src/analysis/v2/mod.rs
pub mod cognitive;
pub mod scope;
pub mod visitor;
pub mod rust;
pub mod metrics;
pub mod patterns;

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
        let mut results: HashMap<PathBuf, Vec<Violation>> = HashMap::new();
        let mut global_scopes = HashMap::new();
        let mut path_map = HashMap::new();

        for path in files {
            Self::process_file_patterns(path, &mut results);
            Self::process_file_scopes(path, &mut global_scopes, &mut path_map);
        }

        Self::merge_scope_violations(&global_scopes, &path_map, &mut results);
        results
    }

    fn process_file_patterns(path: &Path, results: &mut HashMap<PathBuf, Vec<Violation>>) {
        let Ok(source) = std::fs::read_to_string(path) else { return };
        let violations = patterns::detect_all(path, &source);
        if !violations.is_empty() {
            results.entry(path.to_path_buf()).or_default().extend(violations);
        }
    }

    fn process_file_scopes(
        path: &Path,
        global_scopes: &mut HashMap<String, scope::Scope>,
        path_map: &mut HashMap<String, PathBuf>,
    ) {
        let Some((scopes, p_str)) = Self::parse_file(path) else { return };
        for (name, scope) in scopes {
            let key = format!("{p_str}::{name}");
            global_scopes.insert(key, scope);
            path_map.insert(p_str.clone(), path.to_path_buf());
        }
    }

    fn parse_file(path: &Path) -> Option<(HashMap<String, scope::Scope>, String)> {
        let source = std::fs::read_to_string(path).ok()?;
        let ext = path.extension()?.to_str()?;
        let lang = Lang::from_ext(ext)?;
        if lang != Lang::Rust { return None; }

        let mut parser = Parser::new();
        parser.set_language(lang.grammar()).ok()?;
        let tree = parser.parse(&source, None)?;
        let visitor = visitor::AstVisitor::new(&source, lang);
        let file_scopes = visitor.extract_scopes(tree.root_node());
        Some((file_scopes, path.to_string_lossy().to_string()))
    }

    fn merge_scope_violations(
        scopes: &HashMap<String, scope::Scope>,
        path_map: &HashMap<String, PathBuf>,
        results: &mut HashMap<PathBuf, Vec<Violation>>,
    ) {
        for (full_name, scope) in scopes {
            let path_str = full_name.split("::").next().unwrap_or("");
            let Some(path) = path_map.get(path_str) else { continue };
            let violations = analyze_scope(scope);
            if !violations.is_empty() {
                results.entry(path.clone()).or_default().extend(violations);
            }
        }
    }
}

fn analyze_scope(scope: &scope::Scope) -> Vec<Violation> {
    let mut violations = Vec::new();
    check_cohesion(scope, &mut violations);
    check_encapsulation(scope, &mut violations);
    check_coupling(scope, &mut violations);
    violations
}

fn check_cohesion(scope: &scope::Scope, out: &mut Vec<Violation>) {
    if scope.is_enum() || !scope.has_behavior() { return; }
    if !scope.methods().values().any(|m| !m.field_access.is_empty()) { return; }
    if scope.methods().len() < 4 { return; }

    let lcom4 = metrics::ScopeMetrics::calculate_lcom4(scope);
    if lcom4 > 1 {
        out.push(build_lcom4_violation(scope, lcom4));
    }
}

fn build_lcom4_violation(scope: &scope::Scope, lcom4: usize) -> Violation {
    Violation::with_details(
        scope.row(),
        format!("Class '{}' has low cohesion (LCOM4: {})", scope.name(), lcom4),
        "LCOM4",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![
                format!("Connected components: {lcom4}"),
                format!("Methods: {}", scope.methods().len()),
                "Methods don't share fields or call each other.".into(),
            ],
            suggestion: Some(format!("Split '{}' into {} smaller types.", scope.name(), lcom4)),
        },
    )
}

fn check_encapsulation(scope: &scope::Scope, out: &mut Vec<Violation>) {
    if scope.is_enum() { return; }
    if is_data_struct(scope) { return; }
    if !scope.has_behavior() { return; }
    if scope.fields().is_empty() { return; }

    let ahf = metrics::ScopeMetrics::calculate_ahf(scope);
    if ahf < 60.0 {
        out.push(build_ahf_violation(scope, ahf));
    }
}

fn is_data_struct(scope: &scope::Scope) -> bool {
    scope.derives().contains("Serialize")
        || scope.derives().contains("Deserialize")
        || scope.derives().contains("Parser")
        || scope.derives().contains("Args")
}

fn build_ahf_violation(scope: &scope::Scope, ahf: f64) -> Violation {
    let private_count = scope.fields().values().filter(|f| !f.is_public).count();
    Violation::with_details(
        scope.row(),
        format!("Class '{}' exposes too much state (AHF: {:.1}%)", scope.name(), ahf),
        "AHF",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![
                format!("AHF is {ahf:.1}% (min 60%)"),
                format!("{} of {} fields are private.", private_count, scope.fields().len()),
            ],
            suggestion: Some("Encapsulate fields with accessors.".into()),
        },
    )
}

fn check_coupling(scope: &scope::Scope, out: &mut Vec<Violation>) {
    let cbo = metrics::ScopeMetrics::calculate_cbo(scope);
    let sfout = metrics::ScopeMetrics::calculate_max_sfout(scope);

    if cbo > 9 {
        out.push(build_cbo_violation(scope, cbo));
    }
    if sfout > 7 {
        out.push(build_sfout_violation(scope, sfout));
    }
}

fn build_cbo_violation(scope: &scope::Scope, cbo: usize) -> Violation {
    Violation::with_details(
        scope.row(),
        format!("Class '{}' is tightly coupled (CBO: {})", scope.name(), cbo),
        "CBO",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![format!("External dependencies: {cbo}")],
            suggestion: Some("Reduce dependencies. Consider facade pattern.".into()),
        },
    )
}

fn build_sfout_violation(scope: &scope::Scope, sfout: usize) -> Violation {
    Violation::with_details(
        scope.row(),
        format!("Class '{}' has high fan-out (SFOUT: {})", scope.name(), sfout),
        "SFOUT",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![format!("Max outgoing calls: {sfout}")],
            suggestion: Some("Delegate to helper functions.".into()),
        },
    )
}