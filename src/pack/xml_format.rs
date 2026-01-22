// src/pack/xml_format.rs
//! XML output format logic.
//! Extracted from formats.rs to satisfy Law of Atomicity.

use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::skeleton;
use crate::utils::compute_sha256;
use super::{FocusContext, PackOptions};

/// Packs files into XML format with focus awareness.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_xml_focus<F>(
    files: &[PathBuf],
    out: &mut String,
    opts: &PackOptions,
    focus: &FocusContext,
    on_progress: &F,
) -> Result<()>
where
    F: Fn(usize, usize, &str) + Sync,
{
    let total = files.len();
    if focus.foveal.is_empty() && focus.peripheral.is_empty() {
        writeln!(out, "<documents>")?;
        for (i, path) in files.iter().enumerate() {
            on_progress(i + 1, total, &format!("Packing {}", path.display()));
            write_xml_doc(out, path, should_skeletonize(path, opts), None)?;
        }
        writeln!(out, "</documents>")?;
        return Ok(());
    }

    writeln!(out, "<documents>")?;
    write_xml_foveal(out, files, focus, on_progress)?;
    write_xml_peripheral(out, files, focus, on_progress)?;
    writeln!(out, "</documents>")?;

    Ok(())
}

fn write_xml_foveal<F>(
    out: &mut String,
    files: &[PathBuf],
    focus: &FocusContext,
    on_progress: &F,
) -> Result<()>
where
    F: Fn(usize, usize, &str) + Sync,
{
    let foveal: Vec<_> = files.iter().filter(|f| focus.foveal.contains(*f)).collect();
    for (i, path) in foveal.iter().enumerate() {
        on_progress(i + 1, foveal.len(), &format!("Packing Foveal: {}", path.display()));
        write_xml_doc(out, path, false, Some("foveal"))?;
    }
    Ok(())
}

fn write_xml_peripheral<F>(
    out: &mut String,
    files: &[PathBuf],
    focus: &FocusContext,
    on_progress: &F,
) -> Result<()>
where
    F: Fn(usize, usize, &str) + Sync,
{
    let peripheral: Vec<_> = files.iter().filter(|f| focus.peripheral.contains(*f)).collect();
    for (i, path) in peripheral.iter().enumerate() {
        on_progress(i + 1, peripheral.len(), &format!("Packing Peripheral: {}", path.display()));
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