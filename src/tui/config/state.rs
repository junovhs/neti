// src/tui/config/state.rs
use super::helpers;
use super::view;
use crate::config::{save_to_file, Config, Preferences, RuleConfig};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use std::collections::HashMap;
use std::time::Duration;

pub struct ConfigApp {
    pub rules: RuleConfig,
    pub preferences: Preferences,
    pub commands: HashMap<String, Vec<String>>,
    // 0=Preset, 1-5=Rules, 6-9=Workflow, 10=Theme, 11=Progress
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
            preferences: config.preferences,
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
            terminal.draw(|f| view::draw(f, self))?;
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
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1),
            KeyCode::Left | KeyCode::Char('h') => self.adjust_value(false),
            KeyCode::Right | KeyCode::Char('l') => self.adjust_value(true),
            KeyCode::Enter | KeyCode::Char('s') => self.save(),
            _ => {}
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    fn move_cursor(&mut self, delta: i32) {
        let new_pos = (self.selected_field as i32) + delta;
        if new_pos < 0 {
            self.selected_field = 11;
        } else if new_pos > 11 {
            self.selected_field = 0;
        } else {
            self.selected_field = new_pos as usize;
        }
    }

    fn adjust_value(&mut self, increase: bool) {
        self.modified = true;
        match self.selected_field {
            0 => helpers::cycle_preset(self, increase),
            1..=5 => helpers::adjust_rule(self, increase),
            6..=11 => helpers::adjust_pref(self, increase),
            _ => {}
        }
    }

    fn save(&mut self) {
        if let Err(e) = save_to_file(&self.rules, &self.preferences, &self.commands) {
            self.saved_message = Some((format!("Error: {e}"), std::time::Instant::now()));
        } else {
            self.saved_message = Some((
                "Saved slopchop.toml!".to_string(),
                std::time::Instant::now(),
            ));
            self.modified = false;
        }
    }
}
