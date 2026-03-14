// src/graph/rank/tags.rs
//! Tag types representing definitions and references.

use std::path::PathBuf;

/// A tag representing either a definition or a reference.
#[derive(Debug, Clone)]
pub struct Tag {
    pub file: PathBuf,
    pub name: String,
    pub kind: TagKind,
    pub line: usize,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    Def,
    Ref,
}
