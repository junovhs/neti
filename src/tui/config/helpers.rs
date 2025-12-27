// src/tui/config/helpers.rs
use super::state::ConfigApp;

#[derive(Clone, Copy)]
struct Preset {
    tokens: usize,
    complexity: usize,
    depth: usize,
}

const PRESETS: [Preset; 3] = [
    Preset { tokens: 1500, complexity: 4, depth: 2 }, // Strict
    Preset { tokens: 2000, complexity: 8, depth: 3 }, // Standard
    Preset { tokens: 3000, complexity: 12, depth: 4 }, // Relaxed
];

pub fn adjust_rule(app: &mut ConfigApp, increase: bool) {
    match app.selected_field {
        1 => adjust_int(&mut app.rules.max_file_tokens, 100, 100, increase),
        2 => adjust_int(&mut app.rules.max_cyclomatic_complexity, 1, 1, increase),
        3 => adjust_int(&mut app.rules.max_nesting_depth, 1, 1, increase),
        4 => adjust_int(&mut app.rules.max_function_args, 1, 1, increase),
        5 => adjust_int(&mut app.rules.max_function_words, 1, 1, increase),
        _ => {}
    }
}

pub fn adjust_pref(app: &mut ConfigApp, _increase: bool) {
    // Toggles for AutoCopy, AutoFormat, Progress, RequirePlan
    // (UI Theme index 8 removed)
    match app.selected_field {
        6 => app.preferences.auto_copy = !app.preferences.auto_copy,
        7 => app.preferences.auto_format = !app.preferences.auto_format,
        8 => app.preferences.progress_bars = !app.preferences.progress_bars,
        9 => app.preferences.require_plan = !app.preferences.require_plan,
        _ => {}
    }
}

fn adjust_int(val: &mut usize, step: usize, min: usize, increase: bool) {
    if increase { *val = val.saturating_add(step); }
    else { *val = val.saturating_sub(step).max(min); }
}

pub fn cycle_preset(app: &mut ConfigApp, forward: bool) {
    let current = detect_preset_index(app);
    let next_idx = if forward { (current + 1) % 3 } else { (current + 2) % 3 };
    let p = PRESETS[next_idx];
    app.rules.max_file_tokens = p.tokens;
    app.rules.max_cyclomatic_complexity = p.complexity;
    app.rules.max_nesting_depth = p.depth;
}

fn detect_preset_index(app: &ConfigApp) -> usize {
    if app.rules.max_file_tokens <= 1500 { 0 }
    else if app.rules.max_file_tokens <= 2000 { 1 }
    else { 2 }
}

const FIELD_INFOS: [&str; 10] = [
    "GLOBAL PROTOCOL: Select a predefined security level.",
    "LAW OF ATOMICITY: Limits file size to maintain modularity.",
    "LAW OF COMPLEXITY: Limits control flow branching factor.",
    "LAW OF COMPLEXITY: Limits indentation/nesting depth.",
    "LAW OF COMPLEXITY: Limits function argument count.",
    "LAW OF BLUNTNESS: Limits function naming verbosity.",
    "WORKFLOW: Auto-copy context.txt to clipboard.",
    "WORKFLOW: Run formatter immediately after apply.",
    "VISUALS: Show animated progress bars.",
    "WORKFLOW: Force payloads to include a PLAN block.",
];

#[must_use]
pub fn get_active_label(field: usize) -> &'static str {
    if field == 0 { "GLOBAL PROTOCOL" } else if field < 6 { "3 LAWS" } else { "WORKFLOW" }
}

#[must_use]
pub fn get_active_description(field: usize) -> &'static str {
    FIELD_INFOS.get(field).copied().unwrap_or("")
}

#[must_use]
pub fn detect_preset(app: &ConfigApp) -> &'static str {
    match detect_preset_index(app) {
        0 => "STRICT",
        1 => "STANDARD",
        2 => "RELAXED",
        _ => "CUSTOM",
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn get_integrity_score(app: &ConfigApp) -> f64 {
    let t = calculate_metric_weight(app.rules.max_file_tokens, 1000.0, 2000.0);
    let c = calculate_metric_weight(app.rules.max_cyclomatic_complexity, 1.0, 14.0);
    let d = calculate_metric_weight(app.rules.max_nesting_depth, 1.0, 4.0);
    
    (1.0 - (t + c + d) / 3.0).clamp(0.0, 1.0)
}

#[allow(clippy::cast_precision_loss)]
fn calculate_metric_weight(val: usize, min: f64, range: f64) -> f64 {
    (val as f64 - min) / range
}