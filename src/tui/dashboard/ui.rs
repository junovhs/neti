// src/tui/dashboard/ui.rs
use crate::roadmap_v2::types::{TaskStatus, TaskStore};
use crate::tui::dashboard::state::{DashboardApp, Tab, TaskStatusFilter};
use crate::types::FileReport;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use std::fmt::Write;

pub fn draw(f: &mut Frame, app: &mut DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);

    match app.active_tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[1]),
        Tab::Roadmap => draw_roadmap(f, app, chunks[1]),
        Tab::Config => draw_config(f, app, chunks[1]),
        Tab::Logs => draw_logs(f, app, chunks[1]),
    }

    draw_footer(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let titles: Vec<_> = ["[1] Dashboard", "[2] Roadmap", "[3] Config", "[4] Logs"]
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let mut block = Block::default().borders(Borders::ALL).title("SlopChop");

    if app.has_pending_payload() {
        block = block.border_style(Style::default().fg(Color::Yellow));
    }

    let tabs = Tabs::new(titles)
        .block(block)
        .select(app.active_tab as usize)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );

    f.render_widget(tabs, area);
}

fn draw_dashboard(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let status_text = build_status_text(app);
    let status =
        Paragraph::new(status_text).block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[0]);

    draw_logs_mini(f, app, chunks[1]);
}

fn build_status_text(app: &DashboardApp) -> String {
    let mut text = String::new();

    if let Some(report) = &app.scan_report {
        let _ = writeln!(text, "Files: {}", report.files.len());
        let _ = writeln!(
            text,
            "Violations: {}",
            report
                .files
                .iter()
                .map(FileReport::violation_count)
                .sum::<usize>()
        );
        let _ = writeln!(text, "Clean: {}", report.clean_file_count());
    } else {
        text.push_str("Scanning...\n");
    }

    if app.has_pending_payload() {
        text.push_str("\n📋 PAYLOAD READY\n");
        text.push_str("Press 'a' to apply");
    }

    text
}

fn draw_roadmap(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let Some(store) = &app.roadmap else {
        let p =
            Paragraph::new("No roadmap loaded.\nCreate tasks.toml or run: slopchop roadmap init")
                .block(Block::default().borders(Borders::ALL).title("Roadmap"));
        f.render_widget(p, area);
        return;
    };

    let items = build_roadmap_items(store, app.roadmap_filter);
    let filter_label = match app.roadmap_filter {
        TaskStatusFilter::All => "All",
        TaskStatusFilter::Pending => "Pending",
        TaskStatusFilter::Done => "Done",
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Roadmap Tasks ({filter_label}) - 'f' to filter")),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

fn build_roadmap_items(store: &TaskStore, filter: TaskStatusFilter) -> Vec<ListItem<'_>> {
    store
        .tasks
        .iter()
        .filter(|t| match filter {
            TaskStatusFilter::All => true,
            TaskStatusFilter::Pending => t.status == TaskStatus::Pending,
            TaskStatusFilter::Done => matches!(t.status, TaskStatus::Done | TaskStatus::NoTest),
        })
        .map(|t| {
            let style = if t.status == TaskStatus::Done {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            let prefix = match t.status {
                TaskStatus::Done | TaskStatus::NoTest => "[x]",
                TaskStatus::Pending => "[ ]",
            };
            ListItem::new(format!("{} {}", prefix, t.text)).style(style)
        })
        .collect()
}

fn draw_config(f: &mut Frame, app: &mut DashboardApp, area: Rect) {
    crate::tui::config::view::draw_embed(f, &app.config_editor, area);
}

fn draw_logs(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .map(|s| ListItem::new(Line::from(s.as_str())))
        .collect();

    let list = List::new(logs).block(Block::default().borders(Borders::ALL).title("System Logs"));
    f.render_widget(list, area);
}

fn draw_logs_mini(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(10)
        .map(|s| ListItem::new(Line::from(s.as_str())))
        .collect();

    let list = List::new(logs).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Recent Activity"),
    );
    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let base_text = "q: Quit | TAB: Switch | r: Reload | 1-4: Jump";

    let text = if app.has_pending_payload() {
        format!("{base_text} | a: APPLY PAYLOAD | Esc: Dismiss")
    } else {
        base_text.to_string()
    };

    let style = if app.has_pending_payload() {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let p = Paragraph::new(text).style(style);
    f.render_widget(p, area);
}