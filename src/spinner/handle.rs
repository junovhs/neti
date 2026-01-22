// src/spinner/handle.rs
//! Thread management for the spinner.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use super::render;
use super::safe_hud::SafeHud;

/// Handle for controlling the spinner thread.
pub struct SpinnerHandle {
    handle: thread::JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl SpinnerHandle {
    #[must_use]
    pub fn spawn(hud: SafeHud) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let r_clone = running.clone();
        
        let handle = thread::spawn(move || {
            render::run_hud_loop(&r_clone, &hud);
        });

        Self { handle, running }
    }

    pub fn stop(self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.handle.join();
    }
}