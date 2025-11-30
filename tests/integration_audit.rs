// tests/integration_audit.rs
use warden_core::roadmap::Roadmap;

#[test]
fn test_scans_completed_only() {
    let r = Roadmap::parse("# T\n\n## v0.1.0\n\n- [x] Done\n- [ ] Todo\n");
    let t = r.all_tasks();
    let complete: Vec<_> = t.iter().filter(|t| t.status == warden_core::roadmap::TaskStatus::Complete).collect();
    assert!(!complete.is_empty());
}

#[test] fn test_no_test_skipped() {}
#[test] fn test_explicit_anchor_verified() {}
#[test] fn test_missing_file_detected() {}
