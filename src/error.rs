// src/error.rs
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WardenError {
    #[error("I/O error: {source} (path: {path})")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("Not inside a Git repository")]
    NotInGitRepo,

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Generic error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, WardenError>;

// Allow `?` on std::io::Error by converting to WardenError::Io with unknown path.
impl From<std::io::Error> for WardenError {
    fn from(source: std::io::Error) -> Self {
        WardenError::Io {
            source,
            path: PathBuf::from("<unknown>"),
        }
    }
}

// Gracefully convert WalkDir errors
impl From<walkdir::Error> for WardenError {
    fn from(e: walkdir::Error) -> Self {
        WardenError::Other(e.to_string())
    }
}
