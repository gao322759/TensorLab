//! 把 `cargo xtask layers` 纳入 `cargo test`：分层是项目宪法，
//! 任何破坏分层边界的改动都应让测试变红。

use std::path::PathBuf;
use std::process::Command;

fn xtask_bin() -> PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_xtask") {
        return PathBuf::from(p);
    }
    let exe = std::env::current_exe().expect("current exe");
    let dir = exe
        .parent()
        .and_then(|p| p.parent())
        .expect("target profile dir");
    dir.join(format!("xtask{}", std::env::consts::EXE_SUFFIX))
}

#[test]
fn layering_invariants_hold_for_the_real_workspace() {
    let out = Command::new(xtask_bin())
        .arg("layers")
        .output()
        .expect("spawn xtask layers");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "xtask layers failed (exit={:?})\nstdout:\n{stdout}\nstderr:\n{stderr}",
        out.status.code()
    );
    assert!(
        stdout.contains("layering invariants hold"),
        "unexpected xtask output:\n{stdout}"
    );
}
