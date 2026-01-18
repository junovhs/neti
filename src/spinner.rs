// src/spinner.rs
use colored::Colorize;
use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType},
};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const INTERVAL: u64 = 80;
const ATOMIC_LINES: usize = 3;

/// A multi-level Head-Up Display for process execution.
///
/// Visual Structure:
/// [MACRO]  ⠋ Running cargo check... (12s)
/// [MICRO]  >> Compiling `slopchop_core` v1.6.0
/// [ATOMIC]    Compiling serde v1.0.197
///             Compiling libc v0.2.153
///             Checking memchr v2.7.1
#[derive(Clone)]
pub struct Spinner {
    running: Arc<AtomicBool>,
    /// Shared state for the HUD (title, status, log buffer).
    /// Protected by Mutex to allow safe updates from the main thread while the render thread reads it.
    state: Arc<Mutex<HudState>>,
    /// Handle to the rendering thread.
    /// Wrapped in Mutex<Option<_>> to allow `stop` to take ownership and join it.
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

struct HudState {
    title: String,
    micro_status: String,
    atomic_buffer: VecDeque<String>,
    start_time: Instant,
    final_success: Option<bool>,
}

impl HudState {
    fn snapshot(&self) -> (String, String, VecDeque<String>, Instant) {
        (
            self.title.clone(),
            self.micro_status.clone(),
            self.atomic_buffer.clone(),
            self.start_time,
        )
    }
}

impl Spinner {
    pub fn start(title: impl Into<String>) -> Self {
        let state = Arc::new(Mutex::new(HudState {
            title: title.into(),
            micro_status: "Starting...".to_string(),
            atomic_buffer: VecDeque::with_capacity(ATOMIC_LINES),
            start_time: Instant::now(),
            final_success: None,
        }));

        let running = Arc::new(AtomicBool::new(true));

        let r_clone = running.clone();
        let s_clone = state.clone();

        let handle = thread::spawn(move || {
            run_hud_loop(&r_clone, &s_clone);
        });

        Self {
            running,
            state,
            handle: Arc::new(Mutex::new(Some(handle))),
        }
    }

    pub fn push_log(&self, line: &str) {
        if let Ok(mut guard) = self.state.lock() {
            if guard.atomic_buffer.len() >= ATOMIC_LINES {
                guard.atomic_buffer.pop_front();
            }
            guard.atomic_buffer.push_back(line.to_string());

            if let Some(status) = extract_micro_status(line) {
                guard.micro_status = status;
            }
        }
    }

    pub fn stop(&self, success: bool) {
        // Pass success status to the thread via shared state
        if let Ok(mut guard) = self.state.lock() {
            guard.final_success = Some(success);
        }

        // Signal thread to stop
        if !self.running.swap(false, Ordering::Relaxed) {
            return;
        }

        // Wait for thread to finish cleanup
        if let Ok(mut guard) = self.handle.lock() {
            if let Some(h) = guard.take() {
                let _ = h.join();
            }
        }
    }
}

fn run_hud_loop(running: &Arc<AtomicBool>, state: &Arc<Mutex<HudState>>) {
    let mut frame_idx = 0;
    let mut stdout = io::stdout();

    // Hide cursor for the Matrix aesthetic
    let _ = execute!(stdout, cursor::Hide);

    // Reserve space
    let _ = writeln!(stdout); // Title
    let _ = writeln!(stdout); // Micro
    for _ in 0..ATOMIC_LINES { let _ = writeln!(stdout); }
    
    let height = u16::try_from(ATOMIC_LINES + 2).unwrap_or(5);
    let _ = execute!(stdout, cursor::MoveUp(height));

    while running.load(Ordering::Relaxed) {
        // Snapshot Pattern: Minimize lock time to prevent IO blocking from stalling the build
        let snapshot = if let Ok(guard) = state.lock() {
            Some(guard.snapshot())
        } else {
            None
        };

        if let Some((title, micro, atomic, start)) = snapshot {
            render_frame(&mut stdout, &title, &micro, &atomic, start, frame_idx);
        }
        
        thread::sleep(Duration::from_millis(INTERVAL));
        frame_idx += 1;
    }

    // Cleanup Phase (runs in thread to reduce CBO of Spinner struct)
    let _ = execute!(stdout, cursor::Show);
    
    if let Ok(guard) = state.lock() {
        let _ = clear_lines(ATOMIC_LINES + 2);
        // Default to false if not set (should not happen if stop is called correctly)
        let success = guard.final_success.unwrap_or(false);
        print_final_status(success, &guard.title, guard.start_time.elapsed());
    }
}

fn render_frame(
    stdout: &mut io::Stdout,
    title: &str,
    micro: &str,
    atomic: &VecDeque<String>,
    start: Instant,
    frame_idx: usize
) {
    let spinner = FRAMES.get(frame_idx % FRAMES.len()).unwrap_or(&"+");
    let elapsed = start.elapsed().as_secs();
    
    // 1. MACRO VIEW
    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let title_line = format!(
        "{spinner} {} ({elapsed}s)",
        title.cyan().bold()
    );
    let _ = writeln!(stdout, "{title_line}");

    // 2. MICRO VIEW
    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let micro_line = format!(
        "   {} {}",
        "›".yellow().bold(),
        micro.white()
    );
    let _ = writeln!(stdout, "{micro_line}");

    // 3. ATOMIC VIEW
    for i in 0..ATOMIC_LINES {
        let _ = execute!(stdout, Clear(ClearType::CurrentLine));
        // Use map_or and simpler closure
        let content = atomic.get(i).map_or("", String::as_str);
        
        // Truncate safely
        let trunc_len = 80;
        let safe_content = if content.len() > trunc_len {
            // Find char boundary to avoid panic
            let mut end = trunc_len;
            while !content.is_char_boundary(end) {
                end = end.saturating_sub(1);
            }
            &content[..end]
        } else {
            content
        };
        
        let _ = writeln!(stdout, "     {}", safe_content.dimmed());
    }

    let height = u16::try_from(ATOMIC_LINES + 2).unwrap_or(5);
    let _ = execute!(stdout, cursor::MoveUp(height));
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
    let icon = if success { "ok".green() } else { "err".red() };
    let time = format!("{}s", duration.as_secs()).dimmed();
    println!("   {icon} {title} ({time})");
}

fn extract_micro_status(line: &str) -> Option<String> {
    let trimmed = line.trim();
    
    // Cargo / Rust / Trunk patterns
    if trimmed.starts_with("Compiling") 
        || trimmed.starts_with("Checking")
        || trimmed.starts_with("Downloading")
        || trimmed.starts_with("Finished")
        || trimmed.starts_with("Building")
        || trimmed.starts_with("Bundling") {
        return Some(trimmed.to_string());
    }

    if trimmed.starts_with("Running") || trimmed.starts_with("Executing") {
        return Some(trimmed.to_string());
    }

    if trimmed.starts_with("Scanning") {
        return Some(trimmed.to_string());
    }

    if trimmed.contains("| LAW:") {
        // Use char pattern for split
        if let Some(path) = trimmed.split('|').next() {
            return Some(format!("Scanning {}", path.replace("FILE:", "").trim()));
        }
    }

    None
}