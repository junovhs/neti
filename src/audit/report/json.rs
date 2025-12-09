use crate::audit::types::{AuditReport, AuditStats, Opportunity, OpportunityKind};
use std::fmt::Write;

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
