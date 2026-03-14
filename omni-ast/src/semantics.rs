use crate::harvester::SemanticFingerprint;
use crate::taxonomy::SemanticBadges;

#[path = "semantics_engine.rs"]
mod engine;
#[path = "semantics_queries.rs"]
mod queries;
#[path = "semantics_logic_queries.rs"]
mod logic_queries;
#[path = "semantics_concurrency_queries.rs"]
mod concurrency_queries;
#[path = "semantics_logic_tables.rs"]
mod logic_tables;
#[path = "semantics_tables.rs"]
mod tables;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticLanguage {
    Rust,
    Go,
    Python,
    JavaScript,
    TypeScript,
    Cpp,
    Swift,
}

impl SemanticLanguage {
    #[must_use]
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "py" => Some(Self::Python),
            "js" | "jsx" | "mjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => Some(Self::Cpp),
            "swift" => Some(Self::Swift),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Concept {
    TestContext,
    HeapAllocation,
    Lookup,
    Length,
    Mutation,
    Locking,
    Loop,
    ExportedApi,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticContext {
    pub fingerprint: SemanticFingerprint,
    pub badges: SemanticBadges,
    pub source_text: String,
    pub path_hint: Option<String>,
}

pub trait LangSemantics {
    fn language(&self) -> SemanticLanguage;

    fn is_test_context(&self, context: &SemanticContext) -> bool;

    fn has_concept(&self, concept: Concept, context: &SemanticContext) -> bool;

    fn has_length_boundary_risk(&self, context: &SemanticContext) -> bool;

    fn has_unguarded_collection_access(&self, context: &SemanticContext) -> bool;

    fn has_unwrapped_front_access(&self, context: &SemanticContext) -> bool;

    fn has_guarding_collection_check(&self, context: &SemanticContext) -> bool;

    fn is_async_locking_context(&self, context: &SemanticContext) -> bool;
}

pub use engine::{semantics_for, SharedSemantics};
