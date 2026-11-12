# MATLAB R2026a 风格 UI 重建计划（Craft Anything / Clean-room）

> 状态：**M0-M4 全部完成（2026-11-11）；M5 tl-desktop 集成实施完成（2026-11-11）**
> M5 产物：`apps/tl-desktop/ui/index.html` 换皮（浅色默认/深色）+ 换骨架（titlebar / 4-tab ribbon / pathbar / statusbar）
> + 停靠（折叠/跨列拖拽/分隔条）+ Preferences 抽屉 + 行号 gutter；GUI smoke 15/15 PASS。
> 产物：
> - 证据与测量：[r2026a-reference/design-tokens.md](r2026a-reference/design-tokens.md) + `r2026a-reference/capture/`（6 张参考截图 + 3 张原型截图）
> - 可运行原型：[matlab-ui-proto/index.html](matlab-ui-proto/index.html)（+ styles.css / app.js，零依赖离线可开）
> - 检查报告：[r2026a-reference/check-report.md](r2026a-reference/check-report.md)（10 项交互全 PASS，修复 4 个缺陷）
> - 集成评估：[integration-assessment.md](integration-assessment.md)（三步路径 S/M/M，3 个产品决策：2 项已裁定、1 项待确认）
> - 集成实施：`apps/tl-desktop/ui/index.html` + `ui/tests/gui_smoke.mjs`（15 条断言）
> 关键实测：MATLAB R2026a 桌面为 **CEF（Chromium）渲染**——与 tl-desktop 的 Tauri WebView 同栈，范式可移植性高。
> 决策记录：coding agent = 本项目 agent 直接驱动（不连外部 Claude Code/Codex）；浅色先行；英文占位；主题基线浅色。
> 用户裁定（2026-11-11）：**浅色为默认主题**；**Ribbon 精简为 4 个 tab**（HOME/EDITOR/VIEW/HELP，PLOT 并入 HOME，去掉 Simulink / APPS / PUBLISH）。
> 全程 **Clean-room**（`core_session_show` 确认 `clean_room=true`，未使用 `--deep`，未读任何 MathWorks 源码/资源）。
> 工具：Craft Anything 0.2.x（`F:\Tools\craft-anything`，Rust 逆向/重建引擎，Study → Build → Check → Refine）
> 目标：为 TensorLab `tl-desktop`（Tauri 2.0 + WebView）产出 MATLAB R2026a 风格的 UI 范式参考与可集成前端资产
> 合规：**全程 Clean-room 模式**，映射到 TensorLab 宪法九（不读 MathWorks 内部资源，仅研究可见外观范式）
> 输入文档：本文档；上游：`docs/decisions.md` ADR-004/005（A1 编辑器自研、A2 SVG 渲染器的字面偏离）

## 1. 目标与定位

用 Craft Anything 的 clean-room 工作流研究本机 MATLAB R2026a 的**桌面 UI 外观范式**，产出：

1. **参考规范**：主窗口布局、面板停靠系统、编辑器外观、Command Window、Workspace 表格、图窗窗口、深浅主题的设计 token（间距/颜色/字体/层级）——落盘 `docs/ui/r2026a-reference/`；
2. **静态原型**：纯 HTML/CSS/JS 的 MATLAB 风格 UI shell（mock 数据驱动，无后端），经 Craft Anything Check 与截图对比收敛；
3. **集成评估**：原型组件移植进 `tl-desktop` 前端的清单与工作量估算（实际改造 tl-desktop 是后续独立任务，不在本计划内）。

**为什么用 Craft Anything**：它的 clean-room 模式（不读目标内部，仅从可见外观取证）与宪法九"不参考 MathWorks 源码/内部资源"**语义完全对齐**；Check 步骤的三宽度对比能客观量化"像不像"；全部工具暴露为 CLI/MCP，产出是带置信度的证据记录而非猜测。

## 2. 环境现状（已实测）

| 项 | 状态 | 动作 |
|----|------|------|
| craft-anything-cli doctor | **healthy=true**（CLI ok、MCP self-test 30 tools、Chromium ok、Java ok） | 无 |
| 本机 MATLAB R2026a | **已安装**（`C:\Program Files\MATLAB\R2026a`） | 研究目标就绪 |
| AI/coding agent | **未连接**（doctor: "No AI backend is set up"；Codex installed but not connected） | `craft-anything-cli setup --client codex` 或连接 Copilot CLI（ACP） |
| Ghidra/adb 等 Deep 引擎 | 未装（可选） | **不需要**（全程 Clean room） |

**前置缺口**：Build 步骤需要本机 coding agent。Claude Code 是首选但未装；本机有 Codex（README 标注"适配器未在真实安装上验证过"）与 GitHub Copilot CLI（ACP 通道）。M0 先连接并冒烟验证，两者至少一个可用才进入 M1。

## 3. 合规护栏（宪法九红线映射，强制）

| 红线 | 执行方式 |
|------|----------|
| 不读 MathWorks 源码/内部资源 | Craft Anything **全程 Clean-room 模式**（默认；绝不 `--deep`）；研究手段仅限：本机 MATLAB 窗口截图（用户自己的许可证副本）、可见文本、测量的布局/颜色/字体/间距、菜单展开行为 |
| 不抄 logo/图标/品牌文本 | Craft Anything 内置规则：agent 被告知不复制 logo/照片/文本，除非确认拥有——重建物中 MATLAB 图标、名称、专有字体（JetBrains Mono 等开源字体替代）一律替换为 TensorLab 品牌 |
| 产物可追溯 | 证据落盘 `capture/`（窗口截图/测量）与 `investigation.cany/`（证据链），布局范式源自测量数据而非逆向代码 |
| 不覆盖 MATLAB 功能语义 | 只研究 UI 外观范式；不提取 MATLAB 内置函数行为/文档原文 |

## 4. 研究范围（MATLAB R2026a 桌面 UI）

**做**：主窗口三区布局（左 Current Folder / 中 Editor+Command Window / 右 Workspace）、功能区 ribbon 与工具条、面板停靠/浮动/分组范式、编辑器外观（行号/断点圆点/语法高亮配色/分屏/Tab 栏）、Command Window 提示符与输出排版、Workspace 变量表格、图窗窗口外观（工具条/轴框）、Preferences 对话框、深色/浅色主题、缩放行为。

**不做**：Simulink、App Designer、Live Editor 的富文本语义、任何内核行为、安装器/许可证 UI、MATLAB Online。

## 5. 里程碑

| 阶段 | 内容 | 验收 |
|------|------|------|
| **M0 环境就绪** | 连接 coding agent（`setup --client codex` / Copilot CLI）→ `doctor` 复验 → 用 `craft-anything-cli rebuild - "一个简单的三面板布局 HTML 页" --rounds 1` 冒烟验证 Build 通路 | doctor 全绿 + 冒烟产物可打开 |
| **M1 Study** | Clean-room 研究本机 MATLAB：打开主窗口/编辑器/图窗/Preferences，采集浅色+深色两主题的截图与布局测量；证据落盘 `capture/` | `docs/ui/r2026a-reference/design-tokens.md`（颜色/字体/间距/布局测量表）+ 截图集 |
| **M2 Build** | `rebuild` 生成**静态 HTML/CSS/JS 原型**（栈明确指定 web/HTML-CSS；mock 编辑器文本、mock Workspace 表、mock Command Window 输出；含停靠面板交互） | 原型可离线打开，含深浅主题切换 |
| **M3 Check** | Craft Anything Check 对比三宽度截图 → 差异清单 → refine 迭代 ≤3 轮 | 相似度收敛（差异清单仅剩品牌替换项） |
| **M4 集成评估** | 组件清单（CSS token/布局 shell/面板停靠 JS）+ `tl-desktop` Tauri WebView 集成方案与工作量估算，写入 `docs/ui/integration-assessment.md` | 评估文档评审通过（tl-desktop 实际改造另立任务） |

预计总量：M0 半天、M1-M3 各 1-2 个会话、M4 半天。

## 6. 产出物清单

```
docs/ui/r2026a-reference/     M1 证据与设计 token（人写整理）
docs/ui/matlab-ui-proto/      M2-M3 原型项目（Craft Anything project folder）
docs/ui/integration-assessment.md  M4 集成评估
tests/ui/                     原型截图对比记录（可选）
```

## 7. 风险与对策

| # | 风险 | 对策 |
|---|------|------|
| R1 | Craft Anything 处于 **early alpha**；app 型目标的 Study/Check 能力"partly there"（README 自述） | 随时人工兜底：截图 + 手工测量补充设计 token；Check 差异清单人工复核 |
| R2 | coding agent 适配器未验证（Codex "not verified on a real install"） | M0 冒烟先行；Codex 不行则切 Copilot CLI（ACP）；都不行则 M1 证据仍可用（Study 不需要 agent），M2 改由我直接实现 |
| R3 | MATLAB 窗口截图需要运行中的 GUI 会话 | 本机已装 R2026a，用户交互打开窗口后 Craft Anything 采集；Docked 面板布局用默认布局即可 |
| R4 | 产物被误认为 MathWorks 素材 | 强制品牌替换（TensorLab 图标/名称）；原型标注 "UI paradigm study, not affiliated" |
| R5 | tl-desktop 当前是 egui 之外的 Tauri+WebView —— 产出栈天然匹配（HTML/CSS/JS） | 低风险；M4 评估确认即可 |

## 8. 待确认（已全部裁定）

1. coding agent 选择 → **本项目 agent 直接驱动**（2026-11-11）。
2. 原型主题基线 → **浅色先行，深色作为 refine 项**；产品默认浅色（2026-11-11）。
3. 原型语言 → **英文占位**（2026-11-11）。
4. M4 之后是否立即启动 tl-desktop 前端改造 → **是**，作为 M5 已实施完成（2026-11-11，三步全落地，15/15 GUI 断言 PASS）。
5. Ribbon tab 数 → **精简 4 个**（HOME/EDITOR/VIEW/HELP）（2026-11-11）。
