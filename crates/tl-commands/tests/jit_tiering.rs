//! V0.3 JIT canary：分层编译必须在重复执行后进入 JIT 层，且脚本循环应有加速。
//!
//! RunMat 0.6.2 的 tiering 默认阈值是 8 次调用 / 32 次回边；同一 unit 连续执行
//! 10 次左右后 `used_jit` 才会变为 true。这里用固定文本重复执行，避免依赖
//! 单次长循环的后台编译完成时序。

use tl_commands::Engine;

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted[sorted.len() / 2]
}

#[test]
fn jit_can_be_disabled_for_vm_baseline() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let mut engine = Engine::with_jit(false).expect("VM engine");
            let src = "benchmark = 0; for i = 1:200; benchmark = benchmark + i; end";
            for run in 0..12 {
                let report = engine.run("vm-baseline", src);
                assert!(
                    report.error.is_none(),
                    "VM iteration {run} failed: {:?}",
                    report.error.as_ref().map(|e| &e.message)
                );
                assert!(
                    !report.used_jit,
                    "Engine::with_jit(false) must stay on the interpreter tier"
                );
            }
        })
        .expect("spawn VM test thread")
        .join()
        .expect("VM test panicked");
}

#[test]
fn repeated_script_loop_reaches_jit_tier() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let mut engine = Engine::new().expect("engine");
            let src = "benchmark = 0; for i = 1:2000; benchmark = benchmark + i; end";
            let mut cold = Vec::new();
            let mut steady = Vec::new();
            let mut saw_jit = false;
            for run in 0..12 {
                let report = engine.run("jit-tier", src);
                assert!(
                    report.error.is_none(),
                    "iteration {run} failed: {:?}",
                    report.error.as_ref().map(|e| &e.message)
                );
                if run < 3 {
                    cold.push(report.duration_ms);
                }
                if run >= 9 {
                    steady.push(report.duration_ms);
                }
                saw_jit |= report.used_jit;
            }
            let cold_ms = median(&cold);
            let steady_ms = median(&steady);
            assert!(
                saw_jit,
                "JIT tier never activated after 12 repeated invocations (cold={cold_ms:.1}ms)"
            );
            // 典型实测约 3x 加速；这里只要求 10% 改善，避免 CI 计时抖动导致误报。
            assert!(
                steady_ms < cold_ms * 0.9,
                "JIT steady-state should be faster: cold={cold_ms:.1}ms steady={steady_ms:.1}ms"
            );
        })
        .expect("spawn JIT test thread")
        .join()
        .expect("JIT test panicked");
}
