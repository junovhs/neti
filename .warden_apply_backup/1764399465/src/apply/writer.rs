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
pub fn write_files(files: &ExtractedFiles, root: Option<&Path>) -> Result<ApplyOutcome> {
    let backup_path = create_backup(files, root)?;
    let mut written_paths = Vec::new();

    for (path_str, file_data) in files {
        write_single_file(path_str, &file_data.content, root)?;
        written_paths.push(path_str.clone());
    }

    Ok(ApplyOutcome::Success {
        written: written_paths,
        backed_up: backup_path.is_some(),
    })
}

fn write_single_file(path_str: &str, content: &str, root: Option<&Path>) -> Result<()> {
    let path = resolve_path(path_str, root);
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow!("Failed to create directory {}: {e}", parent.display()))?;
    }
    fs::write(&path, content).map_err(|e| anyhow!("Failed to write {}: {e}", path.display()))?;
    Ok(())
}

fn resolve_path(path_str: &str, root: Option<&Path>) -> PathBuf {
    match root {
        Some(r) => r.join(path_str),
        None => PathBuf::from(path_str),
    }
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

    let latest = find_latest_backup(backup_root)?;
    restore_files_from_backup(&latest)
}

fn find_latest_backup(root: &Path) -> Result<PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(root)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();

    entries.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

    entries
        .first()
        .map(std::fs::DirEntry::path)
        .ok_or_else(|| anyhow!("No backups found in {BACKUP_DIR}"))
}

fn restore_files_from_backup(restore_source: &Path) -> Result<Vec<PathBuf>> {
    let mut restored_files = Vec::new();
    for entry in walkdir::WalkDir::new(restore_source) {
        let entry = entry?;
        if entry.file_type().is_file() {
            restored_files.push(restore_single_file_entry(&entry, restore_source)?);
        }
    }
    Ok(restored_files)
}

fn restore_single_file_entry(entry: &walkdir::DirEntry, restore_source: &Path) -> Result<PathBuf> {
    let rel_path = entry.path().strip_prefix(restore_source)?;
    if let Some(parent) = rel_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(entry.path(), rel_path)?;
    Ok(rel_path.to_path_buf())
}

fn create_backup(files: &ExtractedFiles, root: Option<&Path>) -> Result<Option<PathBuf>> {
    // Only backup files that exist
    let files_to_backup: Vec<&String> = files.keys()
        .filter(|p| resolve_path(p, root).exists())
        .collect();

    if files_to_backup.is_empty() {
        return Ok(None);
    }

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let root_path = root.map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let backup_folder = root_path.join(BACKUP_DIR).join(timestamp.to_string());

    fs::create_dir_all(&backup_folder).context("Failed to create backup directory")?;

    for path_str in files_to_backup {
        backup_single_file(path_str, &backup_folder, root)?;
    }

    Ok(Some(backup_folder))
}

fn backup_single_file(path_str: &str, backup_folder: &Path, root: Option<&Path>) -> Result<()> {
    let src = resolve_path(path_str, root);
    let dest = backup_folder.join(path_str);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&src, &dest).with_context(|| format!("Failed to backup {}", src.display()))?;
    Ok(())
}