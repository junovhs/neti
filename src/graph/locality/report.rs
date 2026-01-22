// src/graph/locality/report.rs
//! Rich output formatting for locality analysis.

use colored::Colorize;

use super::analysis::{TopologyAnalysis, ViolationKind};
use super::ValidationReport;

/// Prints a comprehensive locality report.
pub fn print_full_report(report: &ValidationReport, analysis: &TopologyAnalysis) {
    print_summary(report);

    if analysis.violations.is_empty() {
        println!("{}", "  ✓ All dependencies respect locality.".green());
        return;
    }

    print_violations_by_category(analysis);
    print_layers(report);
    print_god_modules(analysis);
    print_hub_candidates(analysis);
    print_module_coupling(analysis);
    print_entropy(report);
}

#[allow(clippy::cast_precision_loss)]
fn print_summary(report: &ValidationReport) {
    let health = if report.total_edges() > 0 {
        let clean = report.total_edges() - report.failed().len();
        (clean as f64 / report.total_edges() as f64) * 100.0
    } else {
        100.0
    };

    println!("\n{}", "TOPOLOGICAL HEALTH".cyan().bold());
    println!(
        "  Health Score: {:.1}%  ({} clean / {} edges)",
        health,
        report.total_edges() - report.failed().len(),
        report.total_edges()
    );
    println!(
        "  Violations:   {}",
        format_count(report.failed().len())
    );
}

fn format_count(n: usize) -> String {
    if n == 0 {
        n.to_string().green().to_string()
    } else {
        n.to_string().red().to_string()
    }
}

fn print_violations_by_category(analysis: &TopologyAnalysis) {
    let mut by_kind: std::collections::HashMap<&ViolationKind, Vec<_>> =
        std::collections::HashMap::new();

    for v in &analysis.violations {
        by_kind.entry(&v.kind).or_default().push(v);
    }

    let order = [
        ViolationKind::EncapsulationBreach,
        ViolationKind::GodModule,
        ViolationKind::MissingHub,
        ViolationKind::SidewaysDep,
        ViolationKind::UpwardDep,
    ];

    for kind in &order {
        if let Some(violations) = by_kind.get(kind) {
            println!(
                "\n{} {} ({})",
                ">>".yellow(),
                kind.label().yellow().bold(),
                kind.description()
            );

            for v in violations {
                println!(
                    "    {} -> {}",
                    v.edge.from.display(),
                    v.edge.to.display().to_string().red()
                );
                println!("      {}", v.suggestion.dimmed());
            }
        }
    }
}

fn print_god_modules(analysis: &TopologyAnalysis) {
    if analysis.god_modules.is_empty() {
        return;
    }

    println!("\n{}", "GOD MODULES (3+ outbound violations)".red().bold());
    for gm in &analysis.god_modules {
        println!(
            "  {} -> {} violations",
            gm.path.display().to_string().red(),
            gm.outbound_violations
        );
        for target in &gm.targets {
            println!("      -> {}", target.display());
        }
    }
}

fn print_hub_candidates(analysis: &TopologyAnalysis) {
    if analysis.hub_candidates.is_empty() {
        return;
    }

    println!(
        "\n{}",
        "HUB CANDIDATES (frequently imported, not yet Hub)".yellow().bold()
    );
    for hc in &analysis.hub_candidates {
        println!(
            "  {} (fan-in: {}, importers: {})",
            hc.path.display().to_string().yellow(),
            hc.fan_in,
            hc.importers.len()
        );
    }
    println!(
        "\n  {} High fan-in modules are auto-detected as Hubs if they have low fan-out.",
        "->".cyan()
    );
}

fn print_module_coupling(analysis: &TopologyAnalysis) {
    if analysis.module_coupling.is_empty() {
        return;
    }

    println!("\n{}", "MODULE COUPLING".cyan().bold());
    for (a, b, count) in &analysis.module_coupling {
        let bar = "█".repeat((*count).min(20));
        println!("  {a} → {b}: {bar} ({count})");
    }
}

fn print_entropy(report: &ValidationReport) {
    let pct = report.entropy() * 100.0;
    let label = format!("{pct:.1}%");

    let colored = if pct > 30.0 {
        label.red()
    } else if pct > 10.0 {
        label.yellow()
    } else {
        label.green()
    };

    println!("\n  Topological Entropy: {colored}");
}

#[allow(clippy::indexing_slicing)] // Guarded: mods.len() > 10 check before slice
fn print_layers(report: &ValidationReport) {
    if report.layers().is_empty() {
        return;
    }

    println!("\n{}", "LAYER ARCHITECTURE".cyan().bold());
    
    // Group modules by layer
    let mut layers: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
    for (path, layer) in report.layers() {
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
        let name = if name == "mod" {
             path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("mod")
        } else {
             name
        };
        layers.entry(*layer).or_default().push(name.to_string());
    }

    // Sort layers
    let mut sorted_layers: Vec<_> = layers.into_iter().collect();
    sorted_layers.sort_by_key(|(l, _)| *l);

    let max_bar = 30;
    let max_files = sorted_layers.iter().map(|(_, m)| m.len()).max().unwrap_or(1);

    for (layer, modules) in sorted_layers {
        let mut mods = modules;
        // Deduplicate and sort
        mods.sort();
        mods.dedup();
        
        let count = mods.len();
        let bar_len = (count * max_bar) / max_files;
        let bar = "█".repeat(bar_len.max(1));
        
        let role = if layer == 0 { "(leaf)" } else { "" };

        println!("  L{layer} {role:<6} {bar} {count} files");
    }
}