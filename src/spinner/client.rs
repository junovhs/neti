// src/spinner/client.rs
//! Client for sending updates to the spinner.

use super::safe_hud::SafeHud;
use super::state::HudState;

/// Handles updates to the HUD state.
#[derive(Clone)]
pub struct SpinnerClient {
    state: SafeHud,
}

impl SpinnerClient {
    #[must_use]
    pub fn new(state: SafeHud) -> Self {
        Self { state }
    }

    pub fn set_macro_step(&self, current: usize, total: usize, name: impl Into<String>) {
        let n = name.into();
        self.state.modify(|s| s.set_macro_step(current, total, n));
    }

    pub fn set_micro_status(&self, status: impl Into<String>) {
        let s = status.into();
        self.state.modify(|state| state.set_micro_status(s));
    }

    pub fn step_micro_progress(&self, current: usize, total: usize, status: impl Into<String>) {
        let s = status.into();
        self.state.modify(|state| state.step_micro_progress(current, total, s));
    }

    pub fn push_log(&self, line: &str) {
        let l = line.to_string();
        self.state.modify(|state| state.push_log(&l));
    }

    pub fn tick(&self) {
        self.state.modify(HudState::tick);
    }
}