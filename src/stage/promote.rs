// src/stage/promote.rs
//! Transactional promotion from staged worktree to real workspace.

use super::state::TouchedPath;

use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Result of a promotion operation.
#[derive(Debug)]
pub struct PromoteResult {
    pub files_written: Vec<String>,
    pub files_deleted: Vec<String>,
    pub backup_path: Option<PathBuf>,
}

/// Promotes touched paths from stage to real workspace.
///
/// # Errors
/// Returns error if promotion fails or hash mismatch is detected.
pub fn promote_to_workspace(
    repo_root: &Path,
    worktree: &Path,
    to_write: &[&TouchedPath],
    to_delete: &[&TouchedPath],
    backup_dir: &Path,
) -> Result<PromoteResult> {
    // Phase 0: Pre-flight hash verification (Split-Brain Prevention)
    verify_workspace_integrity(repo_root, to_write, to_delete)?;

    let backup_path = create_backup_dir(backup_dir)?;

    // Collect all paths that will be modified
    let all_paths: HashSet<&str> = to_write
        .iter()
        .chain(to_delete.iter())
        .map(|t| t.path.as_str())
        .collect();

    // Phase 1: Backup existing files
    let backed_up = backup_existing_files(repo_root, &all_paths, &backup_path)?;

    // Phase 2: Apply changes (with rollback on failure)
    let result = apply_changes(repo_root, worktree, to_write, to_delete);

    match result {
        Ok((written, deleted)) => Ok(PromoteResult {
            files_written: written,
            files_deleted: deleted,
            backup_path: Some(backup_path),
        }),
        Err(e) => {
            // Rollback: restore backed up files
            if let Err(rollback_err) = rollback_changes(repo_root, &backed_up, &backup_path) {
                return Err(anyhow!(
                    "Promotion failed: {e}. Rollback also failed: {rollback_err}"
                ));
            }
            Err(anyhow!("Promotion failed (rolled back): {e}"))
        }
    }
}

/// Verifies that workspace files haven't changed since they were staged.
fn verify_workspace_integrity(
    repo_root: &Path,
    to_write: &[&TouchedPath],
    to_delete: &[&TouchedPath],
) -> Result<()> {
    for touched in to_write.iter().chain(to_delete.iter()) {
        if let Some(expected_hash) = &touched.base_hash {
            let path = repo_root.join(&touched.path);
            if !path.exists() {
                return Err(anyhow!(
                    "Split-brain detected: {} was expected to exist but is missing.",
                    touched.path
                ));
            }
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {} for integrity check", touched.path))?;
            let actual_hash = crate::apply::patch::common::compute_sha256(&content);
            
            if actual_hash != *expected_hash {
                return Err(anyhow!(
                    "Split-brain detected: {} has been modified manually since it was staged. \
                     Aborting promotion to prevent overwriting your changes.",
                    touched.path
                ));
            }
        }
    }
    Ok(())
}

/// Creates a timestamped backup directory.
fn create_backup_dir(base: &Path) -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let backup_path = base.join(format!("promote_{timestamp}"));
    fs::create_dir_all(&backup_path)
        .with_context(|| format!("Failed to create backup dir: {}", backup_path.display()))?;

    Ok(backup_path)
}

/// Backs up existing files that will be modified.
fn backup_existing_files(
    repo_root: &Path,
    paths: &HashSet<&str>,
    backup_dir: &Path,
) -> Result<Vec<String>> {
    let mut backed_up = Vec::new();

    for path in paths {
        let src = repo_root.join(path);
        if src.exists() && src.is_file() {
            let dest = backup_dir.join(path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dest).with_context(|| format!("Failed to backup {}", src.display()))?;
            backed_up.push((*path).to_string());
        }
    }

    Ok(backed_up)
}

/// Applies write and delete operations to the real workspace.
fn apply_changes(
    repo_root: &Path,
    worktree: &Path,
    to_write: &[&TouchedPath],
    to_delete: &[&TouchedPath],
) -> Result<(Vec<String>, Vec<String>)> {
    let mut written = Vec::new();
    let mut deleted = Vec::new();

    // Apply writes: copy from stage to real workspace
    for touched in to_write {
        let path = &touched.path;
        let src = worktree.join(path);
        let dest = repo_root.join(path);

        if !src.exists() {
            return Err(anyhow!("Stage file missing: {}", src.display()));
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create dir for {}", dest.display()))?;
        }

        fs::copy(&src, &dest)
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;

        written.push(path.clone());
    }

    // Apply deletes
    for touched in to_delete {
        let path = &touched.path;
        let target = repo_root.join(path);
        if target.exists() {
            fs::remove_file(&target)
                .with_context(|| format!("Failed to delete {}", target.display()))?;
            deleted.push(path.clone());
        }
    }

    Ok((written, deleted))
}

/// Rolls back changes by restoring backed up files.
fn rollback_changes(repo_root: &Path, backed_up: &[String], backup_dir: &Path) -> Result<()> {
    for path in backed_up {
        let src = backup_dir.join(path);
        let dest = repo_root.join(path);

        if src.exists() {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dest)
                .with_context(|| format!("Rollback: Failed to restore {}", dest.display()))?;
        }
    }

    Ok(())
}

/// Cleans up old backup directories, keeping only the most recent N.
///
/// # Errors
/// Returns error if cleanup fails.
pub fn cleanup_old_backups(backup_base: &Path, keep_count: usize) -> Result<usize> {
    if !backup_base.exists() {
        return Ok(0);
    }

    let mut entries: Vec<_> = fs::read_dir(backup_base)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter(|e| e.file_name().to_string_lossy().starts_with("promote_"))
        .collect();

    if entries.len() <= keep_count {
        return Ok(0);
    }

    // Sort by name (which includes timestamp, so chronological)
    entries.sort_by_key(std::fs::DirEntry::file_name);

    let to_remove = entries.len() - keep_count;
    let mut removed = 0;

    for entry in entries.into_iter().take(to_remove) {
        if fs::remove_dir_all(entry.path()).is_ok() {
            removed += 1;
        }
    }

    Ok(removed)
}
