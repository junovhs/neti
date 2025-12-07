// src/roadmap_v2/store.rs
use super::types::{RoadmapCommand, Task, TaskStatus, TaskStore, TaskUpdate};
use anyhow::Error;
use std::path::Path;

const DEFAULT_PATH: &str = "tasks.toml";

impl TaskStore {
    /// Load from tasks.toml (or default path).
    ///
    /// # Errors
    /// Returns error if file cannot be read or contains invalid TOML.
    pub fn load(path: Option<&Path>) -> Result<Self, SlopChopError> {
        let path = path.unwrap_or_else(|| Path::new(DEFAULT_PATH));

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;

        toml::from_str(&content)
            .map_err(|e| SlopChopError::Other(format!("Invalid tasks.toml: {e}")))
    }

    /// Save to tasks.toml.
    ///
    /// # Errors
    /// Returns error if serialization fails or file cannot be written.
    pub fn save(&self, path: Option<&Path>) -> Result<(), SlopChopError> {
        let path = path.unwrap_or_else(|| Path::new(DEFAULT_PATH));

        let content = toml::to_string_pretty(self)
            .map_err(|e| SlopChopError::Other(format!("Failed to serialize: {e}")))?;

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Apply a command to the store.
    ///
    /// # Errors
    /// Returns error if task not found or duplicate ID on add.
    pub fn apply(&mut self, cmd: RoadmapCommand) -> Result<(), SlopChopError> {
        match cmd {
            RoadmapCommand::Check { id } => self.set_status(&id, TaskStatus::Done),
            RoadmapCommand::Uncheck { id } => self.set_status(&id, TaskStatus::Pending),
            RoadmapCommand::Add(task) => self.add_task(task),
            RoadmapCommand::Update { id, fields } => self.update_task(&id, fields),
            RoadmapCommand::Delete { id } => self.delete_task(&id),
        }
    }

    fn set_status(&mut self, id: &str, status: TaskStatus) -> Result<(), SlopChopError> {
        let task = self.find_task_mut(id)?;
        task.status = status;
        Ok(())
    }

    fn add_task(&mut self, task: Task) -> Result<(), SlopChopError> {
        if self.tasks.iter().any(|t| t.id == task.id) {
            return Err(SlopChopError::Other(format!(
                "Task already exists: {}",
                task.id
            )));
        }
        self.tasks.push(task);
        Ok(())
    }

    fn update_task(&mut self, id: &str, fields: TaskUpdate) -> Result<(), SlopChopError> {
        let task = self.find_task_mut(id)?;

        if let Some(txt) = fields.text {
            task.text = txt;
        }
        if let Some(tst) = fields.test {
            task.test = Some(tst);
        }
        if let Some(sec) = fields.section {
            task.section = sec;
        }
        if let Some(grp) = fields.group {
            task.group = Some(grp);
        }

        Ok(())
    }

    fn delete_task(&mut self, id: &str) -> Result<(), SlopChopError> {
        let idx = self
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| SlopChopError::Other(format!("Task not found: {id}")))?;
        self.tasks.remove(idx);
        Ok(())
    }

    fn find_task_mut(&mut self, id: &str) -> Result<&mut Task, SlopChopError> {
        self.tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| SlopChopError::Other(format!("Task not found: {id}")))
    }
}
