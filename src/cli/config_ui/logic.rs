use super::items::ConfigItem;
use super::render;
use crate::config::Config;
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Print, Color, SetForegroundColor, ResetColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{stdout, Write};
use super::editor::{ConfigEditor, EventResult, EditResult};

/// Runs the editor event loop.
///
/// # Errors
/// Returns error if terminal setup or event reading fails.
pub fn run_editor(editor: &mut ConfigEditor) -> Result<Option<Config>> {
    terminal::enable_raw_mode()?;
    let result = event_loop(editor);
    terminal::disable_raw_mode()?;
    result
}

fn event_loop(editor: &mut ConfigEditor) -> Result<Option<Config>> {
     loop {
        render::draw(editor.items(), editor.selected(), editor.config())?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            
            match handle_key_event(editor, key.code)? {
                EventResult::Continue => {},
                EventResult::Exit => return Ok(None),
                EventResult::Save(config) => return Ok(Some(*config)),
            }
        }
    }
}

fn handle_key_event(editor: &mut ConfigEditor, code: KeyCode) -> Result<EventResult> {
     match code {
        KeyCode::Up => {
            move_selection(editor, -1);
            Ok(EventResult::Continue)
        }
        KeyCode::Down => {
            move_selection(editor, 1);
            Ok(EventResult::Continue)
        }
        KeyCode::Enter => {
            edit_current(editor)?;
            Ok(EventResult::Continue)
        }
        KeyCode::Char('s' | 'S') => Ok(EventResult::Save(Box::new(editor.config().clone()))),
        KeyCode::Esc | KeyCode::Char('q') => Ok(EventResult::Exit),
        _ => Ok(EventResult::Continue)
    }
}

pub fn move_selection(editor: &mut ConfigEditor, delta: isize) {
    let len = editor.items().len();
    let current = editor.selected();
    
    let new_pos = if delta < 0 {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        current.saturating_add(delta.unsigned_abs())
    };
    
    if new_pos < len {
        editor.set_selected(new_pos);
    }
}

#[allow(clippy::indexing_slicing)] // Guarded: selected is always < items.len()
fn edit_current(editor: &mut ConfigEditor) -> Result<()> {
     let selected = editor.selected();
    let item = editor.items()[selected];
    
    if item.is_boolean() {
        item.toggle_boolean(editor.config_mut());
        editor.set_modified(true);
    } else if item.is_enum() {
        item.cycle_enum(editor.config_mut());
        editor.set_modified(true);
    } else if let Some(new_val) = edit_number(editor, item)? {
        item.set_number(editor.config_mut(), new_val);
        editor.set_modified(true);
    }
    Ok(())
}

fn edit_number(editor: &ConfigEditor, item: ConfigItem) -> Result<Option<usize>> {
     let mut value = item.get_number(editor.config());
    
    loop {
        render_number_editor(editor.selected(), value)?;
        
        match handle_number_input(&mut value)? {
            EditResult::Continue => {}
            EditResult::Commit(val) => return Ok(Some(val)),
            EditResult::Cancel => return Ok(None),
        }
    }
}

fn render_number_editor(selected: usize, value: usize) -> Result<()> {
    let row = u16::try_from(selected).unwrap_or(0) + 2; // +2 to account for header offset
    let mut stdout = stdout();
    
    execute!(
        stdout,
        cursor::MoveTo(40, row),
        Clear(ClearType::UntilNewLine),
        SetForegroundColor(Color::Green),
        Print(format!("[{value}]")),
        ResetColor,
        Print(" \u{2190}\u{2192} (adj) \u{2191}\u{2193} (jump)")
    )?;
    stdout.flush()?;
    Ok(())
}

fn handle_number_input(value: &mut usize) -> Result<EditResult> {
      let Event::Key(key) = event::read()? else {
        return Ok(EditResult::Continue);
    };
    
    if key.kind != KeyEventKind::Press {
        return Ok(EditResult::Continue);
    }

    Ok(process_number_key(key.code, value))
}

// ADAPTIVE STEPPING LOGIC
fn process_number_key(code: KeyCode, value: &mut usize) -> EditResult {
    let small_step = 1;
    let big_step = if *value >= 1000 { 500 } else if *value >= 100 { 100 } else { 10 };

    match code {
        // Fine adjustment
        KeyCode::Left => {
            *value = value.saturating_sub(small_step);
            EditResult::Continue
        }
        KeyCode::Right => {
            *value = value.saturating_add(small_step);
            EditResult::Continue
        }
        
        // Coarse adjustment
        KeyCode::Down => {
            *value = value.saturating_sub(big_step);
            EditResult::Continue
        }
        KeyCode::Up => {
            *value = value.saturating_add(big_step);
            EditResult::Continue
        }
        
        KeyCode::Enter => EditResult::Commit(*value),
        KeyCode::Esc => EditResult::Cancel,
        _ => EditResult::Continue
    }
}