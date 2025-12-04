// tests/unit_project.rs
use slopchop_core::project::ProjectType;
use std::fs::File;
use tempfile::TempDir;

#[test]
fn test_detect_rust() {
    let temp = TempDir::new().unwrap();
    File::create(temp.path().join("Cargo.toml")).unwrap();
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Rust);
}

#[test]
fn test_detect_node() {
    let temp = TempDir::new().unwrap();
    File::create(temp.path().join("package.json")).unwrap();
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Node);
}

#[test]
fn test_detect_python() {
    let temp = TempDir::new().unwrap();
    File::create(temp.path().join("requirements.txt")).unwrap();
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Python);
}

#[test]
fn test_detect_go() {
    let temp = TempDir::new().unwrap();
    File::create(temp.path().join("go.mod")).unwrap();
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Go);
}

#[test]
fn test_detect_unknown() {
    let temp = TempDir::new().unwrap();
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Unknown);
}
