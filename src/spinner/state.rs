// src/spinner/state.rs
//! HUD state management.

use std::collections::VecDeque;
use std::time::Instant;

pub const ATOMIC_LINES: usize = 5;

pub struct HudState {
    pipeline_title: String,
    pipeline_step: Option<(usize, usize)>,
    step_name: String,
    micro_status: String,
    micro_progress: Option<(usize, usize)>,
    atomic_buffer: VecDeque<String>,
    start_time: Instant,
    final_success: Option<bool>,
    activity_tick: usize,
}

pub struct HudSnapshot {
    pub pipeline_title: String,
    pub pipeline_step: Option<(usize, usize)>,
    pub step_name: String,
    pub micro_status: String,
    pub micro_progress: Option<(usize, usize)>,
    pub atomic_buffer: VecDeque<String>,
    pub start_time: Instant,
    pub activity_tick: usize,
}

impl Default for HudSnapshot {
    fn default() -> Self {
        Self {
            pipeline_title: String::new(),
            pipeline_step: None,
            step_name: String::new(),
            micro_status: String::new(),
            micro_progress: None,
            atomic_buffer: VecDeque::new(),
            start_time: Instant::now(),
            activity_tick: 0,
        }
    }
}

impl HudState {
    pub fn new(title: impl Into<String>) -> Self {
        let t = title.into();
        Self {
            pipeline_title: t.clone(),
            pipeline_step: None,
            step_name: t,
            micro_status: "Initializing...".to_string(),
            micro_progress: None,
            atomic_buffer: VecDeque::with_capacity(ATOMIC_LINES),
            start_time: Instant::now(),
            final_success: None,
            activity_tick: 0,
        }
    }

    pub fn set_macro_step(&mut self, current: usize, total: usize, name: String) {
        self.pipeline_step = Some((current, total));
        self.step_name = name;
        self.micro_progress = None;
        self.micro_status = "Starting...".to_string();
        self.activity_tick += 1;
    }

    pub fn set_micro_status(&mut self, status: String) {
        self.micro_status = status;
        self.micro_progress = None;
        self.activity_tick += 1;
    }

    pub fn step_micro_progress(&mut self, current: usize, total: usize, status: String) {
        self.micro_progress = Some((current, total));
        self.micro_status = status;
        self.activity_tick += 1;
    }

    pub fn push_log(&mut self, line: &str) {
        if self.atomic_buffer.len() >= ATOMIC_LINES {
            self.atomic_buffer.pop_front();
        }
        self.atomic_buffer.push_back(line.to_string());
        self.activity_tick += 1;

        if self.micro_progress.is_none() {
            if let Some(s) = extract_status(line) {
                self.micro_status = s;
            }
        }
    }

    pub fn tick(&mut self) {
        self.activity_tick += 1;
    }

    pub fn set_finished(&mut self, success: bool) {
        self.final_success = Some(success);
    }

    pub fn completion_info(&self) -> (bool, &str, Instant) {
        (
            self.final_success.unwrap_or(false),
            &self.pipeline_title,
            self.start_time,
        )
    }

    pub fn snapshot(&self) -> HudSnapshot {
        HudSnapshot {
            pipeline_title: self.pipeline_title.clone(),
            pipeline_step: self.pipeline_step,
            step_name: self.step_name.clone(),
            micro_status: self.micro_status.clone(),
            micro_progress: self.micro_progress,
            atomic_buffer: self.atomic_buffer.clone(),
            start_time: self.start_time,
            activity_tick: self.activity_tick,
        }
    }
}

fn extract_status(line: &str) -> Option<String> {
    let t = line.trim();
    if t.is_empty() {
        return None;
    }
    let prefixes = [
        "Compiling",
        "Checking",
        "Finished",
        "Downloading",
        "Running",
        "Building",
        "Scanning",
        "Analyzing",
    ];
    if prefixes.iter().any(|p| t.starts_with(p)) {
        return Some(t.to_string());
    }
    None
}