// src/analysis/patterns/security.rs
//! Security patterns: X01 (SQL injection), X02 (command injection), X03 (hardcoded secrets).

#[path = "security_x01.rs"]
mod security_x01;
#[path = "security_x02.rs"]
mod security_x02;
#[path = "security_x03.rs"]
mod security_x03;

use crate::types::Violation;
use tree_sitter::Node;

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    security_x01::detect_x01_sql(source, root, &mut out);
    security_x02::detect_x02_command(source, root, &mut out);
    security_x03::detect_x03_secrets(source, root, &mut out);
    out
}
