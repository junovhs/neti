use crate::error::Result;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum GitMode {
    Auto,
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub git_mode: GitMode,
    pub include_patterns: Vec<Regex>,
    pub exclude_patterns: Vec<Regex>,
    pub code_only: bool,
    pub verbose: bool,
}

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self {
            git_mode: GitMode::Auto,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            code_only: false,
            verbose: false,
        }
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration violates rules (currently empty).
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// Pattern constants
pub const PRUNE_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "dist",
    "build",
    "target",
    "gen",
    "schemas",
    "tests",
    "test",
    "__tests__",
    ".venv",
    "venv",
    ".tox",
    ".cache",
    "coverage",
    "vendor",
    "third_party",
];

pub const BIN_EXT_PATTERN: &str = r"(?i)\.(png|jpe?g|gif|svg|ico|icns|webp|woff2?|ttf|otf|pdf|mp4|mov|mkv|avi|mp3|wav|flac|zip|gz|bz2|xz|7z|rar|jar|csv|tsv|parquet|sqlite|db|bin|exe|dll|so|dylib|pkl|onnx|torch|tgz|zst)$";

pub const SECRET_PATTERN: &str = r"(?i)(^\.?env(\..*)?$|/\.?env(\..*)?$|(^|/)(id_rsa(\.pub)?|id_ed25519(\.pub)?|.*\.(pem|p12|jks|keystore|pfx))$)";

pub const CODE_EXT_PATTERN: &str = r"(?i)\.(c|h|cc|hh|cpp|hpp|rs|go|py|js|jsx|ts|tsx|java|kt|kts|rb|php|scala|cs|swift|m|mm|lua|sh|bash|zsh|fish|ps1|sql|html|xhtml|xml|xsd|xslt|yaml|yml|toml|ini|cfg|conf|json|ndjson|md|rst|tex|s|asm|cmake|gradle|proto|graphql|gql|nix|dart|scss|less|css)$";

pub const CODE_BARE_PATTERN: &str =
    r"(?i)(Makefile|Dockerfile|dockerfile|CMakeLists\.txt|BUILD|WORKSPACE)$";
