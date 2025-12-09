use crate::audit::types::{AuditReport, OpportunityKind};
use std::fmt::Write;

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
