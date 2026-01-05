use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{FocusContext, PackOptions};
use crate::apply::patch::common::compute_sha256;
use crate::skeleton;

const SIGIL: &str = "XSC7XSC";

/// Packs files into the `SlopChop` format.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_slopchop(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    for path in files {
        write_slopchop_file(out, path, should_skeletonize(path, opts))?;
    }
    Ok(())
}

/// Packs files into the `SlopChop` format with focus awareness.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_slopchop_focus(
    files: &[PathBuf],
    out: &mut String,
    opts: &PackOptions,
    focus: &FocusContext,
) -> Result<()> {
    if focus.foveal.is_empty() && focus.peripheral.is_empty() {
        return pack_slopchop(files, out, opts);
    }

    write_foveal_section(out, files, focus)?;
    write_peripheral_section(out, files, focus)?;

    Ok(())
}

fn write_foveal_section(out: &mut String, files: &[PathBuf], focus: &FocusContext) -> Result<()> {
    let foveal: Vec<_> = files.iter().filter(|f| focus.foveal.contains(*f)).collect();
    if foveal.is_empty() { return Ok(()); }

    writeln!(out, "\n{SIGIL} FOVEAL {SIGIL} (Full Content)\n")?;
    for path in foveal {
        write_slopchop_file(out, path, false)?;
    }
    Ok(())
}

fn write_peripheral_section(
    out: &mut String,
    files: &[PathBuf],
    focus: &FocusContext,
) -> Result<()> {
    let peripheral: Vec<_> = files.iter().filter(|f| focus.peripheral.contains(*f)).collect();
    if peripheral.is_empty() { return Ok(()); }

    writeln!(out, "\n{SIGIL} PERIPHERAL {SIGIL} (Signatures Only)\n")?;
    for path in peripheral {
        write_slopchop_file_skeleton(out, path)?;
    }
    Ok(())
}

fn write_slopchop_file(out: &mut String, path: &Path, skeletonize: bool) -> Result<()> {
    let p_str = path.to_string_lossy().replace('\\', "/");
    
    match fs::read_to_string(path) {
        Ok(content) => {
            let hash = compute_sha256(&content);
            writeln!(out, "{SIGIL} FILE {SIGIL} {p_str} SHA256:{hash}")?;
            
            if skeletonize {
                out.push_str(&skeleton::clean(path, &content));
            } else {
                out.push_str(&content);
            }
        }
        Err(e) => {
            writeln!(out, "{SIGIL} FILE {SIGIL} {p_str}")?;
            writeln!(out, "// <ERROR READING FILE: {e}>")?;
        }
    }

    writeln!(out, "\n{SIGIL} END {SIGIL}\n")?;
    Ok(())
}

fn write_slopchop_file_skeleton(out: &mut String, path: &Path) -> Result<()> {
    let p_str = path.to_string_lossy().replace('\\', "/");
    
    match fs::read_to_string(path) {
        Ok(content) => {
            let hash = compute_sha256(&content);
            writeln!(out, "{SIGIL} FILE {SIGIL} {p_str} [SKELETON] SHA256:{hash}")?;
            out.push_str(&skeleton::clean(path, &content));
        }
        Err(e) => {
            writeln!(out, "{SIGIL} FILE {SIGIL} {p_str} [SKELETON]")?;
            writeln!(out, "// <ERROR READING FILE: {e}>")?;
        }
    }

    writeln!(out, "\n{SIGIL} END {SIGIL}\n")?;
    Ok(())
}

/// Packs files into an XML format.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_xml(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    writeln!(out, "<documents>")?;
    for path in files {
        write_xml_doc(out, path, should_skeletonize(path, opts), None)?;
    }
    writeln!(out, "</documents>")?;
    Ok(())
}

/// Packs files into XML format with focus awareness.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_xml_focus(
    files: &[PathBuf],
    out: &mut String,
    opts: &PackOptions,
    focus: &FocusContext,
) -> Result<()> {
    if focus.foveal.is_empty() && focus.peripheral.is_empty() {
        return pack_xml(files, out, opts);
    }

    writeln!(out, "<documents>")?;
    write_xml_foveal(out, files, focus)?;
    write_xml_peripheral(out, files, focus)?;
    writeln!(out, "</documents>")?;

    Ok(())
}

fn write_xml_foveal(out: &mut String, files: &[PathBuf], focus: &FocusContext) -> Result<()> {
    for path in files.iter().filter(|f| focus.foveal.contains(*f)) {
        write_xml_doc(out, path, false, Some("foveal"))?;
    }
    Ok(())
}

fn write_xml_peripheral(out: &mut String, files: &[PathBuf], focus: &FocusContext) -> Result<()> {
    for path in files.iter().filter(|f| focus.peripheral.contains(*f)) {
        write_xml_doc(out, path, true, Some("peripheral"))?;
    }
    Ok(())
}

fn write_xml_doc(
    out: &mut String,
    path: &Path,
    skeletonize: bool,
    focus_attr: Option<&str>,
) -> Result<()> {
    let p_str = path.to_string_lossy().replace('\\', "/");
    
    match fs::read_to_string(path) {
        Ok(content) => {
            let hash = compute_sha256(&content);
            let attr = focus_attr.map_or(String::new(), |f| format!(" focus=\"{f}\""));
            writeln!(out, "  <document path=\"{p_str}\" sha256=\"{hash}\"{attr}><![CDATA[")?;

            let text = if skeletonize { skeleton::clean(path, &content) } else { content };
            out.push_str(&text.replace("]]>", "]]]]><![CDATA[>"));
        }
        Err(e) => {
            writeln!(out, "  <document path=\"{p_str}\">")?;
            writeln!(out, "<!-- ERROR: {e} -->")?;
        }
    }

    writeln!(out, "]]></document>")?;
    Ok(())
}

fn should_skeletonize(path: &Path, opts: &PackOptions) -> bool {
    if opts.skeleton { return true; }
    if let Some(target) = &opts.target { return !path.ends_with(target); }
    false
}

/// Packs files into a Markdown specification based on doc comments.
///
/// # Errors
/// Returns error if formatting fails.
pub fn pack_spec(files: &[PathBuf]) -> Result<String> {
    let mut out = String::new();
    let mut has_content = false;

    writeln!(out, "# Codebase Specification")?;
    writeln!(out, "\n*Auto-generated by SlopChop Holographic Spec*\n")?;

    for path in files {
        if let Ok(content) = fs::read_to_string(path) {
             match crate::pack::docs::extract_from_path(path, &content) {
                 Ok(docs) if !docs.trim().is_empty() => {
                     has_content = true;
                     writeln!(out, "## {}\n", path.display())?;
                     writeln!(out, "{docs}\n")?;
                 }
                 _ => {}
             }
        }
    }

    if !has_content {
        writeln!(out, "\nNo documentation found in selected files.")?;
    }

    Ok(out)
}
