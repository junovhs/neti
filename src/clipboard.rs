// src/clipboard.rs
use anyhow::Result;
use std::process::Command;

/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the system clipboard command fails.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    perform_copy(text)
}

/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if the system clipboard command fails.
pub fn read_clipboard() -> Result<String> {
    perform_read()
}

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
    // We use the arboard crate in the actual build, but if you want shell fallback:
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
    // Powershell is slow but reliable without external deps
    let output = Command::new("powershell")
        .args(["-command", "Get-Clipboard"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
