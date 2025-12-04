// src/clipboard/temp.rs
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const TEMP_PREFIX: &str = "slopchop_clipboard_";

/// Writes content to a temporary file.
///
/// # Errors
/// Returns error if file creation or write fails.
pub fn write_to_temp(content: &str) -> Result<PathBuf> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();

    let filename = format!("{TEMP_PREFIX}{timestamp}.txt");
    let mut temp_path = std::env::temp_dir();
    temp_path.push(filename);

    fs::write(&temp_path, content)?;
    Ok(temp_path)
}

pub fn cleanup_temp_files() {
    let temp_dir = std::env::temp_dir();
    let Ok(entries) = fs::read_dir(temp_dir) else {
        return;
    };

    let now = SystemTime::now();
    let fifteen_mins = std::time::Duration::from_secs(15 * 60);

    for entry in entries.flatten() {
        let path = entry.path();
        if should_delete(&path, now, fifteen_mins) {
            let _ = fs::remove_file(path);
        }
    }
}

fn should_delete(path: &Path, now: SystemTime, limit: std::time::Duration) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    if !name.starts_with(TEMP_PREFIX) {
        return false;
    }

    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };

    now.duration_since(modified).unwrap_or_default() > limit
}
