// tests/integration_patch.rs
// slopchop:ignore
use anyhow::Result;
use sha2::{Digest, Sha256};
use slopchop_core::apply::types::{ApplyContext, ApplyInput, ApplyOutcome};
use slopchop_core::config::Config;
use slopchop_core::stage::StageManager;
use std::fs;
use tempfile::TempDir;

const SIGIL: &str = "XSC7XSC";

fn setup_env() -> Result<(TempDir, Config)> {
    let temp = TempDir::new()?;
    let config = Config::default();
    Ok((temp, config))
}

fn apply(root: &std::path::Path, payload: &str) -> Result<ApplyOutcome> {
    let ctx = ApplyContext {
        config: &Config::default(),
        repo_root: root.to_path_buf(),
        force: true, // Bypass confirmations
        dry_run: false,
        input: ApplyInput::Clipboard, // Mocked
        check_after: false,
        auto_promote: false,
        reset_stage: false,
        sanitize: true,
    };
    slopchop_core::apply::process_input(payload, &ctx)
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

#[test]
fn test_patch_success() -> Result<()> {
    let (env, _) = setup_env()?;
    let root = env.path();

    // 1. Setup Base File with footer to avoid EOF newline issues
    let base_payload = format!(
        r#"{SIGIL} MANIFEST {SIGIL}
src/target.rs [NEW]
{SIGIL} END {SIGIL}

{SIGIL} FILE {SIGIL} src/target.rs
fn main() {{
    println!("Old");
}}
// footer
{SIGIL} END {SIGIL}
"#
    );
    apply(root, &base_payload)?;

    // 2. Apply Patch
    let stage = StageManager::new(root);
    let path = stage.worktree().join("src/target.rs");
    let content = fs::read_to_string(&path)?;
    let hash = compute_hash(&content);

    let patch_payload = format!(
        r#"{SIGIL} MANIFEST {SIGIL}
src/target.rs
{SIGIL} END {SIGIL}

{SIGIL} PATCH {SIGIL} src/target.rs
BASE_SHA256: {hash}
MAX_MATCHES: 1
LEFT_CTX:
fn main() {{
OLD:
    println!("Old");
RIGHT_CTX:
}}
NEW:
    println!("New");
{SIGIL} END {SIGIL}
"#
    );

    apply(root, &patch_payload)?;

    // 3. Verify
    let content = fs::read_to_string(path)?;
    
    if !content.contains("println!(\"New\");") {
        eprintln!("CONTENT WAS:\n{content}");
    }

    assert!(content.contains("println!(\"New\");"));
    assert!(!content.contains("println!(\"Old\");"));
    Ok(())
}

#[test]
fn test_patch_reject_ambiguous() -> Result<()> {
    let (env, _) = setup_env()?;
    let root = env.path();

    // 1. Setup Ambiguous File with footer to ensure newlines
    let base_payload = format!(
        r"{SIGIL} MANIFEST {SIGIL}
ambig.rs [NEW]
{SIGIL} END {SIGIL}

{SIGIL} FILE {SIGIL} ambig.rs
repeat
repeat
// footer
{SIGIL} END {SIGIL}
"
    );
    apply(root, &base_payload)?;

    let stage = StageManager::new(root);
    let path = stage.worktree().join("ambig.rs");
    let content = fs::read_to_string(&path)?;
    let hash = compute_hash(&content);

    // 2. Attempt Ambiguous Patch (should match both 'repeat' lines)
    let patch_payload = format!(
        r"{SIGIL} MANIFEST {SIGIL}
ambig.rs
{SIGIL} END {SIGIL}

{SIGIL} PATCH {SIGIL} ambig.rs
BASE_SHA256: {hash}
MAX_MATCHES: 1
LEFT_CTX:
OLD:
repeat
RIGHT_CTX:
NEW:
fixed
{SIGIL} END {SIGIL}
"
    );
    
    let result = apply(root, &patch_payload)?;
    
    match result {
        ApplyOutcome::ParseError(msg) => {
            assert!(msg.contains("Ambiguous"), "Should fail with Ambiguous error, got: {msg}");
        }
        _ => panic!("Should have failed with ParseError due to ambiguity"),
    }
    Ok(())
}

#[test]
fn test_patch_sha256_verification() -> Result<()> {
    let (env, _) = setup_env()?;
    let root = env.path();

    let base_payload = format!(
        r"{SIGIL} MANIFEST {SIGIL}
secure.rs [NEW]
{SIGIL} END {SIGIL}

{SIGIL} FILE {SIGIL} secure.rs
secret_data
{SIGIL} END {SIGIL}
"
    );
    apply(root, &base_payload)?;

    let patch_payload = format!(
        r"{SIGIL} MANIFEST {SIGIL}
secure.rs
{SIGIL} END {SIGIL}

{SIGIL} PATCH {SIGIL} secure.rs
BASE_SHA256: badhash123
MAX_MATCHES: 1
LEFT_CTX:
OLD:
secret_data
RIGHT_CTX:
NEW:
exposed
{SIGIL} END {SIGIL}
"
    );

    let result = apply(root, &patch_payload)?;
    
    match result {
        ApplyOutcome::ParseError(msg) => {
            assert!(msg.contains("Base SHA256 verification failed"), "Should fail SHA check");
        }
        _ => panic!("Should have failed SHA verification"),
    }
    Ok(())
}