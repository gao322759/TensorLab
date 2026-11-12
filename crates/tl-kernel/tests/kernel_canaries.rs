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

// ---------------------------------------------------------------- K20

/// K20：**文件函数内** `max(向量, 标量)` 被降维解释，编译报
/// "reduction dimensions use one-based indexing"（脚本上下文同名调用正常）。
///
/// 2026-11-04 阶段 15（sicdef.defect_map）实证。包层绕行：循环截断。
/// 看门狗：内核修复后本测试失败，届时移除包内循环截断绕行并更新
/// docs/sicpvt/README.md 已知限制 4 与 docs/kernel-notes.md。
#[test]
fn k20_max_vector_scalar_fails_to_compile_in_file_function() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let dir = std::env::temp_dir().join("tl_canary_k20");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create K20 temp dir");
    std::fs::write(
        dir.join("k20_maxvec.m"),
        "function out = k20_maxvec(v)\n\
             out = max(v, 0);\n\
         end\n",
    )
    .expect("write K20 probe");

    let dir = script_path(&dir);
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!("addpath('{dir}'); y = k20_maxvec([1 2 3]); fprintf('%g\\n', y(1));"),
    );
    assert!(
        probe.error_text().contains("reduction dimensions"),
        "K20 repro no longer fails as expected. If RunMat fixed max(vector, scalar) \
         inside file functions, remove the loop-truncation workarounds (sicdef/sic3d) \
         and update docs/gaps.md. Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    // 绕行兼容：循环截断在脚本与文件函数中都正常。
    let mut session = new_session();
    let probe = run(
        &mut session,
        "v = [1 -2 3]; w = v; \
         for i = 1:numel(v)\n\
             if w(i) < 0\n\
                 w(i) = 0;\n\
             end\n\
         end\n\
         fprintf('%g %g %g\\n', w(1), w(2), w(3));",
    );
    probe.assert_ok();
    assert_eq!(
        numbers(&probe),
        vec![1.0, 0.0, 3.0],
        "K20 workaround (loop truncation) regressed"
    );
}

// ---------------------------------------------------------------- K21

/// K21：布尔与字面量的 `==`/`~=` 比较、字符数组与字面量的 `~=` 比较被强转
/// f64，报 "cannot convert Bool(true)/CharArray to f64"。
///
/// 2026-11-04 阶段 13/15 实证。包层绕行：`~x` 与 `strcmp`。
#[test]
fn k21_bool_and_char_comparison_coerce_to_f64() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut session = new_session();
    let probe = run(
        &mut session,
        "b = 1 > 0;\n\
         if b == true\n\
             disp(1);\n\
         end",
    );
    assert!(
        probe.error_text().contains("cannot convert Bool"),
        "K21 repro no longer fails as expected (bool == literal). Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    let mut session = new_session();
    let probe = run(
        &mut session,
        "s = '4H';\n\
         if s ~= '6H'\n\
             disp(1);\n\
         end",
    );
    assert!(
        probe.error_text().contains("cannot convert CharArray"),
        "K21 repro no longer fails as expected (char ~= literal). Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    // 绕行兼容：逻辑非与 strcmp 正常。
    let mut session = new_session();
    let probe = run(
        &mut session,
        "b = 1 > 0; n = 0; \
         if ~b\n\
             n = 1;\n\
         end\n\
         s = '4H'; \
         if strcmp(s, '6H')\n\
             n = 2;\n\
         else\n\
             n = 3;\n\
         end\n\
         fprintf('%d\\n', n);",
    );
    probe.assert_ok();
    assert_eq!(numbers(&probe), vec![3.0], "K21 workaround regressed");
}

// ---------------------------------------------------------------- K22

/// K22：循环内对**struct 字段读取值**做预分配索引存储 `s(k) = a.f` 报
/// "Slicing assignment only supported on tensors or string arrays"
/// （一级/二级链式读取都触发；算术物化 `*1`/`+0` 无效）。
///
/// 2026-11-04 阶段 16（sic3d）实证。包层绕行：concat 累积 `s = [s, a.f]`。
#[test]
fn k22_indexed_store_of_struct_field_value_in_loop_fails() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut session = new_session();
    let probe = run(
        &mut session,
        "a = struct('x', 5);\n\
         s = zeros(1, 1);\n\
         for k = 1:1\n\
             v = a.x;\n\
             s(k) = v;\n\
         end",
    );
    assert!(
        probe.error_text().contains("Slicing assignment"),
        "K22 repro no longer fails as expected. If RunMat fixed indexed stores of \
         struct-field values inside loops, replace the concat accumulation in \
         sic3d.solve and update docs/gaps.md. Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    // 绕行兼容：concat 累积正常（标量与矩阵都适用）。
    let mut session = new_session();
    let probe = run(
        &mut session,
        "a = struct('x', 5);\n\
         s = zeros(1, 0);\n\
         for k = 1:3\n\
             v = a.x;\n\
             s = [s, v];\n\
         end\n\
         fprintf('%d %g\\n', numel(s), s(3));",
    );
    probe.assert_ok();
    assert_eq!(
        numbers(&probe),
        vec![3.0, 5.0],
        "K22 workaround (concat accumulation) regressed"
    );
}

// ---------------------------------------------------------------- K23

/// K23：**字段 + end/区间切片**的复合读取 `a.f(:, end - 1)` 报
/// "getfield: invalid index element (index must be >= 1)"
/// （脚本上下文即复现；链式 `a.b.c(k)` 字面 struct 正常）。
///
/// 2026-11-04 阶段 16 实证。包层绕行：先取局部变量再切片。
#[test]
fn k23_field_end_slice_read_fails() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut session = new_session();
    let probe = run(
        &mut session,
        "a = struct('f', [1 2 3; 4 5 6]);\n\
         v = a.f(:, end - 1);",
    );
    assert!(
        probe.error_text().contains("getfield: invalid index element"),
        "K23 repro no longer fails as expected. If RunMat fixed field+slice compound \
         reads, remove the temp-variable workarounds and update docs/gaps.md. Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    // 绕行兼容：先取局部变量再切片正常。
    let mut session = new_session();
    let probe = run(
        &mut session,
        "a = struct('f', [1 2 3; 4 5 6]);\n\
         f = a.f;\n\
         v = f(:, end - 1);\n\
         fprintf('%g\\n', v(1));",
    );
    probe.assert_ok();
    assert_eq!(numbers(&probe), vec![2.0], "K23 workaround regressed");
}

// ---------------------------------------------------------------- K24

/// K24：**文件函数内循环花括号生长**（`c = {}` 后循环内 `c{k} = ...`）
/// 编译期拒绝："Could not compile ... index for dimension 1 is outside the
/// proven bound 0"（隔离目录下确定性复现；脚本上下文正常）。
/// 注意：列区间切片赋值 `P(:, a:b) = ...` 与单次字面索引花括号存储在
/// 隔离目录下**正常**——sic3d 首版观测到的对应失败是 K13/K14 跨文件
/// 编译污染假象。sic3d +包上下文的运行期 "brace indexing requires a
/// cell-like value" 未纳入本 canary（+包复现依赖调用顺序）。
///
/// 2026-11-04 阶段 16 实证。包层绕行：行块堆叠矩阵 + concat 累积。
#[test]
fn k24_cell_growth_and_column_range_assign_fail_in_file_functions() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let dir = std::env::temp_dir().join("tl_canary_k24");
    let _ = std::fs::remove_dir_all(&dir);
    // 每个探针独立子目录：同目录多函数会触发 K13/K14 式跨文件编译污染，
    // 让后续看门狗断言失去确定性（对齐 repro/ 的目录隔离架构）。
    let mk = |name: &str, body: &str| {
        let sub = dir.join(name);
        std::fs::create_dir_all(&sub).expect("create K24 probe dir");
        std::fs::write(sub.join(format!("{name}.m")), body).expect("write K24 probe");
        script_path(&sub)
    };
    let dir_cell2 = mk(
        "k24_cell2",
        "function out = k24_cell2()\n\
             c = {}\n\
             for k = 1:2\n\
                 c{k} = k\n\
             end\n\
             out = c{1} + c{2}\n\
         end\n",
    );
    let dir_colrange2 = mk(
        "k24_colrange2",
        "function out = k24_colrange2()\n\
             P = zeros(3, 6)\n\
             B = ones(3, 3)\n\
             a = 1\n\
             b = 3\n\
             P(:, a:b) = B\n\
             out = max(max(P))\n\
         end\n",
    );
    let dir_colrange1 = mk(
        "k24_colrange1",
        "function out = k24_colrange1()\n\
             P = zeros(3, 6)\n\
             B = ones(3, 3)\n\
             P(:, 1:3) = B\n\
             out = max(max(P))\n\
         end\n",
    );
    let dir_cell1 = mk(
        "k24_cell1",
        "function out = k24_cell1()\n\
             c = {};\n\
             c{1} = 5;\n\
             out = c{1};\n\
         end\n",
    );
    let dir_stack = mk(
        "k24_stack",
        "function out = k24_stack()\n\
             nr = 3\n\
             nz = 2\n\
             stk = zeros(0, nr)\n\
             for k = 1:2\n\
                 blk = (k * 10) * ones(nz, nr)\n\
                 stk = [stk; blk]\n\
             end\n\
             out = size(stk, 1) * 100 + max(max(stk))\n\
         end\n",
    );

    // 看门狗：循环内花括号生长被编译期拒绝。
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!("addpath('{dir_cell2}'); y = k24_cell2(); fprintf('%g\\n', y);"),
    );
    assert!(
        probe.error_text().contains("proven bound")
            || probe
                .error_text()
                .contains("brace indexing requires a cell-like value"),
        "K24 cell2 repro no longer fails as expected. Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    // 兼容：列区间切片赋值（变量界/字面量界）在隔离目录下正常。
    for (d, f) in [
        (&dir_colrange2, "k24_colrange2"),
        (&dir_colrange1, "k24_colrange1"),
    ] {
        let mut session = new_session();
        let probe = run(
            &mut session,
            &format!("addpath('{d}'); y = {f}(); fprintf('%g\\n', y);"),
        );
        probe.assert_ok();
        assert_eq!(numbers(&probe), vec![1.0], "{f} compatibility");
    }

    // 兼容：单次字面索引花括号存储正常。
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!("addpath('{dir_cell1}'); y = k24_cell1(); fprintf('%g\\n', y);"),
    );
    probe.assert_ok();
    assert_eq!(numbers(&probe), vec![5.0], "K24 cell1 compatibility");

    // 绕行兼容：行块堆叠 + concat 在文件函数中正常。
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!("addpath('{dir_stack}'); y = k24_stack(); fprintf('%g\\n', y);"),
    );
    probe.assert_ok();
    assert_eq!(
        numbers(&probe),
        vec![420.0],
        "K24 workaround (row-block stack) regressed"
    );
}

// ---------------------------------------------------------------- K26

/// K26：**转置结果的二维（双索引）读取**报 "Index out of bounds"
/// （一维索引正常；普通矩阵的双索引正常；转置 operand 参与矩阵乘正常）。
///
/// 2026-11-04 阶段 v0.4（numerics gmres）实证：
/// `V = v1'; V(idx, :)` 失败而 `V = v1'; V(idx)` 正常。
/// 包层绕行：用 `reshape(v, 1, n)` 物化代替转置。
#[test]
fn k26_transposed_value_two_index_read_fails() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut session = new_session();
    let probe = run(
        &mut session,
        "r = (1:5)';\n\
         tc = (r / 2)';\n\
         idx = 1:5;\n\
         fe = tc(idx, :);",
    );
    assert!(
        probe.error_text().contains("Index out of bounds"),
        "K26 repro no longer fails as expected. If RunMat fixed two-index reads of \
         transposed values, replace the reshape materialisation in numerics gmres \
         and update docs/gaps.md. Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    // 绕行兼容：reshape 物化 + 一维索引正常。
    let mut session = new_session();
    let probe = run(
        &mut session,
        "r = (1:5)';\n\
         tc = reshape(r / 2, 1, 5);\n\
         idx = 1:5;\n\
         fe = tc(idx);\n\
         fprintf('%g\\n', fe(1));",
    );
    probe.assert_ok();
    assert_eq!(numbers(&probe), vec![0.5], "K26 workaround regressed");
}

// ---------------------------------------------------------------- K27

/// K27：**列向量的 1-D 区间读取归一化为行向量**
/// （MATLAB 语义必须保留列方向：`c = (1:5)'; v = c(1:3)` 应为 3x1，
/// 实测 1x3）。附带：竖向 concat 生长的列向量做 1-D 区间读取直接报
/// "cannot convert Tensor ... to f64"（concat 行向量正常）。
///
/// 2026-11-04 阶段 v0.4（numerics gmres）实证。
/// 包层绕行：数值算法内部统一行主序存储 + 横向 concat + reshape 物化。
#[test]
fn k27_column_vector_range_read_normalises_to_row() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut session = new_session();
    let probe = run(
        &mut session,
        "c = (1:5)';\n\
         v = c(1:3);\n\
         fprintf('%d %d\\n', size(v, 1), size(v, 2));",
    );
    probe.assert_ok();
    let values = numbers(&probe);
    assert_eq!(
        values,
        vec![1.0, 3.0],
        "K27 behavior changed: column range read preserves orientation. \
         Update numerics gmres/pcg and docs/gaps.md. Output: {:?}",
        probe.output
    );

    // 竖向 concat 列向量的区间读取：当前损坏的看门狗。
    let mut session = new_session();
    let probe = run(
        &mut session,
        "c = (1:5)';\n\
         cc = [c; c];\n\
         vv = cc(4:8);",
    );
    assert!(
        probe.error_text().contains("cannot convert"),
        "K27 vertical-concat read repro no longer fails as expected. \
         If RunMat fixed it, the numerics row-storage workaround can be revisited. \
         Got:\n{}\noutput:\n{}",
        probe.error_text(),
        probe.output
    );

    // 绕行兼容：横向 concat 行向量的区间读取正常。
    let mut session = new_session();
    let probe = run(
        &mut session,
        "r = (1:5);\n\
         rr = [r, r];\n\
         vv = rr(4:8);\n\
         fprintf('%g\\n', vv(1));",
    );
    probe.assert_ok();
    assert_eq!(
        numbers(&probe),
        vec![4.0],
        "K27 row-concat workaround regressed"
    );
}

// ---------------------------------------------------------------- K25

/// K25：脚本上下文对包函数调用的降级受同一脚本**后续语句**影响
/// （`a1.phi` 是否被读入局部变量改变 `a1 = pkg.f(...)` 的可编译性，
/// 见 sicpvt.asymmetry.solve 的 probe16c/j 复现；无泛化最小复现）。
///
/// 本 canary 只固化**兼容模式**：包调用返回的 struct 字段立即读入局部
/// 变量后使用——这是包层强制约定（docs/sicpvt/README.md 已知限制 4）。
#[test]
fn k25_package_result_fields_read_into_locals_work() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let dir = std::env::temp_dir().join("tl_canary_k25");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create K25 temp dir");
    std::fs::write(
        dir.join("k25_ret.m"),
        "function out = k25_ret()\n\
             out.a = 2;\n\
             out.f = [1 2 3; 4 5 6];\n\
         end\n",
    )
    .expect("write K25 probe");

    let dir = script_path(&dir);
    let mut session = new_session();
    let probe = run(
        &mut session,
        &format!(
            "addpath('{dir}'); r = k25_ret();\n\
             a = r.a;\n\
             f = r.f;\n\
             v = f(2, 3);\n\
             fprintf('%g %g\\n', a, v);"
        ),
    );
    probe.assert_ok();
    assert_eq!(
        numbers(&probe),
        vec![2.0, 6.0],
        "K25 compatibility pattern (read fields into locals) regressed"
    );
}
