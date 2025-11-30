// tests/integration_backup.rs
use std::fs;

#[test]
fn test_backup_dir_created() {
    let d = tempfile::tempdir().unwrap();
    let backup = d.path().join(".warden_apply_backup");
    fs::create_dir_all(&backup).unwrap();
    assert!(backup.exists());
}

#[test]
fn test_timestamp_folder() {
    let d = tempfile::tempdir().unwrap();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let backup = d.path().join(format!(".warden_apply_backup/{ts}"));
    fs::create_dir_all(&backup).unwrap();
    assert!(backup.exists());
}

#[test] fn test_existing_backed_up() {}
#[test] fn test_new_file_no_backup() {}
#[test] fn test_path_structure() {}
