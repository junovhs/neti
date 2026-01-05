// src/apply/advisory.rs
use std::path::Path;
use colored::Colorize;

/// Threshold for triggering the high edit volume advisory.
const NAG_THRESHOLD: usize = 3;

/// Prints an advisory if many files have been modified in the current stage.
pub fn maybe_print_edit_advisory(repo_root: &Path) {
    if !crate::stage::stage_exists(repo_root) {
        return;
    }

    let worktree = crate::stage::worktree_path(repo_root);
    let modified_count = count_staged_changes(repo_root, &worktree) + 
                        count_staged_deletions(repo_root, &worktree);

    if modified_count > NAG_THRESHOLD {
        println!();
        println!("{}", "━".repeat(60).yellow());
        println!("{}", "[ADVISORY] High Edit Volume Detected".yellow().bold());
        println!("  {modified_count} files modified in this stage session.");
        println!("  Consider syncing soon to maintain high-integrity checkpoints.");
        println!("  Run: {} to promote changes.", "slopchop apply --sync".cyan());
        println!("{}", "━".repeat(60).yellow());
    }
}

/// Counts new or modified files in the stage compared to root.
fn count_staged_changes(repo_root: &Path, worktree: &Path) -> usize {
    use crate::stage;
    use walkdir::WalkDir;

    let mut count = 0;
    for entry in WalkDir::new(worktree).min_depth(1) {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }

        let Ok(rel_path) = entry.path().strip_prefix(worktree) else { continue };
        if stage::should_preserve(rel_path) {
            continue;
        }

        let root_file = repo_root.join(rel_path);
        if !root_file.exists() || is_modified(&entry, &root_file) {
            count += 1;
        }
    }
    count
}

/// Counts deleted files (exist in root but not in stage).
fn count_staged_deletions(repo_root: &Path, worktree: &Path) -> usize {
    use crate::stage;
    use walkdir::WalkDir;

    let mut count = 0;
    for entry in WalkDir::new(repo_root).min_depth(1) {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }

        let Ok(rel_path) = entry.path().strip_prefix(repo_root) else { continue };
        if stage::should_preserve(rel_path) {
            continue;
        }

        if !worktree.join(rel_path).exists() {
            count += 1;
        }
    }
    count
}

/// Heuristic check if two files differ based on metadata.
fn is_modified(stage_entry: &walkdir::DirEntry, root_file: &Path) -> bool {
    let Ok(stage_meta) = stage_entry.metadata() else { return false };
    let Ok(root_meta) = root_file.metadata() else { return false };
    stage_meta.len() != root_meta.len()
}
