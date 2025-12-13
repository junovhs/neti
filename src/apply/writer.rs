// src/apply/writer.rs
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const BACKUP_DIR: &str = ".slopchop_apply_backup";

/// Writes changes (updates, new files, deletes) to disk atomically.
///
/// # Errors
/// Returns error if file system operations fail or symlink escape detected.
pub fn write_files(
    manifest: &Manifest,
    files: &ExtractedFiles,
    root: Option<&Path>,
) -> Result<ApplyOutcome> {
    let root_path = root.map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let canonical_root = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.clone());

    let backup_path = create_backup(manifest, root)?;
    let mut written = Vec::new();
    let mut deleted = Vec::new();

    for entry in manifest {
        match entry.operation {
            Operation::Delete => {
                delete_file(&entry.path, root, &canonical_root)?;
                deleted.push(entry.path.clone());
            }
            Operation::Update | Operation::New => {
                if let Some(file_data) = files.get(&entry.path) {
                    write_single_file(&entry.path, &file_data.content, root, &canonical_root)?;
                    written.push(entry.path.clone());
                }
            }
        }
    }

    Ok(ApplyOutcome::Success {
        written,
        deleted,
        roadmap_results: Vec::new(),
        backed_up: backup_path.is_some(),
    })
}

fn delete_file(path_str: &str, root: Option<&Path>, canonical_root: &Path) -> Result<()> {
    let path = resolve_path(path_str, root);
    if path.exists() {
        check_symlink_escape(&path, canonical_root)?;
        fs::remove_file(&path).with_context(|| format!("Failed to delete {}", path.display()))?;
    }
    Ok(())
}

fn write_single_file(
    path_str: &str,
    content: &str,
    root: Option<&Path>,
    canonical_root: &Path,
) -> Result<()> {
    let path = resolve_path(path_str, root);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow!("Failed to create directory {}: {e}", parent.display()))?;
    }

    check_symlink_escape(&path, canonical_root)?;

    let temp_path = create_temp_path(&path);

    fs::write(&temp_path, content)
        .map_err(|e| anyhow!("Failed to write temp file {}: {e}", temp_path.display()))?;

    fs::rename(&temp_path, &path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        anyhow!("Atomic rename failed {} to {}: {e}", temp_path.display(), path.display())
    })?;

    Ok(())
}

fn create_temp_path(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .map_or_else(|| "file".to_string(), |n| n.to_string_lossy().to_string());

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let temp_name = format!(".slopchop_tmp_{timestamp}_{file_name}");
    let parent = target.parent().unwrap_or(Path::new("."));
    parent.join(temp_name)
}

fn check_symlink_escape(path: &Path, canonical_root: &Path) -> Result<()> {
    let mut current = PathBuf::new();

    for component in path.components() {
        current.push(component);
        if !current.exists() {
            break;
        }

        let meta = fs::symlink_metadata(&current)
            .with_context(|| format!("Failed to check {}", current.display()))?;

        if meta.file_type().is_symlink() {
            let resolved = current
                .canonicalize()
                .with_context(|| format!("Failed to resolve symlink {}", current.display()))?;

            if !resolved.starts_with(canonical_root) {
                return Err(anyhow!(
                    "Symlink escape blocked: {} points outside repo root",
                    current.display()
                ));
            }
        }
    }
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