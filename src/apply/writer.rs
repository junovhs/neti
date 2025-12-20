// src/apply/writer.rs
use crate::apply::backup;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct WriteTransaction {
    written: Vec<String>,
    deleted: Vec<String>,
    created_fresh: Vec<String>,
}

impl WriteTransaction {
    fn new() -> Self {
        Self {
            written: Vec::new(),
            deleted: Vec::new(),
            created_fresh: Vec::new(),
        }
    }
}

/// Writes changes (updates, new files, deletes) to disk atomically.
///
/// # Errors
/// Returns error if file system operations fail or symlink escape detected.
pub fn write_files(
    manifest: &Manifest,
    files: &ExtractedFiles,
    root: Option<&Path>,
    retention: usize,
) -> Result<ApplyOutcome> {
    let root_path = root.map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let canonical_root = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.clone());

    let backup_path = backup::create_backup(manifest, root)?;

    if backup_path.is_some() && retention > 0 {
        backup::cleanup_old_backups(root, retention);
    }

    let mut tx = WriteTransaction::new();

    for entry in manifest {
        if let Err(e) = apply_entry(entry, files, root, &canonical_root, &mut tx) {
            if let Some(backup) = &backup_path {
                backup::perform_rollback(root, backup, &tx.created_fresh);
            }
            return Err(e);
        }
    }

    Ok(ApplyOutcome::Success {
        written: tx.written,
        deleted: tx.deleted,
        backed_up: backup_path.is_some(),
        staged: false,
    })
}

fn apply_entry(
    entry: &crate::apply::types::ManifestEntry,
    files: &ExtractedFiles,
    root: Option<&Path>,
    canonical_root: &Path,
    tx: &mut WriteTransaction,
) -> Result<()> {
    match entry.operation {
        Operation::Delete => {
            delete_file(&entry.path, root, canonical_root)?;
            tx.deleted.push(entry.path.clone());
        }
        Operation::Update | Operation::New => {
            if let Some(file_data) = files.get(&entry.path) {
                let path = resolve_path(&entry.path, root);
                if !path.exists() {
                    tx.created_fresh.push(entry.path.clone());
                }
                write_single_file(&entry.path, &file_data.content, root, canonical_root)?;
                tx.written.push(entry.path.clone());
            }
        }
    }
    Ok(())
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
        anyhow!(
            "Atomic rename failed {} to {}: {e}",
            temp_path.display(),
            path.display()
        )
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
