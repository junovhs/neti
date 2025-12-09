// src/audit/scoring.rs
//! Impact scoring and prioritization for consolidation opportunities.
//!
//! This module converts raw detections (duplicates, dead code, patterns)
//! into scored and prioritized opportunities.
//!
//! The scoring formula:
//! ```text
//! score = lines_saved × confidence × (1 / difficulty)
//! ```
//!
//! Where:
//! - `lines_saved`: Estimated lines that could be removed
//! - `confidence`: How confident we are this is actually an issue (0-1)
//! - `difficulty`: How hard the refactor would be (1-5)

use super::types::{
    DeadCode, DeadCodeReason, Impact, Opportunity, OpportunityKind, RepeatedPattern,
    SimilarityCluster,
};
use std::collections::HashSet;

/// Converts a similarity cluster into an opportunity.
#[must_use]
pub fn score_duplication(cluster: &SimilarityCluster, id_prefix: &str) -> Opportunity {
    let count = cluster.units.len();
    let affected_files: HashSet<_> = cluster.units.iter().map(|u| u.file.clone()).collect();
    let lines_saved = cluster.potential_savings;

    // Difficulty depends on:
    // - Same file = easier (1-2)
    // - Different files = harder (2-3)
    // - Different modules = hardest (3-4)
    let difficulty = if affected_files.len() == 1 {
        1
    } else if are_same_module(&affected_files) {
        2
    } else {
        3
    };

    // Confidence based on similarity
    let confidence = cluster.similarity;

    let tokens_saved: usize = cluster.units.iter().skip(1).map(|u| u.tokens).sum();

    let kind = cluster.units.first().map_or("unit", |u| u.kind.label());
    let names: Vec<_> = cluster.units.iter().map(|u| u.name.as_str()).collect();

    let title = format!("{} similar {}s: {}", count, kind, names.join(", "));

    let description = build_duplication_description(cluster);
    let recommendation = build_duplication_recommendation(cluster);

    Opportunity {
        id: format!("{id_prefix}-dup-{}", hash_names(&names)),
        title,
        description,
        kind: OpportunityKind::Duplication,
        impact: Impact {
            lines_saved,
            tokens_saved,
            difficulty,
            confidence,
        },
        affected_files,
        recommendation,
    }
}

/// Converts dead code into an opportunity.
#[must_use]
pub fn score_dead_code(dead: &DeadCode, id_prefix: &str) -> Opportunity {
    let lines_saved = dead.unit.line_count();
    let tokens_saved = dead.unit.tokens;

    // Dead code removal is usually easy
    let difficulty = match dead.reason {
        DeadCodeReason::Unused => 1,          // Just delete it
        DeadCodeReason::Unreachable => 1,     // Just delete it
        DeadCodeReason::OnlyDeadCallers => 2, // Need to remove callers too
    };

    // Confidence varies by detection method
    let confidence = match dead.reason {
        DeadCodeReason::Unused => 0.9,          // High confidence
        DeadCodeReason::Unreachable => 0.8,     // Good confidence
        DeadCodeReason::OnlyDeadCallers => 0.7, // Moderate confidence
    };

    let title = format!(
        "Dead {}: {} ({})",
        dead.unit.kind.label(),
        dead.unit.name,
        dead.reason.explanation()
    );

    let description = format!(
        "The {} `{}` in {} appears to be dead code.\n\
         Reason: {}\n\
         Lines: {}-{}",
        dead.unit.kind.label(),
        dead.unit.name,
        dead.unit.file.display(),
        dead.reason.explanation(),
        dead.unit.start_line,
        dead.unit.end_line
    );

    let recommendation = match dead.reason {
        DeadCodeReason::Unused => {
            format!(
                "Remove `{}` from {} - it is defined but never used",
                dead.unit.name,
                dead.unit.file.display()
            )
        }
        DeadCodeReason::Unreachable => {
            format!(
                "Remove `{}` from {} - it cannot be reached from any entry point",
                dead.unit.name,
                dead.unit.file.display()
            )
        }
        DeadCodeReason::OnlyDeadCallers => {
            format!(
                "Remove `{}` along with its dead callers from {}",
                dead.unit.name,
                dead.unit.file.display()
            )
        }
    };

    let mut affected_files = HashSet::new();
    affected_files.insert(dead.unit.file.clone());

    Opportunity {
        id: format!(
            "{id_prefix}-dead-{}-{}",
            dead.unit.name, dead.unit.start_line
        ),
        title,
        description,
        kind: OpportunityKind::DeadCode,
        impact: Impact {
            lines_saved,
            tokens_saved,
            difficulty,
            confidence,
        },
        affected_files,
        recommendation,
    }
}

/// Converts a repeated pattern into an opportunity.
#[must_use]
pub fn score_pattern(pattern: &RepeatedPattern, id_prefix: &str) -> Opportunity {
    let count = pattern.locations.len();
    let affected_files: HashSet<_> = pattern.locations.iter().map(|l| l.file.clone()).collect();
    let lines_saved = pattern.potential_savings;

    // Pattern extraction difficulty depends on complexity
    let difficulty = if count <= 3 {
        2
    } else if count <= 7 {
        3
    } else {
        4 // Many occurrences means more testing needed
    };

    // Lower confidence since patterns may be intentional
    let confidence = 0.6;

    // Rough token estimate
    let tokens_saved = lines_saved * 8; // ~8 tokens per line average

    let title = format!("Pattern: {} ({} occurrences)", pattern.description, count);

    let file_list: Vec<_> = affected_files
        .iter()
        .take(5)
        .map(|f| f.display().to_string())
        .collect();

    let description = format!(
        "Found {} occurrences of: {}\n\nFiles:\n{}{}",
        count,
        pattern.description,
        file_list.join("\n"),
        if affected_files.len() > 5 {
            format!("\n... and {} more", affected_files.len() - 5)
        } else {
            String::new()
        }
    );

    let recommendation = super::patterns::recommend_extraction(pattern);

    Opportunity {
        id: format!("{id_prefix}-pat-{}", hash_string(&pattern.description)),
        title,
        description,
        kind: OpportunityKind::Pattern,
        impact: Impact {
            lines_saved,
            tokens_saved,
            difficulty,
            confidence,
        },
        affected_files,
        recommendation,
    }
}

/// Scores and ranks all opportunities.
pub fn rank_opportunities(mut opportunities: Vec<Opportunity>) -> Vec<Opportunity> {
    // Sort by score (descending)
    opportunities.sort_by(|a, b| {
        let score_a = a.impact.score();
        let score_b = b.impact.score();
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    opportunities
}

// === Helper Functions ===

fn build_duplication_description(cluster: &SimilarityCluster) -> String {
    let mut desc = String::new();
    desc.push_str(&format!(
        "Found {} structurally similar {}s:\n\n",
        cluster.units.len(),
        cluster.units.first().map_or("unit", |u| u.kind.label())
    ));

    for unit in &cluster.units {
        desc.push_str(&format!(
            "- {} in {} (lines {}-{})\n",
            unit.name,
            unit.file.display(),
            unit.start_line,
            unit.end_line
        ));
    }

    desc.push_str(&format!(
        "\nSimilarity: {:.0}%\n\
         Potential savings: ~{} lines",
        cluster.similarity * 100.0,
        cluster.potential_savings
    ));

    desc
}

fn build_duplication_recommendation(cluster: &SimilarityCluster) -> String {
    let files: HashSet<_> = cluster.units.iter().map(|u| &u.file).collect();
    let kind = cluster.units.first().map_or("unit", |u| u.kind.label());

    if files.len() == 1 {
        format!(
            "Consolidate these {} {}s into a single parameterized implementation \
             in {}",
            cluster.units.len(),
            kind,
            files.iter().next().map_or("", |f| f.to_str().unwrap_or(""))
        )
    } else {
        format!(
            "Extract a shared {} to a common module and have all {} locations use it",
            kind,
            cluster.units.len()
        )
    }
}

fn are_same_module(files: &HashSet<std::path::PathBuf>) -> bool {
    let parents: HashSet<_> = files
        .iter()
        .filter_map(|f| f.parent())
        .map(|p| p.to_path_buf())
        .collect();

    parents.len() == 1
}

fn hash_names(names: &[&str]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for name in names {
        name.hash(&mut hasher);
    }
    hasher.finish() % 10000
}

fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() % 10000
}

