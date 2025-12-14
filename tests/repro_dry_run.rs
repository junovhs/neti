use anyhow::Result;
use slopchop_core::apply::types::{ApplyContext, ApplyInput, ApplyOutcome};
use slopchop_core::config::Config;
use tempfile::TempDir;

#[test]
fn test_dry_run_prevents_writes() -> Result<()> {
    // Setup: Create a temp directory
    let temp = TempDir::new()?;
    let root = temp.path();
    let target_file = root.join("should_not_exist.rs");

    // Config setup
    let mut config = Config::default();
    config.preferences.auto_commit = false;
    config.preferences.auto_push = false;

    // Create context with dry_run = true
    let ctx = ApplyContext {
        config: &config,
        force: true,
        dry_run: true,
        no_commit: true,
        no_push: true,
        input: ApplyInput::Clipboard,
    };

    // Construct a payload that attempts to write a file.
    // NOTE: We split the markers to prevent the outer SlopChop extractor
    // from parsing this string literal as an actual file block in THIS output.
    let m = "#__SLOPCHOP";
    let payload = format!(
        r"{m}_MANIFEST__#
should_not_exist.rs [NEW]
{m}_END__#

{m}_FILE__# should_not_exist.rs
fn ghost() {{}}
{m}_END__#
"
    );

    // Action: Run process_input directly.
    // We change CWD to the temp root because process_input uses current dir.
    // This is safe here as this is a standalone integration test file.
    let orig_dir = std::env::current_dir()?;
    std::env::set_current_dir(root)?;

    let outcome_result = slopchop_core::apply::process_input(&payload, &ctx);

    // Restore CWD immediately
    std::env::set_current_dir(orig_dir)?;

    let outcome = outcome_result?;

    // Assertions
    match outcome {
        ApplyOutcome::Success { written, .. } => {
            // 1. Check that the outcome says it verified (dry run message)
            assert!(
                written.iter().any(|s| s.contains("Dry Run")),
                "Expected 'Dry Run' message in written list"
            );

            // 2. CRITICAL: Check that the file does NOT exist on disk
            assert!(
                !target_file.exists(),
                "Dry run failed: File was written to disk at {}",
                target_file.display()
            );
        }
        _ => panic!("Dry run should return Success outcome, got: {outcome:?}"),
    }

    Ok(())
}