// src/roadmap_v2/storage.rs
use super::types::{AddCommand, RoadmapCommand, Task, TaskStatus, TaskStore, TaskUpdate};
use super::validation;
use anyhow::{bail, Context, Result};
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

    /// Saves the task store to disk atomically (temp file + rename).
    ///
    /// # Errors
    /// Returns error if serialization or write fails.
    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let path = path.unwrap_or(Path::new(DEFAULT_PATH));
        let content = toml::to_string_pretty(self).context("Failed to serialize task store")?;

        atomic_write(path, &content)
    }

    /// Saves with backup of the original file.
    ///
    /// # Errors
    /// Returns error if backup or write fails.
    pub fn save_with_backup(&self, path: Option<&Path>) -> Result<()> {
        let path = path.unwrap_or(Path::new(DEFAULT_PATH));

        if path.exists() {
            create_backup(path)?;
        }

        self.save(Some(path))
    }

    /// Applies a roadmap command to the store.
    ///
    /// # Errors
    /// Returns error if the operation is invalid (e.g., task not found).
    pub fn apply(&mut self, cmd: RoadmapCommand) -> Result<()> {
        match cmd {
            RoadmapCommand::Check { id } => self.set_status(&id, TaskStatus::Done),
            RoadmapCommand::Uncheck { id } => self.set_status(&id, TaskStatus::Pending),
            RoadmapCommand::Add(add_cmd) => {
                self.add_task_positioned(add_cmd, None)?;
                Ok(())
            }
            RoadmapCommand::Update { id, fields } => self.update_task(&id, fields),
            RoadmapCommand::Delete { id } => self.delete_task(&id),
        }
    }

    /// Applies a batch of commands with pre-validation.
    ///
    /// # Errors
    /// Returns error if validation fails or any command fails.
    pub fn apply_batch(&mut self, commands: Vec<RoadmapCommand>) -> Result<BatchResult> {
        let report = validation::validate_batch(self, &commands);

        if !report.is_ok() {
            return Ok(BatchResult {
                applied: 0,
                errors: report.errors,
                warnings: report.warnings,
            });
        }

        let mut result = BatchResult {
            warnings: report.warnings,
            ..Default::default()
        };

        let mut last_order: Option<usize> = None;

        for cmd in commands {
            match self.apply_with_context(cmd, last_order) {
                Ok(order) => {
                    result.applied += 1;
                    last_order = order;
                }
                Err(e) => result.errors.push(e.to_string()),
            }
        }

        Ok(result)
    }

    fn apply_with_context(
        &mut self,
        cmd: RoadmapCommand,
        last_order: Option<usize>,
    ) -> Result<Option<usize>> {
        match cmd {
            RoadmapCommand::Add(add_cmd) => {
                let order = self.add_task_positioned(add_cmd, last_order)?;
                Ok(Some(order))
            }
            other => {
                self.apply(other)?;
                Ok(None)
            }
        }
    }

    fn set_status(&mut self, id: &str, status: TaskStatus) -> Result<()> {
        let task = self.find_task_mut(id)?;
        task.status = status;
        Ok(())
    }

    fn add_task_positioned(
        &mut self,
        add_cmd: AddCommand,
        last_order: Option<usize>,
    ) -> Result<usize> {
        if self.tasks.iter().any(|t| t.id == add_cmd.task.id) {
            bail!("Task already exists: {}", add_cmd.task.id);
        }

        let order = validation::resolve_position(self, &add_cmd, last_order)?;
        let mut task = add_cmd.task;
        task.order = order;

        self.tasks.push(task);
        Ok(order)
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
            .ok_or_else(|| anyhow::anyhow!("Task not found: {id}"))?;
        self.tasks.remove(idx);
        Ok(())
    }

    fn find_task_mut(&mut self, id: &str) -> Result<&mut Task> {
        self.tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| anyhow::anyhow!("Task not found: {id}"))
    }
}

/// Result of a batch apply operation.
#[derive(Debug, Default)]
pub struct BatchResult {
    pub applied: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let temp_path = path.with_extension("toml.tmp");

    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write temp file: {}", temp_path.display()))?;

    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename temp to {}", path.display()))?;

    Ok(())
}

fn create_backup(path: &Path) -> Result<()> {
    let backup_path = path.with_extension("toml.bak");

    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;

    Ok(())
}