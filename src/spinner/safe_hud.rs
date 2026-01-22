// src/spinner/safe_hud.rs
//! Thread-safe wrapper for HUD state.

use super::state::{HudSnapshot, HudState};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Thread-safe wrapper around `HudState`.
/// Encapsulates locking to reduce coupling in consumers.
#[derive(Clone)]
pub struct SafeHud {
    /// Internal state protected by mutex.
    inner: Arc<Mutex<HudState>>,
}

impl SafeHud {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HudState::new(title))),
        }
    }

    /// Access the state with a closure for modification.
    /// This reduces API surface area (CBO) compared to delegating every method.
    pub fn modify<F>(&self, f: F)
    where
        F: FnOnce(&mut HudState),
    {
        if let Ok(mut guard) = self.inner.lock() {
            // Clippy fix: use auto-deref coercion instead of explicit `&mut *guard`
            f(&mut guard);
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> HudSnapshot {
        if let Ok(guard) = self.inner.lock() {
            guard.snapshot()
        } else {
            HudSnapshot::default()
        }
    }

    pub fn completion_info(&self) -> (bool, String, Instant) {
        if let Ok(guard) = self.inner.lock() {
            let (s, t, i) = guard.completion_info();
            (s, t.to_string(), i)
        } else {
            (false, String::new(), Instant::now())
        }
    }
}