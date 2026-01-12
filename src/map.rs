// src/map.rs
//! Repository map generation with tree-style visualization.

use crate::config::Config;
use crate::discovery::discover;
use crate::tokens::Tokenizer;
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

/// Tree node representing a file or directory.
#[derive(Debug, Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    file_info: Option<FileInfo>,
}

/// Metadata for a file entry.
#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    tokens: usize,
}

/// Context for rendering a tree entry.
struct RenderCtx<'a> {
    prefix: &'a str,
    connector: &'a str,
    child_prefix: String,
    deps: Option<&'a HashMap<PathBuf, Vec<PathBuf>>>,
}

/// Generates the repository map output.
///
/// # Errors
/// Returns error if file discovery or reading fails.
pub fn generate(deps: bool) -> Result<String> {
    let config = Config::load();
    let files = discover(&config)?;
    let tree = build_tree(&files);

    let dep_map = if deps {
        let root = std::env::current_dir()?;
        let edges = crate::graph::locality::collect_edges(&root, &files)?;
        let mut m = HashMap::new();
        for (from, to) in edges {
            m.entry(from).or_insert_with(Vec::new).push(to);
        }
        Some(m)
    } else {
        None
    };

    let output = render_tree(&tree, dep_map.as_ref());
    Ok(output)
}

/// Builds a tree structure from a flat list of file paths.
fn build_tree(files: &[PathBuf]) -> TreeNode {
    let mut root = TreeNode::default();

    for path in files {
        insert_path(&mut root, path);
    }

    root
}

/// Inserts a file path into the tree, creating intermediate directories.
fn insert_path(root: &mut TreeNode, path: &Path) {
    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let mut current = root;

    for (i, component) in components.iter().enumerate() {
        let is_file = i == components.len() - 1;
        current = current.children.entry(component.clone()).or_default();

        if is_file {
            current.file_info = read_file_info(path);
        }
    }
}

/// Reads file metadata (size and token count).
fn read_file_info(path: &Path) -> Option<FileInfo> {
    let metadata = fs::metadata(path).ok()?;
    let content = fs::read_to_string(path).ok()?;
    let tokens = Tokenizer::count(&content);

    Some(FileInfo {
        path: path.to_path_buf(),
        size: metadata.len(),
        tokens,
    })
}

/// Renders the tree to a string with box-drawing characters.
fn render_tree(root: &TreeNode, dep_map: Option<&HashMap<PathBuf, Vec<PathBuf>>>) -> String {
    let mut output = String::from("# Repository Map\n\n");
    render_node(&mut output, root, "", dep_map);
    output
}

/// Recursively renders a tree node with proper indentation.
fn render_node(
    output: &mut String,
    node: &TreeNode,
    prefix: &str,
    dep_map: Option<&HashMap<PathBuf, Vec<PathBuf>>>,
) {
    let entries: Vec<_> = node.children.iter().collect();
    let count = entries.len();

    for (i, (name, child)) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let ctx = RenderCtx {
            prefix,
            connector: select_connector(is_last),
            child_prefix: build_child_prefix(prefix, is_last),
            deps: dep_map,
        };
        render_entry(output, name, child, &ctx);
    }
}

/// Selects the appropriate tree connector character.
fn select_connector(is_last: bool) -> &'static str {
    if is_last {
        "└─ "
    } else {
        "├─ "
    }
}

/// Builds the prefix string for child nodes.
fn build_child_prefix(prefix: &str, is_last: bool) -> String {
    if is_last {
        format!("{prefix}   ")
    } else {
        format!("{prefix}│  ")
    }
}

/// Renders a single entry (file or directory).
fn render_entry(output: &mut String, name: &str, node: &TreeNode, ctx: &RenderCtx) {
    let is_dir = node.file_info.is_none() && !node.children.is_empty();

    if is_dir {
        let _ = writeln!(output, "{}{}{name}/", ctx.prefix, ctx.connector);
        render_node(output, node, &ctx.child_prefix, ctx.deps);
    } else if let Some(info) = &node.file_info {
        write_file_line(output, ctx, name, info);
    } else {
        let _ = writeln!(output, "{}{}{name}/", ctx.prefix, ctx.connector);
    }
}

/// Writes a file line with size, token info, and optionally dependencies.
fn write_file_line(output: &mut String, ctx: &RenderCtx, name: &str, info: &FileInfo) {
    let size_str = format_size(info.size);
    let tok_str = format_tokens(info.tokens);
    let _ = write!(
        output,
        "{}{}{name} ({size_str}, {tok_str})",
        ctx.prefix, ctx.connector
    );

    if let Some(map) = ctx.deps {
        if let Some(imports) = map.get(&info.path) {
            let names: Vec<_> = imports
                .iter()
                .filter_map(|p| p.file_name())
                .map(|os| os.to_string_lossy())
                .collect();
            if !names.is_empty() {
                let _ = write!(output, " → [{}]", names.join(", "));
            }
        }
    }
    let _ = writeln!(output);
}

/// Formats byte size in human-readable form.
#[allow(clippy::cast_precision_loss)]
fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// Formats token count with suffix.
#[allow(clippy::cast_precision_loss)]
fn format_tokens(tokens: usize) -> String {
    if tokens >= 1000 {
        format!("{:.1}k toks", tokens as f64 / 1000.0)
    } else {
        format!("{tokens} toks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1_572_864), "1.5 MB");
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500 toks");
        assert_eq!(format_tokens(1500), "1.5k toks");
    }

    #[test]
    fn test_build_tree_structure() {
        let paths = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
            PathBuf::from("src/util/mod.rs"),
        ];
        let tree = build_tree(&paths);

        assert!(tree.children.contains_key("src"));

        if let Some(src) = tree.children.get("src") {
            assert!(src.children.contains_key("main.rs"));
            assert!(src.children.contains_key("lib.rs"));
            assert!(src.children.contains_key("util"));
        } else {
            panic!("Expected 'src' directory in tree");
        }
    }
}
