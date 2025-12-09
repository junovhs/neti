use tree_sitter::Node;

/// Represents the relationship between two nodes in a diff.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DiffKind {
    /// Nodes are structurally identical and have identical content.
    Invariant,
    /// Nodes have the same structure/kind but differ in content (variable names, literals).
    Variant {
        a_text: String,
        b_text: String,
    },
    /// Nodes are structurally different (one has a child the other lacks, or different kinds).
    StructuralMismatch,
}

/// A hole in the AST that needs parameterization.
#[derive(Debug, Clone)]
pub struct Hole {
    /// The specific AST node kind (e.g., "identifier", "`integer_literal`").
    pub kind: String,
    /// The values observed at this hole across the variations.
    pub variants: Vec<String>,
    /// The unique path/id of this hole in the tree topology.
    pub path_id: String,
}

/// The result of comparing two ASTs.
#[derive(Debug, Default)]
pub struct DiffModel {
    pub holes: Vec<Hole>,
    // TODO: Store skeleton (common structure) for codegen
}

struct DiffContext<'a> {
    source_a: &'a [u8],
    source_b: &'a [u8],
}

/// Recursively compares two ASTs and builds a `DiffModel`.
#[must_use]
pub fn diff_trees(
    node_a: Node,
    source_a: &[u8],
    node_b: Node,
    source_b: &[u8],
) -> Option<DiffModel> {
    let mut model = DiffModel::default();
    let ctx = DiffContext { source_a, source_b };
    
    if diff_recursive(&ctx, node_a, node_b, "root", &mut model) {
        Some(model)
    } else {
        None // Structurally incompatible
    }
}

fn diff_recursive(
    ctx: &DiffContext,
    a: Node,
    b: Node,
    path: &str,
    model: &mut DiffModel,
) -> bool {
    let kind = compare_nodes(ctx, a, b);

    match kind {
        DiffKind::StructuralMismatch => false,
        DiffKind::Variant { a_text, b_text } => {
            // Found a leaf that differs. Record a hole.
            model.holes.push(Hole {
                kind: a.kind().to_string(),
                variants: vec![a_text, b_text],
                path_id: path.to_string(),
            });
            true
        }
        DiffKind::Invariant => handle_invariant(ctx, a, b, path, model),
    }
}

fn handle_invariant(
    ctx: &DiffContext,
    a: Node,
    b: Node,
    path: &str,
    model: &mut DiffModel,
) -> bool {
    // Structure/Content matches locally. Check children.
    let count_a = a.child_count();
    let count_b = b.child_count();

    if count_a != count_b {
        return false; // Structural mismatch (different number of children)
    }

    let mut cursor_a = a.walk();
    let mut cursor_b = b.walk();

    let children_a = a.children(&mut cursor_a);
    let children_b = b.children(&mut cursor_b);

    for (i, (child_a, child_b)) in children_a.zip(children_b).enumerate() {
        let sub_path = format!("{path}/{}_{}", child_a.kind(), i);
        if !diff_recursive(ctx, child_a, child_b, &sub_path, model) {
            return false;
        }
    }

    true
}

/// Compares two AST nodes and classifies their difference.
#[must_use]
fn compare_nodes(
    ctx: &DiffContext,
    node_a: Node,
    node_b: Node,
) -> DiffKind {
    if node_a.kind() != node_b.kind() {
        return DiffKind::StructuralMismatch;
    }

    let a_text = node_a.utf8_text(ctx.source_a).unwrap_or("");
    let b_text = node_b.utf8_text(ctx.source_b).unwrap_or("");

    if a_text == b_text {
        return DiffKind::Invariant;
    }

    // Identify if this is a leaf node that we can treat as a parameter
    if is_leaf_value(node_a.kind()) {
        return DiffKind::Variant {
            a_text: a_text.to_string(),
            b_text: b_text.to_string(),
        };
    }

    // For non-leaves, if text differs, it might be due to children differing.
    // We return Invariant here to allow recursion to find the specific leaf diffs.
    // UNLESS the node itself has no named children and relies on text?
    // Most likely recursion handles it.
    DiffKind::Invariant
}

fn is_leaf_value(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "string_literal"
            | "raw_string_literal"
            | "integer_literal"
            | "float_literal"
            | "boolean_literal"
            | "field_identifier"
            | "type_identifier"
            | "primitive_type"
    )
}
