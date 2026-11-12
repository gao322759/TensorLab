# M3 Check 报告：原型 vs MATLAB R2026a 参考

> 检查方式：Clean-room 像素/控件比对 + 真实浏览器交互验证。
> 产物：`docs/ui/matlab-ui-proto/`（index.html + styles.css + app.js，纯静态、离线可开）。

## 1. 为什么不使用 `rebuild_check`

Craft Anything 的 `app_rebuild_check` / `rebuild_check` 需要**一个 project 文件夹 + 一个可运行的 rebuild target**（网站 vs 网站、程序 vs 程序）。
本任务是"外观范式研究 → 新写 UI shell"，target 是 MathWorks 桌面（我们既无源码也无重写意图），
因此走 **capture（取证）+ 浏览器实测（功能/像素）** 的组合：这既满足 clean-room，也比自动 likeness 打分更贴合实际目的。

## 2. 结构对齐检查（vs design-tokens.md 测量值）

| 项 | 参考测量 | 原型实现 | 结论 |
|----|---------|---------|------|
| 标题栏高 | 28px | 28px | 对齐 |
| Ribbon tab 行 | 26px | 26px | 对齐 |
| Ribbon 内容行 | 64px | 64px | 对齐（组标签 12px + 按钮 50px + 边距） |
| 地址栏 | 26px | 26px | 对齐 |
| 状态栏 | 22px | 22px | 对齐 |
| Dock tab 行 | 22px | 22px | 对齐 |
| 表格行高 | 18px | 18px | 对齐 |
| 编辑器行高 | 18px | 18px | 对齐 |
| gutter 宽 | 40px | 44px | 近似（容纳 4 位行号） |
| 左 dock 宽 | ~24% | 232px | 近似（1366 宽下 ≈17%，可调） |
| Ribbon tab 数 | 6（主页/绘图/APP/编辑器/发布/视图） | **4**（HOME/EDITOR/VIEW/HELP） | 有意的产品简化，见 §5 |
| 主题 | 浅色 + 深色（ribbon 深蓝、编辑器更深一档） | 双主题 + 三层深度，**浅色为默认** | 对齐 |

### 2.1 Ribbon 宽度行为（实测数字）

- HOME 的 5 个组（FILE/DATA/CODE/PLOT/ENVIRONMENT）总宽 **≈1023px**（含组内边距），
  视口 858px 时 ENVIRONMENT 落在可视区外，需要横向滚动。
- 这是 **MATLAB 原版同样存在的行为**：参考机 1542px 宽、6 个组时，最右的 "Add-Ons" 按钮同样被视口边缘截断
  （见 `capture/matlab-main-window-home-ribbon.png` 右侧）。
- 结论：ribbon 完整显示需要 **≥1030px 窗口宽**；低于该宽度横向滚动（已隐藏滚动条，可用 Shift+滚轮/触控板）。
  若未来需要在 1024 以下窗口内完整显示，可加"窄窗图标模式"（隐藏按钮文字），属集成阶段的可选项。

## 3. 交互检查（真实浏览器，零 console 错误）

| # | 行为 | 结果 |
|---|------|------|
| 1 | Ribbon tab 切换（4 个 tab：HOME/EDITOR/VIEW/HELP，组内容按测量替换） | PASS |
| 2 | Ribbon 下拉菜单（Layout / New） | **首轮 FAIL → 已修**（见 §4.1）→ 复检 PASS |
| 3 | 面板头双击折叠 / 拖拽换列 | PASS（拖拽落点高亮） |
| 4 | 编辑器多 tab、关闭、新建、点击定位 Ln/Col | PASS |
| 5 | 命令行窗口：`plot(...)` / `who` / `clear` / `clc` / `help` / 赋值 / ↑↓ 历史 | PASS |
| 6 | `plot(...)` → 停靠 Figure 面板（含 pan/zoom/cursor 工具条） | PASS |
| 7 | 浅/深主题切换（标题栏齿轮 + VIEW→主题 + Preferences→Colors 三入口） | PASS |
| 8 | Preferences 对话框（左树 + 搜索 + 内容面板 + 实时生效） | PASS |
| 9 | 布局预设（Default / Workspace bottom-left）双入口 | PASS |
| 10 | 全局搜索焦点（Ctrl+Shift+Space） | PASS |

## 4. 发现并修复的缺陷

### 4.1 Ribbon 下拉菜单立即关闭（阻断级）

- 现象：点击 Layout / New 后无菜单出现（DOM 中 `.popup` 数量始终为 0）。
- 根因：`document` 上的 "click outside" 监听在**同一次点击的冒泡阶段**执行，把刚创建的弹层删掉。
- 修复：ribbon 按钮的点击处理器先 `ev.stopPropagation()` 再打开菜单。
- 复检：菜单稳定显示，选择第二预设后 Workspace 从右列移到左列，状态栏同步。

### 4.2 Ribbon 组标签被裁切 + 横向滚动条外露

- 现象：组标签（DATA / CODE / ENVIRONMENT）底部被裁；ribbon 底部出现亮色横向滚动条（深色下尤其刺眼）。
- 根因：按钮文本换行使高度超过 44px，挤出 64px 的内容行；`overflow-x:auto` 暴露了滚动条。
- 修复：按钮固定 50px + 文本 2 行截断；组标签固定 12px；`scrollbar-width:none` + `::-webkit-scrollbar{height:0}`。

### 4.3 深色主题缺少"第三层深度"

- 现象：深色下编辑器与面板同色（#2b2b2b），失去测量到的"标题栏近黑 → 面板深灰 → 编辑器近黑"层次。
- 修复：新增 `--bg-editor`（浅色 #ffffff / 深色 #1e1e1e）并绑定到编辑器面板体；左侧工具条深色下提亮到 #b9b9b9。

### 4.4 图窗标题与实际命令不符

- 现象：原型固定渲染"two-tone spectrum"，但用户输入的是 `plot(sin(1:100))`，会误导读者以为真的求值了。
- 修复：图窗标题改为**原样显示用户命令**，横轴标注 `index (mock series)`，诚实声明是示意数据（真求值属集成阶段）。

## 5. 与原型的差异（有意保留）

| 项 | 参考行为 | 原型 | 原因 |
|----|---------|------|------|
| Ribbon tab 数 | 6 个（主页/绘图/APP/编辑器/发布/视图） | **4 个**（HOME/EDITOR/VIEW/HELP） | 产品决策（2026-11-11 用户裁定）：PLOT 并入 HOME；APPS/PUBLISH 无对应功能，空 tab 损害可信度 |
| Simulink 组 | HOME 含 SIMULINK 组 | 无 | AGENTS.md 明确不做 Simulink |
| 默认主题 | 浅色（参考机） | **浅色默认**，深色可切换 | 产品决策（2026-11-11 用户裁定） |
| 图窗数值 | 真实求值绘图 | 示意曲线 + 命令文本标题 | 原型无计算内核；真求值走 tl-desktop Engine |
| 中文界面 | 参考机为中文 | 英文占位 | 决策 3 |
| Figure ribbon | 独立顶层窗口 + 蓝色 ribbon | 停靠面板 + 面板头 | 原型简化为停靠形态；蓝色 ribbon 主题化记录在 tokens 以便集成 |
| 面板拖拽 | 支持分组、浮动、拆分 | 支持跨列 + 折叠 | 完整 docking 引擎属集成阶段工作量 |
| 高 DPI | 150% 独立密度 | 未单独适配 | 待补采（tokens §9） |

## 6. 复现命令

```powershell
# 静态服务（任选其一）
py -m http.server 8931 --directory docs/ui/matlab-ui-proto
# 打开
http://localhost:8931/index.html
```

无外部依赖、无 CDN、无构建步骤。
