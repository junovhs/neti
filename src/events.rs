// src/events.rs
//! Machine-readable event logging for audit trails.
//!
//! Events are appended to `.neti/events.jsonl`.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    StageCreated {
        id: String,
    },
    StageReset,
    ApplyStarted,
    ApplySucceeded {
        files_written: usize,
        files_deleted: usize,
    },
    ApplyRejected {
        reason: String,
    },
    FileWritten {
        path: String,
        bytes: usize,
    },
    FileDeleted {
        path: String,
    },
    CheckStarted,
    CheckPassed,
    CheckFailed {
        exit_code: i32,
    },
    PromoteStarted,
    PromoteSucceeded {
        files_written: usize,
        files_deleted: usize,
    },
    PromoteFailed {
        error: String,
    },
    SanitizationPerformed {
        path: String,
        lines_removed: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetiEvent {
    pub timestamp: u64,
    pub kind: EventKind,
}

#[derive(Clone)]
pub struct EventLogger {
    log_path: PathBuf,
}

impl EventLogger {
    #[must_use]
    pub fn new(repo_root: &Path) -> Self {
        let log_path = repo_root.join(".neti").join("events.jsonl");
        Self { log_path }
    }

    pub fn log(&self, kind: EventKind) {
        // Logging is best-effort. We swallow errors to avoid crashing main flow.
        if let Ok(json) = Self::serialize_event(kind) {
            let _ = self.append_to_file(&json);
        }
    }

    fn serialize_event(kind: EventKind) -> Result<String> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let event = NetiEvent { timestamp, kind };
        Ok(serde_json::to_string(&event)?)
    }

    fn append_to_file(&self, line: &str) -> Result<()> {
        if let Some(parent) = self.log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }
}