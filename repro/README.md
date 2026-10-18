# K13 / K14 目录级污染：最小复现与上游 issue 草稿

> 内核：RunMat 0.6.2（`runmat-core`，`CompatMode::Matlab`） · 平台：Windows
> 回归看门狗：`crates/tl-kernel/tests/kernel_canaries.rs`
> 运行全部 canary：`cargo test -p tl-kernel --test kernel_canaries`

## 一、目录结构

```text
repro/
  k13-quadprog-pollution/
    fminsearch.m      # 受害者：只调用一次函数句柄
    quadprog.m        # 污染者：公开函数 + 同文件局部函数
  k14-lsqlin-pollution/
    fminsearch.m      # 同一个受害者
    lsqlin.m          # 污染者：公开函数 + 同文件局部函数
```

每个目录**只有两个 `.m` 文件**，故意不依赖 TensorLab 的 `toolbox/`，可直接作为独立仓库/issue 附件。

## 二、一键复现（TensorLab 仓库内）

```powershell
cargo test -p tl-kernel --test kernel_canaries k13_quadprog_pollutes_same_directory_fminsearch -- --nocapture
cargo test -p tl-kernel --test kernel_canaries k14_lsqlin_pollutes_same_directory_fminsearch -- --nocapture
```

## 三、实测结果（2026-10-17）

| 场景 | 目录内函数定义数 | 结果 |
|---|---|---|
| 只放 `fminsearch.m` | 1 | ✅ 正常返回 `f = 5` |
| `fminsearch.m` + `quadprog.m` | 1 + 2 = 3 | ❌ `Not enough input arguments.` |
| `fminsearch.m` + `lsqlin.m` | 1 + 2 = 3 | ❌ `Not enough input arguments.` |
| `fminsearch.m` + 任意单函数文件 | 2 | ✅ 正常 |
| `fminsearch.m` + `lsqnonneg.m` + 任意单函数文件 | 3 | ❌ `shape mismatch for slice assign` |

**结论（最小触发条件）**：同一目录下的语义函数定义达到 **3 个** 时，目录级函数解析表发生错位（实测 3/4/5/6 个均错位，1/2 个正常）。函数句柄调用会绑定到同目录的另一个函数，具体症状取决于被错绑函数的参数个数/函数体：

- 错绑到 2 参数函数 → `Not enough input arguments.`
- 错绑到含切片赋值的函数 → `shape mismatch for slice assign`
- 错绑到 1 参数函数 → **静默返回错误结果**（实测期望 `f = 5`，实际得到兄弟函数返回的 `3`、`4`，最危险）

这解释了 v0.2 的两个现象：

- **K13**：`toolbox/optimization/` 原本只有 `fminsearch.m` + `lsqnonneg.m`（2 个函数），加入完整 `quadprog.m`（工作集 + 乘子下降）后达到 3 个，`fminsearch` 立即报 `Not enough input arguments.`。
- **K14**：向已有目录加入新 `.m` 后，旧函数编译失败或调用被错绑；v0.2 曾观测到 `semantic function lsqnonneg expected 2 inputs, got 9`。

补充证据：目录里存在一个无法编译的兄弟文件（例如使用 `nargin` / `varargin`）时，编译错误会被归属到另一个**无辜文件**：

```text
Could not compile function source '...\lsqlin.m': local may be read before it is assigned
```

## 四、上游 issue 草稿 #1（K13 / quadprog）

**Title:** Directory-level function resolution corruption: 3rd function in a folder breaks function-handle calls (`Not enough input arguments`)

**Body:**

```markdown
## Summary

RunMat 0.6.2 mis-resolves function calls when a directory contains 3 or more
function definitions. A function-handle call `fun(x)` inside one file can be
bound to a sibling function, producing `Not enough input arguments.` or a wrong
result.

## Minimal reproduction (2 files, 1 directory)

`fminsearch.m` (victim):

```matlab
function [x, f] = fminsearch(fun, x0)
    x = x0;
    f = fun(x);
end
```

`quadprog.m` (polluter: public function + local function in the same file):

```matlab
function x = quadprog(H, f)
    x = qp_core(H, f);
end

function x = qp_core(H, f)
    x = H \ f;
end
```

Driver:

```rust
// Cargo.toml: runmat-core = "0.6.2", futures = "0.3"
use runmat_core::abi::{ExecutionRequest, HostExecutionPolicy, SourceInput};
use runmat_core::{CompatMode, RunMatSession};

fn main() {
    let dir = std::env::args().nth(1).expect("usage: repro <dir>").replace('\\', "/");
    let source = format!(
        "addpath('{dir}'); [x, f] = fminsearch(@(v) (v(1)-3)^2 + (v(2)+1)^2, [0 0]); disp(x);"
    );
    let mut session = RunMatSession::with_options(false, false).unwrap();
    let request = ExecutionRequest::for_source(
        SourceInput::Text { name: "repro".into(), text: source },
        CompatMode::Matlab,
        HostExecutionPolicy::default(),
        session.workspace_handle(),
    );
    let response = futures::executor::block_on(session.execute_request(request));
    match response.result {
        Ok(outcome) => {
            for s in &outcome.streams { print!("{}", s.text); }
            for d in &outcome.diagnostics { eprintln!("{:?}: {}", d.severity, d.message); }
        }
        Err(e) => eprintln!("run error: {e:?}"),
    }
}
```

Run:

```bash
cargo new --bin runmat-k13-repro
cd runmat-k13-repro
# Cargo.toml: runmat-core = "0.6.2", futures = "0.3"
# paste main.rs above; copy fminsearch.m + quadprog.m into ./m/
cargo run -- ./m
```

## Expected

```text
 0
 0
```

(The victim only evaluates the handle once at `x0`, so it returns `x = x0` and
`f = fun(x0)`; the point is that no error is raised.)

## Actual

```text
Error: Not enough input arguments.
```

## Controls

- Keep only `fminsearch.m` in the directory: the same driver succeeds.
- Add any third single-function `.m` file: the failure appears.
- A directory with 2 function definitions works; 3 triggers it.

## Impact

This blocks shipping a pure-m `quadprog` next to `fminsearch`/`lsqnonneg`
(a 3-function directory). TensorLab v0.2 withdrew `quadprog` because of this.

## Environment

- runmat-core 0.6.2 (crates.io), CompatMode::Matlab
- Windows, Rust stable
```

## 五、上游 issue 草稿 #2（K14 / lsqlin）

**Title:** Directory-level function resolution corruption: adding `lsqlin.m` breaks sibling function-handle calls / mis-binds calls

**Body:**

```markdown
## Summary

RunMat 0.6.2 resolves functions per directory using an unstable index. Adding a
new `.m` file to an existing directory can:

1. break function-handle calls in a sibling file (`Not enough input arguments.`);
2. bind a call to the wrong sibling (`semantic function lsqnonneg expected 2 inputs, got 9`
   observed in TensorLab v0.2);
3. attribute a sibling's compile error to an innocent file.

## Minimal reproduction (2 files, 1 directory)

`fminsearch.m` (victim):

```matlab
function [x, f] = fminsearch(fun, x0)
    x = x0;
    f = fun(x);
end
```

`lsqlin.m` (public function + local function in the same file):

```matlab
function x = lsqlin(C, d)
    x = lsq_core(C, d);
end

function x = lsq_core(C, d)
    x = C \ d;
end
```

Driver: see issue #1 / the TensorLab repo (`repro/k14-lsqlin-pollution`).
Call `fminsearch(@(v) (v(1)-3)^2 + (v(2)+1)^2, [0 0])` after `addpath`.

## Actual

```text
Error: Not enough input arguments.
```

Control: with only `fminsearch.m` in the directory, the same call succeeds.

## Additional symptom (compile-error misattribution)

Put `lsqlin.m` next to a file that uses `nargin` (unsupported in file functions):

```text
Could not compile function source '.../lsqlin.m': local may be read before it is assigned
```

The error comes from the sibling file, but is reported against `lsqlin.m`.

## Expected

Adding a sibling file must not change the resolution of any existing function.

## Environment

- runmat-core 0.6.2 (crates.io), CompatMode::Matlab
- Windows, Rust stable
```

## 六、提交状态

- 复现、对照实验、canary 测试均已落在仓库内。
- 上游 issue 正文已按 `runmat-org/runmat` 的 issue 格式整理；**提交动作需要 GitHub 凭据/网络，当前环境未执行**。
- 修复后的收尾清单：
  1. 删除 `repro/k13-*`、`repro/k14-*`；
  2. 把 `quadprog` 合回 `toolbox/optimization/`；
  3. 更新 `docs/kernel-notes.md` 坑 25/26、`docs/gaps.md` K13/K14；
  4. 将 canary 断言从"缺陷存在"改为"正常返回"。
