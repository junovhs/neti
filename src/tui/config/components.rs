// src/tui/config/components.rs
use super::helpers;
use super::state::ConfigApp;
use super::view::Palette;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw_header(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
    let title = Span::styled(
        " 🛡️ SLOPCHOP PROTOCOL ",
        Style::default()
            .fg(pal.primary)
            .add_modifier(Modifier::BOLD),
    );

    let status = if let Some((msg, _)) = &app.saved_message {
        Span::styled(format!(" {msg} "), Style::default().fg(Color::Green))
    } else if app.modified {
        Span::styled(" [UNSAVED CHANGES] ", Style::default().fg(Color::Yellow))
    } else {
        Span::styled(" SYSTEM: NOMINAL ", Style::default().fg(pal.secondary))
    };

    let line = Line::from(vec![title, Span::raw(" |"), status]);

    f.render_widget(
        Paragraph::new(line)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(pal.secondary)),
            )
            .alignment(Alignment::Center),
        area,
    );
}

pub fn draw_settings_table(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" [ SYSTEM CONFIGURATION ] ")
        .border_style(Style::default().fg(pal.secondary));

    let header_cells = ["PARAMETER", "VALUE", "STATUS"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(pal.primary)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = build_table_rows(app, pal);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(block)
    .column_spacing(1);

    f.render_widget(table, area);
}

fn build_table_rows(app: &ConfigApp, pal: &Palette) -> Vec<Row<'static>> {
    let preset = helpers::detect_preset(app);
    let preset_color = match preset {
        "STRICT" => Color::Green,
        "STANDARD" => Color::Yellow,
        "RELAXED" => Color::Red,
        _ => pal.text,
    };

    let active_col = Color::Green;

    let items = vec![
        ("Global Preset", preset.to_string(), preset_color, "ACTIVE"),
        (
            "Max File Tokens",
            app.rules.max_file_tokens.to_string(),
            pal.text,
            "ACTIVE",
        ),
        (
            "Cyclo. Complexity",
            app.rules.max_cyclomatic_complexity.to_string(),
            pal.text,
            "ACTIVE",
        ),
        (
            "Nesting Depth",
            app.rules.max_nesting_depth.to_string(),
            pal.text,
            "ACTIVE",
        ),
        (
            "Func. Arguments",
            app.rules.max_function_args.to_string(),
            pal.text,
            "ACTIVE",
        ),
        (
            "Func. Words",
            app.rules.max_function_words.to_string(),
            pal.text,
            "ACTIVE",
        ),
        (
            "Auto-Copy Ctx",
            bool_str(app.preferences.auto_copy),
            bool_col(app.preferences.auto_copy),
            "READY",
        ),
        (
            "Auto-Format",
            bool_str(app.preferences.auto_format),
            bool_col(app.preferences.auto_format),
            "READY",
        ),
        (
            "Auto-Commit",
            bool_str(app.preferences.auto_commit),
            bool_col(app.preferences.auto_commit),
            "STANDBY",
        ),
        (
            "Commit Prefix",
            format!("\"{}\"", app.preferences.commit_prefix),
            pal.text,
            "SET",
        ),
        (
            "UI Theme",
            format!("{:?}", app.preferences.theme).to_uppercase(),
            pal.primary,
            "LOADED",
        ),
        (
            "Progress Bars",
            bool_str(app.preferences.progress_bars),
            pal.text,
            "OKAY",
        ),
    ];

    items
        .into_iter()
        .enumerate()
        .map(|(i, (label, value, color, status))| {
            let is_selected = i == app.selected_field;
            let style = if is_selected {
                Style::default()
                    .bg(pal.highlight)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            Row::new(vec![
                Cell::from(format!("[#] {label}")),
                Cell::from(value),
                Cell::from(status).style(Style::default().fg(active_col)),
            ])
            .style(style)
        })
        .collect()
}

fn bool_str(b: bool) -> String {
    if b {
        "ON".to_string()
    } else {
        "OFF".to_string()
    }
}
fn bool_col(b: bool) -> Color {
    if b {
        Color::Green
    } else {
        Color::DarkGray
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn draw_context_panel(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" [ INTEL DISPLAY ] ")
        .border_style(Style::default().fg(pal.primary));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Min(8),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(format!(
            "> {}",
            helpers::get_active_label(app.selected_field)
        ))
        .style(
            Style::default()
                .fg(pal.primary)
                .add_modifier(Modifier::BOLD),
        ),
        chunks[0],
    );

    f.render_widget(
        Paragraph::new(helpers::get_active_description(app.selected_field))
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(Style::default().fg(pal.text)),
        chunks[1],
    );

    let ratio = helpers::get_integrity_score(app);
    let (color, label) = if ratio > 0.8 {
        (Color::Green, "OPTIMAL")
    } else if ratio > 0.5 {
        (Color::Yellow, "MODERATE")
    } else {
        (Color::Red, "CRITICAL")
    };

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(chunks[2]);

    f.render_widget(
        Paragraph::new("THREAT LEVEL ANALYTICS\nSTATUS: ACTIVE / SCANNING: ON")
            .alignment(Alignment::Center)
            .style(Style::default().fg(pal.secondary)),
        inner_chunks[0],
    );

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pal.secondary)),
        )
        .gauge_style(Style::default().fg(color))
        .use_unicode(true)
        .ratio(ratio)
        .label(Span::styled(
            format!("INTEGRITY: {:.1}% [{label}]", ratio * 100.0),
            Style::default()
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, inner_chunks[1]);

    let decoration = Paragraph::new(
        "\n[LOG] 2025.11.24 ORBITAL_ADJUSTMENT_COMPLETE\n[LOG] SECURITY_PATCH: LVL 5 ACTIVE\n[LOG] SLOPCHOP PROTOCOL ENGAGED"
    ).style(Style::default().fg(Color::DarkGray));
    f.render_widget(decoration, inner_chunks[2]);
}

pub fn draw_footer(f: &mut Frame, area: Rect, pal: &Palette) {
    let text = " [↑/↓] NAVIGATE | [←/→] ADJUST VALUE | [ENTER] SAVE CONFIG | [Q] DISENGAGE ";
    f.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(pal.bg).bg(pal.secondary))
            .alignment(Alignment::Center),
        area,
    );
}

