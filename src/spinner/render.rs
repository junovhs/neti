// src/spinner/render.rs
//! HUD rendering logic.

use super::safe_hud::SafeHud;
use super::state::{HudSnapshot, ATOMIC_LINES};
use crossterm::{
    cursor, execute,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const INTERVAL: u64 = 80;
const DOTS: &[&str] = &["·", "··", "···"];

trait SimpleColor {
    fn cyan(&self) -> String;
    fn yellow(&self) -> String;
    fn green(&self) -> String;
    fn red(&self) -> String;
    fn blue(&self) -> String;
    fn bold(&self) -> String;
    fn dimmed(&self) -> String;
}

impl SimpleColor for str {
    fn cyan(&self) -> String {
        format!("\x1b[36m{self}\x1b[0m")
    }
    fn yellow(&self) -> String {
        format!("\x1b[33m{self}\x1b[0m")
    }
    fn green(&self) -> String {
        format!("\x1b[32m{self}\x1b[0m")
    }
    fn red(&self) -> String {
        format!("\x1b[31m{self}\x1b[0m")
    }
    fn blue(&self) -> String {
        format!("\x1b[34m{self}\x1b[0m")
    }
    fn bold(&self) -> String {
        format!("\x1b[1m{self}\x1b[0m")
    }
    fn dimmed(&self) -> String {
        format!("\x1b[2m{self}\x1b[0m")
    }
}

pub fn run_hud_loop(running: &Arc<AtomicBool>, hud: &SafeHud) {
    let mut frame_idx = 0;
    let mut stdout = io::stdout();
    let _ = execute!(stdout, cursor::Hide);

    let total_lines = 4 + ATOMIC_LINES;
    for _ in 0..total_lines {
        let _ = writeln!(stdout);
    }
    let height = u16::try_from(total_lines).unwrap_or(10);
    let _ = execute!(stdout, cursor::MoveUp(height));

    while running.load(Ordering::Relaxed) {
        render_frame(&mut stdout, &hud.snapshot(), frame_idx);
        thread::sleep(Duration::from_millis(INTERVAL));
        frame_idx += 1;
    }

    let _ = execute!(stdout, cursor::Show);
    let _ = clear_lines(total_lines);
    let (success, title, start) = hud.completion_info();
    print_final(success, &title, start.elapsed());
}

fn render_frame(stdout: &mut io::Stdout, snap: &HudSnapshot, frame_idx: usize) {
    let spinner = FRAMES.get(frame_idx % FRAMES.len()).unwrap_or(&"+");
    let elapsed = snap.start_time.elapsed().as_secs();
    let (term_width, _) = terminal::size().unwrap_or((80, 24));
    let max_w = (term_width as usize).saturating_sub(1);

    render_header(stdout, snap, spinner, elapsed);
    render_progress_bar(stdout, snap, elapsed);
    render_micro_status(stdout, snap, spinner, max_w);
    render_log_buffer(stdout, snap, max_w);

    let height = u16::try_from(4 + ATOMIC_LINES).unwrap_or(10);
    let _ = execute!(stdout, cursor::MoveUp(height));
}

fn render_header(stdout: &mut io::Stdout, snap: &HudSnapshot, spinner: &str, elapsed: u64) {
    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let header = match snap.pipeline_step {
        Some((step, total)) => {
            let prog = format!("[{step}/{total}]").dimmed();
            let step_name = truncate(&snap.step_name, 30);
            format!(
                "{} {} {prog} {}",
                spinner.blue(),
                snap.pipeline_title.blue().bold(),
                step_name.cyan()
            )
        }
        None => format!(
            "{} {} ({elapsed}s)",
            spinner.blue(),
            snap.pipeline_title.blue().bold()
        ),
    };
    let _ = writeln!(stdout, "{header}");
}

fn render_progress_bar(stdout: &mut io::Stdout, snap: &HudSnapshot, elapsed: u64) {
    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    if let Some((step, total)) = snap.pipeline_step {
        let width = 40;
        let safe_total = if total == 0 { 1 } else { total };
        let filled = ((step * width) / safe_total).min(width);
        let bar = format!(
            "{}{}",
            "━".repeat(filled).blue(),
            "─".repeat(width - filled).dimmed()
        );
        let _ = writeln!(stdout, "  {bar} {elapsed}s");
    } else {
        let _ = writeln!(stdout, "{}", "  │".dimmed());
    }
}

fn render_micro_status(stdout: &mut io::Stdout, snap: &HudSnapshot, spinner: &str, max_w: usize) {
    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let display = build_micro_display(snap, spinner, max_w);
    let _ = writeln!(stdout, "{display}");
    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let _ = writeln!(stdout, "{}", "     OUTPUT ::".dimmed());
}

fn build_micro_display(snap: &HudSnapshot, spinner: &str, max_w: usize) -> String {
    match snap.micro_progress {
        Some((curr, total)) if total > 0 => {
            let pct = (curr * 100) / total;
            let bar_w = 30;
            let filled = ((curr * bar_w) / total).min(bar_w);
            let bar = format!(
                "{}{}",
                "█".repeat(filled).yellow(),
                "░".repeat(bar_w - filled).dimmed()
            );
            let status = truncate(&snap.micro_status, max_w.saturating_sub(50).max(10));
            format!("  └─ {bar} {pct:>3}%  {status}")
        }
        _ => {
            let status = truncate(&snap.micro_status, max_w.saturating_sub(15).max(10));
            let dots = DOTS.get(snap.activity_tick % DOTS.len()).unwrap_or(&"·");
            format!("  └─ {} {status} {}", spinner.yellow(), dots.dimmed())
        }
    }
}

fn render_log_buffer(stdout: &mut io::Stdout, snap: &HudSnapshot, max_w: usize) {
    for i in 0..ATOMIC_LINES {
        let _ = execute!(stdout, Clear(ClearType::CurrentLine));
        let offset = ATOMIC_LINES.saturating_sub(snap.atomic_buffer.len());
        if i >= offset {
            let content = snap
                .atomic_buffer
                .get(i - offset)
                .map_or("", String::as_str);
            let _ = writeln!(
                stdout,
                "     {}",
                truncate(content, max_w.saturating_sub(6)).dimmed()
            );
        } else {
            let _ = writeln!(stdout);
        }
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    &s[..end]
}

fn clear_lines(count: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    for _ in 0..count {
        execute!(stdout, Clear(ClearType::CurrentLine))?;
        writeln!(stdout)?;
    }
    execute!(stdout, cursor::MoveUp(u16::try_from(count).unwrap_or(5)))
}

fn print_final(success: bool, title: &str, duration: Duration) {
    let icon = if success {
        "ok".green().bold()
    } else {
        "err".red().bold()
    };
    let time = format!("({:.1}s)", duration.as_secs_f64()).dimmed();
    println!("{icon} {title} {time}");
}