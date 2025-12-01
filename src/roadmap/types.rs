use std::fmt;

#[derive(Debug, Clone)]
pub struct Roadmap {
    pub path: Option<String>,
    pub title: String,
    pub sections: Vec<Section>,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub id: String,
    pub heading: String,
    pub level: u8,
    pub theme: Option<String>,
    pub tasks: Vec<Task>,
    pub subsections: Vec<Section>,
    pub raw_content: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub path: String,
    pub text: String,
    pub status: TaskStatus,
    pub indent: u8,
    pub line: usize,
    pub children: Vec<Task>,
    pub tests: Vec<String>, // Explicitly linked test files
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Complete,
}

#[derive(Debug, Clone)]
pub struct RoadmapStats {
    pub total: usize,
    pub complete: usize,
    pub pending: usize,
}

/// A single command from AI
#[derive(Debug, Clone)]
pub enum Command {
    Check {
        path: String,
    },
    Uncheck {
        path: String,
    },
    Add {
        parent: String,
        text: String,
        after: Option<String>,
    },
    AddSection {
        heading: String,
    },
    Delete {
        path: String,
    },
    Update {
        path: String,
        text: String,
    },
    Note {
        path: String,
        note: String,
    },
    Move {
        path: String,
        position: MovePosition,
    },
    ReplaceSection {
        id: String,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovePosition {
    After(String),
    Before(String),
    EndOfSection(String),
}

/// A batch of commands parsed from AI output
#[derive(Debug, Clone)]
pub struct CommandBatch {
    pub commands: Vec<Command>,
    pub errors: Vec<String>,
}

/// Result of applying a command
#[derive(Debug)]
pub enum ApplyResult {
    Success(String),
    NotFound(String),
    Error(String),
}

impl fmt::Display for ApplyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success(msg) => write!(f, "✓ {msg}"),
            Self::NotFound(msg) => write!(f, "✗ Not found: {msg}"),
            Self::Error(msg) => write!(f, "✗ Error: {msg}"),
        }
    }
}