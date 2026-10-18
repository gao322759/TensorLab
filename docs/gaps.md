# 已知差距与限制

> v0.1 的诚实清单。每条标注：影响、原因、计划。功能对标状态见 [parity-checklist.md](parity-checklist.md)。

## A. 与计划文档不一致的实现选择

计划文档写的是"Monaco 编辑器 + Plotly 绘图"，实际采用自研实现。**功能可用，但字面偏离**。

| # | 项 | 实际实现 | 偏离原因 | 计划 |
|---|---|---|---|---|
| A1 | 编辑器 | 自研 overlay 高亮（textarea + `pre`） | Monaco 约 5MB 且通常需 CDN/npm，与"离线、无 CDN、包体可控"冲突 | 可复议，见 [decisions.md](decisions.md) ADR-005 |
| A2 | 绘图 | 自研 FigureScene→SVG 渲染器 | Plotly.js 全量约 3.5MB | 可复议，见 ADR-004 |

**A1 带来的具体缺失**：无自动补全、无括号匹配、无多光标、无内联错误波浪线、无查找替换、无 minimap。
**A2 带来的具体缺失**：无缩放/平移/hover tooltip/图例点击开关；大数据（>10k 点）无 WebGL 加速；
3D 图（`surf`/`mesh`/`plot3`/`contour`）未实现渲染。

## B. 功能缺口

| # | 缺口 | 影响 | 计划 |
|---|---|---|---|
| B1 | 5 个工具箱未完整 | v0.2 已交付工具箱脚本（`toolbox/` 与 `toolbox/*_extra/`）；Signal/Image/Stats/Control 均达到 20–30 个核心函数，Optimization 补齐至 20 个（`quadprog` 仍受 K13 阻塞） | v0.2 已交付；剩余项见各工具箱文档 |
| B2 | MEX 仅能加载/探测 | v0.2 用 `libloading` 实现真实加载 + `mexFunction` 入口探测（`mex load <path>`，Windows/Linux/macOS 均可）；透明 `foo(x)` 调用仍受 K12 阻塞 | v0.3（待内核注册 API） |
| B3 | ~~JIT 未在宿主层暴露~~ | v0.3 已补：`Engine::with_jit(false)`、`Engine::kernel_stats()`（jit_compiled / interpreter_fallback / osr / vectorized_regions）与 `tensorlab bench --no-jit` | ✅ 已关闭（v0.3-dev） |
| B4 | 无 `.mat` 读写 | `load`/`save` 不可用 | 未排期 |
| B5 | 无文件 IO API | `fopen`/`fread`/`textscan` 不可用 | 未排期 |
| B6 | 3D/图像类绘图未渲染 | 内核能导出对应 plot 类型，桌面端只画 2D | v0.2 候选 |
| B7 | 无安装包 | release 为裸可执行文件，未生成 MSI/NSIS/DMG/AppImage | v0.2 |
| B8 | 变量预览上限 16 元素 | 非数值类型显示 `—`；大矩阵只能看前 16 个 | v0.2（变量编辑器） |
| B9 | `help` 只有 10 个主题 | 非 MATLAB 全文档 | 随内置函数增长同步维护 |
| B10 | 表达式语句的匿名结果值不单独暴露 | `RunReport.resultText` 恒为 `None`（控制台文本里含 `ans = ...`） | 按需 |

## C. 测试覆盖缺口

| # | 缺口 | 影响 | 计划 |
|---|---|---|---|
| C1 | ~~桌面端 UI（HTML/JS）无自动化测试~~ | v0.2 已补：`apps/tl-desktop/ui/tests/gui_smoke.mjs`（Node + Edge headless + CDP，7 条断言），并由 `src-tauri/tests/gui_smoke.rs` 接入 `cargo test` | ✅ 已关闭（v0.2） |
| C2 | 图窗渲染无像素级验证 | 只能确认"有 SVG"，不能确认"画对了" | v0.2 |
| C3 | ~~无性能基准~~ | v0.3 已补：`tensorlab bench`（10+5 协议）、`bench/matlab_bench.m`、`jit_tiering.rs` canary 与 [jit-benchmarks.md](jit-benchmarks.md) | ✅ 已关闭（v0.3-dev） |
| C4 | 仅 Windows 验证 | macOS/Linux 行为未知（大栈线程、路径分隔、HDF5 构建方式） | v0.2 |
| C5 | 大量内核能力"未验证"（见 parity 的 🟡） | 不确定能否工作 | v0.2 优先清 |

## D. 工程与环境

| # | 项 | 说明 |
|---|---|---|
| D1 | 仓库未初始化 git | 环境中无 `git` 可执行文件；v0.1 冻结以文件标记落地，未打 tag |
| D2 | 开发机杀软会锁死新编译 exe | 见 [decisions.md](decisions.md) ADR-009 |
| D3 | 冷构建 10 分钟级 | 内核依赖树（parser/HIR/MIR/VM/runtime + HDF5 + Cranelift + Tauri）庞大 |
| D4 | 需要 VS 自带 CMake | 系统 CMake 不识别 VS 2026 生成器，见 [development.md](development.md) 坑 2 |
| D5 | release 二进制约 100MB 量级 | 与"包体 < 30MB"目标差距大，需专项优化（strip/LTO/裁剪内核特性） |
| D6 | 无 CI | 无自动构建/测试流水线 |
| D7 | 反复运行 `tensorlab.exe` 会残留不可杀进程 | `taskkill /F` 报 `Access is denied`，导致 `cargo test` 报 `failed to remove file ... (os error 5)`；见 [development.md](development.md) 坑 4 变体 |
| D8 | `cargo xtask` 依赖 `.cargo/config.toml` 别名 | 已补上；缺失时报 `no such command: xtask`，见 [development.md](development.md) 坑 7 |

## D2. 内核能力缺口（限制工具箱实现）

由 RunMat 0.6.2 缺失导致的实现受限项（详见 [kernel-notes.md](kernel-notes.md) 坑 14/17/19/20/22）：

| 编号 | 缺失/受限 | 影响 |
|------|-----------|------|
| K1 | 文件函数不支持 `nargin`/`varargin`/`varargout` | 工具箱必须用 `arguments` 块声明可选参数 |
| K2 | 标量不是张量（`x(:)`/切片赋值报错） | 工具箱入口需显式向量化；`fminsearch` 要求 `numel(x0)≥2` |
| K3 | `reshape` 在个别函数体内触发栈溢出 | 需规避并单独实跑验证 |
| K4 | `min`/`max` 偶发编译期误判（`reduction dimensions use one-based indexing`） | 索引/边界钳制一律用标量 `if` |
| K6 | 无 `expm` | state-space `c2d` 不可行（tf 路径可后续实现） |
| K7 | 无 `padarray`/`im2col`/`colfilt`/`nlfilter`/`im2col` | 形态学/滤波用显式循环实现（性能较低） |
| K8 | `classdef` 仅 Path 请求可用 | 0.6.2 解析器支持 classdef，且 `SourceInput::Path` 会加载同目录 class 文件（`Vec2` 实测通过）。v0.3-dev 已新增 `Engine::run_path`，`run_file`/demo/MCP `run` 均改走 Path（[ADR-013](decisions.md)），文件执行可用 classdef；REPL/桌面缓冲区仍为 Text。`zpk`/`pid` 等模型对象尚待实现 |
| K9 | `exist()` 对算子式内置函数返回 0 | 覆盖统计不能只看 `exist`（如 `abs` 实际可用） |
| K10 | `filtfilt` 要求信号长度 > `3*(nfilt-1)` | 短信号需回退 `filter` |
| K11 | 无 `chi2inv`/`gammainc`/`erfc`/`betainc`/`poly` | 已用 `chi2cdf`+二分补齐 `chi2inv`；其余按需自实现 |
| K12 | 无宿主函数注册 API（MEX 透明调用受阻） | `Value::ExternalFunctionHandle` 只是类方法句柄名，不能注册宿主函数；`tl-mex` 因此实现为**动态库加载/入口探测桥**（`mex load <path>`），MATLAB MEX 的 `foo(x)` 透明调用需等待内核提供注册 API。另：`mxArray` 二进制布局未公开，照抄违反 clean-room 红线 |
| K13 | **目录级函数解析污染**（`quadprog` 阻塞） | **已定位最小触发条件**：同一目录下语义函数定义达到 **3 个**时解析表错位。最小复现 `repro/k13-quadprog-pollution/`（`fminsearch.m` + `quadprog.m`，一个目录 + 两个 `.m`）：`fminsearch` 报 `Not enough input arguments`；只放 `fminsearch.m` 时正常。错位也可能表现为 `shape mismatch for slice assign` 或静默错误数值。`quadprog` 实现已验证正确（解 `[1.5,2.5]`、fval −12.5），但为避免破坏 `fminsearch` **暂不交付**【上游 issue 草稿见 `repro/README.md`，待提交】 |
| K14 | **目录级污染（编译错误错位归属）** | 向已有工具箱目录新增 `.m` 会让同目录旧函数编译失败，甚至把调用错误绑定到同目录另一个函数（`semantic function lsqnonneg expected 2 inputs, got 9`）；典型报错会把兄弟文件的错误归到 `lsqlin.m`（`local may be read before it is assigned`）。最小复现 `repro/k14-lsqlin-pollution/`（一个目录 + 两个 `.m`）。规避：新增复杂函数放独立子目录（`toolbox/optim_*`、`toolbox/stats_extra/*`）；详见 kernel-notes 坑 26 |
| K15 | **工具箱函数内的匿名闭包丢环境** | v0.2：`@(z) helper(fun,z,A,b,mu)` 在 `.m` 函数内传给 `fminsearch` 会报 `Not enough input arguments`。**2026-10-17 用 RunMat 0.6.2 实测已不复现**（分离 helper / 同文件局部 helper 均正常，解 ≈ `[1.90909, 1]`）。canary `k15_closure_keeps_captured_environment` 防回退；坑 27 |
| K16 | **切片后缀转置被忽略** | **仍复现**：`x(2:end-1)'` 返回列向量（`30x1`），完整 `x'` 正常（`1x32`），导致隐式广播成矩阵。规避：逐元素循环或显式构造；canary `k16_slice_suffix_transpose_is_ignored`；坑 28 |
| K17 | **标量归约/单根返回差异** | **已对齐 MATLAB**：`max([1 5 3])` 返回 `1x1` 标量；`roots([1 -2])` 返回 `1x1`，`p(1)` 可用。canary `k17_scalar_reduction_and_single_root_are_matlab_compatible` 防回退；坑 29 |
| K18 | **struct 的残留限制** | 无 `isstruct`/`iscell`（用 `strcmp(class(x),...)`）；结构体数组不能自动扩展（`s(2).a=2` 报 `Struct subscript out of bounds`），只能用 `struct('f',{...})` 构造；坑 31 |
| K19 | **性能上限由 RunMat 执行层决定** | release 对标 MATLAB R2023b：`script_loop_5k` 405ms vs 0.04ms、`function_calls_100` 917ms vs 0.023ms、`matmul_300` 236ms vs 3.5ms、`fft_16k` 83ms vs 2.8ms、`vector_math_100k` 139ms vs 16ms。诊断：`used_jit=true`、`native_osr_transfers=0`（顶层脚本循环从未 OSR）、`vectorized_native_regions>0`；release/debug steady 几乎相同；20 次预热（越 specialized 阈值 16）无改善。TensorLab 无法替换内置实现（K12）且 tiering 配置仅 `pub(crate)` 测试接口，因此标量循环/数值内核的进一步优化需上游改动。详见 [jit-benchmarks.md](jit-benchmarks.md) §四 |

## E. 语义层面的注意点

| # | 项 | 说明 |
|---|---|---|
| E1 | 字符串拼接不能用 `+` | 内核 0.6.2 的 `+` 只收 numeric/logical，用 `strcat` |
| E2 | 回显格式由内核决定 | 宿主不渲染 `display_events`（否则重复），因此 `format` 行为不可在宿主侧调整 |
| E3 | 图窗 payload 不可 round-trip | 剥离 `*_source` 后无法 `import_figure_scene`（见 ADR-007） |
| E4 | `magic(4)` 奇异 | 与真 MATLAB 一致（`inv(magic(4))` 报错）；demo 因此用 `magic(5)` |
| E5 | `help` 依赖宿主拦截 | 若绕过 `tl-commands`（直接调 `tl-kernel::Engine`）执行 `help('x')`，会触发内核栈溢出 |

## F. 关闭记录

（每关闭一条，从上面移到此处并注明版本）

| # | 关闭于 | 说明 |
|---|---|---|
| K5 | v0.3-dev（2026-10-17） | 「无 struct」结论错误：`struct()` 构造、字段读写、动态字段、`getfield`/`setfield`/`fieldnames`/`isfield`、`struct('f',{...})` 数组均实测可用。已交付 `strel`/`bwconncomp`/`regionprops` 并接入形态学函数；残留限制见 K18 |
