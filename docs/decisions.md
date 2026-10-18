# 架构决策记录（ADR）

格式：`ADR-NNN 标题` — 状态 / 背景 / 决策 / 影响。新增决策请追加在末尾，不要改写历史条目。

---

## ADR-001 内核采用 RunMat 0.6.2（而非自研解释器）

**状态**：已采纳（v0.1 起生效）

**背景**：项目初期调研一度无法验证"RunMat 0.6.2"是否存在，曾据计划文档判定为调研假设项，转而自研
L0-L4（矩阵类型、值模型、解析器、求值器原型）。随后用户提供了 `github.com/runmat-org/runmat`，
验证其在 crates.io 真实存在且 API 与文档一致。

**决策**：废弃自研原型，改用 RunMat 0.6.2 作为 m-script 运行时。TensorLab 的 L0-L6 变为**薄封装层**，
价值聚焦在 L7/L8 与三个 App。

**影响**：
- ✅ 直接获得完整 m 语法、1200+ 内置函数、Cranelift JIT、绘图语义、GPU 支持
- ✅ 专利/许可清晰（Apache-2.0 / MIT），见 [attribution.md](attribution.md)
- ⚠️ 依赖树巨大（冷构建 10 分钟级），且引入 HDF5 等非可选系统依赖
- ⚠️ 受内核行为约束（如无 `help` 内置函数、`Span` 为字节偏移），需在宿主层适配
- ⚠️ 内核 API 变更会传导到 `tl-kernel`（缓解：所有 RunMat 类型只出现在这一个 crate）

---

## ADR-002 L2 由 `tl-runtime` 改为 `tl-kernel`

**状态**：已采纳

**背景**：原计划中 L2 名为 `tl-runtime`，设想自研 Workspace / Session / 事件通道。内核改为 RunMat 后，
这些职责被内核接管，L2 的实际角色变成"内核门面"。

**决策**：L2 更名 `tl-kernel`，职责改为封装 `RunMatSession`（run / cancel / vars / figures）。

**影响**：`tools/xtask` 的层表同步更新；`tl-commands` 的 `Engine` 成为对内唯一的执行入口。
命名见 [architecture.md](architecture.md) §三。

---

## ADR-003 控制台回显以内核 `streams` 为唯一来源

**状态**：已采纳（v0.1）；回归测试 `statement_echo_is_not_duplicated`

**背景**：内核在 `console::record_value_output` 中已按 `format short/long` 把回显写入 console 缓冲，
随 `ExecutionOutcome.streams` 返回；同时 `display_events` 只携带匿名语句值（`DisplayLabel::Anonymous`，
无 label）。宿主若两者都渲染，控制台会出现重复：

```
x = 42
42        ← 多余
```

**决策**：控制台只消费 `streams`；`display_events` 仅在本次执行**完全没有** console 输出时兜底。
未被采纳的替代方案：
- "只渲染 display_events、丢弃内核回显" —— 不可行：匿名事件无法还原 `x = 42` 的 label 与格式化
- "从 streams 里识别并剔掉回显行" —— 不可行：stream 条目无类型标记，与真实 stdout 无法区分

**影响**：宿主失去对回显格式的控制权（由内核 `format` 决定）；换回显格式须改内核或做文本后处理。

---

## ADR-004 图窗渲染用自研 SVG，不引入 Plotly

**状态**：已采纳（v0.1，可复议）

**背景**：计划文档要求"基本绘图转 Plotly"。但 Plotly.js 全量约 3.5MB（basic 约 1MB），与
"离线、无 CDN、包体可控"的目标冲突。

**决策**：实现自研 FigureScene→SVG 渲染器（约 250 行 JS），支持 line / scatter / bar / stem / stairs /
area + subplot 网格 + 标题/轴标签/网格线。

**影响**：
- ✅ 零额外加载；（小数据量下）渲染延迟低于 Plotly；包体不膨胀
- ✅ 与 `camelCase`/`snake_case` 混合的真实 schema 解耦（内联在 UI 中）
- ⚠️ **无交互能力**（缩放/平移/hover/图例开关）；大数据（>10k 点）无 WebGL 加速
- ⚠️ 需自行维护渲染正确性（有后端契约测试兜底，UI 本身无自动化测试）
- 复议触发条件：需要交互式图窗，或单图数据量稳定超过 10k 点时，改引 WebGL 路径

---

## ADR-005 编辑器用自研 overlay 高亮，不引入 Monaco

**状态**：已采纳（v0.1，可复议）

**背景**：计划要求 "Monaco 编辑器"。Monaco 资源约 5MB 且通常经 CDN 或 npm 打包，与离线目标冲突。

**决策**：`apps/tl-desktop/ui/index.html` 内实现 textarea + `pre` 高亮层的 overlay 编辑器：
MATLAB 关键字 / 内置函数 / 数字 / 字符串 / 注释着色，Tab 缩进，Ctrl+Enter 运行。
`tl-parser` 提供同语义的 tokenizer（供未来替换或服务端高亮）。

**影响**：
- ✅ 单文件、零依赖、启动即用
- ⚠️ 无自动补全、无括号匹配、无多光标、无错误波浪线
- ⚠️ overlay 方案对超长行/字体不一致敏感
- 注：`tl-parser::tokenize_line` 的 `Token.start` 曾恒为 0（已修），若接 Monaco 需保证列号语义正确

---

## ADR-006 三个 App 的执行线程栈设为 64 MiB

**状态**：已采纳（v0.1）

**背景**：RunMat 的 parser/eval 递归深度超出 Windows 主线程默认栈，表现为栈溢出（进程崩溃 / 无输出）。
`cargo test` 的测试线程默认栈更大，故 CI/单测不复现。

**决策**：CLI、MCP 服务器、桌面端（引擎线程）统一在 `stack_size(64 * 1024 * 1024)` 的线程上执行引擎。

**影响**：多占约 64 MiB 虚拟地址空间（按需提交，实际占用远小于此）；换来稳定执行。

---

## ADR-007 图窗载荷剥离 `*_source`（渲染载荷 ≠ round-trip 载荷）

**状态**：已采纳（v0.1）；测试 `strips_source_duplicates_but_keeps_render_data`

**背景**：`export_figure_scene` 为**无损重建**设计，每个 plot 同时导出物化后的 `x`/`y` 与原始
`*_source`（`SceneNumericData`），二者内容完全重复（`x = x_data.materialize_f64()`，
`x_source = Some((&x_data).into())`）。200 点时无所谓；10 万点时 payload、序列化、IPC、`JSON.parse`
四步各翻倍，而这四步比渲染本身更贵。

**决策**：`tl-kernel::export_figures` 在反序列化后剥离所有 `*_source` 键。**该 payload 从此是渲染载荷，
不再可用于 `import_figure_scene` 无损回灌**（已在代码注释与本文档明示）。

**影响**：点数组占主导的图体积约减半；牺牲了 round-trip 能力（当前无此需求）。

---

## ADR-008 `help` 由宿主层提供（内核 0.6.2 无该内置函数）

**状态**：已采纳（v0.1）

**背景**：源码核验 RunMat 0.6.2 的内置函数目录，**没有 `help`**。直接执行 `help('x')` 会走未定义函数
路径并触发内核深递归 → 栈溢出崩溃。

**决策**：在 `tl-commands::repl_line` 拦截 `help <topic>` / `help('<topic>')`，由
`tl-interpreter::help_topic` 返回宿主文档表（10 个主题 + 未知主题提示）。裸 `help` 仍路由到宿主命令
`help`（列命令总览）。`tl-mcp-server` 与桌面控制台同样受此保护。

**影响**：MATLAB 的 `help` 全文档不可用，只能覆盖我们手工维护的主题；新增内置函数文档需同步该表。

---

## ADR-009 桌面端 bin 更名为 `tensorlab-desktop`

**状态**：已采纳（v0.1）

**背景**：`tl-desktop.exe` 在开发机上被本地杀软（360 + 火绒）锁死：无法执行（`Access is denied`）、
无法重命名、无法删除，`icacls /reset` + `takeown` 也无效。原因疑似对**无签名的大型新编译 exe** 的实时扫描。

**决策**：`apps/tl-desktop/src-tauri/Cargo.toml` 中把 bin 名与 package 名分离：

```toml
default-run = "tensorlab-desktop"
[[bin]]
name = "tensorlab-desktop"
path = "src/main.rs"
```

**影响**：产物名变为 `tensorlab-desktop[.exe]`；换名即可绕过被锁文件（杀软按文件路径锁定）。
发布时应在签名后使用正式名（见 [releasing.md](releasing.md)）。

---

## ADR-010 文件打开/保存用 `tauri-plugin-dialog` + 自研读写命令

**状态**：已采纳（v0.1）

**背景**：原计划"基础文件菜单（打开/保存/运行）"。Tauri 2 中文件对话框已从核心 API 移出为插件。

**决策**：引入 `tauri-plugin-dialog`（`capabilities/default.json` 授予 `dialog:allow-open` /
`dialog:allow-save`），路径由原生对话框取得后，读写走**自研 Rust 命令** `read_text_file` /
`write_text_file`（`std::fs`），**不**引入 `tauri-plugin-fs` 及其 scope 权限体系。

**影响**：权限面更小、可控；打开/保存仅限文本（`.m`）；前端调用 `plugin:dialog|open` / `|save`
（`options` 为 camelCase：`defaultPath` / `filters`）。

---

## ADR-011 变量面板预览使用内核 `preview`（上限 16 元素）

**状态**：已采纳（v0.1）

**背景**：`workspace_snapshot()` 的每条 `WorkspaceEntry` 默认带 `preview`（最多
`WORKSPACE_PREVIEW_LIMIT = 16` 个数值），而宿主此前丢弃了它，导致变量面板只有名称/大小/类型。

**决策**：`tl-kernel::variables()` 把内核 preview 格式化为 MATLAB 风格单行文本
（标量 `3.1416`；向量/矩阵 `[1 2 3 …]`，截断加省略号）。非数值类型内核不产出数值预览，显示 `—`。

**影响**：零额外成本获得变量值预览；超出 16 元素只能看到前 16 个。
要突破此限制需走内核 `materialize_variable` + `WorkspaceSliceOptions`（"变量编辑器"，v0.2 范围）。

---

## ADR-012 工具箱缺口以纯 m-script 文件交付（不 fork 内核）

**状态**：草案（v0.2 首个增量落地）

**背景**：v0.2 要对齐 MATLAB R2026a 的 5 个工具箱。实测（`exist()` 探测 246 个目标函数）
发现内核直接提供 109 个（44%），缺口 137 个。需要一个交付缺口函数的机制。

**备选**：
1. fork / patch RunMat 注册自定义内置函数 —— RunMat 的内置注册表
   （`runmat_runtime::builtins::wasm_registry::register_all`）无公开的注入 API，需 fork，维护成本高。
2. 在解释器里加 `match` 特判 —— 违反"Everything is a command"，且解释器不是我们的层。
3. **以纯 m-script `.m` 文件交付，经 `addpath` 挂载** —— 实测 RunMat 支持
   `addpath`/`genpath`/`which`，加入路径的 `.m` 函数可被调用且跨执行持久。

**决策**：采用方案 3。新增仓库根 `toolbox/<领域>/<fn>.m`，`tl-kernel`（L2）在会话
初始化时枚举 `toolbox/` 子目录并逐目录 `addpath`（不再用 `genpath`）。三个 App 经 Engine 引导自动挂载。

**影响**：
- ✅ 零内核改动、零 fork；天然落在 L7；clean-room（m 代码可追溯教科书）
- ✅ 用户可读、可改，符合 MATLAB 工具箱直觉
- ⚠️ 纯 m 实现的性能低于原生（对 MVP 可接受；hot path 后续可下沉）
- ⚠️ 需解决 `toolbox/` 目录的定位（开发期用源目录；发行期嵌入或随产物旁置）
- ⚠️ **文件函数不支持 `nargin`/`varargin`/`varargout`**（内核限制，见 [kernel-notes.md](kernel-notes.md) 坑 14）→ 工具箱统一用 `arguments` 块声明可选参数（R2019b+ 风格，实测可用）

---

## ADR-013 文件执行走 `SourceInput::Path`（解锁 classdef）

**状态**：已采纳（v0.3-dev）

**背景**：RunMat 0.6.2 的解析器支持 `classdef`，但只在 `SourceInput::Path` 请求下加载
脚本同目录的类文件；`SourceInput::Text` 不触发类发现。TensorLab 此前所有执行
（`Engine::run`、`run_file`、demo 验收、MCP `run`）都把文件读成字符串后走 Text，
因此 `class(p)` 对 class 文件报 Undefined，`zpk`/`pid` 等模型对象也无法交付。
实测证据见 `crates/tl-kernel/tests/classdef_path.rs` 与
`crates/tl-commands/tests/engine_contract.rs::run_file_loads_sibling_classdef`。

**决策**：
1. `tl-kernel::Engine` 新增 `run_path(path, source)`，用 `SourceInput::Path`（路径优先
   `canonicalize`，保证同目录类文件可被发现）；`run(text)` 保持不变。
2. 所有**磁盘文件执行**改走 Path：`tl_commands::run_file`、`run_all_demos`、
   `tl_interpreter::run_file_ok`、MCP `run` 工具。
3. classdef 的使用边界：**只允许用于返回 MATLAB 兼容的模型对象**（如后续 `zpk`/`pid`），
   不推广为用户 OOP 能力，也不改变 AGENTS §一「不做完整 OOP」的产品边界。
4. 桌面端 Run 执行的是编辑器缓冲区（无文件路径），继续走 Text；classdef 需在
   CLI/MCP 中运行磁盘脚本。

**影响**：
- ✅ `tensorlab run script.m`、MCP `run`、demo 验收均可加载同目录 classdef
- ✅ 不改变 `Engine::run` 的 REPL/单行语义，IPC 载荷（RunReport）不变
- ⚠️ Path 请求的 workspace delta 使用 `SourceBinding` 键；宿主仍通过
  `workspace_snapshot()` 生成变量面板，因此不受影响
- ⚠️ 桌面端尚未把当前文件路径传入 IPC；若需要编辑器内 classdef，后续给 `run`
  命令增加可选 `path` 参数
