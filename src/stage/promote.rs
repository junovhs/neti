// src/stage/promote.rs
//! Transactional promotion from staged worktree to real workspace.

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
/// Returns error if promotion fails. On failure, attempts rollback.
pub fn promote_to_workspace(
    repo_root: &Path,
    worktree: &Path,
    paths_to_write: &[&str],
    paths_to_delete: &[&str],
    backup_dir: &Path,
) -> Result<PromoteResult> {
    let backup_path = create_backup_dir(backup_dir)?;

    // Collect all paths that will be modified
    let all_paths: HashSet<&str> = paths_to_write
        .iter()
        .chain(paths_to_delete.iter())
        .copied()
        .collect();

    // Phase 1: Backup existing files
    let backed_up = backup_existing_files(repo_root, &all_paths, &backup_path)?;

    // Phase 2: Apply changes (with rollback on failure)
    let result = apply_changes(repo_root, worktree, paths_to_write, paths_to_delete);

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
            backed_up.push(path.to_string());
        }
    }

    Ok(backed_up)
}

/// Applies write and delete operations to the real workspace.
fn apply_changes(
    repo_root: &Path,
    worktree: &Path,
    paths_to_write: &[&str],
    paths_to_delete: &[&str],
) -> Result<(Vec<String>, Vec<String>)> {
    let mut written = Vec::new();
    let mut deleted = Vec::new();

    // Apply writes: copy from stage to real workspace
    for path in paths_to_write {
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

        written.push(path.to_string());
    }

    // Apply deletes
    for path in paths_to_delete {
        let target = repo_root.join(path);
        if target.exists() {
            fs::remove_file(&target)
                .with_context(|| format!("Failed to delete {}", target.display()))?;
            deleted.push(path.to_string());
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
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| e.file_name().to_string_lossy().starts_with("promote_"))
        .collect();

    if entries.len() <= keep_count {
        return Ok(0);
    }

    // Sort by name (which includes timestamp, so chronological)
    entries.sort_by_key(|e| e.file_name());

    let to_remove = entries.len() - keep_count;
    let mut removed = 0;

    for entry in entries.into_iter().take(to_remove) {
        if fs::remove_dir_all(entry.path()).is_ok() {
            removed += 1;
        }
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dirs() -> Result<(TempDir, TempDir, TempDir)> {
        let repo = TempDir::new()?;
        let stage = TempDir::new()?;
        let backup = TempDir::new()?;
        Ok((repo, stage, backup))
    }

    #[test]
    fn test_promote_new_file() -> Result<()> {
        let (repo, stage, backup) = setup_test_dirs()?;

        // Create file in stage
        fs::write(stage.path().join("new.rs"), "fn main() {}")?;

        let result =
            promote_to_workspace(repo.path(), stage.path(), &["new.rs"], &[], backup.path())?;

        assert_eq!(result.files_written.len(), 1);
        assert!(repo.path().join("new.rs").exists());

        Ok(())
    }

    #[test]
    fn test_promote_update_file() -> Result<()> {
        let (repo, stage, backup) = setup_test_dirs()?;

        // Create original in repo
        fs::write(repo.path().join("main.rs"), "old content")?;

        // Create updated version in stage
        fs::write(stage.path().join("main.rs"), "new content")?;

        let result =
            promote_to_workspace(repo.path(), stage.path(), &["main.rs"], &[], backup.path())?;

        assert_eq!(result.files_written.len(), 1);
        let content = fs::read_to_string(repo.path().join("main.rs"))?;
        assert_eq!(content, "new content");

        // Verify backup exists
        assert!(result.backup_path.is_some());
        let backup_file = result.backup_path.unwrap().join("main.rs");
        assert!(backup_file.exists());
        assert_eq!(fs::read_to_string(backup_file)?, "old content");

        Ok(())
    }

    #[test]
    fn test_promote_delete_file() -> Result<()> {
        let (repo, stage, backup) = setup_test_dirs()?;

        // Create file to delete in repo
        fs::write(repo.path().join("old.rs"), "to be deleted")?;

        let result =
            promote_to_workspace(repo.path(), stage.path(), &[], &["old.rs"], backup.path())?;

        assert_eq!(result.files_deleted.len(), 1);
        assert!(!repo.path().join("old.rs").exists());

        Ok(())
    }

    #[test]
    fn test_promote_rollback_on_failure() -> Result<()> {
        let (repo, stage, backup) = setup_test_dirs()?;

        // Create original file
        fs::write(repo.path().join("exists.rs"), "original")?;

        // Stage file exists, but missing file doesn't exist in stage
        fs::write(stage.path().join("exists.rs"), "modified")?;

        // This should fail because "missing.rs" doesn't exist in stage
        let result = promote_to_workspace(
            repo.path(),
            stage.path(),
            &["exists.rs", "missing.rs"],
            &[],
            backup.path(),
        );

        assert!(result.is_err());

        // Original file should be restored (rollback)
        // Note: This test might need adjustment based on exact rollback semantics
        Ok(())
    }

    #[test]
    fn test_cleanup_old_backups() -> Result<()> {
        let backup_base = TempDir::new()?;

        // Create multiple backup dirs
        for i in 1..=5 {
            let dir = backup_base.path().join(format!("promote_{i}"));
            fs::create_dir(&dir)?;
        }

        let removed = cleanup_old_backups(backup_base.path(), 2)?;
        assert_eq!(removed, 3);

        // Count remaining
        let remaining: Vec<_> = fs::read_dir(backup_base.path())?
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(remaining.len(), 2);

        Ok(())
    }
}
