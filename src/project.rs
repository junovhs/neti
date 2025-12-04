// src/project.rs
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Strictness {
    Strict,
    Standard,
    Relaxed,
}

impl ProjectType {
    /// Detects project type from current directory.
    #[must_use]
    pub fn detect() -> Self {
        Self::detect_in(Path::new("."))
    }

    /// Detects project type in a specific directory.
    #[must_use]
    pub fn detect_in(root: &Path) -> Self {
        if root.join("Cargo.toml").exists() {
            return Self::Rust;
        }
        if root.join("package.json").exists() {
            return Self::Node;
        }
        if root.join("pyproject.toml").exists()
            || root.join("requirements.txt").exists()
            || root.join("Pipfile").exists()
        {
            return Self::Python;
        }
        if root.join("go.mod").exists() {
            return Self::Go;
        }
        Self::Unknown
    }

    /// Detects if this is a TypeScript project
    #[must_use]
    pub fn is_typescript() -> bool {
        Path::new("tsconfig.json").exists()
            || Path::new("tsconfig.node.json").exists()
            || has_ts_files()
    }
}

fn has_ts_files() -> bool {
    Path::new("src")
        .read_dir()
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "ts" || ext == "tsx")
            })
        })
        .unwrap_or(false)
}

#[must_use]
pub fn generate_toml(project: ProjectType, strictness: Strictness) -> String {
    let rules = rules_section(strictness);
    let commands = commands_section(project);

    format!("# slopchop.toml\n{rules}\n\n{commands}\n")
}

fn rules_section(strictness: Strictness) -> String {
    let (tokens, complexity, depth) = match strictness {
        Strictness::Strict => (1500, 4, 2),
        Strictness::Standard => (2000, 8, 3),
        Strictness::Relaxed => (3000, 12, 4),
    };

    format!(
        r#"[rules]
max_file_tokens = {tokens}
max_cyclomatic_complexity = {complexity}
max_nesting_depth = {depth}
max_function_args = 5
max_function_words = 5
ignore_naming_on = ["tests", "spec"]"#
    )
}

fn commands_section(project: ProjectType) -> String {
    match project {
        ProjectType::Rust => rust_commands(),
        ProjectType::Node => node_commands(),
        ProjectType::Python => python_commands(),
        ProjectType::Go => go_commands(),
        ProjectType::Unknown => unknown_commands(),
    }
}

fn rust_commands() -> String {
    r#"[commands]
check = [
    "cargo clippy --all-targets -- -D warnings -D clippy::pedantic",
    "cargo test"
]
fix = "cargo fmt""#
        .to_string()
}

fn node_commands() -> String {
    let npx = npx_cmd();

    // Use biome for TypeScript projects
    if ProjectType::is_typescript() {
        format!(
            r#"[commands]
check = "{npx} @biomejs/biome check src/"
fix = "{npx} @biomejs/biome check --write src/""#
        )
    } else {
        format!(
            r#"[commands]
check = "{npx} eslint src/"
fix = "{npx} eslint --fix src/""#
        )
    }
}

fn python_commands() -> String {
    r#"[commands]
check = "ruff check ."
fix = "ruff check --fix .""#
        .to_string()
}

fn go_commands() -> String {
    r#"[commands]
check = "go vet ./..."
fix = "go fmt ./...""#
        .to_string()
}

fn unknown_commands() -> String {
    r#"# No project type detected. Configure commands manually:
# [commands]
# check = "your-lint-command"
# fix = "your-fix-command""#
        .to_string()
}

#[must_use]
pub fn npx_cmd() -> &'static str {
    if cfg!(windows) {
        "npx.cmd"
    } else {
        "npx"
    }
}
