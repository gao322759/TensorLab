//! 内核 canary 测试：固定 K13–K17 的最小复现，防止内核升级后静默回归。
//!
//! 设计原则：
//! - **K13 / K14 / K16 是当前 RunMat 0.6.2 的已知缺陷**。这些测试断言缺陷仍然存在；
//!   一旦内核升级修复它们，测试会主动失败——这是有意的提醒：删除 `repro/` 目录隔离、
//!   把 `quadprog` 合回 `toolbox/optimization/`，并同步更新 docs/gaps.md。
//! - **K15 / K17 断言 MATLAB 兼容行为**。旧缺陷若重新出现，测试会失败。
//!
//! 复现样本在仓库根 `repro/`：一个目录 + 两个 `.m`，对应"加入某文件后同目录函数编译错位"。
//! 每个用例都使用独立的 `RunMatSession`（0.6.2 的搜索路径是 per-session 的）。

use runmat_core::abi::{DiagnosticSeverity, ExecutionRequest, HostExecutionPolicy, SourceInput};
use runmat_core::{CompatMode, RunMatSession};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// 内核的图窗/环境状态仍有进程级成分；串行化让 canary 在任何测试线程数下都确定。
static KERNEL_LOCK: Mutex<()> = Mutex::new(());

struct Probe {
    output: String,
    errors: Vec<String>,
}

impl Probe {
    fn error_text(&self) -> String {
        self.errors.join("\n")
    }

    fn assert_ok(&self) {
        assert!(
            self.errors.is_empty(),
            "unexpected kernel error:\n{}\noutput:\n{}",
            self.error_text(),
            self.output
        );
    }
}

fn new_session() -> RunMatSession {
    RunMatSession::with_options(false, false).expect("RunMat session")
}

fn run(session: &mut RunMatSession, source: &str) -> Probe {
    let request = ExecutionRequest::for_source(
        SourceInput::Text {
            name: "canary".into(),
            text: source.into(),
        },
        CompatMode::Matlab,
        HostExecutionPolicy::default(),
        session.workspace_handle(),
    );
    let response = futures::executor::block_on(session.execute_request(request));
    let outcome = match response.result {
        Ok(outcome) => outcome,
        Err(err) => {
            return Probe {
                output: String::new(),
                errors: vec![format!("{err:?}")],
            }
        }
    };

    let mut output = String::new();
    for stream in &outcome.streams {
        output.push_str(&stream.text);
    }
    let mut errors = Vec::new();
    for diagnostic in &outcome.diagnostics {
        if matches!(diagnostic.severity, DiagnosticSeverity::Error) {
            errors.push(diagnostic.message.clone());
        }
    }
    Probe { output, errors }
}

fn workspace_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn script_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

fn numbers(probe: &Probe) -> Vec<f64> {
    probe
        .output
        .split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .collect()
}

// ---------------------------------------------------------------- K13

/// K13：`quadprog.m`（公开函数 + 同文件局部函数）与 `fminsearch.m` 同目录时，
/// `fminsearch` 内部的函数句柄调用错位，报 `Not enough input arguments.`。
///
/// 这是上游 bug 看门狗：修复后本测试会失败，届时删除 `repro/k13-quadprog-pollution/`
/// 的目录隔离，并把 `quadprog` 合回 `toolbox/optimization/`。
#[test]
fn k13_quadprog_pollutes_same_directory_fminsearch() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = workspace_path("repro/k13-quadprog-pollution");

    // 对照组：同一个 fminsearch.m 单独放一个目录时工作正常。
    let control = std::env::temp_dir().join("tl_canary_k13_control");
    let _ = std::fs::remove_dir_all(&control);
    std::fs::create_dir_all(&control).expect("create K13 control dir");
    std::fs::copy(dir.join("fminsearch.m"), control.join("fminsearch.m"))
        .expect("copy fminsearch victim");
    let control_path = script_path(&control);
    let mut session = new_session();
    let control_probe = run(
        &mut session,
        &format!(
            "addpath('{control_path}'); \
             [x, f] = fminsearch(@(v) v(1)^2 + v(2)^2, [1 2]); \
             fprintf('%g\\n', f);"
        ),
    );
    control_probe.assert_ok();
    assert!(
        numbers(&control_probe).contains(&5.0),
        "K13 control failed: the victim alone must work. Output: {:?}",
        control_probe.output
    );

    // 实验组：加入 quadprog.m（公开函数 + 同文件局部函数）后，同目录调用错位。
    let dir = script_path(&dir);
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!(
            "addpath('{dir}'); \
             [x, f] = fminsearch(@(v) (v(1)-3)^2 + (v(2)+1)^2, [0 0]); \
             disp(x);"
        ),
    );

    assert!(
        probe.error_text().contains("Not enough input arguments"),
        "K13 repro no longer fails as expected. If RunMat fixed the directory pollution, \
         remove repro/k13-quadprog-pollution and update docs/gaps.md. Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );
}

// ---------------------------------------------------------------- K14

/// K14：`lsqlin.m`（公开函数 + 同文件局部函数）与 `fminsearch.m` 同目录时，
/// 同目录的函数句柄调用同样错位。v0.2 的另一种表现是 `lsqlin(9 args)` 被绑定到
/// `lsqnonneg`（`semantic function lsqnonneg expected 2 inputs, got 9`）。
///
/// 这是上游 bug 看门狗：修复后本测试会失败，届时删除 `repro/k14-lsqlin-pollution/`
/// 的目录隔离。
#[test]
fn k14_lsqlin_pollutes_same_directory_fminsearch() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = script_path(&workspace_path("repro/k14-lsqlin-pollution"));
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!(
            "addpath('{dir}'); \
             [x, f] = fminsearch(@(v) (v(1)-3)^2 + (v(2)+1)^2, [0 0]); \
             disp(x);"
        ),
    );

    assert!(
        probe.error_text().contains("Not enough input arguments"),
        "K14 repro no longer fails as expected. If RunMat fixed the directory pollution, \
         remove repro/k14-lsqlin-pollution and update docs/gaps.md. Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );
}

// ---------------------------------------------------------------- K15

/// K15：`.m` 文件函数内捕获外部变量的匿名闭包，传给 `fminsearch` 后必须保留环境。
///
/// 旧缺陷（v0.2）会报 `Not enough input arguments.`；2026-10-17 用完整
/// `toolbox/optimization/fminsearch.m` + 同文件局部 helper 实测正常，本测试防止回退。
/// 注意：若把 `fminsearch` 换成"只调用一次句柄"的最小 victim 并放进 3 函数目录，
/// 会复现另一种错位（属于 K13），不要误判为 K15。
#[test]
fn k15_closure_keeps_captured_environment() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let dir = std::env::temp_dir().join("tl_canary_k15");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create K15 temp dir");
    std::fs::write(
        dir.join("k15_closure.m"),
        "function x = k15_closure()\n\
             A = [1 0];\n\
             b = 2;\n\
             mu = 10;\n\
             fun = @(z) (z(1) - 1)^2 + (z(2) - 1)^2;\n\
             pen = @(z) k15_penalty(fun, z, A, b, mu);\n\
             x = fminsearch(pen, [0 0]);\n\
         end\n\
         function y = k15_penalty(fun, z, A, b, mu)\n\
             r = A * z - b;\n\
             y = fun(z) + mu * sum(r .* r);\n\
         end\n",
    )
    .expect("write K15 probe");

    let toolbox_optimization = script_path(&workspace_path("toolbox/optimization"));
    let probe_dir = script_path(&dir);
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!(
            "addpath('{toolbox_optimization}'); addpath('{probe_dir}'); \
             x = k15_closure(); fprintf('%g %g\\n', x(1), x(2));"
        ),
    );
    probe.assert_ok();

    let values = numbers(&probe);
    assert!(
        values.len() >= 2,
        "K15 expected two numeric values, got output: {:?}",
        probe.output
    );
    assert!(
        (values[0] - 1.90909).abs() < 1e-3,
        "K15 x(1) lost the closure environment: got {}",
        values[0]
    );
    assert!(
        (values[1] - 1.0).abs() < 1e-3,
        "K15 x(2) lost the closure environment: got {}",
        values[1]
    );
}

// ---------------------------------------------------------------- K16

/// K16：对切片做后缀转置 `x(2:end-1)'` 被内核忽略，结果仍是列向量。
///
/// 这是已知缺陷的看门狗：修复后本测试会失败，届时把断言改成 `1 x 30`，
/// 并移除工具箱中"逐元素构造行向量"的规避写法（docs/kernel-notes.md 坑 28）。
#[test]
fn k16_slice_suffix_transpose_is_ignored() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut session = new_session();
    let probe = run(
        &mut session,
        "x = (1:32)'; a = x(2:end-1)'; fprintf('%d %d\\n', size(a, 1), size(a, 2));",
    );
    probe.assert_ok();

    let values = numbers(&probe);
    assert_eq!(
        values,
        vec![30.0, 1.0],
        "K16 behavior changed: slice transpose is no longer ignored. \
         Update the assertion to 1 x 30 and remove the workaround. Output: {:?}",
        probe.output
    );
}

// ---------------------------------------------------------------- K17

/// K17：`max(row_vector)` 必须返回标量；`roots` 单根必须可索引。
///
/// 旧缺陷返回同形状向量 / 标量 `Complex`，导致 `p(1)` 报错。
/// 当前内核已与 MATLAB 对齐，本测试防止回退。
#[test]
fn k17_scalar_reduction_and_single_root_are_matlab_compatible() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut session = new_session();
    let probe = run(
        &mut session,
        "v = [1 5 3]; m = max(v); p = roots([1 -2]); \
         fprintf('%d %d\\n', size(m, 1), size(m, 2)); \
         fprintf('%d\\n', numel(p)); \
         fprintf('%g\\n', p(1));",
    );
    probe.assert_ok();

    let values = numbers(&probe);
    assert_eq!(
        values,
        vec![1.0, 1.0, 1.0, 2.0],
        "K17 regression: max(row) must be 1x1, roots([1 -2]) must be one indexable value. \
         Output: {:?}",
        probe.output
    );
}
