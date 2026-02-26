use crate::types::{Confidence, ScanReport, Violation};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

pub(crate) fn collect_violations(report: &ScanReport) -> Vec<(&Path, &Violation)> {
    let mut all: Vec<(&Path, &Violation)> = Vec::new();
    for file in &report.files {
        for v in &file.violations {
            all.push((&file.path, v));
        }
    }
    all
}

pub(crate) fn rule_counts(all: &[(&Path, &Violation)]) -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for (_, v) in all {
        *counts.entry(v.law).or_insert(0) += 1;
    }
    counts
}

pub(crate) fn next_occurrence(
    shown: &mut HashMap<&'static str, usize>,
    law: &'static str,
) -> usize {
    let entry = shown.entry(law).or_insert(0);
    *entry += 1;
    *entry
}

pub(crate) fn confidence_suffix(v: &Violation) -> String {
    match v.confidence {
        Confidence::Medium => {
            let reason = v
                .confidence_reason
                .as_deref()
                .unwrap_or("pattern match without proof");
            format!("{} — {reason}", v.confidence.label())
        }
        Confidence::High | Confidence::Info => v.confidence.label().to_string(),
    }
}

pub(crate) fn pluralize(word: &str, count: usize) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

pub(crate) fn duration(report: &ScanReport) -> Duration {
    let ms = u64::try_from(report.duration_ms).unwrap_or(u64::MAX);
    Duration::from_millis(ms)
}
