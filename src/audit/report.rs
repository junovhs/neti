// src/audit/report.rs
//! Output formatting for consolidation audit reports.
//!
//! This module provides both human-readable and machine-readable output
//! formats for audit results.

use super::types::{AuditReport, AuditStats, Opportunity, OpportunityKind};
use colored::Colorize;
use std::fmt::Write;

/// Formats the audit report for terminal display.
#[must_use]
pub fn format_terminal(report: &AuditReport) -> String {
    let mut out = String::new();

    // Header
    writeln!(out, "{}", "─".repeat(70).dimmed()).ok();
    writeln!(out, "{}", " 🔍 CONSOLIDATION AUDIT REPORT ".cyan().bold()).ok();
    writeln!(out, "{}", "─".repeat(70).dimmed()).ok();
    writeln!(out).ok();

    // Summary stats
    write_stats(&mut out, &report.stats);

    // Opportunities by category
    if report.opportunities.is_empty() {
        writeln!(
            out,
            "{}",
            "✨ No consolidation opportunities found! Your code is clean.".green()
        )
        .ok();
    } else {
        write_opportunities(&mut out, &report.opportunities);
    }

    // Footer
    writeln!(out).ok();
    writeln!(out, "{}", "─".repeat(70).dimmed()).ok();

    out
}

fn write_stats(out: &mut String, stats: &AuditStats) {
    writeln!(out, "{}", "📊 SUMMARY".cyan().bold()).ok();
    writeln!(out).ok();

    writeln!(
        out,
        "   Files analyzed:    {}",
        stats.files_analyzed.to_string().white()
    )
    .ok();
    writeln!(
        out,
        "   Code units found:  {}",
        stats.units_extracted.to_string().white()
    )
    .ok();
    writeln!(
        out,
        "   Analysis time:     {}ms",
        stats.duration_ms.to_string().white()
    )
    .ok();
    writeln!(out).ok();

    writeln!(
        out,
        "   Similarity clusters: {}",
        format_count(stats.similarity_clusters)
    )
    .ok();
    writeln!(
        out,
        "   Dead code units:     {}",
        format_count(stats.dead_code_units)
    )
    .ok();
    writeln!(
        out,
        "   Repeated patterns:   {}",
        format_count(stats.pattern_instances)
    )
    .ok();
    writeln!(out).ok();

    if stats.total_potential_savings > 0 {
        writeln!(
            out,
            "   {} {} lines could potentially be removed/consolidated",
            "💡".yellow(),
            stats.total_potential_savings.to_string().green().bold()
        )
        .ok();
    }

    writeln!(out).ok();
}

fn format_count(n: usize) -> String {
    if n == 0 {
        "0".dimmed().to_string()
    } else {
        n.to_string().yellow().to_string()
    }
}

fn write_opportunities(out: &mut String, opportunities: &[Opportunity]) {
    writeln!(
        out,
        "{}",
        "🎯 OPPORTUNITIES (sorted by impact)".cyan().bold()
    )
    .ok();
    writeln!(out).ok();

    for (i, opp) in opportunities.iter().enumerate() {
        write_opportunity(out, i + 1, opp);
    }
}

fn write_opportunity(out: &mut String, index: usize, opp: &Opportunity) {
    let severity_color = match opp.kind {
        OpportunityKind::Duplication => "HIGH".red(),
        OpportunityKind::ModuleConsolidation => "HIGH".red(),
        OpportunityKind::Pattern => "MEDIUM".yellow(),
        OpportunityKind::DeadCode => "LOW".green(),
    };

    writeln!(
        out,
        "{}. [{}] {}",
        index,
        severity_color,
        opp.title.white().bold()
    )
    .ok();

    // Impact metrics
    let score = opp.impact.score();
    writeln!(
        out,
        "   {} ~{} lines | difficulty: {}/5 | confidence: {:.0}% | score: {:.1}",
        "📈".dimmed(),
        opp.impact.lines_saved,
        opp.impact.difficulty,
        opp.impact.confidence * 100.0,
        score
    )
    .ok();

    // Files affected
    let file_count = opp.affected_files.len();
    if file_count <= 3 {
        let files: Vec<_> = opp
            .affected_files
            .iter()
            .map(|f| f.display().to_string())
            .collect();
        writeln!(out, "   {} {}", "📁".dimmed(), files.join(", ").dimmed()).ok();
    } else {
        writeln!(out, "   {} {} files affected", "📁".dimmed(), file_count).ok();
    }

    // Recommendation
    writeln!(out, "   {} {}", "💡".dimmed(), opp.recommendation.dimmed()).ok();

    writeln!(out).ok();
}

/// Formats the audit report as JSON for machine consumption.
#[must_use]
pub fn format_json(report: &AuditReport) -> String {
    let mut out = String::new();

    out.push_str("{\n");
    out.push_str("  \"stats\": ");
    write_stats_json(&mut out, &report.stats);
    out.push_str(",\n");

    out.push_str("  \"opportunities\": [\n");
    for (i, opp) in report.opportunities.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        write_opportunity_json(&mut out, opp);
    }
    out.push_str("\n  ]\n");
    out.push_str("}\n");

    out
}

fn write_stats_json(out: &mut String, stats: &AuditStats) {
    write!(
        out,
        r#"{{
    "files_analyzed": {},
    "units_extracted": {},
    "similarity_clusters": {},
    "dead_code_units": {},
    "pattern_instances": {},
    "total_potential_savings": {},
    "duration_ms": {}
  }}"#,
        stats.files_analyzed,
        stats.units_extracted,
        stats.similarity_clusters,
        stats.dead_code_units,
        stats.pattern_instances,
        stats.total_potential_savings,
        stats.duration_ms
    )
    .ok();
}

fn write_opportunity_json(out: &mut String, opp: &Opportunity) {
    let kind = match opp.kind {
        OpportunityKind::Duplication => "duplication",
        OpportunityKind::DeadCode => "dead_code",
        OpportunityKind::Pattern => "pattern",
        OpportunityKind::ModuleConsolidation => "module_consolidation",
    };

    let files: Vec<_> = opp
        .affected_files
        .iter()
        .map(|f| f.display().to_string())
        .collect();

    write!(
        out,
        r#"    {{
      "id": "{}",
      "kind": "{}",
      "title": "{}",
      "impact": {{
        "lines_saved": {},
        "tokens_saved": {},
        "difficulty": {},
        "confidence": {},
        "score": {}
      }},
      "files": [{}],
      "recommendation": "{}"
    }}"#,
        escape_json(&opp.id),
        kind,
        escape_json(&opp.title),
        opp.impact.lines_saved,
        opp.impact.tokens_saved,
        opp.impact.difficulty,
        opp.impact.confidence,
        opp.impact.score(),
        files
            .iter()
            .map(|f| format!("\"{}\"", escape_json(f)))
            .collect::<Vec<_>>()
            .join(", "),
        escape_json(&opp.recommendation)
    )
    .ok();
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Formats the report for AI consumption (concise, actionable).
#[must_use]
pub fn format_ai_prompt(report: &AuditReport) -> String {
    let mut out = String::new();

    writeln!(out, "# Consolidation Audit Results").ok();
    writeln!(out).ok();
    writeln!(
        out,
        "**Potential savings: ~{} lines**",
        report.stats.total_potential_savings
    )
    .ok();
    writeln!(out).ok();
    writeln!(out, "## Opportunities (sorted by impact)").ok();
    writeln!(out).ok();

    for (i, opp) in report.opportunities.iter().take(10).enumerate() {
        let kind_label = match opp.kind {
            OpportunityKind::Duplication => "DUPLICATION",
            OpportunityKind::DeadCode => "DEAD CODE",
            OpportunityKind::Pattern => "PATTERN",
            OpportunityKind::ModuleConsolidation => "MODULE",
        };

        writeln!(out, "{}. **[{}]** {}", i + 1, kind_label, opp.title).ok();
        writeln!(out, "   - Est. savings: {} lines", opp.impact.lines_saved).ok();
        writeln!(out, "   - Files: {}", opp.affected_files.len()).ok();
        writeln!(out, "   - Action: {}", opp.recommendation).ok();
        writeln!(out).ok();
    }

    if report.opportunities.len() > 10 {
        writeln!(
            out,
            "*...and {} more opportunities*",
            report.opportunities.len() - 10
        )
        .ok();
    }

    out
}

/// Formats a single opportunity for detailed display.
#[must_use]
pub fn format_opportunity_detail(opp: &Opportunity) -> String {
    let mut out = String::new();

    writeln!(out, "{}", "═".repeat(70)).ok();
    writeln!(out, "{}", opp.title.bold()).ok();
    writeln!(out, "{}", "═".repeat(70)).ok();
    writeln!(out).ok();

    writeln!(out, "{}", "DESCRIPTION:".cyan()).ok();
    writeln!(out, "{}", opp.description).ok();
    writeln!(out).ok();

    writeln!(out, "{}", "IMPACT:".cyan()).ok();
    writeln!(out, "  Lines saved:  ~{}", opp.impact.lines_saved).ok();
    writeln!(out, "  Tokens saved: ~{}", opp.impact.tokens_saved).ok();
    writeln!(out, "  Difficulty:   {}/5", opp.impact.difficulty).ok();
    writeln!(out, "  Confidence:   {:.0}%", opp.impact.confidence * 100.0).ok();
    writeln!(out, "  Score:        {:.2}", opp.impact.score()).ok();
    writeln!(out).ok();

    writeln!(out, "{}", "AFFECTED FILES:".cyan()).ok();
    for file in &opp.affected_files {
        writeln!(out, "  - {}", file.display()).ok();
    }
    writeln!(out).ok();

    writeln!(out, "{}", "RECOMMENDATION:".cyan()).ok();
    writeln!(out, "{}", opp.recommendation).ok();

    out
}
