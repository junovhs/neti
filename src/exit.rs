// src/exit.rs
//! Standardized process exit codes for `SlopChop`.
//!
//! Provides a stable contract for scripts and automation.

use std::process::Termination;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SlopChopExit {
    /// Operation completed successfully.
    Success = 0,
    /// Generic error (e.g. IO, network, config).
    Error = 1,
    /// Input validation failed (Parser error, Protocol violation, Empty input).
    InvalidInput = 2,
    /// Security/Safety violation (Path traversal, Protected file, Symlink escape).
    SafetyViolation = 3,
    /// Patch application failed (Hash mismatch, Ambiguity, 0 matches).
    PatchFailure = 4,
    /// Promotion failed (Stage->Workspace write error).
    PromoteFailure = 5,
    /// Verification failed (Tests, Lints, or Structural Scan).
    CheckFailed = 6,
}

impl SlopChopExit {
    #[must_use]
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn exit(self) -> ! {
        std::process::exit(self.code())
    }
}

impl Termination for SlopChopExit {
    fn report(self) -> std::process::ExitCode {
        // Rust's std::process::ExitCode implies usage of `u8` on many unix-likes,
        // but we cast to standard 0..255 range implicitly via `u8`.
        // For portable scripts, we generally rely on 0 vs non-0, but specific codes help debug.
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        std::process::ExitCode::from(self.code() as u8)
    }
}

impl From<anyhow::Result<()>> for SlopChopExit {
    fn from(res: anyhow::Result<()>) -> Self {
        match res {
            Ok(()) => Self::Success,
            Err(e) => {
                eprintln!("Error: {e}");
                Self::Error
            }
        }
    }
}// test
