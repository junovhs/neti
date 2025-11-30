// src/tui/config_state.rs
use crate::config::{save_to_file, Config, RuleConfig};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use std::collections::HashMap;
use std::time::Duration;

pub struct ConfigApp {
    pub rules: RuleConfig,
    pub commands: HashMap<String, Vec<String>>,
    /// 0 = Preset, 1..5 = Rules
    pub selected_field: usize,
    pub running: bool,
    pub modified: bool,
    pub saved_message: Option<(String, std::time::Instant)>,
}

impl Default for ConfigApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigApp {
    #[must_use]
    pub fn new() -> Self {
        let mut config = Config::new();
        config.load_local_config();

        Self {
            rules: config.rules,
            commands: config.commands,
            selected_field: 0,
            running: true,
            modified: false,
            saved_message: None,
        }
    }

    /// Runs the config TUI loop.
    ///
    /// # Errors
    /// Returns error if terminal IO or event polling fails.
    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<()> {
        while self.running {
            terminal.draw(|f| crate::tui::config_view::draw(f, self))?;
            self.process_event()?;
        }
        Ok(())
    }

    fn process_event(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                self.handle_input(key.code);
            }
        }
        self.check_message_expiry();
        Ok(())
    }

    fn check_message_expiry(&mut self) {
        if let Some((_, time)) = self.saved_message {
            if time.elapsed() > Duration::from_secs(2) {
                self.saved_message = None;
            }
        }
    }

    fn handle_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Left | KeyCode::Char('h') => self.decrement_val(),
            KeyCode::Right | KeyCode::Char('l') => self.increment_val(),
            KeyCode::Enter | KeyCode::Char('s') => self.save(),
            _ => {}
        }
    }

    fn move_up(&mut self) {
        if self.selected_field > 0 {
            self.selected_field -= 1;
        } else {
            self.selected_field = 5;
        }
    }

    fn move_down(&mut self) {
        if self.selected_field < 5 {
            self.selected_field += 1;
        } else {
            self.selected_field = 0;
        }
    }

    fn increment_val(&mut self) {
        self.modified = true;
        match self.selected_field {
            0 => self.cycle_preset(true),
            1 => self.rules.max_file_tokens += 100,
            2 => self.rules.max_cyclomatic_complexity += 1,
            3 => self.rules.max_nesting_depth += 1,
            4 => self.rules.max_function_args += 1,
            5 => self.rules.max_function_words += 1,
            _ => {}
        }
    }

    fn decrement_val(&mut self) {
        self.modified = true;
        match self.selected_field {
            0 => self.cycle_preset(false),
            1 => {
                self.rules.max_file_tokens =
                    self.rules.max_file_tokens.saturating_sub(100).max(100);
            }
            2 => {
                self.rules.max_cyclomatic_complexity =
                    self.rules.max_cyclomatic_complexity.saturating_sub(1).max(1);
            }
            3 => {
                self.rules.max_nesting_depth =
                    self.rules.max_nesting_depth.saturating_sub(1).max(1);
            }
            4 => {
                self.rules.max_function_args =
                    self.rules.max_function_args.saturating_sub(1).max(1);
            }
            5 => {
                self.rules.max_function_words =
                    self.rules.max_function_words.saturating_sub(1).max(1);
            }
            _ => {}
        }
    }

    fn cycle_preset(&mut self, forward: bool) {
        // Simple 3-state cycle based on token count as a heuristic
        let current = if self.rules.max_file_tokens <= 1500 {
            0 // Strict
        } else if self.rules.max_file_tokens <= 2000 {
            1 // Standard
        } else {
            2 // Relaxed
        };

        let next = if forward {
            (current + 1) % 3
        } else {
            (current + 2) % 3
        };

        match next {
            0 => self.apply_preset(1500, 4, 2),  // Strict
            1 => self.apply_preset(2000, 8, 3),  // Standard
            2 => self.apply_preset(3000, 12, 4), // Relaxed
            _ => {}
        }
    }

    fn apply_preset(&mut self, tokens: usize, complexity: usize, depth: usize) {
        self.rules.max_file_tokens = tokens;
        self.rules.max_cyclomatic_complexity = complexity;
        self.rules.max_nesting_depth = depth;
        // Keep args/words standard across presets for now, or tune them too
        self.rules.max_function_args = 5;
        self.rules.max_function_words = 5;
    }

    fn save(&mut self) {
        if let Err(e) = save_to_file(&self.rules, &self.commands) {
            self.saved_message = Some((format!("Error: {e}"), std::time::Instant::now()));
        } else {
            self.saved_message =
                Some(("Saved warden.toml!".to_string(), std::time::Instant::now()));
            self.modified = false;
        }
    }

    // --- View Helpers ---

    #[must_use]
    pub fn get_active_label(&self) -> &'static str {
        match self.selected_field {
            0 => "GLOBAL PROTOCOL",
            1 => "LAW OF ATOMICITY",
            2..=4 => "LAW OF COMPLEXITY",
            5 => "LAW OF BLUNTNESS",
            _ => "UNKNOWN",
        }
    }

    #[must_use]
    pub fn get_active_description(&self) -> &'static str {
        match self.selected_field {
            0 => "Select a predefined security clearance level.\n\nStrict: Greenfield/Critical systems.\nStandard: Recommended balance.\nRelaxed: Legacy containment.",
            1 => "Limits file size. Large files confuse AI context windows and make verification impossible. \n\nGoal: Modular, atomic units.",
            2 => "Limits control flow paths. High complexity increases hallucination rates and makes code untestable.\n\nGoal: Linear, obvious logic.",
            3 => "Limits indentation. Deep nesting causes AI to lose scope tracking and context.\n\nGoal: Shallow, flat structures.",
            4 => "Limits function inputs. Too many arguments suggests a missing struct or mixed concerns.\n\nGoal: Clean interfaces.",
            5 => "Limits function naming verbosity. Long names often mask poor abstraction.\n\nGoal: Concise intent.",
            _ => "",
        }
    }

    #[must_use]
    pub fn detect_preset(&self) -> &'static str {
        if self.rules.max_file_tokens <= 1500 && self.rules.max_cyclomatic_complexity <= 4 {
             "STRICT"
        } else if self.rules.max_file_tokens >= 3000 {
             "RELAXED"
        } else if self.rules.max_file_tokens == 2000 {
             "STANDARD"
        } else {
             "CUSTOM"
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_containment_integrity(&self) -> f64 {
        // Calculate an aggregate "Integrity Score" based on all metrics.
        // Lower values = Higher Integrity (closer to strict limits)
        // We invert this for the gauge: 1.0 = Strict, 0.0 = Relaxed.

        let t_score = (self.rules.max_file_tokens as f64 - 1000.0) / 3000.0; // 1500->0.16, 3000->0.66
        let c_score = (self.rules.max_cyclomatic_complexity as f64 - 1.0) / 15.0;
        let d_score = (self.rules.max_nesting_depth as f64 - 1.0) / 5.0;

        let raw_avg = (t_score + c_score + d_score) / 3.0;
        
        // Invert: 1.0 minus the "looseness"
        (1.0 - raw_avg).clamp(0.0, 1.0)
    }
}