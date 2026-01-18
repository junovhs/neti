// src/analysis/logic.rs
use crate::config::Config;
use crate::types::{FileReport, Violation, ScanReport};
use crate::analysis::v2;
use crate::analysis::file_analysis;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};

#[must_use]
pub fn run_scan<F>(config: &Config, files: &[PathBuf], on_progress: Option<&F>) -> ScanReport
where
    F: Fn(&Path) + Sync,
{
    let start = std::time::Instant::now();
    
    let mut results: Vec<FileReport> = files
        .par_iter()
        .inspect(|path| {
            if let Some(cb) = on_progress {
                cb(path);
            }
        })
        .map(|path| file_analysis::analyze_file(path, config))
        .collect();

    let v2_engine = v2::ScanEngineV2::new(config.clone());
    let deep_violations = v2_engine.run(files);

    merge_violations(&mut results, &deep_violations);

    ScanReport {
        total_violations: results.iter().map(|r| r.violations.len()).sum(),
        total_tokens: results.iter().map(|r| r.token_count).sum(),
        files: results,
        duration_ms: start.elapsed().as_millis(),
    }
}

fn merge_violations(results: &mut [FileReport], deep: &std::collections::HashMap<PathBuf, Vec<Violation>>) {
    for r in results {
        if let Some(v) = deep.get(&r.path) {
            r.violations.extend(v.clone());
        }
    }
}