//! GUI smoke test: drives the real `ui/index.html` in headless Edge/Chrome via
//! the Node harness (`../ui/tests/gui_smoke.mjs`). The test is skipped when
//! Node.js is not installed, so `cargo test` stays portable.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn desktop_ui_smoke_test_passes() {
    let node = Command::new("node").arg("--version").output();
    let node = match node {
        Ok(out) if out.status.success() => out,
        _ => {
            eprintln!("skipping GUI test: node is not available");
            return;
        }
    };
    assert!(
        String::from_utf8_lossy(&node.stdout)
            .trim()
            .starts_with('v'),
        "unexpected node --version output"
    );

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("ui")
        .join("tests")
        .join("gui_smoke.mjs");
    assert!(script.is_file(), "missing GUI test script: {script:?}");

    let out = Command::new("node")
        .arg(&script)
        .output()
        .expect("spawn node GUI test");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    println!("{stdout}");
    if !stderr.is_empty() {
        eprintln!("{stderr}");
    }
    assert!(
        out.status.success(),
        "GUI smoke test failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("11 passed, 0 failed"),
        "unexpected GUI test summary:\n{stdout}"
    );
}
