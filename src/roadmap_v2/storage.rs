// src/roadmap_v2/storage.rs
use super::types::{RoadmapCommand, Task, TaskStatus, TaskStore, TaskUpdate};
use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::Path;

const DEFAULT_PATH: &str = "tasks.toml";

impl TaskStore {
    /// Loads the task store from disk.
    ///
    /// # Errors
    /// Returns error if file doesn't exist or TOML is invalid.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let path = path.unwrap_or(Path::new(DEFAULT_PATH));

        if !path.exists() {
            bail!("Task file not found: {}", path.display());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        toml::from_str(&content).with_context(|| format!("Invalid TOML in {}", path.display()))
    }

    /// Saves the task store to disk.
    ///
    /// # Errors
    /// Returns error if serialization or write fails.
    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let path = path.unwrap_or(Path::new(DEFAULT_PATH));

        let content = toml::to_string_pretty(self).context("Failed to serialize task store")?;

        fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;

        Ok(())
    }

    /// Applies a roadmap command to the store.
    ///
    /// # Errors
    /// Returns error if the operation is invalid (e.g., task not found).
    pub fn apply(&mut self, cmd: RoadmapCommand) -> Result<()> {
        match cmd {
            RoadmapCommand::Check { id } => self.set_status(&id, TaskStatus::Done),
            RoadmapCommand::Uncheck { id } => self.set_status(&id, TaskStatus::Pending),
            RoadmapCommand::Add(task) => self.add_task(task),
            RoadmapCommand::Update { id, fields } => self.update_task(&id, fields),
            RoadmapCommand::Delete { id } => self.delete_task(&id),
        }
    }

    fn set_status(&mut self, id: &str, status: TaskStatus) -> Result<()> {
        let task = self.find_task_mut(id)?;
        task.status = status;
        Ok(())
    }

    fn add_task(&mut self, task: Task) -> Result<()> {
        if self.tasks.iter().any(|t| t.id == task.id) {
            bail!("Task already exists: {}", task.id);
        }
        self.tasks.push(task);
        Ok(())
    }

    fn update_task(&mut self, id: &str, fields: TaskUpdate) -> Result<()> {
        let task = self.find_task_mut(id)?;

        if let Some(text) = fields.text {
            task.text = text;
        }
        if let Some(test) = fields.test {
            task.test = Some(test);
        }
        if let Some(section) = fields.section {
            task.section = section;
        }
        if let Some(group) = fields.group {
            task.group = Some(group);
        }

        Ok(())
    }

    fn delete_task(&mut self, id: &str) -> Result<()> {
        let idx = self
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| anyhow!("Task not found: {id}"))?;
        self.tasks.remove(idx);
        Ok(())
    }

    fn find_task_mut(&mut self, id: &str) -> Result<&mut Task> {
        self.tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| anyhow!("Task not found: {id}"))
    }
}
