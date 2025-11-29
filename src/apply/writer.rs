// src/apply/writer.rs
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const BACKUP_DIR: &str = ".warden_apply_backup";

/// Writes changes (updates, new files, deletes) to disk.
///
/// # Errors
/// Returns error if file system operations fail.
pub fn write_files(
    manifest: &Manifest,
    files: &ExtractedFiles,
    root: Option<&Path>,
) -> Result<ApplyOutcome> {
    let backup_path = create_backup(manifest, root)?;
    let mut written = Vec::new();
    let mut deleted = Vec::new();

    for entry in manifest {
        match entry.operation {
            Operation::Delete => {
                delete_file(&entry.path, root)?;
                deleted.push(entry.path.clone());
            }
            Operation::Update | Operation::New => {
                if let Some(file_data) = files.get(&entry.path) {
                    write_single_file(&entry.path, &file_data.content, root)?;
                    written.push(entry.path.clone());
                }
            }
        }
    }

    Ok(ApplyOutcome::Success {
        written,
        deleted,
        backed_up: backup_path.is_some(),
    })
}

fn delete_file(path_str: &str, root: Option<&Path>) -> Result<()> {
    let path = resolve_path(path_str, root);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Failed to delete {}", path.display()))?;
    }
    Ok(())
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

fn create_backup(manifest: &Manifest, root: Option<&Path>) -> Result<Option<PathBuf>> {
    let targets: Vec<&String> = manifest
        .iter()
        .map(|e| &e.path)
        .filter(|p| resolve_path(p, root).exists())
        .collect();

    if targets.is_empty() {
        return Ok(None);
    }

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let root_path = root.map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let backup_folder = root_path.join(BACKUP_DIR).join(timestamp.to_string());

    fs::create_dir_all(&backup_folder).context("Failed to create backup directory")?;

    for path_str in targets {
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