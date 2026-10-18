# 路线图

> 版本级目标与验收标准。详细候选功能见 [toolboxes/](toolboxes/) 与 [gaps.md](gaps.md)。
> 已发布版本的契约快照见 [releases/](releases/)。

| 版本 | 状态 | 目标 | 验收标准 |
|---|---|---|---|
| **v0.1.0** | 🔒 已冻结（2026-10-16） | 跑通最小可演示 | Tauri 窗口 + 编辑器 + 变量面板 + 执行 + 绘图 + 5 demo + CLI + MCP；`cargo test` 全绿 |
| **v0.2.0** | 🔒 已冻结（2026-10-17） | 5 个 MVP 工具箱 + MEX 加载桥 + GUI 自动化测试 | 每个工具箱 20-30 个核心函数对齐 R2026a；MEX 可加载/探测；`cargo test` 102 全绿；10 demo；安装包不在本次范围 |
| **v0.3** | 进行中（2026-10-17 启动） | JIT 启用 + 性能达标 | Cranelift JIT 路径启用；关键场景（矩阵/FFT/信号）达 MATLAB 0.5x（TensorLab 耗时 ≤ 2× MATLAB）；文档站 |

---

## v0.2 范围（已冻结 2026-10-17）

**锚定目标：MATLAB R2026a 的函数签名与语义。** 验收与契约快照见 [releases/v0.2.md](releases/v0.2.md)。

### 1. 五个 MVP 工具箱（各 20-30 个核心函数）

| 工具箱 | 计划函数数 | 功能清单 |
|---|---|---|
| Signal Processing | 25 | [toolboxes/signal.md](toolboxes/signal.md) |
| Image Processing | 25 | [toolboxes/image.md](toolboxes/image.md) |
| Optimization | 20 | [toolboxes/optimization.md](toolboxes/optimization.md) |
| Statistics & Machine Learning | 25 | [toolboxes/statistics-ml.md](toolboxes/statistics-ml.md) |
| Control System | 20 | [toolboxes/control.md](toolboxes/control.md) |

**起点实测**（`exist()` 探测 246 个 R2026a 目标函数）：内核已提供 109（44%），缺口 137。
缺口以**纯 m-script 文件**交付（见 [ADR-012](decisions.md)），挂在 L7 `tl-toolbox`；
`addpath` 机制已实测可用。逐函数状态见各工具箱文档。

### 2. MEX 接口

`crates/tl-mex` 已由空桩升级为**动态库加载 / 入口探测桥**（`MexLibrary::load` + `mex` 宿主命令）。
**透明 m 代码调用**（`foo(x)`）仍受阻：RunMat 0.6.2 未提供宿主函数注册 API，且 `mxArray` ABI 不公开
（clean-room 红线）。待内核提供注册入口后补齐，详见 [gaps.md](gaps.md) K12。

### 3. 跨平台打包

脚本已就绪：[../packaging/](../packaging/)（`build-windows.ps1` / `build-unix.sh`）。
Windows 便携 ZIP + 可选 MSI/NSIS；macOS DMG、Linux AppImage/deb 需在对应平台构建。

### 4. 命令行与桌面端增强（候选）

- 变量编辑器（突破 16 元素预览上限，走内核 `materialize_variable` + `WorkspaceSliceOptions`）
- ✅ 桌面端 UI 自动化测试（v0.2 已交付：Node + 无头 Edge + CDP，7 条断言）
- 图窗交互（缩放/平移/hover）—— 若采纳，需复议 [ADR-004](decisions.md)

---

## v0.2 后续行动（2026-10-17 冻结后）

> 按 Kent Beck 的“特性—重构交替”原则，以下工作属于**巩固**而非新特性。
> v0.2.0 快照已冻结；主工作区的改动按 v0.2.x / v0.3 记录，不得静默修改冻结契约。

### P0 — 内核阻塞清障（决定 v0.3 上限）

1. **目录级编译污染（K13/K14）**：✅ 最小复现已固化在 `repro/k13-quadprog-pollution/`
   与 `repro/k14-lsqlin-pollution/`（各一个目录 + 两个 `.m`）；触发条件定位为**同目录 3 个
   语义函数定义**。上游 issue 草稿见 `repro/README.md`（提交需 GitHub 凭据，待人工执行）。
   `toolbox/optim_*`、`toolbox/stats_extra/*` 的目录隔离继续作为临时架构；内核修复后
   合并回常规布局，并删除两个 repro 目录。
2. **匿名闭包丢环境（K15）**：✅ 2026-10-17 用 RunMat 0.6.2 实测已不复现（分离 helper /
   同文件局部 helper 均正常）。已加 canary `k15_closure_keeps_captured_environment`；
   下一步可在 `fmincon` 中恢复 `nonlcon` 闭包写法并补非线性约束。
3. **切片转置 / 标量归约（K16/K17）**：✅ 最小复现已并入 kernel-notes 坑 28/29，并各加
   一条 canary（`k16_slice_suffix_transpose_is_ignored`、
   `k17_scalar_reduction_and_single_root_are_matlab_compatible`）。K17 已对齐 MATLAB，
   K16 仍复现。
4. **MEX 透明调用（K12）**：与内核团队确认“宿主函数注册 API”的路线图；
   同时设计 TensorLab 自己的 `mxArray` ABI（clean-room，不照抄 MATLAB 头文件），
   先做 `mexFunction` 的调用契约原型，不等待上游。

### P1 — 对抗性测试补强（当前测试的薄弱面）

1. ✅ 为 v0.2 新增的 34 个工具箱函数补**退化/反例**测试：空输入、标量、奇异矩阵、
   重复极点、不可行约束、非法 method 字符串。实现：
   `crates/tl-commands/tests/adversarial.rs`（11 个测试）。
   发现并记录：`intlinprog` 不可行时可能挂起、`nlinfit` 发散模型触发 SVD NaN panic、
   `dlqr(...,R=0)` 未校验、重复极点 `margin` 报 0°（见 docs/testing.md、各工具箱文档）。
2. ✅ 把 demo 的关键数值升级为 **golden 快照**
   （`engine_contract.rs::all_demos_run_without_errors` 逐字断言 optim_demo/control_demo）；
   GUI 图窗增加结构级断言（2 个轴、2 条折线、4 个点/线、刻度标签与 viewBox），
   见 `apps/tl-desktop/ui/tests/gui_smoke.mjs`（8 条断言）。
3. ✅ 增加“防作弊”测试：`anti_cheat_two_inputs_change_outputs` 覆盖
   fmincon/lsqlin/intlinprog/kalman/c2d/mle/glmfit/freqs/goertzel/convn。
4. 给 `mex load` 增加负例：非 PE 文件、无导出符号、路径不存在（部分已有）。

### P1 — 打包与跨平台

1. 在 Windows 上实际跑通 `packaging/build-windows.ps1`（便携 ZIP + 可选 NSIS/MSI）。
2. 在 macOS / Linux 上跑 `build-unix.sh`（DMG / AppImage / deb）；记录各平台冷构建坑。
3. 发行前统一升版本号（当前 workspace 仍为 `0.1.0`），并补 `v0.2.0` git tag（引入 git 后）。

### P1 — 架构加固（特性间隙的重构）

1. 收敛工具箱目录：内核修复后移除 `optim_*` / `stats_extra` 的临时隔离。
2. 降低 demo 测试时长（GA/PSO/SA 默认迭代、图像 demo 的 `bwdist`），
   目标把全量 `cargo test` 从约 40 分钟降到 15 分钟以内。
3. 统一工具箱函数的错误消息与 `arguments` 默认值风格，补一份函数模板。
4. 评估把 `tl-mcp-server` 的工具名对齐计划书（`matlab_run` / `matlab_eval` / `matlab_plot`），
   以别名方式兼容现有 `run` / `eval` / `plot`。

### P2 — 进入 v0.3

- JIT 在宿主侧暴露（编译统计/开关）、性能基准（矩阵/FFT/信号 0.5x）、
  文档站、包体与启动时间优化。

### 治理规则

- 冻结后对 IPC 载荷、分层、对外名称、控制台语义的修改必须走新版本并写 [CHANGELOG](../CHANGELOG.md)。
- 每个 PR 必须包含：特性或修复 + 对应测试 + 文档同步（toolboxes / gaps / kernel-notes）。
- 不把 demo 数、函数数、代码行数当绩效指标；以用户能否跑通真实脚本、GUI 是否可用、
  Agent 能否通过 MCP 完成任务为准。

---

## v0.3 范围（进行中，2026-10-17 启动）

### 已完成
- ✅ **JIT 开关/统计**：`Engine::new()`（JIT on）/ `Engine::with_jit(false)`（VM baseline）；
  `RunReport.used_jit` + `tl-jit::stats_of` 暴露执行层。
- ✅ **JIT 基线**：新增 `tensorlab bench`（同一会话 10 次预热 + 5 次测量）和
  `crates/tl-commands/tests/jit_tiering.rs` canary。实测 JIT 生效：
  脚本循环 4.12x、FFT 2.23x、向量数学 1.90x、矩阵乘 1.49x（debug 构建）。
  详见 [jit-benchmarks.md](jit-benchmarks.md)。
- ✅ MATLAB 对照脚本 `bench/matlab_bench.m`（同一 workload / 同一协议），
  已在本机 **MATLAB R2023b** 实测并记录到 [jit-benchmarks.md](jit-benchmarks.md)；
  release 正式对标：matmul 68x、FFT 30x、向量数学 8.7x、标量循环 10125x、
  函数调用 39870x（**0.5x 目标未达成**）。
- ✅ **内核统计与 VM 基线**：`Engine::kernel_stats()`（jit_compiled / interpreter_fallback /
  osr / vectorized_regions）与 `tensorlab bench --no-jit`；诊断证据见
  [jit-benchmarks.md](jit-benchmarks.md) §四。
- ✅ **启动时间**：release 5 次实测中位数 **234ms**、稳定区间 230–300ms，满足 <500ms
  （测量命令见 [jit-benchmarks.md](jit-benchmarks.md) §五）。
- ✅ **文档站 + 教程**：`docs/site/`（首页 + 快速上手 / m-script / 绘图与桌面端 / MCP 四篇教程），
  离线可打开、无外部依赖。

### 下一步
- ⬜ **RunMat 0.6.3 升级**：截至 2026-10-18 crates.io 与 GitHub（含 `main`）最新均为
  0.6.2，0.6.3 尚未发布；发布后升级 `runmat-core/runtime/value` 并重跑全量测试与基准。
- ⛔ **标量循环 JIT（内核受限，K19）**：release 405ms vs MATLAB 0.04ms；
  诊断显示 `used_jit=true` 但 `native_osr_transfers=0`，顶层循环从未 OSR；
  tiering 配置仅 `pub(crate)`，宿主无法调整。需 RunMat 上游修复。
- ⬜ **函数调用路径 JIT（本轮不做）**：`function_calls_100` release 917ms；
  JIT 编译调用方，但文件函数分派/边界开销仍占主导。
- ⛔ **数值内核快速路径（内核受限，K19）**：matmul 68x、FFT 30x、向量数学 8.7x；
  TensorLab 无内置函数注册 API（K12），无法替换 `*` / `fft` / `sin` 实现。
- ⬜ **包体 < 30MB**（本轮不做）：当前 release 二进制约 100MB 量级，需 strip/LTO/裁剪内核特性。

---

## 长期数字目标（v0.3）

| 指标 | 目标 |
|---|---|
| m-script 核心语法兼容 | 70-80% |
| 内置函数覆盖 | 150+（5 工具箱 × 30） |
| 性能 | MATLAB 0.5x（TensorLab 耗时 ≤ 2× MATLAB；关键场景） |
| 启动时间 | < 500ms |
| 包体 | < 30MB |
| 用户 | 1000 早期用户 + 1 付费企业 |

## 明确不做（v1.0 再说）

- ❌ Simulink 替代品
- ❌ 完整 OOP（`classdef` 全特性）
- ❌ Live Script（实时编辑器）
- ❌ App Designer（GUI 设计器）
- ❌ 移动端（iOS / Android）
- ❌ 发明新语言（只解释 m-script）
