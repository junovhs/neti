use crate::config::{Config, PRUNE_DIRS};
use crate::error::{Result, WardenError};
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub struct FileEnumerator {
    config: Config,
}

impl FileEnumerator {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Enumerates files based on configuration.
    ///
    /// # Errors
    ///
    /// Returns error if `git` fails in Git mode.
    pub fn enumerate(&self) -> Result<Vec<PathBuf>> {
        use crate::config::GitMode;

        match self.config.git_mode {
            GitMode::Yes => {
                if !Self::in_git_repo() {
                    return Err(WardenError::NotInGitRepo);
                }
                Self::git_ls_files()
            }
            GitMode::No => Ok(self.walk_all_files()),
            GitMode::Auto => {
                if Self::in_git_repo() {
                    if let Ok(files) = Self::git_ls_files() {
                        return Ok(files);
                    }
                }
                Ok(self.walk_all_files())
            }
        }
    }

    fn in_git_repo() -> bool {
        let out = Command::new("git")
            .arg("rev-parse")
            .arg("--is-inside-work-tree")
            .output();

        matches!(out, Ok(o) if o.status.success())
    }

    fn git_ls_files() -> Result<Vec<PathBuf>> {
        let out = Command::new("git")
            .arg("ls-files")
            .arg("-z")
            .arg("--exclude-standard")
            .output()?;

        if !out.status.success() {
            return Err(WardenError::Other(format!(
                "git ls-files failed: exit {}",
                out.status
            )));
        }

        let mut paths = Vec::new();
        for chunk in out.stdout.split(|b| *b == 0) {
            if chunk.is_empty() {
                continue;
            }
            let s = String::from_utf8_lossy(chunk);
            paths.push(PathBuf::from(s.as_ref()));
        }
        Ok(paths)
    }

    fn walk_all_files(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let mut errors = Vec::new();

        let walker = WalkDir::new(".").follow_links(false).into_iter();

        for item in walker.filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !PRUNE_DIRS.iter().any(|p| name == *p)
        }) {
            let entry = match item {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("walkdir: {e}"));
                    continue;
                }
            };

            if entry.file_type().is_file() {
                let p = entry.path().strip_prefix(".").unwrap_or(entry.path());
                paths.push(p.to_path_buf());
            }
        }

        if !errors.is_empty() && self.config.verbose {
            eprintln!(
                "WARN: Encountered {} errors during file walk:",
                errors.len()
            );
            for (i, err) in errors.iter().take(5).enumerate() {
                eprintln!("  {}. {}", i + 1, err);
            }
        }

        paths
    }
}
