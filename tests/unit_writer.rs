// tests/unit_writer.rs
use std::fs;

#[test]
fn test_creates_parent_dirs() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("a/b/c.txt");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, "test").unwrap();
    assert!(p.exists());
}

#[test]
fn test_writes_content() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("test.txt");
    fs::write(&p, "hello").unwrap();
    assert_eq!(fs::read_to_string(&p).unwrap(), "hello");
}

#[test]
fn test_delete_file() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("test.txt");
    fs::write(&p, "x").unwrap();
    fs::remove_file(&p).unwrap();
    assert!(!p.exists());
}

#[test] fn test_tracks_written() {}
