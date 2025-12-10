// src/roadmap_v2/cli/verify.rs
//! Test verification and ID-to-test mapping for roadmap audit.

use std::path::Path;
use std::process::Command;

/// Result of test verification.
#[derive(Debug, Clone, Copy)]
pub enum VerifyResult {
    Pass,
    NotFound,
    NoAnchor,
    ExecFailed,
    Skipped,
}

/// Verifies a task's test anchor exists (and optionally passes).
/// If no anchor provided, tries to infer from task ID.
pub fn check_test(task_id: &str, test: Option<&str>, root: &Path, exec: bool) -> VerifyResult {
    // Explicit test anchor takes priority
    if let Some(test_path) = test {
        return check_explicit_test(test_path, root, exec);
    }

    // Try to infer test from task ID
    if let Some(found) = find_test_for_id(task_id, root) {
        return check_explicit_test(&found, root, exec);
    }

    VerifyResult::NoAnchor
}

fn check_explicit_test(test_path: &str, root: &Path, exec: bool) -> VerifyResult {
    if test_path == "[no-test]" {
        return VerifyResult::Skipped;
    }

    if !test_exists(root, test_path) {
        return VerifyResult::NotFound;
    }

    if exec && !test_passes(test_path) {
        return VerifyResult::ExecFailed;
    }

    VerifyResult::Pass
}

/// Checks if a test function exists in the codebase.
fn test_exists(root: &Path, test_path: &str) -> bool {
    let parts: Vec<&str> = test_path.split("::").collect();
    let file_part = parts.first().unwrap_or(&"");
    let file_path = root.join(file_part);

    if !file_path.exists() {
        return false;
    }

    if parts.len() <= 1 {
        return true;
    }

    let fn_name = parts.last().unwrap_or(&"");
    let Ok(content) = std::fs::read_to_string(&file_path) else {
        return false;
    };

    content.contains(&format!("fn {fn_name}"))
}

/// Actually runs the test and checks if it passes.
fn test_passes(test_path: &str) -> bool {
    let test_name = test_path.split("::").last().unwrap_or(test_path);

    let output = Command::new("cargo")
        .args(["test", test_name, "--", "--exact", "--nocapture"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Infers a test function name from a task ID.
/// Convention: `my-task-id`  `test_my_task_id`
fn infer_test_from_id(task_id: &str) -> String {
    let normalized = task_id.replace('-', "_");
    format!("test_{normalized}")
}

/// Searches test files for a function matching the inferred name.
fn find_test_for_id(task_id: &str, root: &Path) -> Option<String> {
    let fn_name = infer_test_from_id(task_id);
    let tests_dir = root.join("tests");

    if !tests_dir.exists() {
        return None;
    }

    search_dir_for_fn(&tests_dir, &fn_name)
}

fn search_dir_for_fn(dir: &Path, fn_name: &str) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
            if let Some(found) = check_file_for_fn(&path, fn_name) {
                return Some(found);
            }
        }
    }

    None
}

fn check_file_for_fn(path: &Path, fn_name: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let pattern = format!("fn {fn_name}");

    if content.contains(&pattern) {
        let rel_path = path.to_string_lossy();
        return Some(format!("{rel_path}::{fn_name}"));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_from_id() {
        assert_eq!(infer_test_from_id("my-task"), "test_my_task");
        assert_eq!(infer_test_from_id("foo-bar-baz"), "test_foo_bar_baz");
    }
}