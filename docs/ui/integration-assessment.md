# M4 集成评估：把 MATLAB R2026a 范式 UI 接入 tl-desktop

> 结论：**可接入，且成本低于预期**——因为 tl-desktop 前端是单文件 `ui/index.html`（Tauri WebView + 自研高亮器 + 自研 SVG 绘图），
> 与原型同栈（HTML/CSS/JS），且现有 DOM 契约（`#src` / `#highlight` / `#console` / `#console-input` / `#vars` / `#figures`）可原样保留。
> 建议按"先换皮、后换骨架、最后加停靠"三步走，每步都可独立发布。
>
> **实施状态（2026-11-11）：三步已全部落地**到 `apps/tl-desktop/ui/index.html`（第 1 步 token / 第 2 步 titlebar+ribbon+pathbar+statusbar /
> 第 3 步 停靠 + Preferences），`node apps/tl-desktop/ui/tests/gui_smoke.mjs` **15/15 PASS**（新增 4 条 ribbon/主题/Preferences/停靠断言）。
> 说明：本机 `cargo check -p tl-desktop` 因 `hdf5-metno-src` 的 cmake 找不到 "Visual Studio 18 2026" 生成器而失败（环境问题，非本次改动引入），
> 前端验证走真实浏览器（headless Edge + CDP）。

## 1. 现状对照

| 维度 | tl-desktop 现状（`apps/tl-desktop/ui/index.html`） | 原型（`docs/ui/matlab-ui-proto`） |
|------|--------------------------------------------------|-----------------------------------|
| 结构 | 单文件：`:root` token（L7-10）+ `<style>`（L6-154）+ header/main（L157-192）+ script（L211-802） | 三文件分离：`styles.css` / `index.html` / `app.js` |
| 主题 | **仅深色**（`--bg:#1e1e1e`，accent 已是 `#0072BD`） | 浅色默认 + 深色，双主题 token |
| 顶栏 | 工具条按钮（Open/Save/Save As/About/Run/Stop） | 标题栏 + 6 tab ribbon + 地址栏 |
| 布局 | 三栏：editor / console / right(vars+figures) | 三栏 + 面板头 tab + 可拖拽停靠 + 分隔条 |
| 编辑器 | `pre#highlight` 覆盖 `textarea#src` 自研高亮 | 只读渲染（演示用），高亮规则同源 |
| 绘图 | 自研 FigureScene→SVG（ADR-004） | 示意 SVG（无渲染器） |
| 控制台 | `#console` + `#console-input` + 右键菜单 | 同构（提示符/历史/横幅） |

**关键发现**：现有 `#highlight` 覆盖层方案与原型的高亮视觉一致，**编辑器不需要重写**，只需把 token 喂给现有 `pre`。

## 2. 三步集成路径

### 第 1 步（换皮）：token 层替换 — 复杂度 S

- 把 `:root`（L7-10）替换为实测 token 集（`docs/ui/r2026a-reference/design-tokens.md` §2/§6），并新增 `html[data-theme="light"]` 默认 + `html[data-theme="dark"]`。
- 现有 `--bg/--bg2/--bg3/--panel/--fg/--dim/--accent/--err/--warn/--border/--sel` 保留为**别名**指向新 token，保证 802 行脚本里已有的内联样式引用不失效。
- 度量层（`--h-titlebar: 28px` 等）同时引入，为第 2 步铺路。
- **产出**：外观立刻接近参考（当前深色已与测量值同族，主要是补齐浅色）。**零 JS 改动**。

### 第 2 步（换骨架）：header → 标题栏 + ribbon — 复杂度 M

- `<header>`（L157-167）替换为 `titlebar` + `ribbon` + `pathbar` 三段；现有按钮改为 ribbon 项：
  - FILE 组：Open / Save / Save As
  - CODE 组：Run / Stop
  - RESOURCES 组：About（→ Preferences 入口）
- **DOM 契约保持**：所有 `id`（`open-btn` / `save-btn` / `run-btn` / `stop-btn` / `about-btn`）必须原样保留，脚本的事件绑定零改动。
- Ribbon 由数据驱动（原型 `RIBBONS` 结构），后续加 tab 只改数据。
- 明暗主题切换按钮加入标题栏；默认浅色需产品确认（当前首启是深色）。

### 第 3 步（加停靠）：dock 引擎 + Preferences — 复杂度 M

- 引入原型的面板容器（`dock-col` + `panel-head` + `.hsplit` + 拖拽重排），把 `#editor-pane` / `#console-pane` / `#right-pane` 包进 panel。
- **同样保持 id**：`#editor` / `#console` / `#vars` / `#figures` 不动，脚本与 CSS 选择器不受影响。
- Figure 面板：把自研 SVG 渲染结果放进 `#figures` 的面板体（现有 `#figures` 已是容器，改动很小）。
- Preferences 抽屉：复用原型 `PREF_PAGES` 结构，第一版只放 Appearance（主题/字体）+ Layout + About 信息。
- 双击面板头折叠、跨列拖拽、分隔条拖拽：原型代码可直接移植（约 120 行）。

## 3. 需要产品决策的 3 点

| # | 决策 | 选项 | 建议 |
|---|------|------|------|
| 1 | 首次启动主题 | 深色（现状）/ 浅色（MATLAB 默认） | ✅ **已确认（2026-11-11）：浅色**——目标是"MATLAB 范式"，且参考机默认浅色；保留"记住上次选择" |
| 2 | Ribbon 是否保留全部 6 个 tab | 6 个（含 PLOTS/APPS/PUBLISH，多数为空）/ 精简为 4 个（HOME/EDITOR/VIEW/HELP） | ✅ **已确认（2026-11-11）：精简 4 个**，空 tab 会损害可信度；PLOT 相关并入 HOME |
| 3 | 是否保留 `#highlight`+`textarea` 自研编辑器 | 保留 / 换 Monaco | **保留**（ADR-005 结论不变）；本次范式的收益在骨架不在于编辑器内核 |

## 4. 风险与缓解

| # | 风险 | 等级 | 缓解 |
|---|------|------|------|
| R1 | `color-mix()` 需 WebView2 ≥ 111 | 低 | Tauri 2 用 WebView2 evergreen；关键处提供 `rgba()` 回退 |
| R2 | 单文件 802 行脚本按 id 绑定，结构改动易断 | **中** | 强制"id 不变"约束 + 每步跑 GUI 结构断言（现 11 条断言可扩） |
| R3 | 浅色默认改变首启观感，可能影响既有用户 | 低 | 提供"跟随系统"+ 记住上次主题 |
| R4 | 停靠引擎与现有 flex 布局冲突 | 中 | 分步上线，第 3 步可开关（`data-layout="legacy"` 回退） |
| R5 | 3D 绘图/图像类仍未渲染（gaps.md B6） | 中 | 本次不解决；Figure 面板先只承载 2D，标注已知限制 |
| R6 | 深色参考机的中文界面文案与 tl-desktop 英文混排 | 低 | 文案层统一走 i18n key（现有已有中文 placeholder，需一次梳理） |

## 5. 验收锚点（集成阶段用）

1. `cargo build -p tl-desktop` 通过，`cargo xtask layers` 不受影响（纯前端改动）。
2. 现有 11 条 GUI 结构断言全绿；新增断言：ribbon tab 数、面板数、主题切换后 `data-theme` 变化、Workspace 行数与 `vars` 输出一致。
3. 截图对照：与 `docs/ui/r2026a-reference/capture/` 的浅/深两张参考图并列（人工比对，不做自动打分）。
4. 功能不回归：Open/Save/Run/Stop/About、控制台历史、变量面板、图窗渲染全部手动过一遍。

## 6. 不做什么

- 不复制 MathWorks logo / 图标 / 专有字体 / 文案（宪法九）；原型图标全部为自绘通用符号。
- 不实现 Simulink / App Designer / Live Editor 的对应 UI。
- 不在本轮引入 Monaco 或第三方 UI 框架（保持零依赖、离线）。
- 不改 IPC 契约（`RunReport` / `StreamEntryPayload` tag 已冻结）。
