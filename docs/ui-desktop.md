# 桌面端 UI（`tensorlab-desktop`）

Tauri 2 应用，主产品。Rust 侧在 `apps/tl-desktop/src-tauri/`，前端是**单个静态 HTML**
（`apps/tl-desktop/ui/index.html`），通过 `tauri.conf.json` 的 `frontendDist: "../ui"` 编译期内嵌。

> 无 npm、无构建步骤、无 CDN —— 改完 HTML 重新 `cargo build -p tl-desktop` 即生效。

## 一、布局

UI 范式对齐 MATLAB R2026a（clean-room 测量见 `docs/ui/r2026a-reference/design-tokens.md`）：

```
┌───────────────────────────────────────────────────────────────────────┐
│ [T] TensorLab v0.2.1  script1.m                      ☾  ⓘ             │  titlebar 28（两主题均白底）
├───────────────────────────────────────────────────────────────────────┤
│ HOME  EDITOR  VIEW  HELP            [search workspace]                │  ribbon tabs 34（深蓝 #144074）
│ New Open Save SaveAs │ Run Stop │ Clear Cmds │ Light Dark Preferences │  ribbon body 120
├───────────────────────────────────────────────────────────────────────┤
│ Home > script1.m                                                      │  pathbar 26
├──────────────────────┬───────────────────────────────┬────────────────┤
│ WORKSPACE            │ script1.m                     │ FIGURES        │
│ Name Size Class Value│ 1  %% TensorLab demo …        │ [Scene → SVG]  │
│                      │ (gutter) (highlight overlay)   │                │
│                      ├───────────────────────────────┤                │
│                      │ COMMAND WINDOW    banner       │                │
│                      │ >> 输入行                      │                │
├──────────────────────┴───────────────────────────────┴────────────────┤
│ ready        Ln 1, Col 1 │ UTF-8 │ Light │ RunMat 0.6.2                │  statusbar 22
└───────────────────────────────────────────────────────────────────────┘
```

- **浅色为默认主题**，深色一键切换（标题栏 ☾ / VIEW ribbon / Preferences 三入口），选择写入 `localStorage`。
- Ribbon 精简为 4 个 tab：`HOME` / `EDITOR` / `VIEW` / `HELP`；无 Simulink / APPS / PUBLISH。
- 三栏面板可**双击头部折叠**、**按住头部跨列拖拽**、**拖动分隔条**调整编辑器/命令行比例；VIEW → Reset Layout 复位。
- 度量常量集中在 `:root`（2026-11-12 按 MATLAB 像素复测校正，见 design-tokens.md §1.1：
  `--h-titlebar: 28px`、`--h-ribbon-tabs: 34px`、`--h-ribbon-body: 120px`、
  `--h-pathbar: 26px`、`--h-statusbar: 22px`、`--h-panelhead: 28px`、`--w-gutter: 40px`）；
  旧变量别名（`--bg/--fg/--accent`…）保留。

## 二、Rust 侧与前端契约

前端通过 `__TAURI_INTERNALS__.invoke(cmd, args)` 调用：

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `run` | `{ source: string, path?: string }` | `RunReport` | 执行编辑器缓冲（存在断点时自动走插桩调试路径）；调试暂停期间直接返回错误而不排队 |
| `stop` | — | — | 协作式取消（内核在语句边界响应）；调试暂停时等价 `dbquit` |
| `vars` | — | `VariableInfo[]` | 工作区快照 |
| `debug_set_breakpoints` | `{ lines: number[] }` | — | 整表同步断点（编辑器 gutter 每次变更后调用） |
| `debug_state` | — | `number \| null` | 当前暂停行号（1-based），未暂停为 `null`；前端运行期间每 80 ms 轮询 |
| `debug_resume` | `{ mode: "continue" \| "step" \| "stop" }` | — | 调试恢复：dbcont / dbstep / dbquit |
| `app_info` | — | `{ version, kernel, compat, license }` | About 对话框（版本来自 `CARGO_PKG_VERSION`） |
| `read_text_file` | `{ path }` | `string` | 读 `.m` |
| `write_text_file` | `{ path, contents }` | — | 写 `.m` |

后端 → 前端事件：

| 事件 | 载荷 | 说明 |
|---|---|---|
| `debug-paused` | `number`（1-based 行号） | 命中断点或单步暂停；**快路径**，失败不影响功能（见下） |

> ⚠️ 暂停通知有**两条通道**：`debug-paused` 事件（快路径）与 80 ms 轮询 `debug_state`（主通道）。
> 事件注册可能因权限/时序失败，而引擎此时已阻塞在调试钩子上——只靠事件会出现
> “Run 变灰、界面像死机”的观感。轮询保证暂停一定被 UI 发现；`Stop` 无条件调用后端
> `stop`（后端判断暂停中则转 dbquit），因此即使 UI 没收到暂停通知也能停止执行。
> `run` 命令为 `async` + `spawn_blocking`，不会占住 Tauri 的同步命令线程而挡住 `stop`/`debug_resume`。

插件调用（`tauri-plugin-dialog`）：

| 调用 | 参数 | 返回 |
|---|---|---|
| `plugin:dialog\|open` | `{ options: { title, multiple:false, directory:false, filters } }` | 路径字符串 或 `null` |
| `plugin:dialog\|save` | `{ options: { title, defaultPath, filters } }` | 路径字符串 或 `null` |

⚠️ 插件 `options` 为 **camelCase**（`defaultPath`），路径参数是**普通字符串**（非 `{path:...}` 包装）。

### 引擎线程模型

UI 线程**不**直接跑引擎。`spawn_engine_thread()` 创建一条 64 MiB 栈的线程独占 `Engine`，
通过 `mpsc` 收发：

```
UI(JS) ──invoke("run")──► Tauri command ──mpsc──► 引擎线程(64MiB 栈)
                                                      │ Engine::run
UI(JS) ◄──RunReport─────── Tauri command ◄──mpsc──────┘
```

理由见 [decisions.md](decisions.md) ADR-006（内核 parser/eval 深递归会爆主线程栈）。
Tauri 命令在等待 `recv()` 时会阻塞，但阻塞的是 Tauri 的命令线程而非引擎线程。

## 三、前端实现要点

### 编辑器：自研 overlay 高亮（非 Monaco）

```
<div id="editor">
  <div id="gutter"><div id="gutter-inner"></div></div>   <!-- 行号列 40px，随滚动 translateY -->
  <div id="editor-main">
    <pre id="highlight"></pre>          <!-- 高亮层，pointer-events:none -->
    <textarea id="src" spellcheck="false"></textarea>  <!-- 输入层，文字透明、光标可见 -->
  </div>
</div>
```

- 两层字体/行高/内边距必须**完全一致**，否则高亮错位（`--editor-font` / `--editor-line` 统一驱动）
- 滚动同步：`src.addEventListener("scroll", ...)` 同步 `hl.scrollTop/Left` 与 gutter 的 `translateY`
- 着色规则：关键字 / 内置函数 / 数字 / 字符串 / 注释（正则 + 先保护已生成的 `<span>`）
- Tab 插入 4 空格；`Ctrl+Enter` 运行；行号可在 Preferences → Editor 关闭

已知限制见 [gaps.md](gaps.md)（无自动补全/括号匹配/错误波浪线）。

### About 与版本显示

- 标题栏品牌右侧显示 `v<version>`，版本来自 Tauri 命令 `app_info`（`CARGO_PKG_VERSION`）；
- 点击标题栏 ⓘ、HELP → About 或按 `F1` 打开对话框，显示版本、内核（RunMat 0.6.2）、
  兼容目标（MATLAB R2026a）与许可证（MIT）；
- `Esc`、Close 按钮或点击遮罩关闭；
- 浏览器直开（无 Tauri）时使用内置后备版本号，GUI smoke 用 IPC shim 覆盖该命令。

### 控制台：按事件 tag 分派

```js
for (const e of report.events) {
  if (e.kind === "output") log(e.text);
  else if (e.kind === "error") log(e.text, "err");
  else if (e.kind === "clearConsole") consoleEl.innerHTML = "";
}
if (report.error)    log(`Error${...}: ${report.error.message}`, "err");
for (const w of report.warnings) log(`Warning: ${w.message}`, "warn");
```

⚠️ tag 值是**小写** camelCase（`output`/`error`/`clearConsole`），由
`stream_entry_tags_are_camel_case` 钉死。曾因这里写成 PascalCase 导致**所有**控制台输出不显示。

### 控制台 REPL（宿主命令入口）

控制台底部有输入行（`#console-input`），按 Enter 提交：

- 后端 `run` 命令走 `tl_commands::repl_line`，因此**与 CLI 的 REPL 完全等价**；
- 可直接输入宿主命令：`help`、`vars`、`clear`、`figures`、`version`、`mex load <path>`；
- 也可输入单行 m 代码（`a = 1:5`、`sin(pi/2)`）；
- ↑/↓ 浏览历史；提交行以 `>> ` 前缀回显（`.meta` 样式）。

> 注意：编辑器里的「▶ Run」执行的是**整个脚本**（多行），不会走宿主管道；
> `mex`/`vars`/`help` 这类单行命令请用控制台输入行。

### 变量面板：value 列

`report.variables[].preview` 是后端格式化好的 MATLAB 风格单行文本：

| name | size | class | value |
|---|---|---|---|
| `a` | 1×5 | double | `[1 2 3 4 5]` |
| `A` | 5×5 | double | `[17 24 1 8 15 23 5 7 14 16 …]` |
| `s` | 1×5 | char | — |

标量不加方括号（`42`）；超 16 元素加 `…`；非数值类型为空 → 显示 `—`。

工作区表格行高 18px；≤4096 元素的变量最多展示 128 个数值。全局搜索框按名字过滤 Workspace 行。

### 图窗：`figureSvg()`（约 250 行，非 Plotly）

输入是 `figures[].sceneJson`，真实结构（**与 runmat_plot 的类型声明不一致**，以实测为准）：

```json
{ "createdAt": "...",
  "figure": {
    "layout":   { "axesRows": 2, "axesCols": 1, "axesIndices": [0] },
    "metadata": { "activeAxesIndex": 0,
                  "axesMetadata": [ { "title": "sin(x)", "xLabel": "...", "gridEnabled": true } ] },
    "plots":    [ { "kind": "line", "x": [...], "y": [...],
                    "color_rgba": [0,0,1,1], "line_width": 2,
                    "line_style": "Solid", "axes_index": 0, "label": "sin(x)", "visible": true } ] } }
```

渲染步骤：

1. 解 **envelope**：取 `scene.figure`（若无则用根）
2. 按 `plots[].axes_index` 分组
3. 布局：`layout.axesRows × layout.axesCols` 网格，每格 400×250
4. 每格自算数据范围 → 坐标映射 → 画网格/刻度/数据
5. 子图标题/轴标签/网格取自 `metadata.axesMetadata[i]`

支持的 `kind`：`line` / `stairs` / `scatter` / `bar` / `histogram` / `stem` / `area` / `errorbar`。
颜色用 `color_rgba`（`[f32;4]` 0-1）转 CSS `rgba()`；虚线由 `line_style`
（`Solid`/`Dashed`/`Dotted`/`DashDot`）映射。

### 文件菜单

- Open（Ctrl+O）/ Save（Ctrl+S）/ Save As（Ctrl+Shift+S）位于 HOME、EDITOR 两个 ribbon tab 的 FILE 组
- 标题栏、编辑器 tab、地址栏同步显示文件名与未保存标记 `•`（`document.title` 同步）
- 打开/保存走原生对话框 → `read_text_file` / `write_text_file`
- 打开/保存结果会追加一行 `opened <path>` / `saved <path>` 到控制台
- `Ctrl+,` 打开 Preferences（Appearance / Editor / Workspace / About TensorLab）

### Ribbon / 主题 / 停靠

- Ribbon 由 `app.js` 同文件内的 `RIBBONS` 数据驱动，4 个 tab：HOME / EDITOR / VIEW / HELP；
  按钮统一走 `data-act` + 事件委托，tab 重绘不会丢监听。
- 主题 token 分浅/深两套（`html[data-theme="light|dark"]`），浅色默认。2026-11-12 按 MATLAB 实测校正：
  标题栏**两主题均白底黑字**；ribbon tab 行**两主题均品牌深蓝 `#144074`**（活动 tab 与内容行连体）；
  深色下 ribbon 内容行 `#212121`、内容区统一 `#121212`；活动面板头为蓝底 `#2e71bf`（点击面板即激活）；
  命令行横幅深色 `#433302`。
- 停靠：`.dock-col` × 3；面板头双击折叠，按住头部拖拽可跨列停靠（`.dock-col.drop` 高亮），
  `#hsplit` 拖动调整编辑器/命令行比例；`resetLayout()` 复位。
- 面板内 DOM 契约不变：`#src` / `#highlight` / `#console` / `#console-input` / `#vars` /
  `#figures` / `#status` / `run-btn` / `stop-btn` / `open-btn` / `about-btn`。

## 四、调试与错误处理

### 断点 / 步进 / 停止

RunMat 0.6.2 内核没有调试器 API，桌面端用 **源码语句级插桩 + 阻塞式内置函数**
实现 MATLAB `dbstop`/`dbcont`/`dbstep`/`dbquit` 语义（实现见 `crates/tl-kernel/src/debug.rs`）：

1. `Engine::run_debugged*` 在**存在断点**时，把源码逐语句行行首注入 `__tl_debug__(N);`
   （同行插入，行号不变）；无断点时走普通 `run`，零开销。
2. `__tl_debug__` 是经 `inventory` 注册进内核 builtin 表的内置函数；
   命中断点或单步模式时，先发 `debug-paused` 事件，再在 Condvar 上阻塞等宿主恢复。
3. `stop`（dbquit）置内核中断标志并以错误终止——对齐 dbquit 的终止语义。

注入点会跳过：空行/纯注释、`...` 续行、括号内换行、块注释、`arguments` 块、
结构性关键字行（`function`/`end`/`else`/`elseif`/`case`/`otherwise`/`catch` 等）与整个 `classdef` 文件。

| 操作 | 入口 |
|---|---|
| 切分断点 | 编辑器 gutter 单击；`F12`（当前行）；控制台 `dbstop <line>` |
| 运行 | HOME/EDITOR → Run、`F5`（未暂停时） |
| 单步 | EDITOR/HOME → Step、`F10`/`F11`、`dbstep` |
| 继续 | EDITOR/HOME → Continue、`F5`（暂停时）、`dbcont` |
| 停止 | Stop 按钮、`dbquit` |
| 清除 | HOME → Clear Breakpoints、`dbclear <line>` / `dbclear all` |

暂停时：行背景绿色高亮 + gutter 绿色箭头 + 状态栏 `paused · line N`；Run 禁用，
Continue/Step 启用。暂停期间控制台的新 `run` 请求会被拒绝并提示先 `dbcont`/`dbquit`
（引擎线程被调试钩子占住，排队会死等）。

**已知限制**：步进粒度是语句行（`else`/`case`/`end` 等结构行不暂停，与 MATLAB 行级步进一致）；
暂停期间变量面板不刷新（`vars` 与 `run` 共用一条引擎线程），恢复/结束后更新；
尚不支持函数级 `dbstop in <fn>`、条件断点与 `dbstack`。

### 错误处理

- 控制台错误按 MATLAB 格式输出：`Error (identifier): message` → `at line N` → 调用栈；
  该行可**点击**跳转编辑器并红色高亮出错行（gutter 行号转红）。
- 每次 Run 前清除上一轮错误高亮。
- 警告以 `Warning: ...` 黄色输出；`clearConsole` 事件清空控制台。

## 五、capabilities 权限

`src-tauri/capabilities/default.json`：

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default", "dialog:default", "dialog:allow-open", "dialog:allow-save"]
}
```

文件读写**不**经 `tauri-plugin-fs`（避免其 scope 权限体系），而是自研 Rust 命令（见
[decisions.md](decisions.md) ADR-010）。

## 六、构建与运行

```bash
cargo build -p tl-desktop
target/debug/tensorlab-desktop      # 或 target/release/
```

> ⚠️ 产物名是 `tensorlab-desktop`（不是 `tl-desktop`）。原因见 ADR-009（本地杀软锁死原名）。
> 若启动报 `Access is denied`，是杀软实时扫描在锁定文件，重链或改名即可。
>
> ⚠️ 环境备注（2026-11-11）：本机 `cargo check -p tl-desktop` 会因依赖 `hdf5-metno-src` 的 cmake
> 找不到 "Visual Studio 18 2026" 生成器而失败（与前端改动无关）。前端请用
> `node apps/tl-desktop/ui/tests/gui_smoke.mjs` 独立验证。

## 七、自动化测试

- `apps/tl-desktop/ui/tests/gui_smoke.mjs`：零 npm 依赖，用 headless Edge/Chrome + CDP 加载真实
  `ui/index.html`，注入 Tauri IPC shim，覆盖 **19 条断言**：初始渲染与高亮、startup banner、
  About/版本对齐、Run → 控制台/Workspace/FigureScene→SVG、图窗结构、控制台 REPL 与历史、
  错误样式、右键复制、Ctrl+L、clearConsole、4 tab ribbon、主题切换、Preferences、面板折叠与布局复位、
  gutter 断点切换与后端同步、debug-paused 高亮与按钮联动、Continue/Step/Stop 恢复链路、
  控制台 `dbstop`/`dbcont`/`dbclear` 命令。
- 内核调试器单测：`cargo test -p tl-kernel --lib debug::`（插桩扫描器 10 例 + 断点暂停/恢复端到端）。
- 入口：`node apps/tl-desktop/ui/tests/gui_smoke.mjs`（或 `cargo test -p tl-desktop`，Node 缺失时跳过）。
- 后端数据正确性由 `crates/tl-commands/tests/engine_contract.rs` 保障。
