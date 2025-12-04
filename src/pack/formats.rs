// src/pack/formats.rs
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{FocusContext, PackOptions};
use crate::skeleton;

/// Packs files into the SlopChop format.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_slopchop(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    for path in files {
        write_slopchop_file(out, path, should_skeletonize(path, opts))?;
    }
    Ok(())
}

/// Packs files into the SlopChop format with focus awareness.
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
    if foveal.is_empty() {
        return Ok(());
    }

    writeln!(out, "# ═══ FOVEAL (full content) ═══\n")?;
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
    let peripheral: Vec<_> = files
        .iter()
        .filter(|f| focus.peripheral.contains(*f))
        .collect();
    if peripheral.is_empty() {
        return Ok(());
    }

    writeln!(out, "# ═══ PERIPHERAL (signatures only) ═══\n")?;
    for path in peripheral {
        write_slopchop_file_skeleton(out, path)?;
    }
    Ok(())
}

fn write_slopchop_file(out: &mut String, path: &Path, skeletonize: bool) -> Result<()> {
    let p_str = path.to_string_lossy().replace('\\', "/");
    writeln!(out, "#__SLOPCHOP_FILE__# {p_str}")?;

    match fs::read_to_string(path) {
        Ok(content) if skeletonize => out.push_str(&skeleton::clean(path, &content)),
        Ok(content) => out.push_str(&content),
        Err(e) => writeln!(out, "// <ERROR READING FILE: {e}>")?,
    }

    writeln!(out, "\n#__SLOPCHOP_END__#\n")?;
    Ok(())
}

fn write_slopchop_file_skeleton(out: &mut String, path: &Path) -> Result<()> {
    let p_str = path.to_string_lossy().replace('\\', "/");
    writeln!(out, "#__SLOPCHOP_FILE__# {p_str} [SKELETON]")?;

    match fs::read_to_string(path) {
        Ok(content) => out.push_str(&skeleton::clean(path, &content)),
        Err(e) => writeln!(out, "// <ERROR READING FILE: {e}>")?,
    }

    writeln!(out, "\n#__SLOPCHOP_END__#\n")?;
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
    let attr = focus_attr.map_or(String::new(), |f| format!(" focus=\"{f}\""));

    writeln!(out, "  <document path=\"{p_str}\"{attr}><![CDATA[")?;

    match fs::read_to_string(path) {
        Ok(content) => {
            let text = if skeletonize {
                skeleton::clean(path, &content)
            } else {
                content
            };
            out.push_str(&text.replace("]]>", "]]]]><![CDATA[>"));
        }
        Err(e) => writeln!(out, "<!-- ERROR: {e} -->")?,
    }

    writeln!(out, "]]></document>")?;
    Ok(())
}

fn should_skeletonize(path: &Path, opts: &PackOptions) -> bool {
    if opts.skeleton {
        return true;
    }
    if let Some(target) = &opts.target {
        return !path.ends_with(target);
    }
    false
}
