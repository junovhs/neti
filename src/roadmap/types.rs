// src/roadmap/types.rs
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
    pub tests: Vec<String>,
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
    AddSubsection {
        parent: String,
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

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check { path } => write!(f, "CHECK {path}"),
            Self::Uncheck { path } => write!(f, "UNCHECK {path}"),
            Self::Delete { path } => write!(f, "DELETE {path}"),
            Self::AddSection { heading } => write!(f, "SECTION \"{heading}\""),
            Self::AddSubsection { parent, heading } => {
                write!(f, "SUBSECTION {parent} \"{heading}\"")
            }
            Self::ReplaceSection { id, .. } => write!(f, "REPLACE {id}"),
            _ => write!(f, "{}", format_complex_command(self)),
        }
    }
}

fn format_complex_command(cmd: &Command) -> String {
    match cmd {
        Command::Update { path, text } => format!("UPDATE {path} \"{text}\""),
        Command::Add { parent, text, .. } => format!("ADD {parent} \"{text}\""),
        Command::Note { path, note } => format!("NOTE {path} \"{note}\""),
        Command::Move { path, position } => format!("MOVE {path} {position}"),
        _ => String::new(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovePosition {
    After(String),
    Before(String),
    EndOfSection(String),
}

impl fmt::Display for MovePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::After(t) => write!(f, "AFTER {t}"),
            Self::Before(t) => write!(f, "BEFORE {t}"),
            Self::EndOfSection(s) => write!(f, "TO {s}"),
        }
    }
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