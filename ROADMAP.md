# TensorLab 路线图（ROADMAP）

> **MATLAB-compatible, Rust-powered.**
> 完整版本目标见 [docs/roadmap.md](docs/roadmap.md)。本文件为根目录速览。

## 版本里程碑

| 版本 | 状态 | 主题 | 验收标准 |
|------|------|------|----------|
| **v0.1.0** | 🔒 已冻结（2026-10-16） | 最小可演示 | 桌面端 + 编辑器 + 变量面板 + 解释器 + 绘图 + 5 demo + CLI + MCP；`cargo test` 全绿（94 测试） |
| **v0.2.0** | 🔒 已冻结（2026-10-17） | 5 个 MVP 工具箱 + MEX 加载桥 + GUI 自动化测试 | 每个工具箱 20–30 函数对齐 **MATLAB R2026a**；`mex load` 可加载/探测；`cargo test` 102 全绿；10 demo；安装包不在本次范围 |
| **v0.3** | 规划 | JIT 启用 + 性能达标 | Cranelift JIT 路径启用；关键场景达 MATLAB 0.5x（TensorLab 耗时 ≤ 2× MATLAB）；文档站 |

## 当前阶段：v0.2

**锚定 MATLAB R2026a** 的函数签名与语义。

### 五个工具箱（各 20–30 核心函数）

| 工具箱 | 清单 |
|--------|------|
| 信号处理 | [docs/toolboxes/signal.md](docs/toolboxes/signal.md) |
| 图像处理 | [docs/toolboxes/image.md](docs/toolboxes/image.md) |
| 优化 | [docs/toolboxes/optimization.md](docs/toolboxes/optimization.md) |
| 统计与机器学习 | [docs/toolboxes/statistics-ml.md](docs/toolboxes/statistics-ml.md) |
| 控制系统 | [docs/toolboxes/control.md](docs/toolboxes/control.md) |

**实测起点**：对 246 个 R2026a 目标函数探测，内核已提供 **109（44%）**，缺口 137；
缺口以纯 m-script 工具箱文件交付（[ADR-012](docs/decisions.md)）。

### 其他 v0.2 交付

- **MEX 桥**：`tl-mex` 已由空桩升级为动态库加载/入口探测桥（`mex load <path>`）；
  透明 m 代码调用待内核提供函数注册 API（见 [docs/gaps.md](docs/gaps.md) K12）
- **跨平台打包**：脚本见 [packaging/](packaging/)（Windows 便携/安装包；macOS/Linux 在对应平台构建）
- **变量编辑器**：突破 16 元素预览上限

## 长期数字目标（v0.3）

| 指标 | 目标 |
|------|------|
| m-script 核心语法兼容 | 70–80% |
| 内置函数覆盖 | 150+ |
| 性能 | MATLAB 0.5x（TensorLab 耗时 ≤ 2× MATLAB；关键场景） |
| 启动时间 | < 500ms |
| 包体 | < 30MB |

## 明确不做（v1.0 再说）

❌ Simulink · ❌ 完整 OOP（`classdef` 全特性）· ❌ Live Script · ❌ App Designer · ❌ 移动端 · ❌ 发明新语言（只解释 m-script）

## 相关文档

- 项目宪法：[AGENTS.md](AGENTS.md)
- 架构：[docs/architecture.md](docs/architecture.md)
- 决策记录：[docs/decisions.md](docs/decisions.md)
- 发布流程：[docs/releasing.md](docs/releasing.md)
- 版本冻结快照：[docs/releases/](docs/releases/)
- 变更日志：[CHANGELOG.md](CHANGELOG.md)
