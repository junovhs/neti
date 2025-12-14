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
    retention: usize,
) -> Result<ApplyOutcome> {
    let root_path = root.map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let canonical_root = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.clone());

    let backup_path = create_backup(manifest, root)?;

    if backup_path.is_some() && retention > 0 {
        cleanup_old_backups(root, retention);
    }

    let mut written = Vec::new();
    let mut deleted = Vec::new();
    let mut created_fresh = Vec::new(); // Track strictly new files for rollback

    for entry in manifest {
        if let Err(e) = apply_entry(
            entry,
            files,
            root,
            &canonical_root,
            &mut written,
            &mut deleted,
            &mut created_fresh,
        ) {
            if let Some(backup) = &backup_path {
                perform_rollback(root, backup, &created_fresh);
            }
            return Err(e);
        }
    }

    Ok(ApplyOutcome::Success {
        written,
        deleted,
        roadmap_results: Vec::new(),
        backed_up: backup_path.is_some(),
    })
}

fn apply_entry(
    entry: &crate::apply::types::ManifestEntry,
    files: &ExtractedFiles,
    root: Option<&Path>,
    canonical_root: &Path,
    written: &mut Vec<String>,
    deleted: &mut Vec<String>,
    created_fresh: &mut Vec<String>,
) -> Result<()> {
    match entry.operation {
        Operation::Delete => {
            delete_file(&entry.path, root, canonical_root)?;
            deleted.push(entry.path.clone());
        }
        Operation::Update | Operation::New => {
            if let Some(file_data) = files.get(&entry.path) {
                let path = resolve_path(&entry.path, root);
                if !path.exists() {
                    created_fresh.push(entry.path.clone());
                }
                write_single_file(&entry.path, &file_data.content, root, canonical_root)?;
                written.push(entry.path.clone());
            }
        }
    }
    Ok(())
}

fn perform_rollback(root: Option<&Path>, backup_folder: &Path, created_fresh: &[String]) {
    eprintln!("Error during apply. Rolling back...");
    let root_path = root.map_or_else(|| PathBuf::from("."), Path::to_path_buf);

    // 1. Restore backed up files (Reverts Updates and Deletes)
    if backup_folder.exists() {
        restore_dir_recursive(backup_folder, &root_path);
    }

    // 2. Delete newly created files (Reverts News)
    for path_str in created_fresh {
        let path = root_path.join(path_str);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
    eprintln!("Rollback complete.");
}

fn restore_dir_recursive(src: &Path, target_root: &Path) {
    if !src.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(src) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name() else {
            continue;
        };
        let dest = target_root.join(name);

        if path.is_dir() {
            restore_dir_recursive(&path, &dest);
        } else {
            restore_file(&path, &dest);
        }
    }
}

fn restore_file(src: &Path, dest: &Path) {
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::copy(src, dest);
}

fn cleanup_old_backups(root: Option<&Path>, retention: usize) {
    let root_path = root.map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let backup_dir = root_path.join(BACKUP_DIR);

    if !backup_dir.exists() {
        return;
    }

    let Ok(entries) = fs::read_dir(&backup_dir) else {
        return;
    };

    let mut timestamps: Vec<(u64, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_name()?.to_string_lossy();
            let ts: u64 = name.parse().ok()?;
            Some((ts, path))
        })
        .collect();

    timestamps.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, path) in timestamps.into_iter().skip(retention) {
        let _ = fs::remove_dir_all(path);
    }
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