// src/constants.rs
//! Shared constants for file filtering and pattern matching.

pub const PRUNE_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    "gen",
    ".venv",
    "venv",
    ".tox",
    "__pycache__",
    "coverage",
    "vendor",
    ".neti_apply_backup",
];

pub const PRUNE_FILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lockb",
    "go.sum",
    "Gemfile.lock",
];

pub const SKIP_DIRS: &[&str] = &["spec", "examples", "fixtures"];

pub const BIN_EXT_PATTERN: &str =
    r"(?i)\.(png|jpg|gif|svg|ico|webp|woff2?|ttf|pdf|mp4|zip|gz|tar|exe|dll|so|dylib|class|pyc)$";

pub const SECRET_PATTERN: &str =
    r"(?i)(^\.?env(\..*)?$|/\.?env(\..*)?$|(^|/)(id_rsa|id_ed25519|.*\.(pem|p12|key|pfx))$)";

pub const CODE_EXT_PATTERN: &str = r"(?i)\.(rs|go|py|js|jsx|ts|tsx|java|c|cpp|h|hpp|cs|php|rb|sh|sql|html|css|scss|json|toml|yaml|md)$";

pub const CODE_BARE_PATTERN: &str = r"(?i)(Makefile|Dockerfile|CMakeLists\.txt)$";

/// Checks if a directory name should be pruned during traversal.
#[must_use]
pub fn should_prune(name: &str) -> bool {
    PRUNE_DIRS.contains(&name) || PRUNE_FILES.contains(&name) || SKIP_DIRS.contains(&name)
}
