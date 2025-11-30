// tests/unit_project.rs
use std::fs;
use tempfile::TempDir;
use warden_core::project::ProjectType;

fn temp() -> TempDir { tempfile::tempdir().unwrap() }

#[test]
fn test_detect_rust() {
    let d = temp();
    fs::write(d.path().join("Cargo.toml"), "[package]").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    assert_eq!(ProjectType::detect(), ProjectType::Rust);
}

#[test]
fn test_detect_node() {
    let d = temp();
    fs::write(d.path().join("package.json"), "{}").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    assert_eq!(ProjectType::detect(), ProjectType::Node);
}

#[test]
fn test_detect_python() {
    let d = temp();
    fs::write(d.path().join("pyproject.toml"), "[project]").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    assert_eq!(ProjectType::detect(), ProjectType::Python);
}

#[test]
fn test_detect_go() {
    let d = temp();
    fs::write(d.path().join("go.mod"), "module test").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    assert_eq!(ProjectType::detect(), ProjectType::Go);
}

#[test]
fn test_detect_unknown() {
    let d = temp();
    std::env::set_current_dir(d.path()).unwrap();
    assert_eq!(ProjectType::detect(), ProjectType::Unknown);
}
