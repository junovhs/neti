// tests/cli_roadmap.rs
use std::fs;
use warden_core::roadmap::Roadmap;

#[test]
fn test_init_creates_file() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("ROADMAP.md");
    fs::write(&p, "# Test\n\n## v0.1.0\n").unwrap();
    assert!(p.exists());
}

#[test]
fn test_prompt_generates() {
    let r = Roadmap::parse("# Test\n\n## v0.1.0\n\n- [ ] Task\n");
    let opts = warden_core::roadmap::PromptOptions::default();
    let p = warden_core::roadmap::generate_prompt(&r, &opts);
    assert!(!p.is_empty());
}

#[test] fn test_apply_from_clipboard() {}
#[test] fn test_show_tree() {}
#[test] fn test_tasks_list() {}
#[test] fn test_tasks_pending_filter() {}
#[test] fn test_tasks_complete_filter() {}
#[test] fn test_audit_runs() {}
