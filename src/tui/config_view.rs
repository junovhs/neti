// src/tui/config_view.rs
use crate::tui::config_state::ConfigApp;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &ConfigApp) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &ConfigApp, area: Rect) {
    let title = Span::styled(
        " 🛡️ WARDEN PROTOCOL ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let status = if let Some((msg, _)) = &app.saved_message {
        Span::styled(format!(" {msg} "), Style::default().fg(Color::Green))
    } else if app.modified {
        Span::styled(" [Modified] ", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![title, Span::raw(" |"), status]);

    f.render_widget(
        Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_main(f: &mut Frame, app: &ConfigApp, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_settings_table(f, app, layout[0]);
    draw_context_panel(f, app, layout[1]);
}

fn draw_settings_table(f: &mut Frame, app: &ConfigApp, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Configuration ")
        .border_style(Style::default().fg(Color::DarkGray));

    let header_cells = ["Parameter", "Value"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let preset_color = match app.detect_preset() {
        "STRICT" => Color::Green,
        "STANDARD" => Color::Yellow,
        "RELAXED" => Color::Red,
        _ => Color::White,
    };

    let items = vec![
        ("Global Preset", app.detect_preset().to_string(), preset_color),
        ("Max File Tokens", app.rules.max_file_tokens.to_string(), Color::White),
        ("Cyclomatic Complexity", app.rules.max_cyclomatic_complexity.to_string(), Color::White),
        ("Nesting Depth", app.rules.max_nesting_depth.to_string(), Color::White),
        ("Function Arguments", app.rules.max_function_args.to_string(), Color::White),
        ("Function Words", app.rules.max_function_words.to_string(), Color::White),
    ];

    let rows: Vec<Row> = items
        .into_iter()
        .enumerate()
        .map(|(i, (label, value, color))| {
            let is_selected = i == app.selected_field;
            let style = if is_selected {
                Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };
            
            Row::new(vec![
                Cell::from(label),
                Cell::from(value),
            ]).style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(70), Constraint::Percentage(30)])
        .header(header)
        .block(block)
        .column_spacing(2);

    f.render_widget(table, area);
}

#[allow(clippy::cast_precision_loss)]
fn draw_context_panel(f: &mut Frame, app: &ConfigApp, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Intel ")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Law Title
            Constraint::Length(6), // Description
            Constraint::Length(3), // Gauge
            Constraint::Min(1),    // Padding
        ])
        .split(inner);

    // 1. Law Title
    f.render_widget(
        Paragraph::new(app.get_active_label())
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    // 2. Description
    f.render_widget(
        Paragraph::new(app.get_active_description())
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(Style::default().fg(Color::White)),
        chunks[1],
    );

    // 3. Integrity Gauge (Clean visuals)
    let ratio = app.get_containment_integrity();
    let (color, label) = if ratio > 0.8 {
        (Color::Green, "SECURE")
    } else if ratio > 0.5 {
        (Color::Yellow, "MODERATE")
    } else {
        (Color::Red, "COMPROMISED")
    };

    let gauge = Gauge::default()
        .block(Block::default().title(" Containment Integrity ").borders(Borders::NONE))
        .gauge_style(Style::default().fg(color))
        .use_unicode(true) // Uses smooth block characters
        .ratio(ratio)
        .label(Span::styled(
            format!("{label} ({:.0}%)", ratio * 100.0),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    
    f.render_widget(gauge, chunks[2]);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let text = " [↑/↓] Select | [←/→] Adjust | [Enter] Save | [q] Quit ";
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Black).bg(Color::DarkGray)),
        area,
    );
}