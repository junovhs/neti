// src/spinner.rs
use colored::Colorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const FRAMES: &[&str] = &["�", "", "*", "?", "?", "?", "+"];
const INTERVAL: u64 = 80; // Speed up animation

#[derive(Clone)]
pub struct Spinner {
    running: Arc<AtomicBool>,
    /// Thread-safe label for the spinner animation.
    label: Arc<Mutex<String>>,
    // Use Arc<Mutex<Option>> for handle to allow Clone, though only the creator can join
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl Spinner {
    pub fn start(label: impl Into<String>) -> Self {
        let label_mtx = Arc::new(Mutex::new(label.into()));
        let running = Arc::new(AtomicBool::new(true));
        
        let r_clone = running.clone();
        let l_clone = label_mtx.clone();

        let handle = thread::spawn(move || {
            let mut i = 0;
            while r_clone.load(Ordering::Relaxed) {
                // Use .get() to be safe, though modulo guarantees bounds
                let frame = FRAMES.get(i % FRAMES.len()).unwrap_or(&"+");
                
                // Get label safely
                let text = l_clone.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                
                // Truncate if too long to prevent wrapping weirdness
                let display_text = if text.len() > 60 { &text[..60] } else { &text };
                
                print!("\r\x1B[2K   {} {}", frame.cyan(), display_text.dimmed());
                let _ = io::stdout().flush();
                thread::sleep(Duration::from_millis(INTERVAL));
                i += 1;
            }
        });

        Self {
            running,
            label: label_mtx,
            handle: Arc::new(Mutex::new(Some(handle))),
        }
    }

    pub fn set_message(&self, msg: impl Into<String>) {
        if let Ok(mut guard) = self.label.lock() {
            *guard = msg.into();
        }
    }

    pub fn stop(&self, success: bool) {
        if !self.running.swap(false, Ordering::Relaxed) {
            return; // Already stopped
        }
        
        // Wait for thread
        if let Ok(mut guard) = self.handle.lock() {
            if let Some(h) = guard.take() {
                let _ = h.join();
            }
        }

        let icon = if success { "ok".green().bold() } else { "err".red().bold() };
        let text = self.label.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        println!("\r\x1B[2K   {} {}", icon, text.dimmed());
    }
}