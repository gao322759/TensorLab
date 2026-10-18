# RunMat 0.6.2 内核嵌入笔记

> TensorLab 的内核是 [RunMat](https://github.com/runmat-org/runmat) 0.6.2（**Apache-2.0 / MIT 双许可**），
> 通过 `runmat-core` 的 `RunMatSession` 嵌入。本文件记录**经源码验证**的嵌入 API 契约，来源：
> `crates.io` 发布的 `runmat-core 0.6.2` / `runmat-runtime 0.6.2` 源码包（clean-room 使用：只读公开 API，不抄实现）。

## 版本与兼容目标

| 项 | 值 |
|----|----|
| 兼容 pin | MATLAB R2026a（`version("-release")` 返回该 pin；`verLessThan` 比较 26.1） |
| 发布日期 | 2026-08-28 |
| 许可 | **Apache-2.0 / MIT 双许可**（crate 元数据声明 MIT，仓库主许可 Apache-2.0；两者均允许静态链接闭源/开源衍生） |
| 内置函数 | 1200+（含 linalg、fft、统计、绘图 30+ 类型、ODE、表格） |

## Crate 拓扑（仅列我们依赖的）

- `runmat-core 0.6.2` — session 引擎 + ABI。feature：`default=["jit"]`。
- `runmat-runtime 0.6.2` — 内置函数、绘图语义、replay 编解码。feature：`default=["plot-core"]`。
- `runmat-value` — Value 类型。
- `runmat-plot` — Figure/FigureScene 图形模型（plot-core 拉入）。

## Session 执行（runmat-core）

```rust
// 创建：JIT 默认开启（feature "jit"）
let mut session = RunMatSession::new()?;            // 或 with_options(enable_jit, verbose)
session.set_compat_mode(runmat_core::CompatMode::Matlab); // 默认即 Matlab

// TensorLab 宿主封装：Engine::new() = JIT on；Engine::with_jit(false) = 解释器-only
// let mut engine = tl_commands::Engine::with_jit(false)?;
// JIT 生效需要同一 unit 重复执行（tiering 默认 8 次调用 / 32 次回边）；
// 一次性 tensorlab run 报 [VM] 是预期行为，见 docs/jit-benchmarks.md。

// 请求
let req = runmat_core::abi::ExecutionRequest::for_source(
    runmat_core::abi::SourceInput::Text { name: "script.m".into(), text: src },
    session.compat_mode(),
    runmat_core::abi::HostExecutionPolicy::default(),
    session.workspace_handle(),
);
let resp: ExecutionResponse = futures::executor::block_on(session.execute_request(req));
// ExecutionResponse { source_context, result: Result<ExecutionOutcome, RunError> }
```

`ExecutionOutcome` 关键字段：

| 字段 | 类型 | 用途 |
|------|------|------|
| `flow` | `RuntimeFlow`（NoValue/Single/OutputList/CommaList/DynamicList） | 语句级返回值 |
| `streams` | `Vec<ExecutionStreamEntry{stream: Stdout|Stderr|ClearScreen, text, timestamp_ms}>` | 控制台输出（disp/fprintf/clc） |
| `display_events` | `Vec<DisplayEvent{label: DisplayLabel, value: Value, span}>` | 回显（无分号语句的 `x = ...`） |
| `diagnostics` | `Vec<RuntimeDiagnostic{code, severity: Error|Warning|Info|Hint, message, span, callstack}>` | 错误/警告（成功也可带 warning） |
| `figures_touched` | `Vec<u32>` | 本次执行触及的 figure handle |
| `effects` | `Vec<ObservedEffect>` | workspace clear / cd / path 变化等副作用 |
| `workspace_delta` | `WorkspaceDelta{version, upserts, removals, full_snapshot_required}` | 变量面板增量 |
| `execution_time_ms` / `used_jit` | u64 / bool | 状态栏 |
| `profiling` | `Option<ExecutionProfiling{total_ms, cpu_ms, gpu_ms, kernel_count}>` | 性能 |

错误：`RunError::Runtime(RuntimeError)`（`.message()`、`.identifier()`、callstack）。

## Workspace（runmat-core::session）

- `session.workspace_snapshot() -> WorkspaceSnapshot{full, version, values: Vec<WorkspaceEntry>}`，字段：
  `name / class_name / dtype / shape / is_gpu / size_bytes / preview / residency / preview_token`。
- `session.materialize_variable(target, options) -> MaterializedVariable`（async；展开大变量时才用）。
- `session.clear_variables()`。
- `session.export_workspace_state(mode) -> Option<Vec<u8>>` / `import_workspace_state(bytes)`（字节级 replay，用于 save/load 会话）。

## 图窗（runmat-runtime::builtins::plotting，feature "plot-core"）

- 执行后按 `figures_touched` 导出：
  `runmat_runtime::builtins::plotting::export_figure_scene(handle) -> BuiltinResult<Option<Vec<u8>>>`（async）。
- 返回 JSON：`{schemaVersion, layout: FigureLayout, metadata: FigureMetadata, plots: [ScenePlot…]}`（camelCase）。
- `ScenePlot` 为 tag=kind 的枚举：`line / scatter / bar / stairs / stem / area / errorbar / surface / mesh / line3 / scatter3 / contour / pie / referenceLine / quiver / patch …`，
  数值字段为 `{storage:{dtype,values}, shape}`；surface/mesh/scatter3 大数据可能为 `{refKind:"runmat-data-array-v1", datasetPath, array, shape, chunks}` 数据集引用（decode 时自动水合）。
- `import_figure_scene(bytes)` 可恢复 figure（MCP/桌面端回放用）。
- `clone_figure(handle)`、`current_figure_handle_if_exists()`、`close_all` 语义走 `close` builtins（`close all` 在 m 代码里执行）。

## 控制与输入

- 取消：`session.cancel_execution()`（协作式；VM 在步骤间轮询 interrupt flag）。
- `input()`：`session.install_async_input_handler(Fn(InputRequest)->Fut<InputResponse>)`；CLI 可不装（回落终端）。

## display 语义

- 无分号回显 = `display_events`（label=变量名或 Anonymous），值格式化由 runmat-value 的 FormatMode 完成；
  TensorLab L0 另提供 UI 预览格式化（short/long）。
- `type_info`：分号抑制输出时的类型提示。

## 已验证的坑

1. `execute_request` 是 `async fn`，同 session 不允许并发执行（`ActiveExecutionGuard`）；桌面端 stop 需 `cancel_execution` + 等待返回。
2. runmat-core 依赖树很大（parser/hir/mir/vm/runtime 全量链接）；冷构建时间长是预期内的。
3. codeload.github.com 对大 tarball 易截断/限流；用 `static.crates.io` 的 .crate 包做源码核对更可靠。
4. `docs.rs` 对 runmat 0.6.x 构建失败（部分依赖编译环境问题），API 以源码包为准。
5. session 默认 `compat_mode = CompatMode::Matlab`（init.rs），无需手动设置。
6. **无 `help` 内置函数**（源码核验：builtin catalog 仅 6 条元数据，`fft`/`findobj`/`version`/`fprintf` 均已注册，`help` 缺失）。直接执行 `help('x')` 会触发内核未定义函数深递归 → **主线程栈溢出**。TensorLab 在 tl-commands `repl_line` 层拦截 `help`（tl-interpreter 提供宿主文档表）。
7. **深递归栈溢出**：即使正常代码，内核 parser/eval 递归深度也远超 Windows 主线程默认栈。三个 App 必须在 **64 MiB 大栈工作线程**上跑 Engine（`std::thread::Builder::stack_size(64<<20)`）；`cargo test` 的测试线程默认栈较大故不复现。
8. `Span { start, end }` 是**字节偏移**（非行列），行号需宿主持有源码换算（数 `\n`）。
9. `BindingName(pub String)`：`DisplayLabel::Binding` 取 `&name.0` 得 `&str`。
10. 语义注意：`'a' + 'b'` 是字符数组算术（非拼接）；string `"a" + "b"` 拼接在 0.6.2 中**不支持**（plus 仅收 numeric/logical），用 `strcat`。`magic(4)` 行列式为 0（奇异，真 MATLAB 同样拒绝 inv）。
11. **FigureScene 实测 schema（与 runmat_plot 类型声明不一致！）**：`export_figure_scene` 输出的 replay 载荷是 envelope `{ createdAt, figure: { layout, metadata, plots } }`；`layout`/`metadata` 为 camelCase（`axesRows`/`axesMetadata`/`title`/`xLabel`/`gridEnabled`，per-axes 元数据在 `metadata.axesMetadata[i]`）；但 `plots` 数组元素是**内部 `kind` 标记 + snake_case 字段**：`{ kind: "line", x, y, color_rgba: [f32;4], line_width, line_style: "Solid"|"Dashed"|…, axes_index, label, visible }`（bar 用 `labels`/`values`/`bar_width`，stem 用 `baseline`/`marker_size`）。前端渲染器见 `apps/tl-desktop/ui/index.html` 的 `figureSvg()`；防回归测试 `demo_reports_have_expected_shape`。
12. 内核 figure 状态是**进程级全局**：并行测试/cargo test 多测试共享 handle 计数，`figures_touched` 可能混入其他执行创建的 figure。断言/渲染不要依赖 figure 顺序。
13. **搜索路径支持纯 m 工具箱**（`addpath`/`genpath`/`which`/`path` 均为 builtin，实测 `exist=5`）：`addpath('<dir>')` 后该目录下的 `<fn>.m` 函数**立即可调用并跨执行持久**；`which('fn')` 返回绝对路径；`exist('fn')` 返回 2。这是 v0.2 交付工具箱缺口的机制（见 [decisions.md](decisions.md) ADR-012）。
14. **文件函数不支持 `nargin`/`varargin`**（重要约束）：从搜索路径加载的 `.m` 函数若使用 `nargin` → 编译失败 `local may be read before it is assigned`（MIR `RM-MIR0001`）；若使用 `varargin{...}` → 编译失败 `brace indexing requires a cell-like value`。同样的写法在**内联定义的函数**里是支持的（见 runmat-core `tests.rs`）。**替代方案：用 `arguments` 块声明可选参数**（R2019b+ 现代风格），实测文件函数可用：
    ```matlab
    function y = opt(x, p)
        arguments
            x double
            p double = 1     % 默认值
        end
        y = x * p;
    end
    ```
    > ⚠️ 错误信息里的**文件名可能被误归属**：`addpath` 会预编译目录内所有 `.m`，某个文件编译失败时报出的名字可能是同目录另一个文件。排查时逐个文件隔离。
15. **静态分析较严**：`if c, p = 1; else, p = 2; end` 后的 `p` 可能被判为 `RM-MIR0002`（部分路径未赋值）。写工具箱代码时**先给默认值再在分支里覆盖**，或直接用 `arguments` 块默认值。
16. **标量不是"张量"，切片操作会报错**：
    - `x(:)` 对标量 → `Slicing only supported on tensors`；
    - `V(:,1) = scalar` → `Slicing assignment only supported on tensors or string arrays`；
    - `scalar(1)` → 同上。
    标量与 1×1 张量不等价。工具箱代码若可能收到标量，需在入口显式转成向量（但 `reshape` 见坑 17）。
17. **`reshape` 在个别函数体内会触发内核栈溢出**（进程级崩溃 `0xC00000FD`，不可 try/catch）：实测 `x0 = reshape(x0, numel(x0), 1)` 在 `fminsearch.m` 内稳定崩溃，而同样的 `reshape(x, numel(x), 1)` 在 `xcorr.m`/`moment.m` 内正常。规避：优先用 `x(:)`（非标量时）；必须在函数体内 reshape 时做等价改写并**单独验证**。**任何新增工具箱函数都必须在 `target/debug/tensorlab.exe` 上实跑验证，不能只看编译通过。**
18. **`exist()` 不是可靠的覆盖判据**：`exist('abs')` 返回 0，但 `abs(-3)` 正常工作（部分函数以特殊形式/算子注册）。做函数覆盖统计时需对返回 0 的项再实测调用一次。（v0.2 的 137 个"缺口"里因此可能混入少量实际可用的项。）
19. **内核缺失的常用函数**（实测 `exist=0` 且调用报 Undefined function）：`expm`、`padarray`、`im2col`、`colfilt`、`nlfilter`、`medfilt2`、`ordfilt2`、`chi2inv`、`gammainc`、`erfc`、`beta`、`betainc`、`poly`、`dct`/`idct`、`interp3`、`griddata`、`struct2cell`、`cell2struct`、`isstruct`、`iscell`、`fieldnames`（对 handle/对象）、`nargin`/`varargin`（见坑 14）。这些由 TensorLab 工具箱补齐或在文档中标 ➖/⬜。**注意：`struct` 本身可用（见坑 31），`exist('struct')` 返回 0 属于坑 18 的误判。**
20. **`min`/`max` 偶发编译期误判**：在特定组合下（实测含 `arguments` 默认值 + 对整个索引向量做 `min/max(vector, scalar)` 钳制）函数文件编译失败：`reduction dimensions use one-based indexing`。同样的写法在脚本或简化函数中正常。**规避：对索引/边界的钳制一律用标量 `if`，不要对索引向量整体调用 `min`/`max`。**
21. **`filtfilt` 长度约束**：要求信号长度 > `3*(nfilt-1)`，短信号直接报错 `signal length must be greater than ...`。工具箱中需按长度回退到 `filter`。
22. **`tf` / `ss` 模型对象的属性可读**（无需 OOP 支持）：
    - `tf(1,[1 2 1])` → `class` 为 `tf`；可读 `.Numerator`（double 向量）、`.Denominator`、`.Variable`（char，'s'）、`.Ts`（0 表示连续）、`.InputDelay`；
    - `ss(A,B,C,D)` → `class` 为 `ss`；可读 `.A`/`.B`/`.C`/`.D`/`.Ts`（无 `.Numerator`）；
    - 因此工具箱可用纯 m 实现 `tfdata`/`series`/`parallel`/`evalfr`/`freqresp`/`bode`/`margin`/`isct`/`isdt` 等（`zpk`/`pid` 需要 classdef 模型类；0.6.2 的 classdef 仅在 Path 请求下可用，见坑 31，`Engine::run` 用的是 Text 请求）。
23. **1×1 张量 + 变量下标赋值会失败**：`W = zeros(1,1); k = 1; W(k) = 1;` → `Slicing assignment only supported on tensors or string arrays`；但**字面量**下标 `W(1) = 1` 正常。规避：把数组至少分配 2 个元素（`zeros(n+1,1)`），或对 n==1 走专门分支。实测受影响：`quadprog` 的工作集/乘子数组、`tf2ss`/`ss2tf` 的一阶系统。
24. **`fminsearch`/`ss2tf` 等对 `reshape` 的规避已验证有效**：改用元素循环读写后不再崩溃；新增函数一律不写 `reshape`。
25. **目录级函数解析污染（K13，RunMat 0.6.2，2026-10-17 定位）**：同一目录下的**语义函数定义达到 3 个**时，目录级函数解析表会错位（实测 3/4/5/6 个均错位，1/2 个正常）。最小复现见 `repro/k13-quadprog-pollution/`（一个目录 + 两个 `.m`）：`fminsearch.m`（只做一次 `fun(x)` 调用）+ `quadprog.m`（公开函数 + 同文件局部函数）→ `addpath` 后调用 `fminsearch` 报 `Not enough input arguments.`；只放 `fminsearch.m` 时正常。错位目标不同，症状也不同：`Not enough input arguments`、`shape mismatch for slice assign`，或**静默返回错误数值**（错绑到 1 参数兄弟函数时）。v0.2 的完整 `quadprog`（工作集循环 + 乘子下降）正是因此被阻塞。回归看门狗：`cargo test -p tl-kernel --test kernel_canaries k13_quadprog_pollutes_same_directory_fminsearch -- --nocapture`。上游 issue 草稿见 [../repro/README.md](../repro/README.md)。
26. **目录级污染的第二种表现（K14）**：编译错误会被归属到同目录的另一个无辜文件——`lsqlin.m` 旁边放一个使用 `nargin`/`varargin` 的兄弟文件时，调用 `lsqlin` 报 `Could not compile function source '.../lsqlin.m': local may be read before it is assigned`（错误实际来自兄弟文件）。v0.2 还观测到调用被错误绑定：`semantic function lsqnonneg expected 2 inputs, got 9`。最小复现 `repro/k14-lsqlin-pollution/` 用函数句柄调用稳定复现同一解析错位：`cargo test -p tl-kernel --test kernel_canaries k14_lsqlin_pollutes_same_directory_fminsearch -- --nocapture`。**规避：把新增的复杂函数放进独立子目录**（`addpath(genpath(...))` 会递归挂载）；风险函数交付后回归同目录旧函数。
27. **工具箱函数里的匿名函数闭包会丢环境（K15，已不复现）**：v0.2 在 `.m` 文件函数内写 `pen = @(z) helper(fun, z, A, b, mu)` 并传给 `fminsearch`，会报 `Not enough input arguments`。**2026-10-17 用 RunMat 0.6.2 + 完整 `fminsearch` 实测两种写法（helper 独立文件 / 同文件局部函数）都正常**，解 ≈ `[1.90909, 1]`。注意：若把 `fminsearch` 换成"只调用一次句柄"的最小 victim 并放进 3 函数目录，会触发 K13 的目录级错位（`Inner matrix dimensions must agree`），不要误判为 K15。canary `k15_closure_keeps_captured_environment` 防止回退；v0.2 的 `fmincon`/`ga`/`patternsearch` 仍保留"惩罚项在调用点内联展开"的写法，可读性优先时再改回闭包。
28. **对切片做后缀转置会被忽略（K16，仍复现）**：`a = x(2:end-1)'` 在 RunMat 0.6.2 上仍返回列向量（`size(a)` 为 `30x1`），而完整转置 `x'` 正常（`1x32`）。**规避：不要依赖"切片后立即转置"，改成逐元素循环或显式构造行向量**。canary `k16_slice_suffix_transpose_is_ignored` 固定该行为；内核修复后测试会主动失败，按提示改断言并移除规避写法。
29. **标量归约/单根返回（K17，已对齐 MATLAB）**：2026-10-17 实测 `max([1 5 3])` 返回 `1x1` 标量；`roots([1 -2])` 返回 `1x1`，`p(1)` 可正常取值。旧行为（同形状向量 / 标量 `Complex`）不再复现。canary `k17_scalar_reduction_and_single_root_are_matlab_compatible` 防止回退；工具箱里已有的 `max(max(v))` 写法可以保留，但不是必需。
30. **`tf` 会裁剪分子前导零**：`tf([0 1],[1 1])` 的 `.Numerator` 只保留 `[1]`；`tf2zp` 因此需要先剥离前导零再 `roots`（v0.2 已修）。同理，`ss2tf` 返回的严格真分式分子含前导 0，直接喂给 `tf2zp` 会多出一个原点零点。
31. **`struct` 与 `classdef` 的真实可用性（v0.3 实测，推翻旧结论）**：
    - `struct()` 可用：`s = struct('a', 1, 'b', 2)`、`s.c = 3`、`s.(name) = v` 动态字段赋值、`getfield`/`setfield`/`fieldnames`/`isfield` 均正常；`class(s)` 返回 `struct`。
    - 结构体数组只能用构造器创建：`s = struct('f', {v1, v2})` 得到 1×2；`s(2).a = 2` 自动扩展会报 `Struct subscript out of bounds`。
    - 仍缺 `isstruct`/`iscell`：用 `strcmp(class(x), 'struct')` / `strcmp(class(x), 'cell')` 判断。
    - **`classdef` 可用，但只在 `SourceInput::Path` 请求下生效**：0.6.2 会加载脚本同目录的 class 文件（`Vec2.m` + `main.m` 实测 `class(p)=='Vec2'`、`p.magnitude()==5`）；`SourceInput::Text` 不加载 classdef。Path 请求的 workspace delta 使用 `WorkspaceBindingKey::SourceBinding`，不是 `Interactive`。
    - 因此 `strel`/`bwconncomp`/`regionprops` 等 struct 返回函数已可交付（v0.3 已落地）；`zpk`/`pid` 等 class 对象函数在文件执行（CLI/MCP/demo）下已具备运行条件，`Engine::run_path` 与 `run_file` 已切换为 Path 请求（见 [ADR-013](decisions.md)）；REPL/桌面缓冲区仍走 Text。

## 参考文档

- Session Engine: docs/session/index.md（GitHub HEAD）
- Execution Requests: docs/session/execution-requests.md
- Plotting: docs/plotting/index.md、docs/plotting/host-integration.md
