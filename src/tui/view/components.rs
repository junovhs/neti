use crate::tui::state::App;
use crate::types::FileReport;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph}; // Removed Wrap
use ratatui::Frame;

pub fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 📂 File List ");

    let items: Vec<ListItem> = app
        .view_indices
        .iter()
        .map(|&idx| {
            let file = &app.report.files[idx];
            let name = file.path.to_string_lossy();

            let is_clean = file.is_clean();
            let color = if !is_clean {
                Color::Red
            } else if file.token_count > 1000 {
                Color::Yellow
            } else {
                Color::Green
            };
            let icon = if is_clean { "✓" } else { "!" };

            let bars = (file.token_count / 200).clamp(0, 10);
            let bar_vis: String = "I".repeat(bars);

            let content = Line::from(vec![
                Span::styled(
                    format!("{icon} "),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{name:<30} ")),
                Span::styled(
                    format!("{bar_vis:<10}"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(list, area, &mut state);
}

#[allow(clippy::cast_precision_loss)]
pub fn draw_inspector(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🕵️ Inspector ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(file) = app.get_selected_file() {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Length(6),
                    Constraint::Min(5),
                ]
                .as_ref(),
            )
            .split(inner);

        let header = Paragraph::new(Line::from(vec![
            Span::styled("TARGET: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                file.path.to_string_lossy(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));
        f.render_widget(header, layout[0]);

        draw_stats(f, file, layout[1]);
        draw_issues(f, file, layout[2]);
    } else {
        f.render_widget(
            Paragraph::new("No file selected").alignment(Alignment::Center),
            inner,
        );
    }
}

#[allow(clippy::cast_precision_loss)]
fn draw_stats(f: &mut Frame, file: &FileReport, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let token_ratio = (file.token_count as f64 / 2000.0).clamp(0.0, 1.0);
    let token_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE).title("Size"))
        .gauge_style(Style::default().fg(if token_ratio > 0.8 {
            Color::Red
        } else {
            Color::Green
        }))
        .ratio(token_ratio)
        .label(format!("{} toks", file.token_count));
    f.render_widget(token_gauge, chunks[0]);

    let violation_count = file.violations.len();
    let complex_ratio = (violation_count as f64 / 5.0).clamp(0.0, 1.0);
    let complex_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE).title("Issues"))
        .gauge_style(Style::default().fg(if violation_count > 0 {
            Color::Red
        } else {
            Color::Green
        }))
        .ratio(complex_ratio)
        .label(format!("{violation_count} Found"));
    f.render_widget(complex_gauge, chunks[1]);
}

fn draw_issues(f: &mut Frame, file: &FileReport, area: Rect) {
    if file.is_clean() {
        let p = Paragraph::new("✨ Clean.")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(p, area);
    } else {
        let items: Vec<ListItem> = file
            .violations
            .iter()
            .map(|v| {
                let header = Line::from(vec![
                    Span::styled(
                        format!("[{}] ", v.law),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("Line {}", v.row + 1)),
                ]);
                let msg = Line::from(Span::styled(
                    format!("  └─ {}", v.message),
                    Style::default().fg(Color::White),
                ));
                ListItem::new(vec![header, msg, Line::from("")])
            })
            .collect();

        let list =
            List::new(items).block(Block::default().borders(Borders::TOP).title(" Violations "));
        f.render_widget(list, area);
    }
}
