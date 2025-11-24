pub mod components;
pub mod layout;

use crate::tui::state::App;
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    layout::render_dashboard(f, app, area);
}
