// src/apply/backup.rs
use crate::apply::types::Manifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const BACKUP_DIR: &str = ".slopchop_apply_backup";

/// Creates a backup of the files listed in the manifest.
///
/// # Errors
/// Returns error if directory creation or file copying fails.
pub fn create_backup(manifest: &Manifest, root: Option<&Path>) -> Result<Option<PathBuf>> {
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

pub fn perform_rollback(root: Option<&Path>, backup_folder: &Path, created_fresh: &[String]) {
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

pub fn cleanup_old_backups(root: Option<&Path>, retention: usize) {
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

fn resolve_path(path_str: &str, root: Option<&Path>) -> PathBuf {
    match root {
        Some(r) => r.join(path_str),
        None => PathBuf::from(path_str),
    }
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
