# TensorLab 文档索引

> 本目录是 TensorLab 的开发文档。项目宪法（不可违反的规则）在仓库根的 [AGENTS.md](../AGENTS.md)。

## 上手

| 文档 | 内容 |
|---|---|
| [文档站](site/index.html) | 离线 HTML 教程站：快速上手 / m-script 基础 / 绘图与桌面端 / MCP 接入 |
| [architecture.md](architecture.md) | 分层架构（L0-L8）、数据流、IPC 契约、命令注册表 |
| [development.md](development.md) | 环境搭建、构建、调试、工具链（Windows/MSVC） |
| [testing.md](testing.md) | 测试策略与 122 个测试的组织方式 |
| [contributing.md](contributing.md) | 贡献流程与检查清单 |

## 使用

| 文档 | 内容 |
|---|---|
| [cli.md](cli.md) | `tensorlab` 命令行参考（run / REPL / 退出码） |
| [mcp.md](mcp.md) | MCP stdio 服务器参考（握手、5 个工具、协议细节） |
| [host-commands.md](host-commands.md) | 宿主命令注册表（help / vars / clear / figures / version） |
| [ui-desktop.md](ui-desktop.md) | 桌面端 UI 结构与前端渲染器 |
| [language-support.md](language-support.md) | m-script 语法与内置函数支持范围 |

## 兼容性与规划

| 文档 | 内容 |
|---|---|
| [parity-checklist.md](parity-checklist.md) | MATLAB R2026a 兼容性对标清单 |
| [gaps.md](gaps.md) | 已知差距与限制（v0.1 → v0.2） |
| [roadmap.md](roadmap.md) | 版本路线图（v0.1 / v0.2 / v0.3） |
| [toolboxes/](toolboxes/) | v0.2 五个 MVP 工具箱的功能清单（锚定 R2026a） |
| [../packaging/README.md](../packaging/README.md) | 跨平台打包（Windows/macOS/Linux 产物与脚本） |

## 决策与发布

| 文档 | 内容 |
|---|---|
| [decisions.md](decisions.md) | 架构决策记录（ADR） |
| [releasing.md](releasing.md) | 发布流程与冻结操作 |
| [releases/v0.1.md](releases/v0.1.md) | 🔒 v0.1.0 发布说明与冻结声明 |
| [releases/v0.2.1.md](releases/v0.2.1.md) | ✅ v0.2.1 发行说明（Windows MSI / 便携 ZIP） |
| [releases/v0.2.md](releases/v0.2.md) | 🔒 v0.2.0 发布说明与冻结声明 |
| [../CHANGELOG.md](../CHANGELOG.md) | 变更日志 |

## 内核

| 文档 | 内容 |
|---|---|
| [kernel-notes.md](kernel-notes.md) | RunMat 0.6.2 嵌入 API 笔记（含 12 个已验证的坑）★ 改内核相关代码前必读 |

## 法务

| 文档 | 内容 |
|---|---|
| [attribution.md](attribution.md) | 第三方组件与许可归属 |
| [security.md](security.md) | 安全模型（本地代码执行边界） |
