// tests/integration_self_host.rs
use slopchop_core::analysis::RuleEngine;
use slopchop_core::config::Config;
use slopchop_core::discovery;

#[test]
fn test_slopchop_passes_own_rules() {
    // 1. Load the actual configuration of the project
    let mut config = Config::new();
    config.load_local_config();

    // 2. Discover files in the current directory (project root)
    // This respects .slopchopignore, so generated files/target are skipped.
    let files = discovery::discover(&config).expect("Discovery failed");

    assert!(!files.is_empty(), "Self-host check found no files!");

    // 3. Run the analysis
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    // 4. Fail if any violations are found
    if report.has_errors() {
        println!("\n⚠️  SLOPCHOP SELF-HOST VIOLATIONS FOUND:");
        for file in &report.files {
            for v in &file.violations {
                println!(
                    "  {}:{}: {} [{}]",
                    file.path.display(),
                    v.row + 1,
                    v.message,
                    v.law
                );
            }
        }
        panic!(
            "SlopChop failed its own rules! Found {} violations.",
            report.total_violations
        );
    }
}
