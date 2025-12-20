// src/stage/copy.rs
//! Directory copy logic with exclusions for stage creation.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Directories to always exclude from stage copy.
const EXCLUDED_DIRS: &[&str] = &[
    ".slopchop",
    ".git",
    "node_modules",
    "target",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "vendor",
];

/// Files to always exclude from stage copy.
const EXCLUDED_FILES: &[&str] = &[".DS_Store", "Thumbs.db", "desktop.ini"];

/// Copies the source directory to destination, excluding heavy/ignored paths.
///
/// # Errors
/// Returns error if filesystem operations fail.
pub fn copy_repo_to_stage(src: &Path, dest: &Path) -> Result<CopyStats> {
    let mut stats = CopyStats::default();

    // Ensure destination exists
    fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create stage dir: {}", dest.display()))?;

    // Walk the source directory
    for entry in WalkDir::new(src).min_depth(1) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                stats.errors += 1;
                eprintln!("Warning: Failed to read entry: {e}");
                continue;
            }
        };

        let rel_path = match entry.path().strip_prefix(src) {
            Ok(p) => p,
            Err(_) => continue,
        };

        // Skip excluded paths
        if should_exclude(rel_path) {
            if entry.file_type().is_dir() {
                stats.dirs_skipped += 1;
            } else {
                stats.files_skipped += 1;
            }
            continue;
        }

        let dest_path = dest.join(rel_path);

        if entry.file_type().is_dir() {
            if let Err(e) = create_dir_safe(&dest_path) {
                stats.errors += 1;
                eprintln!("Warning: Failed to create dir {}: {e}", dest_path.display());
            } else {
                stats.dirs_copied += 1;
            }
        } else if entry.file_type().is_file() {
            if let Err(e) = copy_file_safe(entry.path(), &dest_path) {
                stats.errors += 1;
                eprintln!("Warning: Failed to copy {}: {e}", entry.path().display());
            } else {
                stats.files_copied += 1;
            }
        } else if entry.file_type().is_symlink() {
            // Handle symlinks conservatively: skip them
            stats.symlinks_skipped += 1;
        }
    }

    Ok(stats)
}

/// Statistics from a copy operation.
#[derive(Debug, Default)]
pub struct CopyStats {
    pub files_copied: usize,
    pub dirs_copied: usize,
    pub files_skipped: usize,
    pub dirs_skipped: usize,
    pub symlinks_skipped: usize,
    pub errors: usize,
}

impl CopyStats {
    /// Returns true if the copy completed without errors.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors == 0
    }

    /// Returns a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Copied {} files, {} dirs. Skipped {} files, {} dirs, {} symlinks.",
            self.files_copied,
            self.dirs_copied,
            self.files_skipped,
            self.dirs_skipped,
            self.symlinks_skipped
        )
    }
}

/// Checks if a relative path should be excluded from copying.
fn should_exclude(rel_path: &Path) -> bool {
    // Check each component of the path
    for component in rel_path.components() {
        if let std::path::Component::Normal(name) = component {
            let name_str = name.to_string_lossy();

            // Check against excluded directories
            if EXCLUDED_DIRS.iter().any(|&d| d == name_str) {
                return true;
            }

            // Check against excluded files
            if EXCLUDED_FILES.iter().any(|&f| f == name_str) {
                return true;
            }
        }
    }

    false
}

/// Creates a directory, ensuring parents exist.
fn create_dir_safe(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Copies a file, creating parent directories as needed.
fn copy_file_safe(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        create_dir_safe(parent)?;
    }

    fs::copy(src, dest)
        .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;

    Ok(())
}

/// Removes a directory and all its contents.
///
/// # Errors
/// Returns error if removal fails.
pub fn remove_stage(stage_dir: &Path) -> Result<()> {
    if stage_dir.exists() {
        fs::remove_dir_all(stage_dir)
            .with_context(|| format!("Failed to remove stage: {}", stage_dir.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_should_exclude_slopchop() {
        assert!(should_exclude(Path::new(".slopchop")));
        assert!(should_exclude(Path::new(".slopchop/stage")));
    }

    #[test]
    fn test_should_exclude_git() {
        assert!(should_exclude(Path::new(".git")));
        assert!(should_exclude(Path::new(".git/objects")));
    }

    #[test]
    fn test_should_exclude_node_modules() {
        assert!(should_exclude(Path::new("node_modules")));
        assert!(should_exclude(Path::new("node_modules/package")));
    }

    #[test]
    fn test_should_not_exclude_src() {
        assert!(!should_exclude(Path::new("src")));
        assert!(!should_exclude(Path::new("src/main.rs")));
    }

    #[test]
    fn test_copy_basic() -> Result<()> {
        let src = TempDir::new()?;
        let dest = TempDir::new()?;

        // Create source structure
        fs::write(src.path().join("file.txt"), "hello")?;
        fs::create_dir(src.path().join("subdir"))?;
        fs::write(src.path().join("subdir/nested.txt"), "world")?;

        let stats = copy_repo_to_stage(src.path(), dest.path())?;

        assert!(stats.is_success());
        assert_eq!(stats.files_copied, 2);
        assert!(dest.path().join("file.txt").exists());
        assert!(dest.path().join("subdir/nested.txt").exists());

        Ok(())
    }

    #[test]
    fn test_copy_excludes_git() -> Result<()> {
        let src = TempDir::new()?;
        let dest = TempDir::new()?;

        // Create source with .git
        fs::create_dir(src.path().join(".git"))?;
        fs::write(src.path().join(".git/config"), "git config")?;
        fs::write(src.path().join("src.rs"), "code")?;

        let stats = copy_repo_to_stage(src.path(), dest.path())?;

        assert!(stats.is_success());
        assert!(dest.path().join("src.rs").exists());
        assert!(!dest.path().join(".git").exists());

        Ok(())
    }

    #[test]
    fn test_copy_excludes_slopchop() -> Result<()> {
        let src = TempDir::new()?;
        let dest = TempDir::new()?;

        // Create source with .slopchop
        fs::create_dir_all(src.path().join(".slopchop/stage"))?;
        fs::write(src.path().join(".slopchop/state.json"), "{}")?;
        fs::write(src.path().join("main.rs"), "fn main() {}")?;

        let stats = copy_repo_to_stage(src.path(), dest.path())?;

        assert!(stats.is_success());
        assert!(dest.path().join("main.rs").exists());
        assert!(!dest.path().join(".slopchop").exists());

        Ok(())
    }
}
