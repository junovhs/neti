// src/audit/report.rs
//! Output formatting for consolidation audit reports.

use super::types::{AuditReport, AuditStats, Opportunity, OpportunityKind};
use colored::Colorize;
use std::fmt::Write;

/// Formats the audit report for terminal display.
#[must_use]
pub fn format_terminal(report: &AuditReport) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "{}", "─".repeat(70).dimmed());
    let _ = writeln!(out, "{}", " 🔍 CONSOLIDATION AUDIT REPORT ".cyan().bold());
    let _ = writeln!(out, "{}", "─".repeat(70).dimmed());
    let _ = writeln!(out);

    write_stats(&mut out, &report.stats);

    if report.opportunities.is_empty() {
        let _ = writeln!(
            out,
            "{}",
            "✨ No consolidation opportunities found! Your code is clean.".green()
        );
    } else {
        write_opportunities(&mut out, &report.opportunities);
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "{}", "─".repeat(70).dimmed());

    out
}

fn write_stats(out: &mut String, stats: &AuditStats) {
    let _ = writeln!(out, "{}", "📊 SUMMARY".cyan().bold());
    let _ = writeln!(out);

    let _ = writeln!(
        out,
        "   Files analyzed:    {}",
        stats.files_analyzed.to_string().white()
    );
    let _ = writeln!(
        out,
        "   Code units found:  {}",
        stats.units_extracted.to_string().white()
    );
    let _ = writeln!(
        out,
        "   Analysis time:     {}ms",
        stats.duration_ms.to_string().white()
    );
    let _ = writeln!(out);

    let _ = writeln!(
        out,
        "   Similarity clusters: {}",
        format_count(stats.similarity_clusters)
    );
    let _ = writeln!(
        out,
        "   Dead code units:     {}",
        format_count(stats.dead_code_units)
    );
    let _ = writeln!(
        out,
        "   Repeated patterns:   {}",
        format_count(stats.pattern_instances)
    );
    let _ = writeln!(out);

    if stats.total_potential_savings > 0 {
        let _ = writeln!(
            out,
            "   {} {} lines could potentially be removed/consolidated",
            "💡".yellow(),
            stats.total_potential_savings.to_string().green().bold()
        );
    }

    let _ = writeln!(out);
}

fn format_count(n: usize) -> String {
    if n == 0 {
        "0".dimmed().to_string()
    } else {
        n.to_string().yellow().to_string()
    }
}

fn write_opportunities(out: &mut String, opportunities: &[Opportunity]) {
    let _ = writeln!(
        out,
        "{}",
        "🎯 OPPORTUNITIES (sorted by impact)".cyan().bold()
    );
    let _ = writeln!(out);

    for (i, opp) in opportunities.iter().enumerate() {
        write_opportunity(out, i + 1, opp);
    }
}

fn write_opportunity(out: &mut String, index: usize, opp: &Opportunity) {
    let severity_color = match opp.kind {
        OpportunityKind::Duplication | OpportunityKind::ModuleConsolidation => "HIGH".red(),
        OpportunityKind::Pattern => "MEDIUM".yellow(),
        OpportunityKind::DeadCode => "LOW".green(),
    };

    let _ = writeln!(
        out,
        "{}. [{}] {}",
        index,
        severity_color,
        opp.title.white().bold()
    );

    let score = opp.impact.score();
    let _ = writeln!(
        out,
        "   {} ~{} lines | difficulty: {}/5 | confidence: {:.0}% | score: {:.1}",
        "📈".dimmed(),
        opp.impact.lines_saved,
        opp.impact.difficulty,
        opp.impact.confidence * 100.0,
        score
    );

    let file_count = opp.affected_files.len();
    if file_count <= 3 {
        let files: Vec<_> = opp
            .affected_files
            .iter()
            .map(|f| f.display().to_string())
            .collect();
        let _ = writeln!(out, "   {} {}", "📁".dimmed(), files.join(", ").dimmed());
    } else {
        let _ = writeln!(out, "   {} {file_count} files affected", "📁".dimmed());
    }

    let _ = writeln!(out, "   {} {}", "💡".dimmed(), opp.recommendation.dimmed());

    let _ = writeln!(out);
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
    let _ = write!(
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
    );
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

    let _ = write!(
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
    );
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

    let _ = writeln!(out, "# Consolidation Audit Results");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "**Potential savings: ~{} lines**",
        report.stats.total_potential_savings
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Opportunities (sorted by impact)");
    let _ = writeln!(out);

    for (i, opp) in report.opportunities.iter().enumerate() {
        let kind_label = match opp.kind {
            OpportunityKind::Duplication => "DUPLICATION",
            OpportunityKind::DeadCode => "DEAD CODE",
            OpportunityKind::Pattern => "PATTERN",
            OpportunityKind::ModuleConsolidation => "MODULE",
        };

        let _ = writeln!(out, "{}. **[{kind_label}]** {}", i + 1, opp.title);
        let _ = writeln!(out, "   - Est. savings: {} lines", opp.impact.lines_saved);
        let _ = writeln!(out, "   - Files: {}", opp.affected_files.len());
        let _ = writeln!(out, "   - Action: {}", opp.recommendation);
        let _ = writeln!(out);
    }

    out
}

/// Formats a single opportunity for detailed display.
#[must_use]
pub fn format_opportunity_detail(opp: &Opportunity) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "{}", "═".repeat(70));
    let _ = writeln!(out, "{}", opp.title.bold());
    let _ = writeln!(out, "{}", "═".repeat(70));
    let _ = writeln!(out);

    let _ = writeln!(out, "{}", "DESCRIPTION:".cyan());
    let _ = writeln!(out, "{}", opp.description);
    let _ = writeln!(out);

    let _ = writeln!(out, "{}", "IMPACT:".cyan());
    let _ = writeln!(out, "  Lines saved:  ~{}", opp.impact.lines_saved);
    let _ = writeln!(out, "  Tokens saved: ~{}", opp.impact.tokens_saved);
    let _ = writeln!(out, "  Difficulty:   {}/5", opp.impact.difficulty);
    let _ = writeln!(out, "  Confidence:   {:.0}%", opp.impact.confidence * 100.0);
    let _ = writeln!(out, "  Score:        {:.2}", opp.impact.score());
    let _ = writeln!(out);

    let _ = writeln!(out, "{}", "AFFECTED FILES:".cyan());
    for file in &opp.affected_files {
        let _ = writeln!(out, "  - {}", file.display());
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "{}", "RECOMMENDATION:".cyan());
    let _ = writeln!(out, "{}", opp.recommendation);

    out
}
