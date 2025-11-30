use crate::clipboard;
use crate::roadmap::{
    apply_commands, generate_prompt, CommandBatch, PromptOptions, Roadmap, TaskStatus,
};
use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Subcommand, Debug, Clone)]
pub enum RoadmapCommand {
    Init {
        #[arg(short, long, default_value = "ROADMAP.md")]
        output: PathBuf,
        #[arg(short, long)]
        name: Option<String>,
    },
    Prompt {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long)]
        full: bool,
        #[arg(long)]
        examples: bool,
        #[arg(long)]
        stdout: bool,
    },
    Apply {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        stdin: bool,
        #[arg(short, long)]
        verbose: bool,
    },
    Show {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long, default_value = "tree")]
        format: String,
    },
    Tasks {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long)]
        pending: bool,
        #[arg(long)]
        complete: bool,
    },
}

/// Entry point for roadmap commands
/// # Errors
/// Returns error if IO fails or clipboard access fails
pub fn handle_command(cmd: RoadmapCommand) -> Result<()> {
    match cmd {
        RoadmapCommand::Init { output, name } => run_init(&output, name),
        RoadmapCommand::Prompt {
            file,
            full,
            examples,
            stdout,
        } => run_prompt(&file, full, examples, stdout),
        RoadmapCommand::Apply {
            file,
            dry_run,
            stdin,
            verbose,
        } => run_apply(&file, dry_run, stdin, verbose),
        RoadmapCommand::Show { file, format } => run_show(&file, &format),
        RoadmapCommand::Tasks {
            file,
            pending,
            complete,
        } => run_tasks(&file, pending, complete),
    }
}

fn run_init(output: &Path, name: Option<String>) -> Result<()> {
    if output.exists() {
        return Err(anyhow!(
            "{} already exists. Use --output.",
            output.display()
        ));
    }
    let n = name.unwrap_or_else(|| "Project".to_string());
    std::fs::write(output, template(&n))?;
    println!("✓ Created {}", output.display());
    Ok(())
}

fn run_prompt(file: &Path, full: bool, examples: bool, stdout: bool) -> Result<()> {
    let r = load(file)?;
    let p = generate_prompt(
        &r,
        &PromptOptions {
            full,
            examples,
            project_name: None,
        },
    );
    if stdout {
        println!("{p}");
    } else {
        clipboard::smart_copy(&p).map_err(|e| anyhow!("Clipboard: {e}"))?;
        println!("✓ Copied prompt.");
    }
    Ok(())
}

fn run_apply(file: &Path, dry_run: bool, stdin: bool, verbose: bool) -> Result<()> {
    let mut roadmap = load(file)?;
    let input = get_input(stdin)?;
    let batch = CommandBatch::parse(&input);

    if batch.commands.is_empty() {
        print_errs(&batch.errors);
        return Err(anyhow!("No commands found."));
    }

    println!(
        "Found {} commands: {}",
        batch.commands.len(),
        batch.summary()
    );
    if verbose {
        print_errs(&batch.errors);
    }

    if dry_run {
        println!("[DRY RUN]");
        return Ok(());
    }

    let results = apply_commands(&mut roadmap, &batch);
    if results
        .iter()
        .any(|r| matches!(r, crate::roadmap::ApplyResult::Success(_)))
    {
        roadmap.save(file)?;
        println!("✓ Saved.");
    }
    for r in &results {
        println!("{r}");
    }
    Ok(())
}

fn run_show(file: &Path, format: &str) -> Result<()> {
    let r = load(file)?;
    if format == "stats" {
        let s = r.stats();
        println!(
            "Tasks: {} ({} done, {} pending)",
            s.total, s.complete, s.pending
        );
    } else {
        println!("{}", r.compact_state());
    }
    Ok(())
}

fn run_tasks(file: &Path, pending: bool, complete: bool) -> Result<()> {
    let r = load(file)?;
    for t in r.all_tasks() {
        if should_show_task(t.status, pending, complete) {
            let mark = if t.status == TaskStatus::Complete {
                "[x]"
            } else {
                "[ ]"
            };
            println!("{mark} {} - {}", t.path, t.text);
        }
    }
    Ok(())
}

fn should_show_task(status: TaskStatus, pending: bool, complete: bool) -> bool {
    match (pending, complete) {
        (true, false) => status == TaskStatus::Pending,
        (false, true) => status == TaskStatus::Complete,
        _ => true,
    }
}

fn load(path: &Path) -> Result<Roadmap> {
    Roadmap::from_file(path).context("Load failed")
}

fn get_input(stdin: bool) -> Result<String> {
    if stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        clipboard::read_clipboard().context("Clipboard read failed")
    }
}

fn print_errs(errors: &[String]) {
    for e in errors {
        eprintln!("Warning: {e}");
    }
}

fn template(name: &str) -> String {
    format!("# {name} Roadmap\n\n## v0.1.0\n\n- [ ] Init\n\n## v0.2.0\n\n## v0.3.0\n")
}
