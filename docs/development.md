# 开发指南

> 面向在本仓库写代码的人。命令均在仓库根执行。

## 一、环境要求

| 组件 | 版本 | 说明 |
|---|---|---|
| Rust | stable（实测 1.99.0，MSVC toolchain） | `rustup default stable` |
| 平台 | Windows 10/11 x64（MSVC） | v0.1 仅验证 Windows |
| Visual Studio | 2022/2026 Community（含 C++ 桌面开发负载） | 提供 MSVC 链接器 |
| CMake | **VS 自带**（实测 4.3.1-msvc1） | ⚠️ 见下文"坑 2" |

### 为什么需要 CMake

内核依赖链里的 `hdf5-metno` 在 Windows 上是**非可选**依赖。我们在 `crates/tl-kernel/Cargo.toml`
启用 `features = ["static"]`，走 `hdf5-metno-src` 捆绑源码 + CMake 本地构建。因此**首次构建必须有 CMake**。

## 二、构建与运行

```bash
cargo check                                  # 全工作区编译检查
cargo build -p tl-cli -p tl-mcp-server        # 构建两个无 GUI 入口
cargo build -p tl-desktop                     # 构建桌面端（Tauri 2）
cargo run -p tl-cli -- run demos/fft_demo.m   # 运行 demo
cargo run -p tl-cli                           # 进入 REPL
cargo run -p tl-mcp-server                    # 启动 MCP 服务器（stdio）
```

产物：`target/debug/` 下 `tensorlab`、`tl-mcp-server`、`tensorlab-desktop`。

### 首次构建耗时

内核依赖树极大（RunMat 整套 parser/HIR/MIR/VM/runtime + HDF5 + Cranelift + Tauri）。
**冷构建 10 分钟级是预期内的**，不是卡死。后续增量构建通常 < 1 分钟（除链接外）。

## 三、必跑检查（提交前）

```bash
cargo fmt --all
cargo test                                   # 102 个测试（v0.2 基线）
cargo xtask layers                           # 分层校验（也被纳入 cargo test）
```

四项全过才算通过。清单见 [AGENTS.md](../AGENTS.md) §五。

## 四、AI 辅助开发工作流（Kent Beck 实操清单）

> 本仓库大量使用 AI 生成代码。以下规则把 Kent Beck 的观点落成可执行动作，
> 并与现有机制（demos are the spec、`cargo test`、GUI smoke、ADR）一一对应。
> 核心：**人主导 + AI 辅助**；代码行数是负债，不是资产。

### 1. 交付节奏：特性与重构严格交替

- **原则：推进一步，巩固一步。**
- 每交付一个特性（新函数 / 新命令 / 新 UI 能力）后，强制安排一次“架构加固时间”：
  - 消除重复、提炼抽象、修正脆弱接口与隐式假设；
  - 补可测试性：把不可测的大函数拆成可单测的纯逻辑；
  - 回归 `demos/` 与全部测试，确认没有“只在新路径上能跑”。
- **判断信号**：当 AI 说“搞定，下一个做啥”时，就是踩刹车的时刻。
- 本仓库落点：新增工具箱函数必须同步 `docs/toolboxes/*.md` 与 demo；
  冻结版本内的结构调整走 v0.2.x，不静默混入新特性。

### 2. 代码审查：从“信任”转为“对抗性验证”

- AI 代码**默认不可信**；审查目标是“证伪”，不是“读懂”。
- 每个数值函数至少三条证据：
  1. 已知解析解或公开参考值（与 MATLAB 期望值比对）；
  2. 退化输入（空、标量、单元素、奇异、重复极点）；
  3. 在**真实二进制**（`target/debug/tensorlab.exe`）上实跑，而不是只看编译通过。
- 警惕“测试作弊”：硬编码返回值、只覆盖 happy path、把断言写成“有输出即可”。
- 关键路径（IPC 契约、控制台语义、图窗载荷、GUI 渲染）必须有真实进程 / 真实浏览器验证。
- **红线**：屏幕上飞速滚动的语法正确代码 ≠ 健壮软件。

### 3. 学习方式：把工程当成“学习机器”

- 首要产出是**对 m-script 语义与内核边界的认知**，可运行代码只是副产品。
- 当实现被内核 bug 污染（例如 K13 目录级编译污染）时，敢于推倒重来；
  先写最小复现，再决定规避方案。
- 每次踩坑必须沉淀为三件套：
  1. `docs/kernel-notes.md` 一条（现象 + 最小复现 + 规避）；
  2. `docs/gaps.md` 一条（影响 + 计划）；
  3. 一条回归测试（防止同类问题再次进入）。

### 3.1 工具箱脚本约定（v0.2.1）

- **目录隔离**：K13/K14 目录级解析污染仍存在于 RunMat 0.6.2。合并探针显示
  `toolbox/optimization`（5 个函数）会让 `fminsearch`/`fmincon`/`ga` 等函数句柄调用报
  `Not enough input arguments`，`toolbox/stats`（16 个函数）会让 `nlinfit` 失败。
  因此 `toolbox/optim_*`、`toolbox/stats_extra/*`、`toolbox/stats_pdf/` 的隔离**不是历史遗留，
  而是当前必需**；`kernel_canaries` 的 K13/K14 测试会在内核修复后主动失败，届时再合并。
- **`arguments` 默认值**：
  - 必选参数不写类型；可选数值写 `name double = <value>`；
  - 可选矩阵/向量/options 写 `name = []`；可选字符串写 `name = '...'`；
  - `arguments` 块统一缩进 4 空格，放在函数签名之后。
- **错误消息**：
  - 用 `error('function:reason', 'message')`，`reason` 用 lowerCamelCase；
  - `message` 全小写开头、无句号、不加函数名前缀；
  - 反例测试断言消息子串，不依赖完整文案。
- **默认迭代**：启发式求解器（`ga`/`particleswarm`/`simulannealbnd`/`patternsearch`）
  默认值以“demo 可收敛 + 全量测试 < 15 分钟”为准；2026-10-17 已下调，
  精度回归由 `optim_demo` 的容差快照看守。

### 4. 价值衡量：从“产出”转向“使命”

- 不以代码行数、PR 数或生成 Token 数作为绩效指标。
- 衡量标准是**用户行为是否改变**：能否真的跑通 MATLAB 脚本、GUI 是否可用、
  AI Agent 能否通过 MCP 完成实际任务。
- 警惕古德哈特定律：demo 数量、函数数量只是中间指标；为凑数字把函数塞进会触发内核 bug 的目录，
  就是“撕裂架构去迎合数字”。

### 5. 明确反对的做法

| 反模式 | 为什么失败 | 本仓库的对应约束 |
|---|---|---|
| 黑灯软件工厂（完全自主 Agent） | 无人学习，脱轨时看起来仍异常壮观 | 每个 PR 必须人工审查；CI 不自动合并 |
| 规格驱动 / One-shot 生成 | 换了马甲的瀑布模型，假设一变证明链崩塌 | 小步特性 + 重构交替；决策写 ADR |
| 形式化验证单发 | 惩罚变化而非拥抱变化 | 回归测试 + demo 持续演进，允许重写实现 |
| 代码行数 KPI | 纯负债，诱导团队破坏架构 | 以 demo、契约测试、用户可用性衡量 |

**一句话**：成功的 AI 编程团队 = 人主导 + AI 辅助 + 特性间隙重构 + 对抗性测试 +
持续学习 + 使命对齐，而不是黑灯自治、单发生成、把代码行数当生产力。

## 五、环境坑（已验证，省你几小时）

### 坑 1：crates.io 索引极慢 / 拉取失败

配置国内稀疏镜像。`%USERPROFILE%\.cargo\config.toml`：

```toml
[source.crates-io]
replace-with = "rsproxy"

[source.rsproxy]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "sparse+https://rsproxy.cn/index/"
```

⚠️ 不要重复定义同一个 `[source.*]`，否则报 "source 重复定义"。

### 坑 2：CMake 不识别 VS 2026 生成器

`cmake` crate 会传 `-G "Visual Studio 18 2026"`，而**系统安装的** CMake（如 3.31）只认到 VS17，
报 `could not find any instance of Visual Studio`。

→ 使用 **VS 自带的 CMake**（实测 `.../2026/Community/Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin`），
并让它排在 PATH 前面：

```powershell
$env:Path = "<VS CMake bin>;$env:USERPROFILE\.cargo\bin;$env:Path"
cargo build
```

### 坑 3：构建中断后出现"找不到 crate"（E0463）

中断（Ctrl+C / 进程被杀）会留下脏指纹，之后报

```
error[E0463]: can't find crate for `ed25519_dalek`
```

而该 crate 的 rlib **确实存在**。修复：

```bash
cargo clean -p <报错的上游 crate>    # 例：cargo clean -p runmat-execution
```

### 坑 4：杀软锁死新编译的 exe

360 / 火绒 会对无签名的大型新编译 exe 做实时扫描，导致 `Access is denied`，
甚至 `Remove-Item`、`Rename-Item`、`icacls /reset`、`takeown` 全部失败。

→ 缓解：`cargo clean -p tl-desktop` 重链，或换 bin 名（见 [decisions.md](decisions.md) ADR-009）。
→ 项目已将桌面端 bin 命名为 `tensorlab-desktop` 规避。

**变体：残留进程占用 exe**。反复运行 `target\debug\tensorlab.exe` 后，偶尔会留下
无法被杀掉的 `tensorlab` 进程（`taskkill /F` 报 `Access is denied`），进而导致
`cargo test` 报 `failed to remove file ... \tensorlab.exe`（os error 5）。

→ 缓解：先 `taskkill /F /IM tensorlab.exe`；若仍失败，直接 `Remove-Item` 该 exe
（实测此时删除可能反而成功），再重跑 cargo。必要时 `cargo clean -p tl-cli`。

### 坑 5：链接期内存不足

`cargo test` 默认并行会同时链接多个 1GB 级 rlib，可能报
`memory allocation of ... failed` / `failed to mmap file`。

→ 用 `cargo test -j 2` 限制并行度。

### 坑 6：PowerShell 写测试脚本要无 BOM

`Set-Content`（PowerShell 5）默认编码会带 BOM，`.m` 文件解析失败。用：

```powershell
$enc = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText("$d\a.m", "x = 42`n", $enc)
```

### 坑 7：`cargo xtask` 需要别名

仓库根的 [.cargo/config.toml](../.cargo/config.toml) 定义了
`xtask = "run --package xtask --"`。**若该文件缺失**，`cargo xtask layers` 会报
`error: no such command: xtask`。等价的直接写法：`cargo run -p xtask -- layers`。

## 六、调试技巧

### 看一次执行的完整结果

```bash
echo "x = 1:5" | target/debug/tensorlab        # REPL
target/debug/tensorlab run script.m
```

CLI 把控制台输出写到 **stdout**，`[VM xx ms]` 状态行写到 **stderr** —— 便于脚本只取纯输出。

### 内核行为不明时

**先读 `docs/kernel-notes.md`**（12 个已验证的坑），再读内核源码：

```
%USERPROFILE%\.cargo\registry\src\<mirror-hash>\runmat-core-0.6.2\src\
```

关键入口：`session/run.rs`（执行 + 回显 + streams）、`workspace/emit.rs`（变量快照与预览）、
`abi.rs`（`ExecutionOutcome` 全部字段）、`runmat-plot/src/event.rs`（FigureScene schema）。

### 分层违规

`cargo run -p xtask -- layers` 会指出具体是哪条依赖边违规（crate + 层号）。
把 RunMat 类型限制在 `tl-kernel` 是设计的核心约束，不要为了图方便在别处直接 `use runmat_core`。

## 七、目录导航

```
crates/tl-types         L0  显示格式化 + serde 载荷（只允许 serde/serde_json/runmat-value）
crates/tl-linalg        L1  纯 Rust LU/det/solve/inv
crates/tl-kernel        L2  Engine 门面（唯一接触 RunMat 类型）
crates/tl-parser        L3  编辑器 tokenizer
crates/tl-interpreter   L4  结果后处理 / help 主题 / demo 验收器
crates/tl-jit           L5  桩
crates/tl-mex           L6  桩（v0.2 实现）
crates/tl-toolbox       L7  宿主命令注册表（inventory）
crates/tl-commands      L8  应用门面（run / repl_line / host_command / demos）
apps/tl-cli             CLI（bin: tensorlab）
apps/tl-mcp-server      MCP stdio 服务器
apps/tl-desktop         Tauri 2 桌面端（src-tauri/ + ui/）
tools/xtask             分层校验器
demos/                  5 个验收脚本（demos are the spec）
docs/                   文档
```
