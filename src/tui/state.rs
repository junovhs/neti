// src/tui/state.rs
use crate::types::{FileReport, ScanReport};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Path,
    Tokens,
    Violations,
}

pub struct App {
    pub report: ScanReport,
    pub view_indices: Vec<usize>,
    pub selected_index: usize,
    pub running: bool,
    pub sort_mode: SortMode,
    pub only_violations: bool,
}

impl App {
    #[must_use]
    pub fn new(report: ScanReport) -> Self {
        let mut app = Self {
            report,
            view_indices: Vec::new(),
            selected_index: 0,
            running: true,
            sort_mode: SortMode::Path,
            only_violations: false,
        };
        app.update_view();
        app
    }

    fn update_view(&mut self) {
        let mut indices: Vec<usize> = self
            .report
            .files
            .iter()
            .enumerate()
            .filter(|(_, f)| !self.only_violations || !f.is_clean())
            .map(|(i, _)| i)
            .collect();

        self.sort_indices(&mut indices);
        self.view_indices = indices;
        self.clamp_selection();
    }

    fn sort_indices(&self, indices: &mut [usize]) {
        let files = &self.report.files;
        indices.sort_by(|&a, &b| {
            let f1 = &files[a];
            let f2 = &files[b];
            match self.sort_mode {
                SortMode::Path => f1.path.cmp(&f2.path),
                SortMode::Tokens => f2.token_count.cmp(&f1.token_count),
                SortMode::Violations => f2.violations.len().cmp(&f1.violations.len()),
            }
        });
    }

    fn clamp_selection(&mut self) {
        if self.view_indices.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.view_indices.len() {
            self.selected_index = self.view_indices.len() - 1;
        }
    }

    /// Runs TUI loop.
    /// # Errors
    /// Returns error on IO failure.
    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<()> {
        while self.running {
            terminal.draw(|f| crate::tui::view::draw(f, self))?;
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
        Ok(())
    }

    fn handle_input(&mut self, code: KeyCode) {
        if self.handle_nav(code) {
            return;
        }
        if self.handle_quit(code) {
            return;
        }
        self.handle_toggles(code);
    }

    fn handle_nav(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                true
            }
            _ => false,
        }
    }

    fn handle_quit(&mut self, code: KeyCode) -> bool {
        if matches!(code, KeyCode::Char('q') | KeyCode::Esc) {
            self.running = false;
            return true;
        }
        false
    }

    fn handle_toggles(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('s') => self.cycle_sort(),
            KeyCode::Char('f') => self.toggle_filter(),
            _ => {}
        }
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn move_down(&mut self) {
        if !self.view_indices.is_empty() && self.selected_index < self.view_indices.len() - 1 {
            self.selected_index += 1;
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Path => SortMode::Tokens,
            SortMode::Tokens => SortMode::Violations,
            SortMode::Violations => SortMode::Path,
        };
        self.update_view();
    }

    fn toggle_filter(&mut self) {
        self.only_violations = !self.only_violations;
        self.update_view();
    }

    #[must_use]
    pub fn get_selected_file(&self) -> Option<&FileReport> {
        if let Some(&real_index) = self.view_indices.get(self.selected_index) {
            self.report.files.get(real_index)
        } else {
            None
        }
    }
}
