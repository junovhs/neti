// tests/smoke_test.rs
use std::fs;
use std::process::Command;
use std::path::Path;

fn run_slopchop(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--quiet").arg("--");
    for arg in args {
        cmd.arg(arg);
    }
    cmd.output().expect("failed to execute slopchop")
}

#[test]
fn test_happy_path_transaction() {
    // 1. Initial cleanup / check
    // We assume we are in the repo root
    
    // 2. Create branch
    let output = run_slopchop(&["branch", "--force"]);
    assert!(output.status.success(), "branch command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // 3. Apply a change via stdin
    let test_file = "src/smoke_test_dummy.rs";
    let sig = "XSC7XSC";
    let payload = format!(
        "{sig} PLAN {sig}\nGOAL: Smoke test\nCHANGES: Add dummy file\n{sig} END {sig}\n{sig} MANIFEST {sig}\n{test_file} [NEW]\n{sig} END {sig}\n{sig} FILE {sig} {test_file}\npub fn smoke() {{ println!(\"Smoke test\"); }}\n{sig} END {sig}"
    );
    
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--", "apply", "--stdin", "--force"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn slopchop apply");

    use std::io::Write;
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write_all(payload.as_bytes()).expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success(), "apply command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(Path::new(test_file).exists(), "Test file was not created");
    
    // 4. Promote
    let output = run_slopchop(&["promote"]);
    assert!(output.status.success(), "promote command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(Path::new(test_file).exists(), "Test file disappeared after promote");
    
    // Cleanup
    fs::remove_file(test_file).ok();
    // We should probably also switch back to main if needed, 
    // but the promote command should have done that.
}

#[test]
fn test_syntax_validation_rejects_invalid_code() {
    let test_file = "src/invalid_syntax.rs";
    let sig = "XSC7XSC";
    let payload = format!(
        "{sig} PLAN {sig}\nGOAL: Test syntax rejection\nCHANGES: Add invalid file\n{sig} END {sig}\n{sig} MANIFEST {sig}\n{test_file} [NEW]\n{sig} END {sig}\n{sig} FILE {sig} {test_file}\nfn broken( {{ \n{sig} END {sig}"
    );
    
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--", "apply", "--stdin", "--force"])
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn slopchop apply");

    use std::io::Write;
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write_all(payload.as_bytes()).expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stderr");
    // It should NOT succeed because of the syntax error
    assert!(!output.status.success(), "apply should have failed due to syntax error");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Syntax error detected"), "Error message should mention syntax error");
    
    assert!(!Path::new(test_file).exists(), "Invalid file should NOT have been created");
}
