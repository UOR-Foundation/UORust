//! R5: planted defects prove both Pages identity preflights refuse stale origins.
//! The existing `test` obligation runs this via `cargo test --workspace` in
//! full `just vv`, and both workflow jobs invoke the same Python controls.

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the repository root")
        .to_path_buf()
}

#[test]
fn pages_identity_workflow_negative_controls() {
    let output = Command::new("python3")
        .args(["-I", "scripts/ci/test_pages_identity.py"])
        .current_dir(root())
        .output()
        .expect("Python is available for CI negative controls");
    assert!(
        output.status.success(),
        "Pages identity planted-defect tests failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
