// src/mutate/runner.rs
//! Parallel mutation test runner.
//!
//! Executes tests against mutated code to identify surviving mutants.

use crate::mutate::mutations::{apply_mutation, MutationPoint};
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Result of testing a single mutation.
#[derive(Debug, Clone)]
pub struct MutationResult {
    pub point: MutationPoint,
    pub survived: bool,
    pub duration_ms: u64,
}

/// Configuration for the mutation runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub test_command: String,
    pub test_args: Vec<String>,
    pub timeout_secs: u64,
    pub workers: usize,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            test_command: "cargo".to_string(),
            test_args: vec!["test".to_string(), "--lib".to_string()],
            timeout_secs: 30,
            workers: get_worker_count(),
        }
    }
}

/// Gets a reasonable worker count based on available CPUs.
fn get_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4)
}

impl RunnerConfig {
    /// Creates config for Rust projects.
    #[must_use]
    pub fn rust() -> Self {
        Self::default()
    }

    /// Creates config for TypeScript/Node projects.
    #[must_use]
    pub fn typescript() -> Self {
        Self {
            test_command: "npm".to_string(),
            test_args: vec!["test".to_string()],
            timeout_secs: 60,
            workers: get_worker_count(),
        }
    }

    /// Creates config for Python projects.
    #[must_use]
    pub fn python() -> Self {
        Self {
            test_command: "pytest".to_string(),
            test_args: vec!["-x".to_string(), "-q".to_string()],
            timeout_secs: 60,
            workers: get_worker_count(),
        }
    }
}

/// Runs all mutations and collects results.
///
/// NOTE: Mutations run serially because parallel execution requires
/// separate workspace copies. This is a v1 limitation.
///
/// # Errors
/// Returns error if file operations fail.
pub fn run_mutations(
    points: &[MutationPoint],
    config: &RunnerConfig,
    workdir: &Path,
    on_progress: impl Fn(usize, usize, &MutationResult) + Sync,
) -> Result<Vec<MutationResult>> {
    let total = points.len();
    let mut results = Vec::with_capacity(total);

    // Run serially to avoid race conditions
    // (parallel would require N copies of the workspace)
    for (idx, point) in points.iter().enumerate() {
        let result = test_mutation(point, config, workdir);
        on_progress(idx + 1, total, &result);
        results.push(result);
    }

    Ok(results)
}

/// Tests a single mutation point.
fn test_mutation(point: &MutationPoint, config: &RunnerConfig, workdir: &Path) -> MutationResult {
    let start = Instant::now();

    // Read original file
    let file_path = workdir.join(&point.file);
    let Ok(original) = fs::read_to_string(&file_path) else {
        return MutationResult {
            point: point.clone(),
            survived: false,
            duration_ms: 0,
        };
    };

    // Apply mutation
    let mutated = apply_mutation(&original, point);
    if fs::write(&file_path, &mutated).is_err() {
        let _ = fs::write(&file_path, &original); // Restore
        #[allow(clippy::cast_possible_truncation)]
        return MutationResult {
            point: point.clone(),
            survived: false,
            duration_ms: start.elapsed().as_millis() as u64,
        };
    }

    // Run tests
    let survived = run_test_command(config, workdir);

    // Restore original
    let _ = fs::write(&file_path, &original);

    #[allow(clippy::cast_possible_truncation)]
    MutationResult {
        point: point.clone(),
        survived,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Runs the test command and returns true if tests PASSED (mutant survived).
fn run_test_command(config: &RunnerConfig, workdir: &Path) -> bool {
    let result = Command::new(&config.test_command)
        .args(&config.test_args)
        .current_dir(workdir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match result {
        Ok(status) => status.success(), // Tests passed = mutant survived
        Err(_) => false,                // Command failed = assume caught
    }
}

/// Calculates summary statistics from results.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn summarize(results: &[MutationResult]) -> MutationSummary {
    let total = results.len();
    let survived = results.iter().filter(|r| r.survived).count();
    let killed = total - survived;
    let total_ms: u64 = results.iter().map(|r| r.duration_ms).sum();

    MutationSummary {
        total,
        killed,
        survived,
        score: if total > 0 {
            (killed as f64 / total as f64) * 100.0
        } else {
            100.0
        },
        total_duration_ms: total_ms,
    }
}

/// Summary statistics for a mutation run.
#[derive(Debug, Clone)]
pub struct MutationSummary {
    pub total: usize,
    pub killed: usize,
    pub survived: usize,
    pub score: f64,
    pub total_duration_ms: u64,
}
