// src/stage/mod.rs
//! Staged Workspace Management
//!
//! Implements the "implicit staged workspace" architecture from the Pivot Brief.
//! All changes are applied to a shadow worktree first, then promoted to the
//! real workspace after verification passes.

mod copy;
mod manager;
mod promote;
mod state;

pub use manager::StageManager;
pub use state::{StageState, TouchKind, TouchedPath};

use std::path::{Path, PathBuf};

/// The root directory for all SlopChop ephemeral state.
pub const SLOPCHOP_DIR: &str = ".slopchop";

/// The stage subdirectory within `.slopchop/`.
pub const STAGE_DIR: &str = "stage";

/// The shadow worktree within the stage directory.
pub const WORKTREE_DIR: &str = "worktree";

/// The state file tracking stage metadata.
pub const STATE_FILE: &str = "state.json";

/// Backups directory for promotion rollback.
pub const BACKUPS_DIR: &str = "backups";

/// Computes the path to `.slopchop/` relative to a repo root.
#[must_use]
pub fn slopchop_path(repo_root: &Path) -> PathBuf {
    repo_root.join(SLOPCHOP_DIR)
}

/// Computes the path to `.slopchop/stage/` relative to a repo root.
#[must_use]
pub fn stage_path(repo_root: &Path) -> PathBuf {
    slopchop_path(repo_root).join(STAGE_DIR)
}

/// Computes the path to `.slopchop/stage/worktree/` relative to a repo root.
#[must_use]
pub fn worktree_path(repo_root: &Path) -> PathBuf {
    stage_path(repo_root).join(WORKTREE_DIR)
}

/// Computes the path to `.slopchop/stage/state.json` relative to a repo root.
#[must_use]
pub fn state_file_path(repo_root: &Path) -> PathBuf {
    stage_path(repo_root).join(STATE_FILE)
}

/// Computes the path to `.slopchop/backups/` relative to a repo root.
#[must_use]
pub fn backups_path(repo_root: &Path) -> PathBuf {
    slopchop_path(repo_root).join(BACKUPS_DIR)
}

/// Checks if a stage currently exists for the given repo root.
#[must_use]
pub fn stage_exists(repo_root: &Path) -> bool {
    worktree_path(repo_root).is_dir()
}

/// Returns the effective working directory for operations.
/// If a stage exists, returns the staged worktree path.
/// Otherwise, returns the repo root.
#[must_use]
pub fn effective_cwd(repo_root: &Path) -> PathBuf {
    if stage_exists(repo_root) {
        worktree_path(repo_root)
    } else {
        repo_root.to_path_buf()
    }
}
