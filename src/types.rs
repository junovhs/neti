use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Violation {
    pub row: usize,
    pub message: String,
    pub law: &'static str,
}

#[derive(Debug, Clone)]
pub struct FileReport {
    pub path: PathBuf,
    pub token_count: usize,
    pub complexity_score: usize,
    pub violations: Vec<Violation>,
}

impl FileReport {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScanReport {
    pub files: Vec<FileReport>,
    pub total_tokens: usize,
    pub total_violations: usize,
    pub duration_ms: u128,
}
