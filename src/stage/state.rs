// src/stage/state.rs
//! Stage state persistence and touched path tracking.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents the kind of touch operation performed on a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TouchKind {
    /// File was written (created or updated).
    Write,
    /// File was deleted.
    Delete,
}

/// A record of a file that was touched during the staged session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchedPath {
    /// Relative path from repo root.
    pub path: String,
    /// The kind of operation performed.
    pub kind: TouchKind,
    /// Timestamp of the operation (Unix epoch seconds).
    pub timestamp: u64,
}

/// Persistent state for an active stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageState {
    /// Unique identifier for this stage session.
    pub id: String,
    /// When the stage was created (Unix epoch seconds).
    pub created_at: u64,
    /// When the stage was last modified (Unix epoch seconds).
    pub updated_at: u64,
    /// Set of paths touched during this stage session.
    pub touched: Vec<TouchedPath>,
    /// Number of successful applies in this stage.
    pub apply_count: u32,
}

impl StageState {
    /// Creates a new stage state with a fresh ID.
    #[must_use]
    pub fn new() -> Self {
        let now = current_timestamp();
        Self {
            id: generate_stage_id(),
            created_at: now,
            updated_at: now,
            touched: Vec::new(),
            apply_count: 0,
        }
    }

    /// Loads state from a JSON file.
    ///
    /// # Errors
    /// Returns error if file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read state file: {}", path.display()))?;
        let state: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse state file: {}", path.display()))?;
        Ok(state)
    }

    /// Saves state to a JSON file.
    ///
    /// # Errors
    /// Returns error if file cannot be written.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create state dir: {}", parent.display()))?;
        }
        let content = serde_json::to_string_pretty(self).context("Failed to serialize state")?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write state file: {}", path.display()))?;
        Ok(())
    }

    /// Records a file write operation.
    pub fn record_write(&mut self, path: &str) {
        self.record_touch(path, TouchKind::Write);
    }

    /// Records a file delete operation.
    pub fn record_delete(&mut self, path: &str) {
        self.record_touch(path, TouchKind::Delete);
    }

    /// Records a touch operation, updating the timestamp if path already exists.
    fn record_touch(&mut self, path: &str, kind: TouchKind) {
        let now = current_timestamp();
        self.updated_at = now;

        // Remove any existing entry for this path
        self.touched.retain(|t| t.path != path);

        // Add the new touch record
        self.touched.push(TouchedPath {
            path: path.to_string(),
            kind,
            timestamp: now,
        });
    }

    /// Increments the apply count and updates timestamp.
    pub fn record_apply(&mut self) {
        self.apply_count += 1;
        self.updated_at = current_timestamp();
    }

    /// Returns all paths that need to be promoted (written files).
    #[must_use]
    pub fn paths_to_write(&self) -> Vec<&str> {
        self.touched
            .iter()
            .filter(|t| t.kind == TouchKind::Write)
            .map(|t| t.path.as_str())
            .collect()
    }

    /// Returns all paths that need to be deleted during promotion.
    #[must_use]
    pub fn paths_to_delete(&self) -> Vec<&str> {
        self.touched
            .iter()
            .filter(|t| t.kind == TouchKind::Delete)
            .map(|t| t.path.as_str())
            .collect()
    }

    /// Returns all unique touched paths (for backup).
    #[must_use]
    pub fn all_touched_paths(&self) -> HashSet<&str> {
        self.touched.iter().map(|t| t.path.as_str()).collect()
    }

    /// Clears all touched paths (used after successful promotion).
    pub fn clear_touched(&mut self) {
        self.touched.clear();
        self.updated_at = current_timestamp();
    }
}

impl Default for StageState {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a short, unique stage ID.
fn generate_stage_id() -> String {
    let now = current_timestamp();
    let random: u32 = (now as u32).wrapping_mul(1_103_515_245).wrapping_add(12345);
    format!("{now:x}-{random:04x}")
}

/// Returns current Unix timestamp in seconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_state_has_id() {
        let state = StageState::new();
        assert!(!state.id.is_empty());
        assert!(state.created_at > 0);
    }

    #[test]
    fn test_record_write() {
        let mut state = StageState::new();
        state.record_write("src/main.rs");
        assert_eq!(state.touched.len(), 1);
        assert_eq!(state.touched[0].kind, TouchKind::Write);
    }

    #[test]
    fn test_record_delete() {
        let mut state = StageState::new();
        state.record_delete("src/old.rs");
        assert_eq!(state.touched.len(), 1);
        assert_eq!(state.touched[0].kind, TouchKind::Delete);
    }

    #[test]
    fn test_save_and_load() -> Result<()> {
        let temp = TempDir::new()?;
        let path = temp.path().join("state.json");

        let mut state = StageState::new();
        state.record_write("src/lib.rs");
        state.save(&path)?;

        let loaded = StageState::load(&path)?;
        assert_eq!(loaded.id, state.id);
        assert_eq!(loaded.touched.len(), 1);
        Ok(())
    }

    #[test]
    fn test_paths_to_write() {
        let mut state = StageState::new();
        state.record_write("src/a.rs");
        state.record_delete("src/b.rs");
        state.record_write("src/c.rs");

        let writes = state.paths_to_write();
        assert_eq!(writes.len(), 2);
        assert!(writes.contains(&"src/a.rs"));
        assert!(writes.contains(&"src/c.rs"));
    }

    #[test]
    fn test_overwrite_touch_updates_kind() {
        let mut state = StageState::new();
        state.record_write("src/file.rs");
        state.record_delete("src/file.rs");

        assert_eq!(state.touched.len(), 1);
        assert_eq!(state.touched[0].kind, TouchKind::Delete);
    }
}
