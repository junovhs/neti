use super::diff::DiffModel;
use colored::Colorize;
use std::fmt::Write;

/// Formats the diff analysis in a compiler-grade visual format.
#[must_use]
pub fn format_diff(model: &DiffModel, _src_a: &str, _src_b: &str) -> String {
    let mut out = String::new();

    if model.holes.is_empty() {
        return "No structural differences found (identical).".green().to_string();
    }

    let _ = writeln!(out, "{}", "Visual AST Diff:".bold().blue());
    let _ = writeln!(out, "{}", "----------------".blue());

    // Simple textual representation of holes for now.
    // A full side-by-side with source context would require
    // mapping AST nodes back to byte ranges and extracting text lines.
    // Given the current architecture, we'll list the variations clearly.

    for (i, hole) in model.holes.iter().enumerate() {
        let _ = writeln!(out, "\n{}. {}: {}", i + 1, "Variation".yellow(), hole.kind);
        let _ = writeln!(out, "   path: {}", hole.path_id.dimmed());
        
        // Show side-by-side values
        if hole.variants.len() >= 2 {
            let val_a = &hole.variants[0];
            let val_b = &hole.variants[1];
            
            let _ = writeln!(out, "   {}", "┌─────────────────────┐".dimmed());
            let _ = writeln!(out, "   │ {:<19} │", val_a.red());
            let _ = writeln!(out, "   │ {:<19} │", val_b.green());
            let _ = writeln!(out, "   {}", "└─────────────────────┘".dimmed());
        } else {
             let _ = writeln!(out, "   values: {:?}", hole.variants);
        }
    }
    
    out
}

