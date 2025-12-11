// src/roadmap_v2/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskStore {
    pub meta: RoadmapMeta,
    #[serde(default)]
    pub sections: Vec<Section>,
    #[serde(default)]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapMeta {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

impl Default for RoadmapMeta {
    fn default() -> Self {
        Self {
            title: "Project Roadmap".to_string(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub status: SectionStatus,
    #[serde(default)]
    pub order: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionStatus {
    #[default]
    Pending,
    Current,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub status: TaskStatus,
    pub section: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default)]
    pub order: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Pending,
    Done,
    #[serde(rename = "no-test")]
    NoTest,
}

/// Commands that can modify the roadmap.
#[derive(Debug, Clone)]
pub enum RoadmapCommand {
    Check { id: String },
    Uncheck { id: String },
    Add(AddCommand),
    Update { id: String, fields: TaskUpdate },
    Delete { id: String },
}

/// Specifies where to insert a new task.
#[derive(Debug, Clone, Default)]
pub enum AfterTarget {
    #[default]
    End,
    /// After the previous ADD command in this batch
    Previous,
    /// After the task with this exact ID
    Id(String),
    /// After the first task containing this text
    Text(String),
    /// At a specific order index
    Line(usize),
}

/// Extended ADD command with positioning.
#[derive(Debug, Clone)]
pub struct AddCommand {
    pub task: Task,
    pub after: AfterTarget,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskUpdate {
    pub text: Option<String>,
    pub test: Option<String>,
    pub section: Option<String>,
    pub group: Option<String>,
}