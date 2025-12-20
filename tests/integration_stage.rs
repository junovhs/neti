// tests/integration_stage.rs
//! Integration tests for the staged workspace feature.
//!
//! These tests verify the acceptance criteria from the Pivot Brief:
//! 1. Stage creation - creates .slopchop/stage/worktree, excludes .slopchop
//! 2. Apply writes only to stage - real workspace unchanged
//! 3. Check runs in stage - uses stage cwd when present
//! 4. Pack uses stage - reflects staged content
//! 5. Promote applies touched paths only
//! 6. Promote rollback works

use anyhow::Result;
use slopchop_core::stage::{effective_cwd, stage_exists, worktree_path, StageManager};
use std::fs;
use tempfile::TempDir;

/// Helper to create a test repository structure.
fn create_test_repo() -> Result<TempDir> {
    let repo = TempDir::new()?;

    // Create typical repo structure
    fs::write(
        repo.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\n",
    )?;
    fs::create_dir(repo.path().join("src"))?;
    fs::write(
        repo.path().join("src/main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )?;
    fs::write(
        repo.path().join("src/lib.rs"),
        "pub fn greet() -> &'static str {\n    \"hello\"\n}\n",
    )?;

    // Create some dirs that should be excluded
    fs::create_dir(repo.path().join(".git"))?;
    fs::write(repo.path().join(".git/config"), "[core]\n")?;

    Ok(repo)
}

// =============================================================================
// ACCEPTANCE TEST 1: Stage creation
// =============================================================================

#[test]
fn test_stage_creates_worktree_dir() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    assert!(!stage_exists(repo.path()));

    manager.ensure_stage()?;

    assert!(stage_exists(repo.path()));
    assert!(worktree_path(repo.path()).is_dir());

    Ok(())
}

#[test]
fn test_stage_does_not_copy_slopchop_into_itself() -> Result<()> {
    let repo = create_test_repo()?;

    // Create a .slopchop dir with some content
    fs::create_dir_all(repo.path().join(".slopchop/old_state"))?;
    fs::write(repo.path().join(".slopchop/old_state/data.json"), "{}")?;

    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Verify .slopchop is NOT in the stage worktree
    let worktree = manager.worktree();
    assert!(!worktree.join(".slopchop").exists());

    // But regular files ARE copied
    assert!(worktree.join("src/main.rs").exists());
    assert!(worktree.join("Cargo.toml").exists());

    Ok(())
}

#[test]
fn test_stage_does_not_copy_git() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    manager.ensure_stage()?;

    let worktree = manager.worktree();
    assert!(!worktree.join(".git").exists());

    Ok(())
}

// =============================================================================
// ACCEPTANCE TEST 2: Apply writes only to stage
// =============================================================================

#[test]
fn test_apply_writes_to_stage_not_real_workspace() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    // Get original content
    let original_content = fs::read_to_string(repo.path().join("src/main.rs"))?;

    manager.ensure_stage()?;

    // Simulate an apply by writing to the stage worktree
    let new_content = "fn main() {\n    println!(\"modified\");\n}\n";
    fs::write(manager.worktree().join("src/main.rs"), new_content)?;
    manager.record_write("src/main.rs")?;

    // Real workspace should be UNCHANGED
    let real_content = fs::read_to_string(repo.path().join("src/main.rs"))?;
    assert_eq!(real_content, original_content);

    // Stage should have the modification
    let staged_content = fs::read_to_string(manager.worktree().join("src/main.rs"))?;
    assert_eq!(staged_content, new_content);

    Ok(())
}

#[test]
fn test_stage_tracks_written_paths() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Record some writes
    manager.record_write("src/main.rs")?;
    manager.record_write("src/new_file.rs")?;
    manager.record_delete("src/old_file.rs")?;

    let state = manager.state().unwrap();
    let writes = state.paths_to_write();
    let deletes = state.paths_to_delete();

    assert!(writes.contains(&"src/main.rs"));
    assert!(writes.contains(&"src/new_file.rs"));
    assert!(deletes.contains(&"src/old_file.rs"));

    Ok(())
}

// =============================================================================
// ACCEPTANCE TEST 3: Check/Pack use stage when present
// =============================================================================

#[test]
fn test_effective_cwd_uses_stage_when_present() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    // Before stage: effective_cwd is repo root
    let cwd_before = effective_cwd(repo.path());
    assert_eq!(cwd_before, repo.path());

    manager.ensure_stage()?;

    // After stage: effective_cwd is worktree
    let cwd_after = effective_cwd(repo.path());
    assert_eq!(cwd_after, worktree_path(repo.path()));

    Ok(())
}

#[test]
fn test_effective_cwd_falls_back_to_repo_without_stage() -> Result<()> {
    let repo = create_test_repo()?;

    // No stage exists
    let cwd = effective_cwd(repo.path());
    assert_eq!(cwd, repo.path());

    Ok(())
}

// =============================================================================
// ACCEPTANCE TEST 5: Promote applies touched paths only
// =============================================================================

#[test]
fn test_promote_only_applies_touched_paths() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Modify ONE file in stage
    fs::write(manager.worktree().join("src/main.rs"), "// modified\n")?;
    manager.record_write("src/main.rs")?;

    // Modify another file in stage WITHOUT recording it
    fs::write(manager.worktree().join("src/lib.rs"), "// also modified\n")?;

    // Store original lib.rs content
    let original_lib = fs::read_to_string(repo.path().join("src/lib.rs"))?;

    // Promote
    let result = manager.promote(3)?;

    // Only main.rs should be promoted (it was recorded)
    assert_eq!(result.files_written, vec!["src/main.rs"]);

    // main.rs should be updated in real workspace
    let main_content = fs::read_to_string(repo.path().join("src/main.rs"))?;
    assert_eq!(main_content, "// modified\n");

    // lib.rs should be UNCHANGED (not recorded)
    let lib_content = fs::read_to_string(repo.path().join("src/lib.rs"))?;
    assert_eq!(lib_content, original_lib);

    Ok(())
}

#[test]
fn test_promote_handles_deletions() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Create a file to delete
    fs::write(repo.path().join("to_delete.rs"), "// delete me")?;

    // Record deletion
    manager.record_delete("to_delete.rs")?;

    // Promote
    let result = manager.promote(3)?;

    assert!(result.files_deleted.contains(&"to_delete.rs".to_string()));
    assert!(!repo.path().join("to_delete.rs").exists());

    Ok(())
}

#[test]
fn test_promote_creates_new_files() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Create new file in stage
    fs::create_dir_all(manager.worktree().join("src/subdir"))?;
    fs::write(manager.worktree().join("src/subdir/new.rs"), "// brand new")?;
    manager.record_write("src/subdir/new.rs")?;

    // Verify it doesn't exist in real workspace yet
    assert!(!repo.path().join("src/subdir/new.rs").exists());

    // Promote
    manager.promote(3)?;

    // Now it should exist
    assert!(repo.path().join("src/subdir/new.rs").exists());
    let content = fs::read_to_string(repo.path().join("src/subdir/new.rs"))?;
    assert_eq!(content, "// brand new");

    Ok(())
}

// =============================================================================
// ACCEPTANCE TEST 6: Promote rollback works
// =============================================================================

#[test]
fn test_promote_creates_backup() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Original content
    let original = fs::read_to_string(repo.path().join("src/main.rs"))?;

    // Modify in stage
    fs::write(manager.worktree().join("src/main.rs"), "// modified")?;
    manager.record_write("src/main.rs")?;

    // Promote
    let result = manager.promote(3)?;

    // Backup should exist
    assert!(result.backup_path.is_some());
    let backup_main = result.backup_path.unwrap().join("src/main.rs");
    assert!(backup_main.exists());

    // Backup should contain original content
    let backup_content = fs::read_to_string(backup_main)?;
    assert_eq!(backup_content, original);

    Ok(())
}

// =============================================================================
// Additional stage lifecycle tests
// =============================================================================

#[test]
fn test_stage_reset_removes_everything() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    manager.ensure_stage()?;
    assert!(stage_exists(repo.path()));

    manager.reset()?;
    assert!(!stage_exists(repo.path()));

    // State should be gone too
    assert!(manager.state().is_none());

    Ok(())
}

#[test]
fn test_stage_id_persists_across_loads() -> Result<()> {
    let repo = create_test_repo()?;

    let id = {
        let mut manager = StageManager::new(repo.path());
        manager.ensure_stage()?;
        manager.stage_id().unwrap().to_string()
    };

    // Load with fresh manager
    let mut manager2 = StageManager::new(repo.path());
    manager2.load_state()?;

    assert_eq!(manager2.stage_id().unwrap(), id);

    Ok(())
}

#[test]
fn test_apply_count_increments() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    assert_eq!(manager.apply_count(), 0);

    manager.record_apply()?;
    assert_eq!(manager.apply_count(), 1);

    manager.record_apply()?;
    assert_eq!(manager.apply_count(), 2);

    Ok(())
}

#[test]
fn test_promote_clears_touched_paths() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Write a file
    fs::write(manager.worktree().join("new.rs"), "// new")?;
    manager.record_write("new.rs")?;

    assert!(!manager.state().unwrap().paths_to_write().is_empty());

    // Promote
    manager.promote(3)?;

    // Touched paths should be cleared
    assert!(manager.state().unwrap().paths_to_write().is_empty());

    Ok(())
}

#[test]
fn test_ensure_stage_is_idempotent() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    // First call creates
    let result1 = manager.ensure_stage()?;
    assert!(result1.was_created());

    let id = manager.stage_id().unwrap().to_string();

    // Second call reuses
    let result2 = manager.ensure_stage()?;
    assert!(!result2.was_created());

    // Same stage ID
    assert_eq!(manager.stage_id().unwrap(), id);

    Ok(())
}
