// src/tui/config/status_panel.rs
use super::helpers;
use super::state::ConfigApp;
use super::view::Palette;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use ratatui::Frame;

#[allow(clippy::cast_precision_loss)]
pub fn draw(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
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

    draw_selection_info(f, app, chunks[0], chunks[1], pal);
    draw_threat_analytics(f, app, chunks[2], pal);
}

fn draw_selection_info(
    f: &mut Frame,
    app: &ConfigApp,
    title_area: Rect,
    desc_area: Rect,
    pal: &Palette,
) {
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
        title_area,
    );

    f.render_widget(
        Paragraph::new(helpers::get_active_description(app.selected_field))
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(pal.text)),
        desc_area,
    );
}

#[allow(clippy::cast_precision_loss)]
fn draw_threat_analytics(f: &mut Frame, app: &ConfigApp, area: Rect, pal: &Palette) {
    let ratio = helpers::get_integrity_score(app);
    let (color, label) = if ratio > 0.8 {
        (Color::Green, "OPTIMAL")
    } else if ratio > 0.5 {
        (Color::Yellow, "MODERATE")
    } else {
        (Color::Red, "CRITICAL")
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    f.render_widget(
        Paragraph::new("THREAT LEVEL ANALYTICS\nSTATUS: ACTIVE / SCANNING: ON")
            .alignment(Alignment::Center)
            .style(Style::default().fg(pal.secondary)),
        chunks[0],
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

    f.render_widget(gauge, chunks[1]);

    let decoration = Paragraph::new(
        "\n[LOG] 2025.11.24 ORBITAL_ADJUSTMENT_COMPLETE\n[LOG] SECURITY_PATCH: LVL 5 ACTIVE\n[LOG] SLOPCHOP PROTOCOL ENGAGED"
    ).style(Style::default().fg(Color::DarkGray));
    f.render_widget(decoration, chunks[2]);
}