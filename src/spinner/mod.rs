// src/spinner/mod.rs
//! Triptych HUD (Head-Up Display) for process execution feedback.

pub mod render;
pub mod state;

use state::HudState;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

/// A multi-level Head-Up Display for process execution.
#[derive(Clone)]
pub struct Spinner {
    /// Flag indicating whether the render loop should continue running.
    running: Arc<AtomicBool>,
    /// Shared HUD state protected by mutex for thread-safe updates.
    state: Arc<Mutex<HudState>>,
    /// Handle to the background rendering thread.
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl Spinner {
    pub fn start(title: impl Into<String>) -> Self {
        let state = Arc::new(Mutex::new(HudState::new(title)));
        let running = Arc::new(AtomicBool::new(true));

        let r_clone = running.clone();
        let s_clone = state.clone();

        let handle = thread::spawn(move || {
            render::run_hud_loop(&r_clone, &s_clone);
        });

        Self {
            running,
            state,
            handle: Arc::new(Mutex::new(Some(handle))),
        }
    }

    pub fn set_macro_step(&self, current: usize, total: usize, name: impl Into<String>) {
        if let Ok(mut guard) = self.state.lock() {
            guard.set_macro_step(current, total, name.into());
        }
    }

    pub fn set_micro_status(&self, status: impl Into<String>) {
        if let Ok(mut guard) = self.state.lock() {
            guard.set_micro_status(status.into());
        }
    }

    pub fn step_micro_progress(&self, current: usize, total: usize, status: impl Into<String>) {
        if let Ok(mut guard) = self.state.lock() {
            guard.step_micro_progress(current, total, status.into());
        }
    }

    pub fn push_log(&self, line: &str) {
        if let Ok(mut guard) = self.state.lock() {
            guard.push_log(line);
        }
    }

    pub fn tick(&self) {
        if let Ok(mut guard) = self.state.lock() {
            guard.tick();
        }
    }

    pub fn stop(&self, success: bool) {
        if let Ok(mut guard) = self.state.lock() {
            guard.set_finished(success);
        }

        if !self.running.swap(false, Ordering::Relaxed) {
            return;
        }

        if let Ok(mut guard) = self.handle.lock() {
            if let Some(h) = guard.take() {
                let _ = h.join();
            }
        }
    }
}
