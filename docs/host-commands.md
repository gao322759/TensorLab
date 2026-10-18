# 宿主命令参考

`tl-toolbox` 的命令注册表（`inventory` 编译期收集）。**只收宿主命令** —— m 语言的内置函数
（`sin` / `fft` / `plot` / …）由 RunMat 内核提供，不经此表。

在 REPL / 桌面控制台里直接输入命令名即可（不必加括号）：

```
>> vars
>> clear
```

`help` 可列出全部命令：

```
>> help
>> help vars
```

## 命令清单

| 名称 | 分类 | 说明 | 返回 |
|---|---|---|---|
| `help` | help | 列出全部宿主命令；`help <name>` 查单条；`help commands` 同裸 `help` | 文本 |
| `vars` | workspace | 工作区变量表（whos 风格：NAME / SIZE / BYTES / CLASS） | 文本 |
| `clear` | workspace | 清空工作区变量（宿主级重置） | 无输出 |
| `figures` | figures | 列出打开的图窗句柄 | 文本 |
| `version` | session | TensorLab 与内核版本 | 文本 |
| `mex` | session | `mex load <path>` 加载并探测原生动态库（MEX 风格）；`mex` / `mex status` 看状态 | 文本 |

### `help`

```
>> help
TensorLab host commands:

  clear         Clear all workspace variables (host-level reset)
  figures       List open figures (handle + title if any)
  help          List host commands; help <name> for details
  vars          Show workspace variables (whos-style table)
  version       Show TensorLab and kernel versions

MATLAB builtins: run help('<topic>') in the console, e.g. help('fft').
```

⚠️ **两种 help 是不同机制**：

- `help` / `help vars` → **宿主命令**（上面的注册表）
- `help('fft')` / `help fft` → **宿主拦截的主题文档表**（`tl-interpreter::help_topic`），
  因为 RunMat 0.6.2 内核**没有** `help` 内置函数，直接调用会触发内核深递归栈溢出

主题文档表当前覆盖：`fft` / `plot` / `eig` / `inv` / `det` / `magic` / `linspace` / `fprintf` / `strcat`
（共 10 个，含未知主题的兜底提示）。新增内置函数文档需同步 `crates/tl-interpreter/src/lib.rs`。

### `vars`

```
>> a = 1:5; b = magic(3);
>> vars
  NAME           SIZE            BYTES  CLASS
  ------------  ---------------  ---------------
  a             1x5                 40  double
  b             3x3                 72  double
```

- 引擎空工作区时返回 `No variables in the workspace.`
- 表中的 `preview`（变量面板里的 value 列）由内核 `preview` 提供，上限 16 元素；
  `vars` 命令的文本表格不含 value 列（value 列只在桌面端变量面板展示）

### `clear`

执行 `Engine::clear()`，无输出。与 m 语言的 `clear` 语句（内核实现）不同：
宿主 `clear` 是**无条件清空整个工作区**。

### `figures`

通过执行 `findobj('Type','figure')` 取句柄；无图窗时返回 `No open figures.`。

### `version`

```
>> version
TensorLab 0.1.0 (kernel: RunMat 0.6.2)
ans = R2026a
```

（`ans = R2026a` 是内核对 `version('-release')` 的回显，即内核的兼容目标版本。）

## 新增一个宿主命令

```rust
struct MyCmd;

impl Command for MyCmd {
    fn name(&self) -> &'static str { "mycmd" }                    // 必须小写
    fn category(&self) -> &'static str { "app" }                  // workspace/figures/session/help/app
    fn description(&self) -> &'static str { "One-line summary" }   // help 列表会展示
    fn run(&self, engine: &mut Engine, args: &[String]) -> CommandResult {
        CommandResult::Text("done".into())                         // 或 Report(RunReport) / None
    }
}

inventory::submit! { CommandEntry(&MyCmd) }
```

约束（由 `tl-toolbox` 单测强制）：

- `name` 非空且全小写
- `category` / `description` 非空
- 名称唯一，且 `all_commands()` 按 name 有序
- `help_overview()` 必须包含每个已注册命令名

`CommandResult` 三种形态：

| 变体 | 含义 |
|---|---|
| `Text(String)` | 纯文本输出（CLI/控制台直接打印） |
| `Report(RunReport)` | 完整运行报告（前端据此更新控制台/变量/图窗面板） |
| `None` | 无输出 |
