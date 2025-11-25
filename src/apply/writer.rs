// src/apply/writer.rs
use crate::apply::types::{ApplyOutcome, ExtractedFiles};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const BACKUP_DIR: &str = ".warden_apply_backup";

/// Writes the extracted files to disk.
///
/// # Errors
/// Returns error if file system operations fail.
pub fn write_files(files: &ExtractedFiles) -> Result<ApplyOutcome> {
    // 1. Create Backup
    let backup_path = create_backup(files)?;

    // 2. Write Files
    let mut written_paths = Vec::new();

    for (path_str, file_data) in files {
        let path = Path::new(path_str);

        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create directory {}: {e}", parent.display()))?;
        }

        // Write content
        fs::write(path, &file_data.content)
            .map_err(|e| anyhow!("Failed to write {path_str}: {e}"))?;

        written_paths.push(path_str.clone());
    }

    Ok(ApplyOutcome::Success {
        written: written_paths,
        backed_up: backup_path.is_some(),
    })
}

/// Restores files from the latest backup.
///
/// # Errors
/// Returns error if no backup exists or restore fails.
pub fn restore_backup() -> Result<Vec<PathBuf>> {
    let backup_root = Path::new(BACKUP_DIR);
    if !backup_root.exists() {
        return Err(anyhow!("No backup directory found at {BACKUP_DIR}"));
    }

    // Find latest backup timestamp
    let mut entries: Vec<_> = fs::read_dir(backup_root)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();

    // Sort by name (timestamp) descending
    entries.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

    let latest = entries
        .first()
        .ok_or_else(|| anyhow!("No backups found in {BACKUP_DIR}"))?;

    let restore_source = latest.path();
    let mut restored_files = Vec::new();

    // Walk the backup and copy back
    for entry in walkdir::WalkDir::new(&restore_source) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let rel_path = entry.path().strip_prefix(&restore_source)?;

            // Ensure parent exists in target
            if let Some(parent) = rel_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(entry.path(), rel_path)?;
            restored_files.push(rel_path.to_path_buf());
        }
    }

    Ok(restored_files)
}

fn create_backup(files: &ExtractedFiles) -> Result<Option<PathBuf>> {
    // Check if any files exist to back up
    let files_to_backup: Vec<&String> = files.keys().filter(|p| Path::new(p).exists()).collect();

    if files_to_backup.is_empty() {
        return Ok(None);
    }

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let backup_folder = Path::new(BACKUP_DIR).join(timestamp.to_string());

    fs::create_dir_all(&backup_folder).context("Failed to create backup directory")?;

    for path_str in files_to_backup {
        let src = Path::new(path_str);
        let dest = backup_folder.join(path_str);

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(src, dest).with_context(|| format!("Failed to backup {path_str}"))?;
    }

    Ok(Some(backup_folder))
}
