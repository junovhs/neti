#![allow(unused_imports)] // Context is used on some OS targets but not others


use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::tokens::Tokenizer;

const TEMP_PREFIX: &str = "warden_clipboard_";

// --- Public API ---

/// Smartly copies text or file handles based on size.
///
/// # Errors
/// Returns error if clipboard access fails or temp file creation fails.
pub fn smart_copy(text: &str) -> Result<String> {
// 1. The Garbage Man: Clean up old artifacts first
cleanup_temp_files();

code
Code
download
content_copy
expand_less
// 2. Check Size
let token_count = Tokenizer::count(text);

if token_count < 1500 {
    // Small? Text Copy.
    perform_copy(text)?;
    Ok("Text copied to clipboard".to_string())
} else {
    // Huge? File Copy.
    let file_path = write_to_temp(text)?;
    copy_file_handle(&file_path)?;
    
    // Fixed clippy::map_unwrap_or violation
    let filename = file_path
        .file_name()
        .map_or_else(|| "temp_file".into(), |n| n.to_string_lossy());

    Ok(format!(
        "Large content ({token_count} tokens). Copied as file attachment: {filename}"
    ))
}

}

/// Legacy wrapper for existing calls.
///
/// # Errors
/// Returns error if clipboard access fails.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
let _ = smart_copy(text)?;
Ok(())
}

/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if clipboard access fails.
pub fn read_clipboard() -> Result<String> {
perform_read()
}

// --- Internal Logic ---

fn write_to_temp(content: &str) -> Result<PathBuf> {
let timestamp = SystemTime::now()
.duration_since(UNIX_EPOCH)?
.as_nanos();

code
Code
download
content_copy
expand_less
let filename = format!("{TEMP_PREFIX}{timestamp}.txt");
let mut temp_path = std::env::temp_dir();
temp_path.push(filename);

fs::write(&temp_path, content)?;
Ok(temp_path)

}

fn cleanup_temp_files() {
let temp_dir = std::env::temp_dir();
let Ok(entries) = fs::read_dir(temp_dir) else { return; };

code
Code
download
content_copy
expand_less
let now = SystemTime::now();
let fifteen_mins = std::time::Duration::from_secs(15 * 60);

for entry in entries.flatten() {
    let path = entry.path();
    if should_delete(&path, now, fifteen_mins) {
         let _ = fs::remove_file(path);
    }
}

}

// Helper to reduce cyclomatic complexity of cleanup_temp_files
fn should_delete(path: &Path, now: SystemTime, limit: std::time::Duration) -> bool {
let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
return false;
};

code
Code
download
content_copy
expand_less
if !name.starts_with(TEMP_PREFIX) {
    return false;
}

let Ok(metadata) = fs::metadata(path) else { return false; };
let Ok(modified) = metadata.modified() else { return false; };

now.duration_since(modified).unwrap_or_default() > limit

}

// --- Platform Specifics (File Handles) ---

#[cfg(target_os = "windows")]
fn copy_file_handle(path: &Path) -> Result<()> {
let path_str = path.to_string_lossy();
// Escape single quotes for PowerShell (replace ' with '')
let escaped_path = path_str.replace(''', "''");
let cmd = format!("Set-Clipboard -Path '{escaped_path}'");

code
Code
download
content_copy
expand_less
Command::new("powershell")
    .args(["-NoProfile", "-NonInteractive", "-Command", &cmd])
    .output()
    .context("Failed to set clipboard via PowerShell")?;
Ok(())

}

#[cfg(target_os = "macos")]
fn copy_file_handle(path: &Path) -> Result<()> {
let path_str = path.to_string_lossy();
let script = format!("set the clipboard to POSIX file "{path_str}"");

code
Code
download
content_copy
expand_less
Command::new("osascript")
    .arg("-e")
    .arg(&script)
    .output()
    .context("Failed to set clipboard via osascript")?;
Ok(())

}

#[cfg(target_os = "linux")]
fn copy_file_handle(path: &Path) -> Result<()> {
let path_str = path.to_string_lossy();

code
Code
download
content_copy
expand_less
// Try wl-copy (Wayland)
if Command::new("wl-copy").arg(&*path_str).output().is_ok() {
    return Ok(());
}

// X11 fallback
let uri = format!("file://{path_str}");
let mut child = Command::new("xclip")
    .args(["-selection", "clipboard", "-t", "text/uri-list", "-i"])
    .stdin(std::process::Stdio::piped())
    .spawn()?;

if let Some(mut stdin) = child.stdin.take() {
    use std::io::Write;
    write!(stdin, "{uri}")?;
}
child.wait()?;
Ok(())

}

// --- Platform Specifics (Text Read/Write) ---

#[cfg(target_os = "macos")]
fn perform_copy(text: &str) -> Result<()> {
use std::io::Write;
let mut child = Command::new("pbcopy")
.stdin(std::process::Stdio::piped())
.spawn()?;
if let Some(mut stdin) = child.stdin.take() {
stdin.write_all(text.as_bytes())?;
}
child.wait()?;
Ok(())
}

#[cfg(target_os = "macos")]
fn perform_read() -> Result<String> {
let output = Command::new("pbpaste").output()?;
Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "linux")]
fn perform_copy(text: &str) -> Result<()> {
use std::io::Write;
// Try xclip
if let Ok(mut child) = Command::new("xclip")
.args(["-selection", "clipboard", "-in"])
.stdin(std::process::Stdio::piped())
.spawn()
{
if let Some(mut stdin) = child.stdin.take() {
stdin.write_all(text.as_bytes())?;
}
child.wait()?;
return Ok(());
}

code
Code
download
content_copy
expand_less
// Fallback to wl-copy
let mut child = Command::new("wl-copy")
    .stdin(std::process::Stdio::piped())
    .spawn()?;
if let Some(mut stdin) = child.stdin.take() {
    stdin.write_all(text.as_bytes())?;
}
child.wait()?;
Ok(())

}

#[cfg(target_os = "linux")]
fn perform_read() -> Result<String> {
if let Ok(output) = Command::new("xclip")
.args(["-selection", "clipboard", "-out"])
.output()
{
return Ok(String::from_utf8_lossy(&output.stdout).to_string());
}
let output = Command::new("wl-paste").output()?;
Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "windows")]
fn perform_copy(text: &str) -> Result<()> {
use std::io::Write;
let mut child = Command::new("clip")
.stdin(std::process::Stdio::piped())
.spawn()?;
if let Some(mut stdin) = child.stdin.take() {
stdin.write_all(text.as_bytes())?;
}
child.wait()?;
Ok(())
}

#[cfg(target_os = "windows")]
fn perform_read() -> Result<String> {
let output = Command::new("powershell")
.args(["-command", "Get-Clipboard"])
.output()?;
Ok(String::from_utf8_lossy(&output.stdout).to_string())
}