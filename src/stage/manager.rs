// src/stage/manager.rs
//! Stage Manager - orchestrates all staged workspace operations.

use super::copy::{copy_repo_to_stage, remove_stage, CopyStats};
use super::promote::{cleanup_old_backups, promote_to_workspace, PromoteResult};
use super::state::StageState;
use super::{backups_path, stage_path, state_file_path, worktree_path};
use crate::events::{EventKind, EventLogger};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Manages the staged workspace lifecycle.
pub struct StageManager {
    repo_root: PathBuf,
    state: Option<StageState>,
}

impl StageManager {
    /// Creates a new stage manager for the given repository root.
    #[must_use]
    pub fn new(repo_root: &Path) -> Self {
        Self {
            repo_root: repo_root.to_path_buf(),
            state: None,
        }
    }

    /// Returns the repository root path.
    #[must_use]
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Returns the stage directory path.
    #[must_use]
    pub fn stage_dir(&self) -> PathBuf {
        stage_path(&self.repo_root)
    }

    /// Returns the staged worktree path.
    #[must_use]
    pub fn worktree(&self) -> PathBuf {
        worktree_path(&self.repo_root)
    }

    /// Returns true if a stage currently exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.worktree().is_dir()
    }

    /// Returns the effective working directory for operations.
    /// If a stage exists, returns the worktree. Otherwise, returns repo root.
    #[must_use]
    pub fn effective_cwd(&self) -> PathBuf {
        if self.exists() {
            self.worktree()
        } else {
            self.repo_root.clone()
        }
    }

    /// Loads existing state or returns None if no stage exists.
    ///
    /// # Errors
    /// Returns error if state file exists but cannot be read.
    pub fn load_state(&mut self) -> Result<Option<&StageState>> {
        let state_path = state_file_path(&self.repo_root);

        if !state_path.exists() {
            self.state = None;
            return Ok(None);
        }

        let state = StageState::load(&state_path)?;
        self.state = Some(state);
        Ok(self.state.as_ref())
    }

    /// Returns the current state, if loaded.
    #[must_use]
    pub fn state(&self) -> Option<&StageState> {
        self.state.as_ref()
    }

    /// Returns a mutable reference to the current state.
    #[must_use]
    pub fn state_mut(&mut self) -> Option<&mut StageState> {
        self.state.as_mut()
    }

    /// Ensures a stage exists, creating one if necessary.
    ///
    /// # Errors
    /// Returns error if stage creation fails.
    pub fn ensure_stage(&mut self) -> Result<EnsureResult> {
        if self.exists() {
            self.load_state()?;
            return Ok(EnsureResult::Existed);
        }

        self.create_stage()
    }

    /// Creates a new stage by copying the repo to the worktree.
    ///
    /// # Errors
    /// Returns error if copy fails.
    pub fn create_stage(&mut self) -> Result<EnsureResult> {
        let worktree = self.worktree();

        // Remove any partial stage
        if worktree.exists() {
            remove_stage(&worktree)?;
        }

        // Copy repo to stage
        let copy_stats = copy_repo_to_stage(&self.repo_root, &worktree)?;

        // Initialize state
        let new_state = StageState::new();
        new_state.save(&state_file_path(&self.repo_root))?;

        // Log event
        let logger = EventLogger::new(&self.repo_root);
        logger.log(EventKind::StageCreated { id: new_state.id.clone() });

        self.state = Some(new_state);

        Ok(EnsureResult::Created(copy_stats))
    }

    /// Records a file write operation in the stage state.
    ///
    /// # Errors
    /// Returns error if state cannot be saved.
    pub fn record_write(&mut self, path: &str) -> Result<()> {
        self.ensure_state_loaded()?;

        if let Some(state) = &mut self.state {
            state.record_write(path);
            state.save(&state_file_path(&self.repo_root))?;
        }

        Ok(())
    }

    /// Records a file delete operation in the stage state.
    ///
    /// # Errors
    /// Returns error if state cannot be saved.
    pub fn record_delete(&mut self, path: &str) -> Result<()> {
        self.ensure_state_loaded()?;

        if let Some(state) = &mut self.state {
            state.record_delete(path);
            state.save(&state_file_path(&self.repo_root))?;
        }

        Ok(())
    }

    /// Records a successful apply operation.
    ///
    /// # Errors
    /// Returns error if state cannot be saved.
    pub fn record_apply(&mut self) -> Result<()> {
        self.ensure_state_loaded()?;

        if let Some(state) = &mut self.state {
            state.record_apply();
            state.save(&state_file_path(&self.repo_root))?;
        }

        Ok(())
    }

    /// Promotes staged changes to the real workspace.
    ///
    /// # Errors
    /// Returns error if promotion fails (with rollback attempted).
    pub fn promote(&mut self, retention: usize) -> Result<PromoteResult> {
        self.ensure_state_loaded()?;

        let state = self.state.as_ref().context("No stage state found")?;

        let paths_to_write: Vec<&str> = state.paths_to_write();
        let paths_to_delete: Vec<&str> = state.paths_to_delete();

        if paths_to_write.is_empty() && paths_to_delete.is_empty() {
            return Ok(PromoteResult {
                files_written: vec![],
                files_deleted: vec![],
                backup_path: None,
            });
        }

        let backup_dir = backups_path(&self.repo_root).join("promote");
        let worktree = self.worktree();

        let result = promote_to_workspace(
            &self.repo_root,
            &worktree,
            &paths_to_write,
            &paths_to_delete,
            &backup_dir,
        )?;

        // Cleanup old backups before reset (which clears stage dir)
        if retention > 0 {
            let _ = cleanup_old_backups(&backup_dir, retention);
        }

        // Clear entire stage after successful promotion to prevent contamination
        self.reset()?;

        Ok(result)
    }

    /// Resets (removes) the current stage.
    ///
    /// # Errors
    /// Returns error if removal fails.
    pub fn reset(&mut self) -> Result<()> {
        let stage = self.stage_dir();

        if stage.exists() {
            fs::remove_dir_all(&stage)
                .with_context(|| format!("Failed to reset stage: {}", stage.display()))?;
        }

        // Log event
        let logger = EventLogger::new(&self.repo_root);
        logger.log(EventKind::StageReset);

        self.state = None;
        Ok(())
    }

    /// Gets the stage ID if a stage exists.
    #[must_use]
    pub fn stage_id(&self) -> Option<&str> {
        self.state.as_ref().map(|s| s.id.as_str())
    }

    /// Gets the number of applies in the current stage.
    #[must_use]
    pub fn apply_count(&self) -> u32 {
        self.state.as_ref().map_or(0, |s| s.apply_count)
    }

    /// Ensures state is loaded, loading it if necessary.
    fn ensure_state_loaded(&mut self) -> Result<()> {
        if self.state.is_none() && self.exists() {
            self.load_state()?;
        }
        Ok(())
    }
}

/// Result of ensuring a stage exists.
#[derive(Debug)]
pub enum EnsureResult {
    /// Stage already existed.
    Existed,
    /// Stage was created with the given copy stats.
    Created(CopyStats),
}

impl EnsureResult {
    /// Returns true if a new stage was created.
    #[must_use]
    pub fn was_created(&self) -> bool {
        matches!(self, EnsureResult::Created(_))
    }
}