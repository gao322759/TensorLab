//! Workspace 快照/恢复 canary（plan.md 4.2，v0.5-B）。
//!
//! 断言：RunMat 0.6.2 的 `export_workspace_state(Force)` /
//! `import_workspace_state` 字节级 replay 在"快照→清空→恢复"后保真
//! （变量值逐位一致），支撑 CLI/桌面/MCP 的长会话持久化
//! （+sict 断点续算的宿主侧升级路径）。

use tl_kernel::Engine;

fn locked_engine() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn preview_of(report: &tl_types::RunReport, name: &str) -> String {
    report
        .variables
        .iter()
        .find(|v| v.name == name)
        .map(|v| v.preview.clone())
        .unwrap_or_default()
}

#[test]
fn workspace_snapshot_restore_roundtrip_is_faithful() {
    let _guard = locked_engine();

    // 第一段：计算 + 快照
    let mut engine = Engine::new().expect("engine");
    let r1 = engine.run(
        "snap_a",
        "x = sqrt(2); M = magic(4) * 1.5; s = 'tensorlab';",
    );
    assert!(r1.error.is_none(), "part A failed: {:?}", r1.error);
    let bytes = engine.snapshot_workspace().expect("snapshot");
    assert!(!bytes.is_empty(), "snapshot must not be empty");

    // 第二段（同一会话）：继续计算得到参考结果
    let r2 = engine.run("snap_b", "y = M(2, 3) + x * 2; z = numel(s) * 2 + 1;");
    assert!(r2.error.is_none(), "part B failed: {:?}", r2.error);
    let y_ref = preview_of(&r2, "y");
    let z_ref = preview_of(&r2, "z");
    assert!(!y_ref.is_empty(), "y must exist");

    // 第三段：全新会话 + 恢复 + 重放第二段 —— 必须逐位一致
    let mut engine2 = Engine::new().expect("engine 2");
    engine2.restore_workspace(&bytes).expect("restore");
    let r3 = engine2.run("snap_b2", "y = M(2, 3) + x * 2; z = numel(s) * 2 + 1;");
    assert!(r3.error.is_none(), "part B replay failed: {:?}", r3.error);
    let y2 = preview_of(&r3, "y");
    let z2 = preview_of(&r3, "z");
    assert_eq!(y_ref, y2, "y must be bit-identical after restore");
    assert_eq!(z_ref, z2, "z must be bit-identical after restore");

    // 清空后恢复也应可用（断点续算的典型序列）
    engine2.clear();
    let r4 = engine2.run("emptied", "q = exist('M');");
    assert!(r4.error.is_none());
    let cleared = preview_of(&r4, "q");
    assert_eq!(cleared, "0", "M must be gone after clear");
    engine2.restore_workspace(&bytes).expect("restore 2");
    let r5 = engine2.run("snap_b3", "y = M(2, 3) + x * 2;");
    assert!(
        r5.error.is_none(),
        "post-clear restore failed: {:?}",
        r5.error
    );
    assert_eq!(preview_of(&r5, "y"), y_ref, "y must survive clear+restore");
}
