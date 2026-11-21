//! CI-01: acceptance of the trusted-base, advisory change-impact reporter.
//! Python fixtures plant missing refs, graph-consumed documentation, renames,
//! symlinks, mixed changes and an untrusted PR-head classifier override.

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
        .to_path_buf()
}

#[test]
fn trusted_change_impact_classification_ci_01() {
    let output = Command::new("python3")
        .args([
            "-I",
            "-m",
            "unittest",
            "discover",
            "-s",
            "scripts/ci",
            "-p",
            "test_pr_stage.py",
            "-v",
        ])
        .current_dir(root())
        .output()
        .expect("Python regression suite is executable");
    assert!(
        output.status.success(),
        "CI-01 regression suite failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// CI-02 shares the strict Python suite invoked by CI-01, but its registered
/// test name makes the new behavior independently visible to conformance.
#[test]
fn trusted_factory_obligation_plan_ci_02() {
    let output = Command::new("python3")
        .args(["-I", "scripts/ci/test_obligation_plan.py"])
        .current_dir(root())
        .output()
        .expect("CI-02 fixture suite is executable");
    assert!(
        output.status.success(),
        "CI-02 negative controls failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
