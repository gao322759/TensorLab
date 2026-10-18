# AGENTS.md — TensorLab 项目宪法

> **TensorLab：MATLAB-compatible, Rust-powered.**
> 一个完全兼容 m-script 语法的科学计算平台。所有进入本仓库的代码都必须遵守本宪法。

## 一、项目使命与边界

**使命**：用 Rust 生态构建 m-script 兼容的科学计算平台（内核 / CLI / 桌面端 / MCP）。

**明确不做**（v0.x 阶段边界）：
- ❌ Simulink
- ❌ 发明新语言（只解释 m-script）
- ❌ 抄袭 MathWorks 专有源码、内部算法、文档原文、测试用例（clean-room：教科书重写 + 公开 API 兼容）
- ❌ 完整 OOP（用户级 classdef）、Live Script、App Designer、移动端（例外：`zpk`/`pid` 等**内核模型对象**可在文件执行路径使用 classdef，见 [ADR-013](docs/decisions.md)）

**内核决策记录**：内核采用 **RunMat 0.6.2**（crates.io `runmat`，github.com/runmat-org/runmat，**Apache-2.0 / MIT 双许可**，兼容目标 R2026a，自带 parser/VM/Cranelift JIT/内置函数/绘图语义/GPU）。TensorLab 的 L0-L6 变为对 RunMat 的薄封装层，L7/L8 与三个 App 是我们的核心增量价值。已在 docs/kernel-notes.md 记录嵌入 API 要点（`RunMatSession::execute_request` / `workspace_snapshot` / figure scene replay）。

## 二、分层架构（L0-L8，由 `cargo xtask layers` 强制校验）

| 层 | Crate | 职责 |
|----|-------|------|
| L0 | `tl-types` | 基础类型：Value / 复数 Matrix / Cell / Struct / 错误 / 图窗状态 / 显示引擎 |
| L1 | `tl-linalg` | 线性代数：矩阵乘 / LU 求解 / inv / det / 对称特征分解 / 范数 |
| L2 | `tl-runtime` | 运行时：Workspace / Session / 事件通道 / **Command 注册表** |
| L3 | `tl-parser` | 词法 + 递归下降解析 → AST |
| L4 | `tl-interpreter` | 求值器：运算 / 索引 / 控制流 / 用户函数 / 多输出 |
| L5 | `tl-jit` | JIT 桥接（v0.3 启用 Cranelift；当前桩） |
| L6 | `tl-mex` | MEX 接口桩（v0.2 实现） |
| L7 | `tl-toolbox` | 内置命令实现：core / linalg / signal / stats / plot / io |
| L8 | `tl-commands` | 应用门面 Engine（run/eval/vars），供三个 App 复用 |

**Apps**（不参与分层，只能被依赖、不能依赖其它 App）：
- `apps/tl-cli` — REPL + `tensorlab run script.m`
- `apps/tl-desktop` — Tauri 2.0 桌面应用（主产品）
- `apps/tl-mcp-server` — MCP stdio 服务器（AI Agent 入口）

**分层规则**：
1. 层 L 只能依赖层号 **小于** L 的 crate；同层禁止互依。
2. Apps 可依赖任何 crate，crate 禁止依赖 App。
3. 外部依赖不受限制，但 `tl-types`（L0）只允许 `serde` / `num-*` / `std`。

## 三、核心哲学（PhotoCraft 风格）

1. **Everything is a command.** 每个内置函数（`sin`、`fft`、`plot`、`who`……）都是一个实现 `tl_runtime::Command` 的类型，经 `inventory` 自动注册。解释器、CLI、Tauri IPC、MCP 共享同一个注册表。
2. **Engine-first / headless-first.** 一切功能先在无头 Engine（`tl-commands::Engine`）里可用，桌面 UI 只是 Engine 的一个视图。
3. **Demos are the spec.** `demos/*.m` 是 v0.1 的验收标准，任何时候它们必须零错误运行。
4. **Layered, enforced.** 违反分层的 PR 无法通过 `cargo xtask layers`。

## 四、常用命令

```bash
cargo check                       # 全工作区编译检查
cargo test                        # 全量测试（含 5 个 demo 脚本验收）
cargo xtask layers                # 分层规则校验
cargo run -p tl-cli -- run demos/fft_demo.m    # 运行演示脚本
cargo run -p tl-cli               # 进入 REPL
cargo build -p tl-desktop         # 构建桌面端（Tauri 2）
cargo run -p tl-mcp-server        # 启动 MCP stdio 服务器
```

## 五、改动检查清单（每次提交前）

- [ ] `cargo check` 零警告新增
- [ ] `cargo test` 全绿（含 `tl-commands` 的 5 个 demo 验收测试）
- [ ] `cargo xtask layers` 通过
- [ ] `cargo fmt` 已格式化
- [ ] 新增内置函数 = 新增一个 `Command` 实现（name / category / description 必填，`help <name>` 依赖它）
- [ ] 不引入同层依赖、不向下依赖

## 六、代码风格

- Rust 2021 edition，`rustfmt` 默认风格。
- 错误统一走 `tl_types::TlError`（MATLAB 风格英文消息 + 行号），不 panic。
- 数值内核：`Matrix` 以 `Vec<Complex<f64>>` 列主序存储（MATLAB 语义：一切皆复数矩阵）。
- 1-based 索引、`end` 关键字、隐式广播（R2016b+ 语义）——兼容最新版 MATLAB 是硬要求。
- 命令命名：crate 前缀 `tl-`；注册表命令名小写。

## 七、路线图

| 版本 | 交付物 | 状态 |
|------|--------|------|
| v0.1 | Tauri 窗口 + 编辑器 + 变量面板 + 解释器 + 绘图 + 5 demo + CLI + MCP | 🔒 **已发行并冻结**（v0.1.0，2026-10-16，94 测试全绿） |
| v0.2 | 5 个 MVP 工具箱（锚定 MATLAB R2026a）+ MEX 加载桥 + GUI 自动化测试 | 🔒 **已冻结**（v0.2.0，2026-10-17，102 测试全绿 + 10 demo；安装包不在本次范围） |
| v0.2.1 | Windows MSI 打包 + 测试提速（< 15 分钟目标）+ 对抗性/golden/GUI 结构测试 | ✅ **已发行**（2026-10-17，119 Rust 测试 + 8 GUI 断言；MSI 84.41 MiB） |
| v0.3 | Cranelift JIT 启用 + 性能对标 MATLAB 0.5x（耗时 ≤ 2× MATLAB） | 进行中（2026-10-17 启动；release 对标 MATLAB R2023b：matmul 68x、FFT 30x、向量数学 8.7x、标量循环 10125x、函数调用 39870x，0.5x 未达成；见 docs/jit-benchmarks.md） |

> v0.1.0 的契约快照与冻结范围见 [docs/releases/v0.1.md](docs/releases/v0.1.md)；
> 冻结后对 IPC 载荷、分层、对外名称、控制台输出语义的变更必须走新版本。
> v0.2 的工具箱逐函数清单见 [docs/toolboxes/](docs/toolboxes/)（实测起点：R2026a 目标函数 246 个中内核已提供 109 个）。

## 八、IPC / 事件契约（v0.1 已冻结）

实际类型见 `crates/tl-types/src/payload.rs`（serde `camelCase`）：

```
Engine::run(source_name, source)  →  RunReport {
    events: Vec<StreamEntryPayload>,   // internally-tagged { kind: "output"|"error"|"clearConsole", text }
    resultText: Option<String>,        // 单行 eval 取值
    error: Option<DiagnosticPayload>,  // { identifier, message, line, callstack, severity }
    warnings: Vec<DiagnosticPayload>,
    variables: Vec<VariableInfo>,      // { name, className, shape, bytes, preview, isGpu }
    figures: Vec<FigurePayload>,       // { handle, sceneJson }（FigureScene replay 载荷）
    durationMs: f64,
    usedJit: bool,
}
```

- CLI / MCP 消费 `RunReport::console_text()`；桌面端按 `events[].kind` 渲染控制台、按 `figures[].sceneJson` 渲染 SVG。
- **tag 命名是前端契约**，改动即破坏桌面端渲染（有 `stream_entry_tags_are_camel_case` 测试钉死）。
- 绘图：内核 `export_figure_scene` 输出 FigureScene JSON（envelope `{ createdAt, figure: { layout, metadata, plots } }`，见 docs/kernel-notes.md §11），桌面端用**自研 SVG 渲染器**绘制（非 Plotly，离线无 CDN）。

## 九、clean-room 红线

- ✅ 可参考：MATLAB 公开文档中的语义描述、教科书算法（LAPACK/NumPy 论文）、通用函数名
- ❌ 不可参考：MathWorks 源码、其测试数据文件、文档原文摘抄
- 所有算法实现必须能追溯到教科书 / 公开论文出处
