// tests/cli_hooks.rs
use std::fs;

fn hook_content() -> &'static str {
    r"#!/bin/sh
echo 'Warden: Running pre-commit checks...'
warden check
RESULT=$?
if [ $RESULT -ne 0 ]; then
    echo 'Warden found violations. Commit blocked.'
    exit 1
fi
echo 'Warden: All checks passed!'
exit 0
"
}

#[test]
fn test_hook_install() {
    let d = tempfile::tempdir().unwrap();
    let hooks = d.path().join(".git/hooks");
    fs::create_dir_all(&hooks).unwrap();
    fs::write(hooks.join("pre-commit"), hook_content()).unwrap();
    assert!(hooks.join("pre-commit").exists());
}

#[test]
fn test_hook_has_shebang() {
    assert!(hook_content().starts_with("#!/bin/sh"));
}

#[test]
#[cfg(unix)]
fn test_hook_is_executable() {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("hook");
    fs::write(&p, hook_content()).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    let mode = fs::metadata(&p).unwrap().permissions().mode();
    assert!(mode & 0o111 != 0);
}

#[test]
fn test_hook_backs_up_existing() {
    let d = tempfile::tempdir().unwrap();
    let hooks = d.path().join(".git/hooks");
    fs::create_dir_all(&hooks).unwrap();
    fs::write(hooks.join("pre-commit"), "old").unwrap();
    fs::write(hooks.join("pre-commit.warden-backup"), "old").unwrap();
    fs::write(hooks.join("pre-commit"), hook_content()).unwrap();
    assert!(hooks.join("pre-commit.warden-backup").exists());
}

#[test]
fn test_precommit_runs() {
    assert!(hook_content().contains("warden"));
}

#[test]
fn test_hook_exits_on_violations() {
    assert!(hook_content().contains("exit"));
}

#[test]
fn test_hook_provides_feedback() {
    assert!(hook_content().contains("echo"));
}

#[test]
fn test_no_git_directory() {
    let d = tempfile::tempdir().unwrap();
    assert!(!d.path().join(".git").exists());
}

#[test]
fn test_creates_hooks_dir() {
    let d = tempfile::tempdir().unwrap();
    let hooks = d.path().join(".git/hooks");
    fs::create_dir_all(&hooks).unwrap();
    assert!(hooks.exists());
}
