pub mod commands;
pub mod parser;
pub mod prompt;
pub mod cli; // This will hold the adapted logic from main.rs

pub use commands::{apply_commands, Command, CommandBatch, ApplyResult};
pub use parser::{Roadmap, Section, Task, TaskStatus};
pub use prompt::{generate_prompt, PromptOptions};
