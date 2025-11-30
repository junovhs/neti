// tests/unit_manifest.rs
use warden_core::apply::manifest;

#[test]
fn test_parse_manifest() {
    let input = "∇∇∇ MANIFEST ∇∇∇\na.rs\nb.rs [NEW]\n∆∆∆";
    let m = manifest::parse_manifest(input).unwrap();
    assert!(m.is_some());
}

#[test]
fn test_new_marker() {
    let input = "∇∇∇ MANIFEST ∇∇∇\na.rs [NEW]\n∆∆∆";
    let m = manifest::parse_manifest(input).unwrap().unwrap();
    assert!(m.iter().any(|e| e.operation == warden_core::apply::types::Operation::New));
}

#[test]
fn test_delete_marker() {
    let input = "∇∇∇ MANIFEST ∇∇∇\na.rs [DELETE]\n∆∆∆";
    let m = manifest::parse_manifest(input).unwrap().unwrap();
    assert!(m.iter().any(|e| e.operation == warden_core::apply::types::Operation::Delete));
}

#[test]
fn test_default_update() {
    let input = "∇∇∇ MANIFEST ∇∇∇\na.rs\n∆∆∆";
    let m = manifest::parse_manifest(input).unwrap().unwrap();
    assert!(m.iter().any(|e| e.operation == warden_core::apply::types::Operation::Update));
}
