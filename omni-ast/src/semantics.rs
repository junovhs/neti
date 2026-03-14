use crate::harvester::SemanticFingerprint;
use crate::taxonomy::SemanticBadges;

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
}

pub trait LangSemantics {
    fn language(&self) -> SemanticLanguage;

    fn is_test_context(&self, context: &SemanticContext) -> bool;

    fn has_concept(&self, concept: Concept, context: &SemanticContext) -> bool;
}
