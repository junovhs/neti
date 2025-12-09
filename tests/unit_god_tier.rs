use slopchop_core::audit::{diff, parameterize, codegen};
use tree_sitter::{Parser, Node};
use slopchop_core::audit::diff::{DiffKind, Hole};
use slopchop_core::audit::parameterize::RefactorStrategy;
use std::fs;
use tempfile::TempDir;
use slopchop_core::audit::types::{Opportunity, OpportunityKind, Impact, CodeUnit, CodeUnitKind, Fingerprint};
use slopchop_core::audit::enhance;
use slopchop_core::config::Config;
use std::collections::HashSet;

fn parse(code: &str) -> (tree_sitter::Tree, tree_sitter::Parser) {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    (tree, parser)
}

#[test]
fn test_diff_simple_variant() {
    let code_a = "fn test() { let x = 1; }";
    let code_b = "fn test() { let x = 2; }";
    
    let (tree_a, _) = parse(code_a);
    let (tree_b, _) = parse(code_b);
    
    let root_a = tree_a.root_node();
    let root_b = tree_b.root_node();
    
    let model = diff::diff_trees(root_a, code_a.as_bytes(), root_b, code_b.as_bytes()).unwrap();
    
    // Should find at least one hole (the "1" vs "2")
    assert!(!model.holes.is_empty());
    
    // Debug output if fails
    for hole in &model.holes {
        println!("Found hole: {:?}", hole);
    }

    let hole = model.holes.iter().find(|h| h.kind == "integer_literal").unwrap();
    assert_eq!(hole.variants, vec!["1", "2"]);
}

#[test]
fn test_parameterize_strategy_enum() {
    let hole = Hole {
        kind: "string_literal".to_string(),
        variants: vec!["mode_a".to_string(), "mode_b".to_string()],
        path_id: "test".to_string(),
    };
    
    let model = slopchop_core::audit::diff::DiffModel {
        holes: vec![hole],
    };
    
    let strategies = parameterize::infer_strategies(&model);
    assert_eq!(strategies.len(), 1);
    
    match &strategies[0] {
        RefactorStrategy::ExtractEnum { name, variants } => {
            assert!(name.contains("STR")); // check base type heuristic
            assert_eq!(variants.len(), 2);
        },
        _ => panic!("Expected ExtractEnum"),
    }
}

#[test]
fn test_codegen_enum() {
    let strategy = RefactorStrategy::ExtractEnum {
        name: "MyEnum".to_string(),
        variants: vec!["Variant1".to_string(), "Variant2".to_string()],
    };
    
    let output = codegen::generate_refactor(&strategy, "original.rs").unwrap();
    assert!(output.contains("enum MyEnum"));
    assert!(output.contains("Variant1"));
    assert!(output.contains("#__SLOPCHOP_FILE__#"));
}

#[test]
fn test_enhance_plan_generation() {
    let dir = TempDir::new().unwrap();
    let file_a = dir.path().join("a.rs");
    let file_b = dir.path().join("b.rs");
    
    let src_a = "fn foo() { println!(\"Hello\"); }";
    let src_b = "fn foo() { println!(\"World\"); }";
    
    fs::write(&file_a, src_a).unwrap();
    fs::write(&file_b, src_b).unwrap();
    
    let unit_a = CodeUnit {
        file: file_a.clone(),
        name: "foo".to_string(),
        kind: CodeUnitKind::Function,
        start_line: 1,
        end_line: 1, 
        fingerprint: Fingerprint { hash: 0, depth: 0, node_count: 0 },
        tokens: 10,
    };
    
    let unit_b = CodeUnit {
        file: file_b.clone(),
        ..unit_a.clone()
    };
    
    let opportunity = Opportunity {
        id: "test".to_string(),
        title: "dup".to_string(),
        description: "dup".to_string(),
        kind: OpportunityKind::Duplication,
        impact: Impact { lines_saved: 0, tokens_saved: 0, difficulty: 0, confidence: 1.0 },
        affected_files: HashSet::new(),
        recommendation: "".to_string(),
        refactoring_plan: None,
        units: vec![unit_a, unit_b],
    };
    
    let mut opts = vec![opportunity];
    let config = Config::default();
    enhance::enhance_opportunities(&mut opts, 1, &config);
    
    assert!(opts[0].refactoring_plan.is_some(), "Should generate a plan");
    let plan = opts[0].refactoring_plan.as_ref().unwrap();
    assert!(plan.contains("enum") || plan.contains("ENUM"), "Plan should suggest enum logic");
}
