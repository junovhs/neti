// src/stage/tests_promote.rs
use super::promote::*;
use super::state::{TouchedPath, TouchKind};
use anyhow::Result;
use std::fs;
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

    let t1 = TouchedPath {
        path: "new.rs".to_string(),
        kind: TouchKind::Write,
        base_hash: None,
        timestamp: 0,
    };

    let result =
        promote_to_workspace(repo.path(), stage.path(), &[&t1], &[], backup.path())?;

    assert_eq!(result.files_written.len(), 1);
    assert!(repo.path().join("new.rs").exists());

    Ok(())
}

#[test]
fn test_promote_update_file() -> Result<()> {
    let (repo, stage, backup) = setup_test_dirs()?;

    // Create original in repo
    let old_content = "old content";
    fs::write(repo.path().join("main.rs"), old_content)?;
    let old_hash = crate::apply::patch::common::compute_sha256(old_content);

    // Create updated version in stage
    fs::write(stage.path().join("main.rs"), "new content")?;

    let t1 = TouchedPath {
        path: "main.rs".to_string(),
        kind: TouchKind::Write,
        base_hash: Some(old_hash),
        timestamp: 0,
    };

    let result =
        promote_to_workspace(repo.path(), stage.path(), &[&t1], &[], backup.path())?;

    assert_eq!(result.files_written.len(), 1);
    let content = fs::read_to_string(repo.path().join("main.rs"))?;
    assert_eq!(content, "new content");

    Ok(())
}

#[test]
fn test_promote_split_brain_prevention() -> Result<()> {
    let (repo, stage, backup) = setup_test_dirs()?;

    // Create original in repo
    let old_content = "old content";
    fs::write(repo.path().join("main.rs"), old_content)?;
    let old_hash = crate::apply::patch::common::compute_sha256(old_content);

    // AI stages a change
    fs::write(stage.path().join("main.rs"), "ai content")?;
    let t1 = TouchedPath {
        path: "main.rs".to_string(),
        kind: TouchKind::Write,
        base_hash: Some(old_hash),
        timestamp: 0,
    };

    // USER manually modifies the file BEFORE promote
    fs::write(repo.path().join("main.rs"), "user manual modification")?;

    // Promotion should fail
    let result = promote_to_workspace(repo.path(), stage.path(), &[&t1], &[], backup.path());

    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert!(err.contains("Split-brain detected"));

    // Content should still be the user's manual modification
    let content = fs::read_to_string(repo.path().join("main.rs"))?;
    assert_eq!(content, "user manual modification");

    Ok(())
}

#[test]
fn test_promote_delete_file() -> Result<()> {
    let (repo, stage, backup) = setup_test_dirs()?;

    // Create file to delete in repo
    let content = "to be deleted";
    fs::write(repo.path().join("old.rs"), content)?;
    let hash = crate::apply::patch::common::compute_sha256(content);

    let t1 = TouchedPath {
        path: "old.rs".to_string(),
        kind: TouchKind::Delete,
        base_hash: Some(hash),
        timestamp: 0,
    };

    let result =
        promote_to_workspace(repo.path(), stage.path(), &[], &[&t1], backup.path())?;

    assert_eq!(result.files_deleted.len(), 1);
    assert!(!repo.path().join("old.rs").exists());

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
        .filter_map(std::result::Result::ok)
        .collect();
    assert_eq!(remaining.len(), 2);

    Ok(())
}