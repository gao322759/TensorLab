# 测试

> v0.1 基线：**94 个测试 / 11 个测试目标 / 0 失败**。
> v0.2 基线（2026-10-17）：**102 个测试 / 13 个测试目标 / 0 失败**，
> 另有 GUI 自动化 **7 条断言**（由 `gui_smoke` 测试以独立 Node 进程执行并计入 102 条）。
> v0.3-dev 基线（2026-10-18）：**122 个测试 / 19 个测试目标 / 0 失败**
> （含 kernel canaries、adversarial、golden 快照、JIT tiering 与内核统计）；
> GUI 自动化 **11 条断言**（新增 About 版本对话框与 Command Window 右键复制、`Ctrl+L` 清空）。
> 冷构建 + 全量测试约 40 分钟（链接重）；`cargo test --workspace -j 1` 可避免并发链接冲突。

## 一、为什么这样组织

测试按**观察视角**分层，而不是按目录：

| 类型 | 观察对象 | 位置 | 数量 |
|---|---|---|---|
| 白盒单测 | 单个函数/模块的内部行为 | 各 crate 的 `#[cfg(test)] mod tests` | 58 |
| 跨层集成 | `tl-commands` 的**公开 API**（不看内部） | `crates/tl-commands/tests/` | 30 |
| 内核契约 | 内核行为/回归看守（canary、classdef Path） | `crates/tl-kernel/tests/` | 6 |
| 黑盒 | **独立进程**的 argv/stdin/stdout/stderr/退出码 | `apps/*/tests/` | 26 |
| GUI 自动化 | 真实 `index.html` + 无头浏览器 + CDP | `apps/tl-desktop/ui/tests/` + `src-tauri/tests/` | 1（内含 11 断言） |
| 架构守门 | 分层不变量 | `tools/xtask/tests/` | 1 |

## 二、明细

| 目标 | 数量 | 关注点 |
|---|---|---|
| `tl-types`（单测） | 11 | `class_name` / `size_of` / `body_text` / `compact` / `echo_line`；`StreamEntryPayload` 的 tag 命名契约 |
| `tl-linalg`（单测） | 8 | LU 分解、`det` 置换符号、`solve`/`inv` 往返、奇异矩阵一律返回 `None` |
| `tl-parser`（单测） | 9 | tokenize **不变量**（token 文本无缝拼回原行）、`start` 为绝对字节列、注释吞行、多字节列号 |
| `tl-kernel`（单测） | 7 | 图窗 `*_source` 剥离、数值预览格式化（标量/向量/截断/空）、`variable_table` 排版、`kernel_stats` 的 VM 回退 |
| `tl-kernel/tests/classdef_path.rs` | 1 | `SourceInput::Path` 加载同目录 classdef：`class(p)=='Vec2'`、`p.magnitude()==5` |
| `tl-kernel/tests/kernel_canaries.rs` | 5 | 内核升级回归看守：K13/K14 目录级污染、K15 闭包环境、K16 切片转置、K17 标量归约 |
| `tl-toolbox`（单测） | 9 | 命令元数据完整且小写、注册表有序无重、`find` 命中/未命中、help 分派、`mex` 子命令 |
| `tl-mex`（单测） | 2 | 缺失库报错含路径；加载系统库成功且不误报 `mexFunction` |
| `tl-interpreter`（单测） | 2 | help 主题表（含未知主题）、错误路径 |
| `tl-commands`（单测） | 11 | demo 验收（10 个）、变量预览、repl 路由、help 拦截（宿主命令优先）、payload camelCase、**工具箱纯 m 函数挂载与数值回归（4 条，含 struct 返回值）** |
| `tl-commands/tests/engine_contract.rs` | 17 | **契约**：RunReport 键名 camelCase、事件 tag 小写、图窗 envelope 且无 `*_source`、**回显不重复**、文件执行（含同目录 classdef）、10 demo 全绿（64 MiB 栈线程）、`DEMOS` 与 `demos/` 一致 |
| `tl-commands/tests/adversarial.rs` | 11 | 退化/反例：空输入、标量、奇异矩阵、不可行约束、非法 method；防作弊（两组输入输出必须变化） |
| `tl-commands/tests/jit_tiering.rs` | 2 | `Engine::with_jit(false)` 保持 VM；重复脚本循环进入 JIT 且 steady-state 改善 |
| `apps/tl-cli/tests/cli.rs` | 12 | 独立进程：`version`/`help` 输出、**5 个 demo 全部 exit 0 且有 stdout**、fprintf 可见、缺失参数/文件/未知子命令的失败语义、运行出错 exit 1 + stderr、REPL（求值/`exit`/EOF/出错后继续） |
| `apps/tl-mcp-server/tests/mcp_stdio.rs` | 14 | 独立进程：`initialize` 握手、`tools/list` 5 工具与 schema、`eval`/`plot`/`vars`/`reset`/`run`、未知工具、**坏 JSON 不崩**、**通知不回包**、多请求 id 对应 |
| `apps/tl-desktop/src-tauri/tests/gui_smoke.rs` | 1 | 启动 Node harness：真实 `ui/index.html` + 无头 Edge + CDP；断言编辑器高亮、About 版本对话框、Run 输出/变量表/SVG 图、控制台 REPL 与历史、错误样式、右键复制、`Ctrl+L` 清空、`clearConsole`（11 条） |
| `tools/xtask/tests/layers.rs` | 1 | 分层不变量（把 `cargo xtask layers` 纳入 `cargo test`） |

## 三、设计原则

### 1. 断言"应该怎样"，不是"现在怎样"

测试暴露缺陷时**改代码**，不把错误行为写进断言。v0.1 期间由此发现并修复：

- `tl-parser::Token.start` 恒为 0（断言"应为绝对字节列" → 红 → 修）
- MCP 对 JSON-RPC 通知回包（断言"通知不得有响应" → 红 → 修）
- 控制台回显重复（断言"精确等于 `x = 42`" → 红 → 修）

### 2. 钉死跨进程契约

跨 IPC 的名字一旦错位，症状往往极其隐蔽（v0.1 就因 serde tag 大小写不一致导致**所有**控制台输出丢失）。
因此契约用测试钉死：

- `stream_entry_tags_are_camel_case` —— tag 只能是 `output` / `error` / `clearConsole`
- `run_report_json_keys_are_camel_case` / `variable_entries_expose_camel_case_fields`
- `figure_payload_serializes_camel_case`
- 各 payload 的**反例**也断言：不得出现 `class_name` 这类 snake_case 泄漏

### 3. 避免脆弱断言

- 不用单个数字断言（banner 里也有数字），如 CLI 测试断言 `ans` 而非 `2`
- **不用绝对体积阈值**：图窗 payload 只断言"不含 `*_source` 键"这一精确契约，而非"< N KB"
- 涉及内核**进程级全局状态**（figure handle 计数）时，遍历全部导出图、不依赖顺序
- 用 `trim()`/逐行比较替代整串相等，容忍无关空白

### 4. 黑盒测试怎么找到二进制

Cargo 为集成测试提供 `CARGO_BIN_EXE_<binname>`。名称含连字符时不总是可靠，因此测试里做了回退：

```rust
fn server_bin() -> PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_tl-mcp-server") { return p.into(); }
    // 回退：<target>/<profile>/deps/<test>.exe → <target>/<profile>/<bin>[.exe]
}
```

### 5. MCP 测试的会话模型

写入全部请求 → 关闭 stdin（服务器在 EOF 处退出）→ 收集所有 stdout 行 → 逐行 `serde_json` 解析。
简单、无竞态、无需超时管理。

### 6. 对抗性测试（AI 生成代码）

AI 生成的实现**默认不可信**。测试不是“证明它能跑”，而是“尽力证明它不能跑”：

- **三类用例**：
  1. 参考值：已知解析解或 MATLAB 期望值（如 `intlinprog → [4,0], f=-12`）；
  2. 退化输入：空、标量、1×1、单元素、奇异矩阵、重复极点、零长度信号；
  3. 反例：非法参数、维度不匹配、不可行约束 —— 断言错误消息与退出语义。
  实现见 `crates/tl-commands/tests/adversarial.rs`（覆盖 v0.2 新增的 34 个函数）。
- **防作弊**：同一函数至少用两组不同输入，确认输出随输入变化；
  拒绝“返回硬编码常量也能过”的断言；不接受“结果非空/有输出”作为正确性证据。
  同一文件中的 `anti_cheat_two_inputs_change_outputs` 覆盖 fmincon/lsqlin/intlinprog/
  kalman/c2d/mle/glmfit/freqs/goertzel/convn。
- **真实环境**：数值函数必须在 `target/debug/tensorlab.exe` 实跑；GUI 必须用无头浏览器
  驱动真实 `index.html`（见 `apps/tl-desktop/ui/tests/gui_smoke.mjs`）。
- **Golden values**：`engine_contract.rs::all_demos_run_without_errors` 对
  `optim_demo` / `control_demo` 的关键打印行做**逐字快照断言**（`fmincon`、
  `lsqlin`、`intlinprog`、`kalman`、`c2d`、`dlqr` 等）；GUI 测试额外断言轴数量、
  折线点数与刻度标签，而不是只判断 `<polyline>` 存在。
- **已知对抗性发现（暂不修复，测试中标注）**：
  - `intlinprog` 在不可行约束上可能长时间不返回，CI 只测合法分支和非法 `intcon`；
  - `nlinfit` 对发散指数模型会触发内核 SVD 的 NaN panic，CI 只测收敛模型与
    模型返回长度错误的反例；
  - `dlqr(...,R=0)` 未做校验，当前返回有限值；
  - 重复极点系统的 `margin` 报 `Pm=0°`（不是 `Inf`），demo 快照固定了当前行为。
- **重构安全网**：每轮“架构加固”前后都跑全量测试；测试红时改代码，不把错误行为写进断言。

> 判断标准：如果测试只能证明“AI 说它对了”，那它就不是测试。

## 四、运行

```bash
# 日常快速全量：不编译 Tauri 桌面 crate；GUI smoke 直接跑 Node
cargo test --workspace --exclude tl-desktop -j 2
node apps/tl-desktop/ui/tests/gui_smoke.mjs

# 完整（含桌面 crate 的 Rust 包装测试，冷构建会慢很多）
cargo test --workspace -j 1

cargo test -p tl-types           # 单 crate（快）
cargo test -p tl-commands --lib  # 单目标
cargo test -p tl-cli --test cli  # 单个黑盒目标

# 看失败详情
cargo test -p tl-commands --test engine_contract -- --nocapture
cargo test -p tl-kernel --lib formats_scalar -- --nocapture
```

## 五、未覆盖（已知）

| 未覆盖 | 原因 | 计划 |
|---|---|---|
| 桌面端 UI（HTML/JS 渲染器、文件菜单、FigureScene→SVG） | 需要 WebDriver/tauri-driver 或 JS 测试运行时 | v0.2 |
| 图窗渲染的像素级正确性 | 同上 | v0.2 |
| 跨平台（macOS/Linux） | 无对应构建环境 | v0.2 打包阶段 |
| 性能回归 | 无基准框架 | v0.3（配合 JIT 目标） |

桌面端 UI 目前靠后端契约测试间接保障：数据正确性有保证（`engine_contract.rs`），
但"渲染出来长什么样"无人验证。
