# 贡献指南（Contributing）

> 面向任何向 TensorLab 提交代码的人。**先读 [AGENTS.md](../AGENTS.md)（项目宪法）**——它是最高约束，本文件是其操作化补充。

## 一、三条铁律

1. **分层不可破**：层 L 只能依赖层号更小的 crate，同层禁止互依。违反者 `cargo xtask layers` 直接失败。见 [architecture.md](architecture.md)。
2. **Everything is a command**：新增内置函数 = 新增一个 `tl_runtime::Command` 实现（经 `inventory` 自动注册），不在解释器里写 `match` 特判。
3. **Engine-first**：功能先在无头 `tl_commands::Engine` 可用，再考虑 UI。

## 二、开发环境

```powershell
# 1. 工具链：rustup stable（MSVC）
rustup show

# 2. 环境前缀（每终端一次；让 cargo 找到 VS2026 自带 CMake 与 rustup bin）
$env:PATH = "D:\Program Files\Microsoft Visual Studio\2026\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"

# 3. 编译
cargo check
```

完整环境坑（镜像、CMake 生成器、大栈线程、HDF5 静态链接等）见 [development.md](development.md)。

## 三、提交前必跑

```powershell
cargo fmt                     # 格式化
cargo check                   # 编译
cargo test                    # 全量测试（含 10 demo 验收）
cargo xtask layers            # 分层校验
```

四项全绿才可提交。详见 [testing.md](testing.md)。

## 四、新增一个内置命令（最常见贡献）

**Step 1**：在对应层选 crate —— 数值 / 线代 → `tl-toolbox`；纯宿主能力（`help`/`vars`/`clear`）→ `tl-commands`（见 [host-commands.md](host-commands.md)）。

**Step 2**：实现 `Command`：

```rust
use tl_runtime::{Command, CommandContext, CommandResult};

pub struct MyFunc;

impl Command for MyFunc {
    fn name(&self) -> &'static str { "my_func" }
    fn category(&self) -> &'static str { "core" }       // help 分组
    fn description(&self) -> &'static str { "One-line summary." }  // help <name> 依赖
    fn execute(&self, ctx: &mut CommandContext) -> CommandResult { /* ... */ }
}

inventory::submit! { MyFunc }   // 自动注册
```

**Step 3**：加测试（至少在 `tl-commands/tests/engine_contract.rs` 加一条端到端断言）。

**约束**：
- `name` 小写；`category` / `description` 必填（`help` 依赖）。
- 不 panic，错误走 `tl_types::TlError`（MATLAB 风格英文消息 + 行号）。
- 不写 `match` 特判进解释器。

## 五、代码风格

- Rust 2021，`rustfmt` 默认风格。
- 注释用中文，标识符与错误消息用英文（AGENTS.md §六）。
- 数值内核：`Matrix` 以 `Vec<Complex<f64>>` **列主序**存储（MATLAB 语义）。
- 兼容最新版 MATLAB 是硬要求：1-based 索引、`end`、隐式广播、复数默认。

## 六、clean-room 红线

- ✅ 可参考：MATLAB 公开文档的语义描述、教科书算法（LAPACK/NumPy 论文）、通用函数名。
- ❌ 不可参考：MathWorks 源码、其测试数据、文档原文摘抄。

所有算法实现必须能追溯到教科书 / 公开论文。见 [attribution.md](attribution.md)。

## 七、测试要求

| 改动类型 | 必须新增 |
|----------|----------|
| 新命令 | `engine_contract.rs` 一条端到端断言 |
| 改 IPC 载荷 | `tl-types` 契约测试（serde tag / 字段名） |
| 改 MCP 协议 | `mcp_stdio.rs` 报文测试 |
| 改 CLI 行为 | `apps/tl-cli/tests/cli.rs` |
| 改分层依赖 | `tools/xtask/tests/layers.rs` 会拦 |

断言原则见 [testing.md](testing.md) §设计原则。

### AI 生成代码的额外审查（对抗性验证）

本仓库大量使用 AI 生成代码，审查规则按 Kent Beck 的实操清单执行（完整版见
[development.md](development.md) §四）：

1. **默认不可信**：审查目标是“证伪”，不是“读懂”。先问“它真的能用吗？我要测到什么程度才能信它？”
2. **三条证据**：已知解析解/参考值、退化输入（空/标量/奇异/重复极点）、真实二进制实跑。
3. **防测试作弊**：拒绝硬编码返回值、只覆盖 happy path、断言“有输出即可”；
   用不同输入验证输出确实随输入变化。
4. **关键路径必须真实环境验证**：IPC 契约、控制台语义、图窗载荷、GUI 渲染不能只靠“看起来合理”。
5. **特性与重构交替**：每交付一个特性，安排一次架构加固（消重复、收敛抽象、补可测试性），
   再进入下一个需求；禁止“能跑就行，别动了”。

> 红线：屏幕上飞速滚动的语法正确代码 ≠ 健壮软件。代码行数是负债，不是资产。


## 八、提交信息

```
<type>: <简短祈使句>

<可选正文：为什么这样改>
```

`type` ∈ `feat` / `fix` / `docs` / `refactor` / `test` / `chore` / `perf`。

## 九、常见陷阱

- **ctrl 撞车**：新建 crate 后忘了在根 `Cargo.toml` 的 `members` 注册 → 不被构建。
- **同层依赖**：在 `tl-toolbox` 里 `use tl_commands::...` → 分层校验失败。
- **BOM**：用 PowerShell 写 `.m` 文件会带 BOM 致解析失败，用 `UTF8Encoding($false)`。
- **脏指纹**：构建中断后 `E0463` → `cargo clean -p <crate>` 重来。
