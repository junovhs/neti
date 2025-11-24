use crate::types::ScanReport;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub struct App {
    pub report: ScanReport,
    pub selected_index: usize,
    pub running: bool,
    pub scroll_offset: u16,
}

impl App {
    #[must_use]
    pub fn new(report: ScanReport) -> Self {
        Self {
            report,
            selected_index: 0,
            running: true,
            scroll_offset: 0,
        }
    }

    /// Runs the TUI loop.
    ///
    /// # Errors
    ///
    /// Returns error if drawing to terminal fails or event polling errors.
    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> anyhow::Result<()> {
        let mut tick_count = 0;
        let tick_limit = 100_000; // Law of Paranoia: Loop Brake

        while self.running {
            // Loop Brake
            tick_count += 1;
            if tick_count > tick_limit {
                break;
            }

            terminal.draw(|f| crate::tui::view::draw(f, self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                        KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                        KeyCode::Down | KeyCode::Char('j') => self.move_down(),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn move_down(&mut self) {
        if !self.report.files.is_empty() && self.selected_index < self.report.files.len() - 1 {
            self.selected_index += 1;
        }
    }
}
