// src/stage/tests_manager.rs
use super::*;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

fn setup_repo() -> Result<TempDir> {
    let repo = TempDir::new()?;
    fs::write(repo.path().join("main.rs"), "fn main() {}")?;
    fs::create_dir(repo.path().join("src"))?;
    fs::write(repo.path().join("src/lib.rs"), "// lib")?;
    Ok(repo)
}

#[test]
fn test_new_manager_no_stage() -> Result<()> {
    let repo = setup_repo()?;
    let manager = StageManager::new(repo.path());

    assert!(!manager.exists());
    assert_eq!(manager.effective_cwd(), repo.path());

    Ok(())
}

#[test]
fn test_create_stage() -> Result<()> {
    let repo = setup_repo()?;
    let mut manager = StageManager::new(repo.path());

    let result = manager.ensure_stage()?;
    assert!(result.was_created());
    assert!(manager.exists());

    // Verify files copied
    let worktree = manager.worktree();
    assert!(worktree.join("main.rs").exists());
    assert!(worktree.join("src/lib.rs").exists());

    // Verify .slopchop not copied into stage
    assert!(!worktree.join(".slopchop").exists());

    Ok(())
}

#[test]
fn test_ensure_stage_idempotent() -> Result<()> {
    let repo = setup_repo()?;
    let mut manager = StageManager::new(repo.path());

    let result1 = manager.ensure_stage()?;
    assert!(result1.was_created());

    let result2 = manager.ensure_stage()?;
    assert!(!result2.was_created());

    Ok(())
}

#[test]
fn test_record_and_track_writes() -> Result<()> {
    let repo = setup_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    manager.record_write("src/new.rs", None)?;
    manager.record_write("src/another.rs", Some("hash123".to_string()))?;
    manager.record_delete("src/old.rs", None)?;

    let state = manager.state().expect("State should be loaded");
    assert_eq!(state.paths_to_write().len(), 2);
    assert_eq!(state.paths_to_delete().len(), 1);

    Ok(())
}

#[test]
fn test_effective_cwd_with_stage() -> Result<()> {
    let repo = setup_repo()?;
    let mut manager = StageManager::new(repo.path());

    // Before stage: effective_cwd is repo root
    assert_eq!(manager.effective_cwd(), repo.path());

    manager.ensure_stage()?;

    // After stage: effective_cwd is worktree
    assert_eq!(manager.effective_cwd(), manager.worktree());

    Ok(())
}

#[test]
fn test_reset_stage() -> Result<()> {
    let repo = setup_repo()?;
    let mut manager = StageManager::new(repo.path());

    manager.ensure_stage()?;
    assert!(manager.exists());

    manager.reset()?;
    assert!(!manager.exists());
    assert!(manager.state().is_none());

    Ok(())
}

#[test]
fn test_promote_basic() -> Result<()> {
    let repo = setup_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Write a new file to stage
    let new_file = manager.worktree().join("new.rs");
    fs::write(&new_file, "// new file")?;
    manager.record_write("new.rs", None)?;

    // Promote
    let result = manager.promote(3)?;
    assert_eq!(result.files_written.len(), 1);

    // Verify file in real repo
    assert!(repo.path().join("new.rs").exists());

    Ok(())
}

#[test]
fn test_stage_id_persists() -> Result<()> {
    let repo = setup_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    let id1 = manager
        .stage_id()
        .expect("Stage should have ID")
        .to_string();

    // Create new manager and reload
    let mut manager2 = StageManager::new(repo.path());
    manager2.load_state()?;

    let id2 = manager2
        .stage_id()
        .expect("Stage should have ID")
        .to_string();

    assert_eq!(id1, id2);

    Ok(())
}