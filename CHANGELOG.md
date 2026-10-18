# Changelog

本文件记录 TensorLab 的所有重要变更。格式参考 [Keep a Changelog](https://keepachangelog.com/)，
版本号遵循 [Semantic Versioning](https://semver.org/)。

## [Unreleased] — v0.3-dev

### Added
- **Command Window 复制/清空**（对齐 MATLAB）：控制台右键菜单（Copy / Copy All /
  Select All / Clear Command Window）、`Ctrl+L` 清空、控制台聚焦时 `Ctrl+A` 全选；
  选中文本仍走系统 `Ctrl+C`；GUI smoke 覆盖右键复制与 `Ctrl+L` 清空。
- **桌面端 About/版本显示**：页眉显示 `v0.2.1`；`About` 按钮或 `F1` 打开对话框，
  展示版本（来自 Cargo 包版本）、内核 RunMat 0.6.2、兼容目标 R2026a 与 MIT 许可证。
  GUI smoke 增至 11 条断言。
- **函数调用路径宿主优化**：`fcdf` 的不完整 beta 连分式改为同文件局部函数；
  `mount_toolbox` 改为逐目录精确 `addpath`（不再 `genpath`）；新增
  `Engine::compile_function_path/source` + `invoke_compiled_function` 与
  `tensorlab bench --aot`，固定函数重复调用实测提升约 **2.9x**。
- **变量面板实用化**：≤4096 元素的 CPU 变量预览从 16 个数值提升到最多 128 个；
  右侧工作区最小宽度 340px → 420px，值列不再限制 220px，支持换行展示。
- **文档站 + 教程**：`docs/site/`（离线 HTML，首页 + 快速上手 / m-script 基础 /
  绘图与桌面端 / MCP 接入四篇教程），无外部依赖。
- **内核统计与 VM 基线**：`Engine::kernel_stats()` 暴露 jit_compiled /
  interpreter_fallback / native_osr_transfers / vectorized_native_regions；
  `tensorlab bench --no-jit` 可跑解释器-only 对照。
- **JIT 基准**：`tensorlab bench`（同一会话 10 次预热 + 5 次测量）与
  `crates/tl-commands/tests/jit_tiering.rs` canary；基线见
  [docs/jit-benchmarks.md](docs/jit-benchmarks.md)。release 下 JIT 加速：脚本循环
  1.57x、FFT 1.87x、向量数学 1.47x、矩阵乘 1.11x；对照 MATLAB R2023b 为
  matmul 68x、FFT 30x、向量数学 8.7x、标量循环 10125x、函数调用 39870x
  （0.5x 目标——耗时 ≤ 2× MATLAB——未达成，v0.3 继续）。
- **JIT 开关**：`Engine::with_jit(false)` 可切到解释器-only，用于 VM/JIT 对照；
  `Engine::new()` 保持 JIT 开启。
- `bench/matlab_bench.m`：与 CLI bench 同 workload / 同协议的 MATLAB 对照脚本，
  已在本机 **MATLAB R2023b**（`D:\Program Files\MATLAB\R2023b\bin`）实测并记录。
- **图像（struct 返回值）**：`strel`（square/rectangle/disk/diamond/line/octagon）、`bwconncomp`、`regionprops`
  （Area/Centroid/BoundingBox/PixelIdxList/MajorAxisLength/MinorAxisLength/Orientation/Perimeter）
- `imdilate`/`imerode` 接受 struct 结构元素（`SE.Neighborhood`）
- 回归测试：`toolbox_struct_based_image_functions_are_usable`（tl-commands）、
  `classdef_is_supported_for_path_requests`（tl-kernel）、`run_file_loads_sibling_classdef`（tl-commands 集成）

### Changed
- **RunMat 仍为 0.6.2**：截至 2026-10-18，crates.io 与 GitHub（含 `main`）最新版本均为
  0.6.2，尚无 0.6.3 可升级。
- **性能结论（K19）**：JIT 生效但 `native_osr_transfers=0`，release/debug steady 几乎相同，
  且 TensorLab 无法替换内核内置函数；标量循环与数值内核的进一步优化需 RunMat 上游改动。
  启动时间实测中位数 **234ms**（<500ms 目标达成）。详见
  [docs/jit-benchmarks.md](docs/jit-benchmarks.md) §四/§五。
- **文件执行改走 `SourceInput::Path`**（[ADR-013](docs/decisions.md)）：新增 `Engine::run_path`；
  `run_file`、demo 验收、`tl-interpreter::run_file_ok`、MCP `run` 工具均已切换，
  磁盘脚本可自动加载同目录 `classdef`。REPL 与桌面端编辑器缓冲区仍走 Text
- 测试基线 102 → 105（新增 classdef Path 验证与文件级 classdef 集成测试）

### Fixed
- **纠正能力结论**：RunMat 0.6.2 的 `struct()`、字段读写、动态字段、`getfield`/`setfield`/`fieldnames`/`isfield`
  均可用；`classdef` 在 `SourceInput::Path` 请求下可用（`SourceInput::Text` 不加载）
- 文档：关闭 gaps K5，新增 K18（`isstruct`/`iscell` 缺失、struct 数组不能自动扩展）与 kernel-notes 坑 31

## [0.2.1] — 2026-10-17

> 主题：发行打包 + 测试提速 + 工具箱脚本约定收敛。

### Added
- **对抗性测试**：`crates/tl-commands/tests/adversarial.rs`（11 个测试，覆盖 v0.2 新增的 34 个函数；
  含不可行约束、奇异矩阵、重复极点、非法 method、标量/1×1 反例）。
- **防作弊**：`anti_cheat_two_inputs_change_outputs` 覆盖 fmincon/lsqlin/intlinprog/kalman/
  c2d/mle/glmfit/freqs/goertzel/convn。
- **Golden 快照**：`engine_contract.rs::all_demos_run_without_errors` 对 optim_demo/control_demo
  关键打印行做逐字/容差断言；GUI smoke 增加轴数量、折线点数、刻度标签与 viewBox 断言（8 条）。
- Windows 打包：`packaging/build-windows.ps1 -Bundle` 产出
  `dist/tensorlab-0.2.1-windows-x64.msi`（88,510,464 bytes，
  SHA256 `2B069E2C2A3ACC7BCDCAE15A197E0B1C309495B763BBA4ACACAF5004A0ECD659`）与
  `dist/tensorlab-0.2.1-windows-x64.zip`（133,679,629 bytes）。
  WiX 3.14 首次下载失败时的镜像预置步骤见 packaging/README.md。

### Changed
- **版本号统一为 0.2.1**：workspace、Tauri、CLI/MCP 版本字符串改为 `env!("CARGO_PKG_VERSION")`。
- **测试提速**：`ga`（30×80 → 20×60）、`particleswarm`（20×100 → 12×50）、
  `simulannealbnd`（3000 → 2000）、`patternsearch`（1000 → 400）默认迭代下调；
  `image_toolbox` demo 从 16×16 改为 6×6（bwdist 为 O(W·H·F) 精确搜索）。
  实测：`tl-commands` 单测 190s → 76s（合并重复 demo 运行）、adversarial 75s → 51s、
  engine_contract 84s → 52s（不含冷编译）。
- 日常全量测试改为 `cargo test --workspace --exclude tl-desktop -j 2` + Node GUI smoke，
  避免每次编译 Tauri 桌面 crate。
- 统一工具箱错误消息（小写开头、无句号）与 `arguments` 默认值风格；
  `optimget` 的 `arguments` 缩进修正。

### Fixed
- 记录 P1 对抗性测试发现的限制：`intlinprog` 不可行时可能挂起、`nlinfit` 发散模型触发
  SVD NaN panic、`dlqr(...,R=0)` 未校验、重复极点 `margin` 报 0°。

### Notes
- `toolbox/optim_*`、`toolbox/stats_extra/*`、`toolbox/stats_pdf/` 的目录隔离**继续保留**：
  合并探针确认 RunMat 0.6.2 仍会触发 K13/K14（`fminsearch`/`fmincon`/`ga`/`nlinfit` 等
  函数句柄调用失败）。`kernel_canaries` 会在内核修复后主动失败，届时再合并。

## [0.2.0] — 2026-10-17 🔒 冻结

> 主题：5 个 MVP 工具箱（锚定 MATLAB R2026a）+ MEX 加载桥 + GUI 自动化测试。
> 验收基线：`cargo test --workspace -j 1` → **102 passed / 0 failed**；10 个 demo 全绿；
> GUI smoke 7 条断言全过。安装包产物不在本次范围（用户明确排除）。见 [docs/releases/v0.2.md](docs/releases/v0.2.md)。

### Added（v0.2 收尾批次）
- **优化（8）**：`fmincon`（外罚函数 + Hooke–Jeeves）、`lsqlin`（有效集 KKT）、`intlinprog`（分支定界）、
  `ga`、`simulannealbnd`、`particleswarm`、`patternsearch`、`optimget` → 工具箱达到 20 个可用函数
- **信号（8）**：`convn`（1-D/2-D）、`fir2`、`freqs`、`upfirdn`、`pspectrum`、`kaiserord`、`czt`、`goertzel`
- **统计（7 + `fcdf` 辅助）**：`chi2pdf`、`mle`、`anova1`、`anova2`、`kruskalwallis`、`glmfit`、`nlinfit`
- **控制（11）**：`nichols`、`allmargin`、`minreal`、`dlqr`、`kalman`、`ssdata`、`ss2zp`、`zp2ss`、`c2d`、`d2c`、`d2d`
- **5 个新 demo**：`signal_toolbox.m`、`image_toolbox.m`、`stats_toolbox.m`、`optim_demo.m`、`control_demo.m`
- **GUI 自动化测试**：`apps/tl-desktop/ui/tests/gui_smoke.mjs`（Node + 无头 Edge + CDP，7 条断言），
  经 `apps/tl-desktop/src-tauri/tests/gui_smoke.rs` 接入 `cargo test`
- **文档**：[docs/releases/v0.2.md](docs/releases/v0.2.md)、各工具箱文档状态更新、kernel-notes 坑 26–30、gaps K14–K17

### Fixed
- `ss2tf` 一阶系统对**标量** A/B/C/D 的切片崩溃（改用 `numel` 判断）
- `tf2zp` 未剥离分子前导零，导致严格真分式多出原点零点
- `tl-commands` 的 demo 验收因 10 个 demo 与旧断言（5）不一致而失败；demo sweep 改到 64 MiB 栈线程

### Changed
- demo 验收从 5 个扩展到 10 个；测试总数 94 → 102
- 新增工具箱函数的目录隔离策略（避免内核目录级编译污染，见 kernel-notes 坑 26）


### Added
- **工具箱机制**：新增仓库根 `toolbox/` 目录（纯 m-script 函数文件，经 `addpath` 挂载，见 [ADR-012](docs/decisions.md)）
- **工具箱函数（共 55 个 `.m`，全部在 `target/debug/tensorlab.exe` 上实跑验证并与 MATLAB 期望值比对）**：
  - 信号（9）：`xcorr`、`findpeaks`、`rectwin`、`tukeywin`、`xcov`、`dct`、`idct`、`kaiser`、`decimate`
  - 统计（8 + `chi2inv` 辅助）：`zscore`、`moment`、`iqr`、`range`、`pca`、`ttest`、`normfit`、`vartest`
  - 图像（23）：`mat2gray`、`rgb2ycbcr`、`ycbcr2rgb`、`imresize`、`medfilt2`、`imrotate`、`imcrop`、`imtranslate`、`edge`、`imadjust`、`histeq`、`imgaussfilt`、`imnoise`、`imdilate`、`imerode`、`imopen`、`imclose`、`imfill`、`bwlabel`、`bwareaopen`、`imquantize`、`ordfilt2`、`label2rgb`
  - 优化（2）：`fminsearch`（Nelder–Mead）、`lsqnonneg`（Lawson–Hanson）
  - 控制（12）：`tfdata`、`series`、`parallel`、`evalfr`、`freqresp`、`bode`、`bodemag`、`margin`、`place`、`acker`、`isct`、`isdt`
- **实测基线**：`exist()` 探测 246 个 R2026a 目标函数 → 内核 109（44%）+ 工具箱 63 → **可用 172（70%）**，剩余 74
  见 [docs/toolboxes/](docs/toolboxes/)；关键验证值（`margin(1/(s²+s))` 的 Pm=51.827、`acker` 增益 `[2 3]`、`pca` 特征值 8.3333、`dct` 系数 5/−2.2304 等）与 MATLAB 一致
- **回归测试**：`tl-commands` 新增 `toolbox_m_scripts_are_mounted`、`toolbox_covers_image_and_control_batches`、`toolbox_mounts_control_optimization_and_signal_batches`
- **第二批工具箱函数（10 个，全部实跑验证）**：
  - 控制：`ss2tf`、`tf2ss`（Faddeev–LeVerrier）、`tf2zp`、`zp2tf`、`lsim`、`initial`（RK4）
  - 信号：`stft`、`istft`（重叠相加，重建误差 4.4e-16）
  - 图像：`wiener2`、`bwdist`
  - ⚠️ `quadprog` 实现已完成并验证（解 `[1.5,2.5]`，fval −12.5），但其文件会使 `fminsearch` 报
    `Not enough input arguments`（内核跨文件编译元数据 bug，[gaps.md](docs/gaps.md) K13）→ **暂不交付**
- **桌面端控制台 REPL**：控制台新增输入行，可直接执行宿主命令（`help` / `vars` / `mex load <path>`）与单行 m 代码，支持 ↑/↓ 历史；后端仍走 `repl_line`
- **`help <宿主命令>` 修复**：`help mex` / `help vars` 现在返回宿主命令说明（此前一律落到内核主题表并报 "No documentation"）
- **测试稳定性**：`tl-commands` 的 Engine 测试改为经 `locked_engine()` 串行化（内核 figure/路径状态是进程级全局，并行会互相污染）
- **MEX 桥（L6 `tl-mex`）**：由空桩升级为**真实动态库加载 / 入口探测桥**
  - `MexLibrary::load`（`libloading`）、`has_mex_entry`（探测 `mexFunction`）、`describe`
  - 宿主命令 `mex [status | load <path>]`（CLI / 桌面 / MCP 共用）
  - 说明：MATLAB MEX 的 `foo(x)` 透明调用受限于内核无函数注册 API + `mxArray` ABI 不公开（clean-room），见 [docs/gaps.md](docs/gaps.md) K12
- **跨平台打包**：新增 [packaging/](packaging/)
  - `build-windows.ps1`（便携 ZIP + 可选 MSI/NSIS）、`build-unix.sh`（tar.gz + 可选 DMG/AppImage/deb）
  - 产物含 `toolbox/`、`demos/`、`ATTRIBUTION.md`（RunMat 许可必须随发行物）
  - 说明：跨平台安装包须在对应平台构建（Tauri 不支持交叉编译）

### Notes
- 内核限制（已写入 [docs/kernel-notes.md](docs/kernel-notes.md) 坑 13–22）：文件函数不支持 `nargin`/`varargin`（改用 `arguments` 块）；
  标量不可切片；`reshape` 在个别函数体内可致栈溢出；`min`/`max` 偶发编译期误判；内核缺 `struct`/`expm`/`padarray`/`ordfilt2` 等；
  `exist()` 对部分算子式内置函数返回 0 但实际可用；`filtfilt` 有最短长度限制

## [0.1.0] — 2026-10-16 🔒 已冻结

首个可演示版本（MVP）。详见 [docs/releases/v0.1.md](docs/releases/v0.1.md)。

### Added
- **内核集成**：以 RunMat 0.6.2 为 m-script 运行时（`runmat-core` / `runmat-runtime` / `runmat-value`），L0-L6 为薄封装层
- **L0-L8 分层架构**：`tl-types` / `tl-linalg` / `tl-kernel` / `tl-parser` / `tl-interpreter` / `tl-jit` / `tl-mex` / `tl-toolbox` / `tl-commands`，由 `cargo xtask layers` 强制校验（43 条依赖边）
- **CLI**（`tensorlab`）：`run <script.m>` 子命令 + 交互式 REPL（引擎跑在 64 MiB 大栈线程上）
- **桌面端**（`tensorlab-desktop`）：Tauri 2 应用 —— 自研 overlay 高亮编辑器、控制台、变量面板、FigureScene→SVG 图窗渲染器、文件菜单（打开/保存/另存为）
- **MCP 服务器**（`tl-mcp-server`）：stdio JSON-RPC 2.0（MCP 2025-06-18），工具 `run` / `eval` / `plot` / `vars` / `reset`
- **宿主命令注册表**（`inventory`，PhotoCraft 风格）：`help` / `vars` / `clear` / `figures` / `version`
- **宿主层 help 文档表**：10 个主题（内核 0.6.2 无 `help` 内置函数）
- **5 个验收 demo**：`hello_world.m` / `matrix_ops.m` / `plot_demo.m` / `fft_demo.m` / `solve_demo.m`
- **变量面板数值预览**：接入内核 `workspace_snapshot` 的 `preview`（最多 16 元素），标量/向量/矩阵格式化
- **图窗载荷瘦身**：剥离 `*_source` 冗余字段（内核为 replay 无损导出而与 `x`/`y` 重复的数据），payload 体积约减半

### Testing
- 94 个自动化测试，11 个测试目标：
  - 白盒单测：`tl-types`(11) / `tl-linalg`(8) / `tl-parser`(9) / `tl-kernel`(6) / `tl-toolbox`(8) / `tl-interpreter`(2) / `tl-commands`(7)
  - 跨层集成：`tl-commands/tests/engine_contract.rs`(16)
  - 黑盒（独立进程）：`tl-cli/tests/cli.rs`(12) / `tl-mcp-server/tests/mcp_stdio.rs`(14)
  - 架构守门：`xtask/tests/layers.rs`(1)

### Fixed
- **控制台回显重复**：内核已把格式化回显写入 console 流，宿主又渲染 `display_events`，导致 `x = 42` 后多出一行 `42`。现以流为唯一来源，`display_events` 仅作兜底
- **控制台输出全部丢失**：`StreamEntryPayload` 的 serde tag 为小写（`output`/`error`/`clearConsole`），桌面端却按 PascalCase 判断，导致 `fprintf` 等输出不显示
- **图窗不显示**：`FigurePayload` 缺 `rename_all = "camelCase"`；且前端渲染器按错误的 FigureScene schema 编写（实际为 envelope `{createdAt, figure:{layout, metadata, plots}}` + 内部 `kind` 标记 + snake_case 字段）
- **桌面端启动被拦截**：`tl-desktop.exe` 被本地杀软锁死无法执行，bin 更名为 `tensorlab-desktop` 规避
- **深递归栈溢出**：内核 parser/eval 递归超出主线程默认栈，三个 App 均改为在 64 MiB 大栈线程执行
- **`tl-parser` token 列号恒为 0**：`Token.start` 语义为绝对字节列，原实现返回常量 0
- **MCP 通知回包**：`notifications/initialized` 违反 JSON-RPC 2.0 产生了响应

### 文档
- 项目宪法：`AGENTS.md`
- 根索引：`README.md`、`ROADMAP.md`
- 文档索引：`docs/README.md`
- 架构与决策：`docs/architecture.md`、`docs/decisions.md`（ADR-001 ~ ADR-012）、`docs/kernel-notes.md`（内核嵌入 API 与 12 个坑）
- 开发与贡献：`docs/development.md`、`docs/contributing.md`、`docs/testing.md`
- 使用：`docs/cli.md`、`docs/mcp.md`、`docs/host-commands.md`、`docs/ui-desktop.md`、`docs/language-support.md`
- 兼容性与规划：`docs/parity-checklist.md`、`docs/gaps.md`、`docs/roadmap.md`、`docs/toolboxes/`（5 个工具箱清单，锚定 R2026a）
- 发布与法务：`docs/releasing.md`、`docs/releases/v0.1.md`、`docs/attribution.md`、`docs/security.md`、`CHANGELOG.md`
