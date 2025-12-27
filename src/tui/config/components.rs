// src/tui/config/components.rs
use super::helpers;
use super::state::ConfigApp;
use super::view::Palette;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw_header(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
    let title = Span::styled(" 🛡️ SLOPCHOP PROTOCOL ", Style::default().fg(pal.primary).add_modifier(Modifier::BOLD));
    let status = if let Some((msg, _)) = &app.saved_message {
        Span::styled(format!(" {msg} "), Style::default().fg(Color::Green))
    } else if app.modified {
        Span::styled(" [UNSAVED CHANGES] ", Style::default().fg(Color::Yellow))
    } else {
        Span::styled(" SYSTEM: NOMINAL ", Style::default().fg(pal.secondary))
    };

    let line = Line::from(vec![title, Span::raw(" |"), status]);
    f.render_widget(Paragraph::new(line).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(pal.secondary))).alignment(Alignment::Center), area);
}

pub fn draw_settings_table(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
    let block = Block::default().borders(Borders::ALL).title(" [ SYSTEM CONFIGURATION ] ").border_style(Style::default().fg(pal.secondary));
    let header = Row::new(vec!["PARAMETER", "VALUE", "STATUS"]).style(Style::default().fg(pal.primary)).height(1).bottom_margin(1);

    let rows: Vec<Row> = build_table_data(app).into_iter().enumerate().map(|(i, (k, v, status))| {
        let style = if i == app.selected_field { Style::default().bg(pal.highlight).fg(Color::Black).add_modifier(Modifier::BOLD) } else { Style::default().fg(pal.text) };
        Row::new(vec![k.to_string(), v, status]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(30), Constraint::Percentage(20)]).header(header).block(block);
    f.render_widget(table, area);
}

fn build_table_data(app: &ConfigApp) -> Vec<(&'static str, String, String)> {
    vec![
        ("Global Preset", helpers::detect_preset(app).to_string(), "ACTIVE".to_string()),
        ("Max File Tokens", app.rules.max_file_tokens.to_string(), "ENFORCED".to_string()),
        ("Cyclo. Complexity", app.rules.max_cyclomatic_complexity.to_string(), "ENFORCED".to_string()),
        ("Nesting Depth", app.rules.max_nesting_depth.to_string(), "ENFORCED".to_string()),
        ("Func. Arguments", app.rules.max_function_args.to_string(), "ENFORCED".to_string()),
        ("Func. Words", app.rules.max_function_words.to_string(), "ENFORCED".to_string()),
        ("Auto-Copy Ctx", bool_str(app.preferences.auto_copy), "READY".to_string()),
        ("Auto-Format", bool_str(app.preferences.auto_format), "READY".to_string()),
        ("Progress Bars", bool_str(app.preferences.progress_bars), "OKAY".to_string()),
        ("Require Plan", bool_str(app.preferences.require_plan), "SECURE".to_string()),
    ]
}

fn bool_str(b: bool) -> String { if b { "ON".to_string() } else { "OFF".to_string() } }

pub fn draw_footer(f: &mut Frame, area: Rect, pal: &Palette) {
    let text = " [arrows] NAVIGATE | [h/l] ADJUST | [ENTER] SAVE | [Q] EXIT ";
    f.render_widget(Paragraph::new(text).style(Style::default().fg(pal.bg).bg(pal.secondary)).alignment(Alignment::Center), area);
}