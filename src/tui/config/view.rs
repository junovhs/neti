// src/tui/config/view.rs
use super::components;
use super::state::ConfigApp;
use super::status_panel;
use crate::config::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use ratatui::Frame;

pub struct Palette {
    pub primary: Color,
    pub secondary: Color,
    pub text: Color,
    pub bg: Color,
    pub highlight: Color,
}

fn get_palette(theme: Theme) -> Palette {
    match theme {
        Theme::Nasa => Palette {
            primary: Color::Cyan,
            secondary: Color::Blue,
            text: Color::White,
            bg: Color::Black,
            highlight: Color::Cyan,
        },
        Theme::Cyberpunk => Palette {
            primary: Color::Magenta,
            secondary: Color::Cyan,
            text: Color::Green,
            bg: Color::Black,
            highlight: Color::Magenta,
        },
        Theme::Corporate => Palette {
            primary: Color::White,
            secondary: Color::Gray,
            text: Color::Gray,
            bg: Color::Black,
            highlight: Color::White,
        },
    }
}

pub fn draw(f: &mut Frame, app: &ConfigApp) {
    let pal = get_palette(app.preferences.theme);
    let area = f.area();

    let block = Block::default().style(Style::default().bg(pal.bg));
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    components::draw_header(f, app, chunks[0], &pal);
    draw_main(f, app, chunks[1], &pal);
    components::draw_footer(f, chunks[2], &pal);
}

/// Renders just the main content (Settings + Context), skipping header/footer.
/// Used for embedding in Dashboard.
pub fn draw_embed(f: &mut Frame, app: &ConfigApp, area: Rect) {
    let pal = get_palette(app.preferences.theme);
    draw_main(f, app, area, &pal);
}

fn draw_main(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    components::draw_settings_table(f, app, layout[0], pal);
    status_panel::draw(f, app, layout[1], pal);
}