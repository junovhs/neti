use crate::roadmap::cmd_handlers;
use crate::roadmap::types::{ApplyResult, Command, CommandBatch, Roadmap};

pub fn apply_commands(roadmap: &mut Roadmap, batch: &CommandBatch) -> Vec<ApplyResult> {
    batch.commands.iter().flat_map(|cmd| run_cmd(roadmap, cmd)).collect()
}

fn run_cmd(roadmap: &mut Roadmap, cmd: &Command) -> Vec<ApplyResult> {
    match cmd {
        Command::Chain { parent, items } => expand_chain(roadmap, parent, items),
        _ => vec![run_single(roadmap, cmd)],
    }
}

fn run_single(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Check { path } => cmd_handlers::handle_check(roadmap, path),
        Command::Uncheck { path } => cmd_handlers::handle_uncheck(roadmap, path),
        Command::Delete { path } => cmd_handlers::handle_delete(roadmap, path),
        Command::Add { parent, text, after } => {
            cmd_handlers::handle_add(roadmap, parent, text, after.as_deref())
        }
        _ => run_single_ext(roadmap, cmd),
    }
}

fn run_single_ext(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Update { path, text } => cmd_handlers::handle_update(roadmap, path, text),
        Command::Note { path, note } => cmd_handlers::handle_note(roadmap, path, note),
        Command::AddSection { heading } => cmd_handlers::handle_add_section(roadmap, heading),
        Command::AddSubsection { parent, heading } => {
            cmd_handlers::handle_add_subsection(roadmap, parent, heading)
        }
        Command::Move { path, position } => cmd_handlers::handle_move(roadmap, path, position),
        Command::ReplaceSection { .. } => ApplyResult::Error("Not supported".into()),
        _ => ApplyResult::Error("Unknown command".into()),
    }
}

fn expand_chain(roadmap: &mut Roadmap, parent: &str, items: &[String]) -> Vec<ApplyResult> {
    items
        .iter()
        .map(|text| cmd_handlers::handle_add(roadmap, parent, text, None))
        .collect()
}