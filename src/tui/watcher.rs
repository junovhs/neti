// src/tui/watcher.rs
use crate::clipboard;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

pub enum WatcherEvent {
    PayloadDetected(String),
}

/// Spawns a background thread to poll the clipboard.
pub fn spawn_watcher(tx: Sender<WatcherEvent>) {
    thread::spawn(move || {
        let mut last_content = String::new();

        loop {
            poll_clipboard(&tx, &mut last_content);
            thread::sleep(Duration::from_millis(500));
        }
    });
}

fn poll_clipboard(tx: &Sender<WatcherEvent>, last_content: &mut String) {
    if let Ok(content) = clipboard::read_clipboard() {
        if content != *last_content && is_slopchop_payload(&content) {
            last_content.clone_from(&content);
            let _ = tx.send(WatcherEvent::PayloadDetected(content));
        }
    }
}

fn is_slopchop_payload(text: &str) -> bool {
    text.contains("XSC7XSC PLAN XSC7XSC")
        || text.contains("XSC7XSC MANIFEST XSC7XSC")
        || text.contains("XSC7XSC FILE XSC7XSC")
        || text.contains("XSC7XSC PATCH XSC7XSC")
}