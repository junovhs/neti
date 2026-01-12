// src/spinner.rs
use colored::Colorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// The "8-Point Orbit" frames requested.
const FRAMES: &[&str] = &["∙", "•", "∗", "✻", "✽", "✶", "+"];

const INTERVAL: u64 = 110;

pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    label: String,
}

impl Spinner {
    // Indexing is safe: i % FRAMES.len() is always < FRAMES.len()
    #[allow(clippy::indexing_slicing)]
    pub fn start(label: impl Into<String>) -> Self {
        let label = label.into();
        let running = Arc::new(AtomicBool::new(true));
        let r_clone = running.clone();
        let l_clone = label.clone();

        let handle = thread::spawn(move || {
            let mut i = 0;
            while r_clone.load(Ordering::Relaxed) {
                let frame = FRAMES[i % FRAMES.len()];
                // \r returns to start, \x1B[2K clears line to ensure no artifacts
                print!("\r\x1B[2K   {} {}", frame.cyan(), l_clone.dimmed());
                let _ = io::stdout().flush();
                thread::sleep(Duration::from_millis(INTERVAL));
                i += 1;
            }
        });

        Self {
            running,
            handle: Some(handle),
            label,
        }
    }

    pub fn stop(mut self, success: bool) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }

        let icon = if success {
            "ok".green().bold()
        } else {
            "err".red().bold()
        };

        // Final overwrite
        println!("\r\x1B[2K   {} {}", icon, self.label.dimmed());
    }
}
