// src/trace/output.rs
use std::collections::HashMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use crate::skeleton;

pub fn render(
    anchor: &Path,
    direct: &[PathBuf],
    indirect: &[PathBuf],
    contents: &HashMap<PathBuf, String>,
) -> String {
    let mut output = String::new();
    write_header(&mut output, anchor, direct, indirect);
    write_anchor(&mut output, anchor, contents);
    write_dependencies(&mut output, direct, contents, "DIRECT");
    write_dependencies(&mut output, indirect, contents, "INDIRECT");
    output
}

fn write_header(out: &mut String, anchor: &Path, direct: &[PathBuf], indirect: &[PathBuf]) {
    let _ = writeln!(out, "# Trace Context: {}\n", anchor.display());
    out.push_str("## Dependency Map\n\n🎯 ANCHOR: ");
    let _ = writeln!(out, "{}", anchor.display());

    if !direct.is_empty() {
        out.push_str("\n📎 DIRECT:\n");
        for d in direct { let _ = writeln!(out, "   └── {}", d.display()); }
    }
    if !indirect.is_empty() {
        out.push_str("\n📦 INDIRECT:\n");
        for i in indirect { let _ = writeln!(out, "   └── {}", i.display()); }
    }
    out.push_str("\n---\n\n");
}

fn write_anchor(out: &mut String, anchor: &Path, contents: &HashMap<PathBuf, String>) {
    let _ = writeln!(out, "## {} [FULL]\n", anchor.display());
    if let Some(content) = contents.get(anchor) {
        out.push_str(content);
        if !content.ends_with('\n') { out.push('\n'); }
    }
    out.push_str("\n\n");
}

fn write_dependencies(out: &mut String, deps: &[PathBuf], contents: &HashMap<PathBuf, String>, label: &str) {
    for dep in deps {
        let Some(content) = contents.get(dep) else { continue; };
        let _ = writeln!(out, "## {} [{label}]\n", dep.display());
        // Clean skeleton with docstrings
        out.push_str(&skeleton::clean(dep, content));
        out.push_str("\n\n");
    }
}