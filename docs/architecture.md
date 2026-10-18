# 架构

> 本文描述 TensorLab v0.1 的实际架构。**分层规则由 `cargo xtask layers` 强制校验**，违反即构建失败。
> 不可违反的规则以 [AGENTS.md](../AGENTS.md) 为准。

## 一、总览

```
        ┌─────────────────────────────────────────────────────────────┐
        │  Apps（不参与分层；只能被依赖，不能依赖其它 App）             │
        │  apps/tl-cli        apps/tl-desktop      apps/tl-mcp-server  │
        │  (REPL + run)       (Tauri 2 主产品)      (stdio JSON-RPC)    │
        └───────────┬──────────────────┬──────────────────┬──────────┘
                    └──────────────────┼──────────────────┘
                                       ▼
                            ┌──────────────────────┐
                       L8   │  tl-commands         │  应用门面 Engine
                            │  run / repl_line /   │  三个 App 的唯一入口
                            │  host_command / demos│
                            └──────────┬───────────┘
                                       ▼
                            ┌──────────────────────┐
                       L7   │  tl-toolbox          │  宿主命令注册表（inventory）
                            │  help/vars/clear/... │  ⚠ 只收宿主命令，不收 m 内置函数
                            └──────────┬───────────┘
                                       ▼
                       L6   tl-mex      │  原生库加载/入口探测桥（透明调用待内核注册 API）
                       L5   tl-jit      │  桩（JIT 由内核承担，v0.3 暴露统计）
                                       ▼
                            ┌──────────────────────┐
                       L4   │  tl-interpreter      │  结果后处理 / help 主题 / demo 验收器
                            └──────────┬───────────┘
                            ┌──────────────────────┐
                       L3   │  tl-parser           │  编辑器用 tokenizer（高亮）
                            └──────────┬───────────┘
                            ┌──────────────────────┐
                       L2   │  tl-kernel           │  Engine 门面：唯一接触 RunMat 类型的层
                            │  RunMatSession 封装  │  run / cancel / vars / figures
                            └──────────┬───────────┘
                            ┌──────────────────────┐
                       L1   │  tl-linalg           │  纯 Rust 数值工具（LU/det/solve/inv）
                            └──────────┬───────────┘
                            ┌──────────────────────┐
                       L0   │  tl-types            │  显示格式化 + 跨 IPC serde 载荷
                            └──────────────────────┘
                                       ▲
                                       │
                    ┌──────────────────┴───────────────────┐
                    │  RunMat 0.6.2（外部内核，Apache-2.0 / MIT）  │
                    │  parser / VM / Cranelift JIT /       │
                    │  1200+ 内置函数 / 绘图 / GPU          │
                    └──────────────────────────────────────┘
```

## 二、分层规则（强制）

1. 层 L 只能依赖层号 **严格小于** L 的 crate；同层禁止互依。
2. Apps 可依赖任意内部 crate；内部 crate **禁止**依赖 App。
3. 外部依赖不受限，但 **L0 `tl-types` 只允许** `serde` / `serde_json` / `runmat-value`。
4. 层表中的 crate 必须存在于工作区。

校验方式：

```bash
cargo run -p xtask -- layers     # 也作为 tools/xtask/tests/layers.rs 纳入 cargo test
```

当前状态：**OK（43 条依赖边）**。

## 三、引擎边界（关键设计）

**`tl-kernel` 是唯一接触 RunMat 类型的层。** 其余所有 crate 与 App 只看见 `tl_types` 的载荷类型
（`RunReport` / `VariableInfo` / `FigurePayload` / `StreamEntryPayload` / `DiagnosticPayload`）。

这带来两个好处：

- 内核升级（如 RunMat 0.6.2 → 0.7）只需改 `tl-kernel` 一个 crate；
- 分层校验能机械地防止 RunMat 类型泄漏到 UI 层。

`tl-kernel::Engine` 的职责：

| 方法 | 作用 |
|---|---|
| `new()` | 创建 `RunMatSession` |
| `run(name, source)` | 阻塞执行 m 源码（内部驱动 async），返回 `RunReport` |
| `cancel()` | 协作式取消（内核在语句边界响应） |
| `variables()` | 工作区快照 + 数值预览格式化 |
| `clear()` | 清空工作区 |
| `export_figures(&[u32])` | 逐 handle 导出 FigureScene JSON（并剥离冗余字段） |

## 四、一次执行的数据流

```
用户按下 Run / 执行 `tensorlab run x.m` / MCP tools/call
        │
        ▼
tl-commands::repl_line  ──(宿主命令?)──► tl-toolbox 注册表 ──► CommandResult::Text
        │ 否（是 m 代码）
        ▼
tl-kernel::Engine::run
        │  ExecutionRequest { source, CompatMode::Matlab, host_policy, workspace }
        ▼
RunMatSession::execute_request  ──────  async（futures::executor::block_on）
        ▼
ExecutionOutcome { streams, workspace_delta, display_events,
                   diagnostics, figures_touched, execution_time_ms, used_jit, ... }
        │
        ▼  映射（tl-kernel）
RunReport { events, error, warnings, variables, figures, durationMs, usedJit }
        │
        ├─► CLI：打印 events 到 stdout，`[VM xx ms]` 到 stderr
        ├─► MCP：events 拼成 text，figures 作为 JSON 附在文本后
        └─► 桌面端：按 events[].kind 渲染控制台；figures[].sceneJson → SVG
```

### 控制台输出的唯一来源

内核已经把 MATLAB 风格回显（`x = 42` / `ans = 2`，遵循 `format short/long`）写进 console 缓冲，
随 `ExecutionOutcome.streams` 返回。`display_events` 只携带匿名语句值（无 label）。

因此 **`streams` 是控制台唯一权威来源**，宿主**不得**再渲染 `display_events`，否则输出重复：

```
x = 42        ← 内核 streams
42            ← 宿主 display_events（错误！已移除）
```

`display_events` 仅在本次执行完全没有 console 输出时作为兜底。回归测试：`statement_echo_is_not_duplicated`。

## 五、图窗管线

```
m: figure; plot(x, y); subplot(2,1,1)
        │
        ▼ 内核 plotting 语义（runmat-plot）
FigureScene（replay 载荷）
        │  envelope: { createdAt, figure: { layout, metadata, plots } }
        │  layout/metadata: camelCase（axesRows / axesMetadata / xLabel ...）
        │  plots[]: 内部 kind 标记 + snake_case（kind: "line" / x, y, color_rgba / axes_index ...）
        ▼ tl-kernel::export_figures
  ├─ 剥离冗余：删除 plots[].*_source（与 x/y 内容完全重复的 replay 数据）→ payload 约减半
  └─ FigurePayload { handle, sceneJson }
        ▼
桌面端 figureSvg()：解 envelope → 按 axes_index 分组 → 每个子图建 SVG
```

> ⚠️ 实测 schema 与 `runmat_plot` 的类型声明**不一致**，以实际导出为准。详见 [kernel-notes.md](kernel-notes.md) §11。

## 六、命令注册表（PhotoCraft 哲学：Everything is a command）

`tl-toolbox` 用 `inventory` 在编译期收集 `CommandEntry(&'static dyn Command)`：

```rust
pub trait Command: Sync + Send {
    fn name(&self) -> &'static str;          // 小写
    fn category(&self) -> &'static str;      // workspace / figures / session / help / app
    fn description(&self) -> &'static str;   // help 列表与 UI 提示
    fn run(&self, engine: &mut Engine, args: &[String]) -> CommandResult;
}
```

**边界**：本注册表只收**宿主命令**（GUI/CLI/MCP 面向宿主的操作）。m 语言的内置函数
（`sin` / `fft` / `plot` / …）由 RunMat 内核提供，**不**经此表。这不影响"共享同一入口"的目标 ——
所有入口最终都落到同一个 `Engine`。

宿主命令清单见 [host-commands.md](host-commands.md)。

## 七、三个 App

| App | 形态 | 关键实现细节 |
|---|---|---|
| `tl-cli` | 二进制 `tensorlab` | 引擎跑在 **64 MiB 大栈线程**上（内核 parser/eval 深递归会爆主线程栈）；REPL 读到 EOF 或 `exit`/`quit` 退出；脚本运行出错 → 退出码 1 |
| `tl-mcp-server` | 二进制 `tl-mcp-server` | stdio 逐行 JSON-RPC 2.0；通知不回包；坏 JSON 静默跳过；同样跑在大栈线程 |
| `tl-desktop` | 二进制 `tensorlab-desktop` | Tauri 2；UI 为静态 HTML（`apps/tl-desktop/ui/`，`frontendDist` 内嵌，无 CDN）；Rust 侧把请求经 mpsc 交给**独占的引擎线程**（64 MiB 栈），避免阻塞 UI 线程 |

### 为什么三个 App 都需要大栈线程

RunMat 的 parser/eval 递归深度远超 Windows 主线程默认栈（1 MiB）。`cargo test` 的测试线程默认栈更大，
所以问题只在真实二进制里复现。修复方式统一为：

```rust
std::thread::Builder::new()
    .stack_size(64 * 1024 * 1024)
    .spawn(run)
    .expect("spawn engine thread")
    .join()
    .map_err(|_| anyhow::anyhow!("engine thread panicked"))?
```

## 八、IPC / 事件契约

以 [crates/tl-types/src/payload.rs](../crates/tl-types/src/payload.rs) 为准，且被测试钉死。摘要：

```
RunReport {
    events: Vec<StreamEntryPayload>,   // { kind: "output"|"error"|"clearConsole", text }
    resultText: Option<String>,
    error: Option<DiagnosticPayload>,  // { identifier, message, line, callstack, severity }
    warnings: Vec<DiagnosticPayload>,
    variables: Vec<VariableInfo>,      // { name, className, shape, bytes, preview, isGpu }
    figures: Vec<FigurePayload>,       // { handle, sceneJson }
    durationMs: f64,
    usedJit: bool,
}
```

契约测试：`crates/tl-commands/tests/engine_contract.rs`（camelCase 键名、tag 值、图窗去重、回显不重复）。
