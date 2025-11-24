use crate::tui::state::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.area());

    draw_file_list(f, app, chunks[0]);
    draw_details(f, app, chunks[1]);
}

fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .report
        .files
        .iter()
        .map(|file| {
            let name = file.path.to_string_lossy();
            let style = if file.violations.is_empty() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            ListItem::new(Line::from(vec![
                Span::styled(name.into_owned(), style),
                Span::raw(format!(" ({} toks)", file.token_count)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We keep state local for this simple viewer (Stateless Widget Pattern for V1)
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_index));

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_details(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Inspection");

    if let Some(file) = app.report.files.get(app.selected_index) {
        let mut text = vec![];
        text.push(Line::from(Span::styled(
            format!("Path: {}", file.path.display()),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )));
        text.push(Line::from(format!("Tokens: {}", file.token_count)));
        text.push(Line::from(""));

        if file.violations.is_empty() {
            text.push(Line::from(Span::styled(
                "✅ Clean. No violations found.",
                Style::default().fg(Color::Green),
            )));
        } else {
            text.push(Line::from(Span::styled(
                format!("❌ Found {} Violations:", file.violations.len()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            for v in &file.violations {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled(
                    format!("[Line {}] {}", v.row + 1, v.law),
                    Style::default().fg(Color::Yellow),
                )));
                text.push(Line::from(format!("   {}", v.message)));
            }
        }

        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    } else {
        f.render_widget(Paragraph::new("No file selected").block(block), area);
    }
}
