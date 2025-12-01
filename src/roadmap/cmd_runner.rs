use crate::roadmap::cmd_handlers;
use crate::roadmap::types::{ApplyResult, Command, CommandBatch, Roadmap};

pub fn apply_commands(roadmap: &mut Roadmap, batch: &CommandBatch) -> Vec<ApplyResult> {
    batch
        .commands
        .iter()
        .map(|cmd| run_cmd(roadmap, cmd))
        .collect()
}

fn run_cmd(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Check { .. } | Command::Uncheck { .. } | Command::Delete { .. } => {
            handle_basic_cmd(roadmap, cmd)
        }
        Command::Add { .. } | Command::Update { .. } | Command::Note { .. } => {
            handle_content_cmd(roadmap, cmd)
        }
        Command::Move { .. } | Command::AddSection { .. } => handle_struct_cmd(roadmap, cmd),
        Command::ReplaceSection { .. } => ApplyResult::Error("Command not supported".into()),
    }
}

fn handle_basic_cmd(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Check { path } => cmd_handlers::handle_check(roadmap, path),
        Command::Uncheck { path } => cmd_handlers::handle_uncheck(roadmap, path),
        Command::Delete { path } => cmd_handlers::handle_delete(roadmap, path),
        _ => unreachable!(),
    }
}

fn handle_content_cmd(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Add {
            parent,
            text,
            after,
        } => cmd_handlers::handle_add(roadmap, parent, text, after.as_deref()),
        Command::Update { path, text } => cmd_handlers::handle_update(roadmap, path, text),
        Command::Note { path, note } => cmd_handlers::handle_note(roadmap, path, note),
        _ => unreachable!(),
    }
}

fn handle_struct_cmd(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::AddSection { heading } => cmd_handlers::handle_add_section(roadmap, heading),
        Command::Move { path, position } => cmd_handlers::handle_move(roadmap, path, position),
        _ => unreachable!(),
    }
}