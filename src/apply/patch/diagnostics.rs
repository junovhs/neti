// src/apply/patch/diagnostics.rs
//! Patch failure diagnostics with "Did you mean?" probe logic.

use super::common::PatchInstruction;
use std::fmt::Write;

pub fn diagnose_zero_matches(content: &str, search: &str, instr: &PatchInstruction) -> String {
    let mut msg = String::from("Patch failed: Could not find exact match for the SEARCH block.\n");
    
    // 1. Probe for "Did you mean?"
    if let Some(candidate) = find_closest_candidate(content, search) {
        let _ = writeln!(msg, "\n🔎 Did you mean this region?");
        let _ = writeln!(msg, "{}", "-".repeat(40));
        let _ = writeln!(msg, "{}", indent_lines(&candidate, "  "));
        let _ = writeln!(msg, "{}", "-".repeat(40));

        append_diff_summary(&mut msg, search, &candidate);
    }

    // 2. Context Mismatch Details
    append_mismatch_details(&mut msg, search);

    if let Some(l) = &instr.context_left {
        if !content.contains(l.trim()) {
             msg.push_str("\n⚠️  LEFT_CTX was not found in the file.");
        }
    }
    
    msg.push_str("\n\nNEXT: Regenerate the patch using the correct context, or send the full FILE.");
    msg
}

fn indent_lines(block: &str, prefix: &str) -> String {
    if block.is_empty() { return String::new(); }
    let mut out = String::new();
    for (i, line) in block.lines().enumerate() {
        if i > 0 { out.push('\n'); }
        out.push_str(prefix);
        out.push_str(line);
    }
    out
}

fn append_diff_summary(msg: &mut String, expected: &str, candidate: &str) {
    let _ = writeln!(msg, "\nDiff summary (expected vs found):");
    
    let exp_lines: Vec<&str> = expected.lines().take(8).collect();
    let cand_lines: Vec<&str> = candidate.lines().take(8).collect();
    
    let limit = exp_lines.len().max(cand_lines.len()).min(8);
    
    for i in 0..limit {
        let e = exp_lines.get(i).copied().unwrap_or("");
        let c = cand_lines.get(i).copied().unwrap_or("");
        
        if e == c {
            let _ = writeln!(msg, "  {e}");
        } else {
            let _ = writeln!(msg, "- {e}");
            let _ = writeln!(msg, "+ {c}");
        }
    }
    
    if expected.lines().count() > 8 || candidate.lines().count() > 8 {
        let _ = writeln!(msg, "  ... (truncated)");
    }
}

fn append_mismatch_details(msg: &mut String, search: &str) {
    msg.push_str("\n📝 Context mismatch details:\n");
    let head = search.lines().next().unwrap_or("???");
    let tail = search.lines().last().unwrap_or("???");
    let _ = writeln!(msg, "Expected start: '{}'", head.trim());
    let _ = writeln!(msg, "Expected end:   '{}'", tail.trim());
}

pub fn find_closest_candidate(content: &str, search: &str) -> Option<String> {
    // We need significant enough probes to be useful and safe
    let search_chars: Vec<char> = search.chars().collect();
    if search_chars.len() < 40 { return None; } 

    let head: String = search_chars.iter().take(20).collect();
    let tail: String = search_chars.iter().rev().take(20).rev().collect();

    let head_matches: Vec<_> = content.match_indices(&head).collect();
    let tail_matches: Vec<_> = content.match_indices(&tail).collect();

    find_best_match(content, &head_matches, &tail_matches, search.len())
}

fn find_best_match(
    content: &str,
    head_matches: &[(usize, &str)],
    tail_matches: &[(usize, &str)],
    search_len: usize,
) -> Option<String> {
    for (h_idx, _) in head_matches {
        for (t_idx, _) in tail_matches {
            if *t_idx > *h_idx && is_plausible_match(*h_idx, *t_idx, search_len) {
                return Some(extract_candidate(content, *h_idx, *t_idx));
            }
        }
    }
    None
}

fn is_plausible_match(h_idx: usize, t_idx: usize, search_len: usize) -> bool {
    let dist = t_idx - h_idx;
    let expected_dist = search_len.saturating_sub(40); // Rough bytes est
    
    // Allow wide variance since edits happen (50% tolerance)
    let diff = dist.abs_diff(expected_dist);
    diff < (expected_dist / 2)
}

fn extract_candidate(content: &str, start_idx: usize, end_idx: usize) -> String {
    // Expand around matches
    let context_start = start_idx.saturating_sub(50);
    let context_end = (end_idx + 50).min(content.len());
    
    // Ensure we cut at char boundaries
    let safe_start = floor_char_boundary(content, context_start);
    let safe_end = ceil_char_boundary(content, context_end);
    
    content[safe_start..safe_end].replace('\n', "\n  ")
}

pub fn diagnose_ambiguous(count: usize, matches: &[(usize, &str)], content: &str) -> String {
    let mut msg = format!("Patch failed: Ambiguous match. Found {count} occurrences.\n\n");
    
    msg.push_str("🔎 Occurrences found at:\n");
    for (i, (idx, _)) in matches.iter().enumerate().take(3) {
        let line_num = content[..*idx].lines().count() + 1;
        let safe_end = ceil_char_boundary(content, *idx + 40);
        let snippet = &content[*idx..safe_end].lines().next().unwrap_or("").trim();
        let _ = writeln!(msg, "{}. Line {}: {}...", i + 1, line_num, snippet);
    }
    
    if count > 3 {
        msg.push_str("... and others.\n");
    }

    msg.push_str("\nNEXT: Add more context (LEFT_CTX / RIGHT_CTX) to make the patch unique.");
    msg
}

fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() { return s.len(); }
    while !s.is_char_boundary(idx) {
        idx = idx.saturating_sub(1);
    }
    idx
}

fn ceil_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() { return s.len(); }
    while !s.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}