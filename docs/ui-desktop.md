# 桌面端 UI（`tensorlab-desktop`）

Tauri 2 应用，主产品。Rust 侧在 `apps/tl-desktop/src-tauri/`，前端是**单个静态 HTML**
（`apps/tl-desktop/ui/index.html`），通过 `tauri.conf.json` 的 `frontendDist: "../ui"` 编译期内嵌。

> 无 npm、无构建步骤、无 CDN —— 改完 HTML 重新 `cargo build -p tl-desktop` 即生效。

## 一、布局

```
┌───────────────────────────────────────────────────────────────────────┐
│ TensorLab v0.2.1 ready      Open… Save SaveAs About ▶Run ■Stop        │  header
├──────────────────────┬───────────────────────┬────────────────────────┤
│ EDITOR  script1.m •  │ CONSOLE               │ WORKSPACE              │
│                      │                       │ name size class value  │
│  (高亮覆盖层)         │  输出 / 错误 / 警告    ├────────────────────────┤
│                      │                       │ FIGURES                │
│                      │                       │ [ FigureScene → SVG ]  │
└──────────────────────┴───────────────────────┴────────────────────────┘
```

## 二、Rust 侧与前端契约

前端通过 `__TAURI_INTERNALS__.invoke(cmd, args)` 调用：

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `run` | `{ source: string }` | `RunReport` | 执行编辑器缓冲 |
| `stop` | — | — | 协作式取消（内核在语句边界响应） |
| `vars` | — | `VariableInfo[]` | 工作区快照 |
| `app_info` | — | `{ version, kernel, compat, license }` | About 对话框（版本来自 `CARGO_PKG_VERSION`） |
| `read_text_file` | `{ path }` | `string` | 读 `.m` |
| `write_text_file` | `{ path, contents }` | — | 写 `.m` |

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
  <pre id="highlight"></pre>          <!-- 高亮层，pointer-events:none -->
  <textarea id="src" spellcheck="false"></textarea>  <!-- 输入层，文字透明、光标可见 -->
</div>
```

- 两层字体/行高/内边距必须**完全一致**，否则高亮错位
- 滚动同步：`src.addEventListener("scroll", ...)` 同步 `hl.scrollTop/Left`
- 着色规则：关键字 / 内置函数 / 数字 / 字符串 / 注释（正则 + 先保护已生成的 `<span>`）
- Tab 插入 4 空格；`Ctrl+Enter` 运行

已知限制见 [gaps.md](gaps.md)（无自动补全/括号匹配/错误波浪线）。

### About 与版本显示

- 页眉 logo 右侧显示 `v<version>`，版本来自 Tauri 命令 `app_info`（`CARGO_PKG_VERSION`）；
- 点击页眉 `About` 或按 `F1` 打开对话框，显示版本、内核（RunMat 0.6.2）、
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

- `Open…`（Ctrl+O）/ `Save`（Ctrl+S）/ `Save As…`（Ctrl+Shift+S）
- 标题栏显示文件名与未保存标记 `•`（`document.title` 同步）
- 打开/保存走原生对话框 → `read_text_file` / `write_text_file`
- 打开/保存结果会追加一行 `opened <path>` / `saved <path>` 到控制台

## 四、capabilities 权限

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

## 五、构建与运行

```bash
cargo build -p tl-desktop
target/debug/tensorlab-desktop      # 或 target/release/
```

> ⚠️ 产物名是 `tensorlab-desktop`（不是 `tl-desktop`）。原因见 ADR-009（本地杀软锁死原名）。
> 若启动报 `Access is denied`，是杀软实时扫描在锁定文件，重链或改名即可。

## 六、未自动化测试的部分

UI 的 HTML/JS **没有**自动化测试（需要 WebDriver/tauri-driver 或 JS 测试运行时，见 [testing.md](testing.md)）。
后端数据正确性由 `crates/tl-commands/tests/engine_contract.rs` 保障；"渲染成什么样"目前依赖人工验证。
