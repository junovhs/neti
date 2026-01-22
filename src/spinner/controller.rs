// src/spinner/controller.rs
//! Lifecycle controller for the spinner thread.

use super::handle::SpinnerHandle;
use super::safe_hud::SafeHud;

/// Manages the spinner background thread.
pub struct SpinnerController {
    handle: Option<SpinnerHandle>,
    state: SafeHud,
}

impl SpinnerController {
    #[must_use]
    pub fn new(handle: SpinnerHandle, state: SafeHud) -> Self {
        Self {
            handle: Some(handle),
            state,
        }
    }

    pub fn stop(&mut self, success: bool) {
        self.state.modify(|state| state.set_finished(success));

        if let Some(h) = self.handle.take() {
            h.stop();
        }
    }
}