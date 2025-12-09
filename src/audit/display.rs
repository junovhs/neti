use super::diff::DiffModel;
use colored::Colorize;

/// Prints the diff analysis in a compiler-grade format.
pub fn print_analysis(model: &DiffModel) {
    if model.holes.is_empty() {
        println!("{}", "No structural differences found (identical).".green());
        return;
    }

    println!("{}", "Refactoring Opportunity Detected:".bold().blue());
    println!("{}", "---------------------------------".blue());
    println!("Found {} variation point(s):", model.holes.len());

    for (i, hole) in model.holes.iter().enumerate() {
        println!("\n{}. {}: {}", i + 1, "Variation".yellow(), hole.kind);
        println!("   path: {}", hole.path_id.dimmed());
        print!("   values: ");
        for (j, var) in hole.variants.iter().enumerate() {
            if j > 0 {
                print!(", ");
            }
            print!("{}", var.cyan());
        }
        println!();
    }
}
