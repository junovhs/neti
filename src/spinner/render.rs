// src/spinner/render.rs
use super::state::{HudSnapshot, ATOMIC_LINES};
use crossterm::{cursor, execute, terminal::{self, Clear, ClearType}};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const INTERVAL: u64 = 80;

#[allow(dead_code)]
trait SimpleColor {
    fn cyan(&self) -> String;
    fn yellow(&self) -> String;
    fn green(&self) -> String;
    fn red(&self) -> String;
    fn white(&self) -> String;
    fn blue(&self) -> String;
    fn bold(&self) -> String;
    fn dimmed(&self) -> String;
}

impl SimpleColor for str {
    fn cyan(&self) -> String { format!("\x1b[36m{self}\x1b[0m") }
    fn yellow(&self) -> String { format!("\x1b[33m{self}\x1b[0m") }
    fn green(&self) -> String { format!("\x1b[32m{self}\x1b[0m") }
    fn red(&self) -> String { format!("\x1b[31m{self}\x1b[0m") }
    fn white(&self) -> String { format!("\x1b[37m{self}\x1b[0m") }
    fn blue(&self) -> String { format!("\x1b[34m{self}\x1b[0m") }
    fn bold(&self) -> String { format!("\x1b[1m{self}\x1b[0m") }
    fn dimmed(&self) -> String { format!("\x1b[2m{self}\x1b[0m") }
}

pub fn run_hud_loop(running: &Arc<AtomicBool>, state: &Arc<Mutex<super::state::HudState>>) {
    let mut frame_idx = 0;
    let mut stdout = io::stdout();

    let _ = execute!(stdout, cursor::Hide);

    let total_lines = 4 + ATOMIC_LINES;
    
    for _ in 0..total_lines { let _ = writeln!(stdout); }
    let height = u16::try_from(total_lines).unwrap_or(10);
    let _ = execute!(stdout, cursor::MoveUp(height));

    while running.load(Ordering::Relaxed) {
        let snapshot = if let Ok(guard) = state.lock() {
            Some(guard.snapshot())
        } else {
            None
        };

        if let Some(snap) = snapshot {
            render_hud(&mut stdout, &snap, frame_idx);
        }
        
        thread::sleep(Duration::from_millis(INTERVAL));
        frame_idx += 1;
    }

    let _ = execute!(stdout, cursor::Show);
    
    if let Ok(guard) = state.lock() {
        let _ = clear_lines(total_lines);
        let (success, title, start) = guard.completion_info();
        print_final_status(success, title, start.elapsed());
    }
}

fn render_hud(stdout: &mut io::Stdout, snap: &HudSnapshot, frame_idx: usize) {
    let spinner = FRAMES.get(frame_idx % FRAMES.len()).unwrap_or(&"+");
    let elapsed = snap.start_time.elapsed().as_secs();
    
    // Get terminal width to prevent wrapping
    let (term_width, _) = terminal::size().unwrap_or((80, 24));
    let max_width = (term_width as usize).saturating_sub(1);

    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    
    let macro_text = if let Some((step, total)) = snap.pipeline_step {
        let width = 30;
        // Integer math for progress bar to avoid precision loss
        let safe_total = if total == 0 { 1 } else { total };
        // We don't display the percentage for Macro, just the step count, but we need filled width
        let filled = (step * width) / safe_total;
        
        let filled_len = filled.min(width);
        let empty_len = width.saturating_sub(filled_len);
        
        let bar = format!("{}{}", "━".repeat(filled_len).blue().bold(), "━".repeat(empty_len).dimmed());
        
        let prefix_len = 55; // Approx length of prefix chars
        let avail = max_width.saturating_sub(prefix_len).max(10);
        let safe_name = truncate_safe(&snap.pipeline_name, avail);

        format!(
            "{} [{bar}] Step {step}/{total}: {}",
            "SLOPCHOP".blue().bold(),
            safe_name.bold()
        )
    } else {
        let prefix_len = 25;
        let avail = max_width.saturating_sub(prefix_len).max(10);
        let safe_name = truncate_safe(&snap.pipeline_name, avail);

        format!(
            "{} {spinner} {} ({elapsed}s)",
            "SLOPCHOP".blue().bold(),
            safe_name
        )
    };
    let _ = writeln!(stdout, "{macro_text}");

    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let _ = writeln!(stdout, "{}", "  │".dimmed());

    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    
    let micro_display = if let Some((curr, total)) = snap.micro_progress {
        if total > 0 {
            // Integer math for Micro progress
            let pct = (curr * 100) / total;
            let bar_width = 40;
            let filled = (curr * bar_width) / total;
            
            let filled_len = filled.min(bar_width);
            let empty_len = bar_width.saturating_sub(filled_len);
            
            let bar = format!("{}{}", "█".repeat(filled_len).yellow(), "░".repeat(empty_len).dimmed());
            
            let prefix_len = 55;
            let avail = max_width.saturating_sub(prefix_len).max(10);
            let safe_status = truncate_safe(&snap.micro_status, avail);

            // Fix clippy::uninlined_format_args
            format!("  └─ {bar} {pct}%  {safe_status}")
        } else {
            let avail = max_width.saturating_sub(10).max(10);
            let safe_status = truncate_safe(&snap.micro_status, avail);
            format!("  └─ {} {safe_status}", spinner.yellow())
        }
    } else {
        let avail = max_width.saturating_sub(10).max(10);
        let safe_status = truncate_safe(&snap.micro_status, avail);
        format!("  └─ {} {safe_status}", spinner.yellow())
    };
    let _ = writeln!(stdout, "{micro_display}");

    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let _ = writeln!(stdout, "{}", "     TYPE STREAM ::".dimmed());

    for i in 0..ATOMIC_LINES {
        let _ = execute!(stdout, Clear(ClearType::CurrentLine));
        
        let idx_in_buf = if snap.atomic_buffer.len() < ATOMIC_LINES {
             if i < (ATOMIC_LINES - snap.atomic_buffer.len()) {
                 None
             } else {
                 Some(i - (ATOMIC_LINES - snap.atomic_buffer.len()))
             }
        } else {
             Some(i)
        };

        if let Some(idx) = idx_in_buf {
            let content = snap.atomic_buffer.get(idx).map_or("", String::as_str);
            let safe_content = truncate_safe(content, max_width.saturating_sub(6));
            let _ = writeln!(stdout, "     {}", safe_content.dimmed());
        } else {
            let _ = writeln!(stdout);
        }
    }

    let height = u16::try_from(4 + ATOMIC_LINES).unwrap_or(10);
    let _ = execute!(stdout, cursor::MoveUp(height));
}

fn truncate_safe(s: &str, max: usize) -> &str {
    if s.len() <= max { return s; }
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
    let height = u16::try_from(count).unwrap_or(5);
    execute!(stdout, cursor::MoveUp(height))?;
    Ok(())
}

fn print_final_status(success: bool, title: &str, duration: Duration) {
    let icon = if success { "ok".green().bold() } else { "err".red().bold() };
    let time = format!("{}s", duration.as_secs()).dimmed();
    println!("{icon} {title} ({time})");
}