// src/spinner/mod.rs
//! Triptych HUD (Head-Up Display) for process execution feedback.

pub mod client;
pub mod controller;
pub mod render;
pub mod safe_hud;
pub mod state;
mod handle;

pub use client::SpinnerClient;
pub use controller::SpinnerController;

use handle::SpinnerHandle;
use safe_hud::SafeHud;

/// Starts the spinner and returns the client (for updates) and controller (for lifecycle).
///
/// This split ensures high cohesion: the client is passed to workers, while the
/// controller is held by the main thread to manage the spinner's lifetime.
#[must_use]
pub fn start(title: impl Into<String>) -> (SpinnerClient, SpinnerController) {
    let safe_hud = SafeHud::new(title);
    let handle = SpinnerHandle::spawn(safe_hud.clone());

    let client = SpinnerClient::new(safe_hud.clone());
    let controller = SpinnerController::new(handle, safe_hud);

    (client, controller)
}