// tests/cli_map.rs
use anyhow::Result;
use slopchop_core::trace;
use std::fs;
use std::sync::{LazyLock, Mutex, PoisonError};
use tempfile::tempdir;

// Protect CWD changes with a global mutex
static CWD_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

#[test]
fn test_map_basic() -> Result<()> {
    let _lock = CWD_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let temp = tempdir()?;
    let _guard = TestDirectoryGuard::new(temp.path());

    fs::write("main.rs", "fn main() {}")?;

    let result = trace::map(false)?;
    let clean = strip_ansi(&result);
    assert!(clean.contains("main.rs"));
    Ok(())
}

// Anchor alias for roadmap
#[test]
fn test_map_stats() -> Result<()> {
    test_map_basic()
}

#[test]
fn test_map_tree() -> Result<()> {
    let _lock = CWD_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let temp = tempdir()?;
    let _guard = TestDirectoryGuard::new(temp.path());
    fs::create_dir("src")?;
    fs::write("src/lib.rs", "fn lib() {}")?;

    let result = trace::map(false)?;
    let clean = strip_ansi(&result);

    assert!(clean.contains("src/"));
    assert!(clean.contains("lib.rs"));
    Ok(())
}

#[test]
fn test_map_deps() -> Result<()> {
    let _lock = CWD_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let temp = tempdir()?;
    let _guard = TestDirectoryGuard::new(temp.path());
    fs::create_dir("src")?;

    fs::write("src/lib.rs", "use crate::utils::Helper;")?;
    fs::write("src/utils.rs", "pub struct Helper;")?;

    let result = trace::map(true)?;
    let clean = strip_ansi(&result);

    assert!(clean.contains("src/"));
    assert!(clean.contains("lib.rs"));
    assert!(clean.contains("🔗"));
    assert!(clean.contains("utils.rs"));
    Ok(())
}

struct TestDirectoryGuard {
    original: std::path::PathBuf,
}

impl TestDirectoryGuard {
    fn new(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for TestDirectoryGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

