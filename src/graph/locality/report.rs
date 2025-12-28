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
    print_god_modules(analysis);
    print_hub_candidates(analysis);
    print_module_coupling(analysis);
    print_entropy(report);
}

fn print_summary(report: &ValidationReport) {
    println!(
        "\n{} {} edges | {} passed | {} failed",
        "LOCALITY SCAN".cyan().bold(),
        report.total_edges,
        report.passed.len().to_string().green(),
        format_count(report.failed.len()),
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
        ViolationKind::TightCoupling,
        ViolationKind::SidewaysDep,
    ];

    for kind in &order {
        if let Some(violations) = by_kind.get(kind) {
            println!(
                "\n{} {} ({})",
                "▸".yellow(),
                kind.label().yellow().bold(),
                kind.description()
            );

            for v in violations {
                println!(
                    "    {} → {}",
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
            "  {} → {} violations",
            gm.path.display().to_string().red(),
            gm.outbound_violations
        );
        for target in &gm.targets {
            println!("      → {}", target.display());
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
        "\n  {} Add these to [rules.locality].hubs in slopchop.toml",
        "→".cyan()
    );
}

fn print_module_coupling(analysis: &TopologyAnalysis) {
    if analysis.module_coupling.is_empty() {
        return;
    }

    println!("\n{}", "MODULE COUPLING".cyan().bold());
    for (a, b, count) in &analysis.module_coupling {
        let bar = "█".repeat(*count);
        println!("  {a} ↔ {b}: {bar} ({count})");
    }
}

fn print_entropy(report: &ValidationReport) {
    let pct = report.entropy * 100.0;
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