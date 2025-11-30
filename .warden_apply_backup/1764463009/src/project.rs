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

impl ProjectType {
    #[must_use]
    pub fn detect() -> Self {
        if Path::new("Cargo.toml").exists() {
            return Self::Rust;
        }
        if Path::new("package.json").exists() {
            return Self::Node;
        }
        if Path::new("pyproject.toml").exists()
            || Path::new("requirements.txt").exists()
            || Path::new("Pipfile").exists()
        {
            return Self::Python;
        }
        if Path::new("go.mod").exists() {
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
pub fn generate_toml() -> String {
    let project = ProjectType::detect();
    let rules = rules_section();
    let commands = commands_section(project);

    format!("# warden.toml\n{rules}\n\n{commands}\n")
}

fn rules_section() -> String {
    r#"[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
max_function_words = 5
ignore_naming_on = ["tests", "spec"]"#
        .to_string()
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
check = "cargo clippy --all-targets -- -D warnings -D clippy::pedantic"
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