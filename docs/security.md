# 安全策略（Security）

> 适用范围：TensorLab v0.1.x。本期为**本地科学计算工具**，不提供网络服务、不上传数据、不含遥测。

## 一、威胁模型

TensorLab 是一个**运行用户自己写的 m-script 的本地工具**，与 MATLAB 的信任模型一致：

| 信任边界 | 说明 |
|----------|------|
| 脚本内容 | **可信**。用户运行的脚本等同于本机代码，`system`/文件读写等能力不做沙箱隔离。 |
| 输入文件 | 半可信。`load`/`read_text_file` 读取的路径由用户显式选择。 |
| 进程边界 | MCP 服务器（stdio）与桌面端均以当前用户权限运行，无提权。 |
| 网络 | **无**。v0.1 不发起任何出站网络请求，不加载 CDN 资源（图窗为自研 SVG）。 |

**结论**：v0.1 的安全目标不是"防御恶意脚本"（这正是 MATLAB/解释器的设计前提），而是"**不引入额外攻击面、不泄露用户数据、依赖供应链可控**"。

## 二、已知攻击面

| 面 | 位置 | 风险 | 缓解 |
|----|------|------|------|
| MCP stdio 协议解析 | `apps/tl-mcp-server` | 畸形 JSON-RPC 导致 panic | 全部解析走 `serde_json::from_str` + `Result`，错误回 `-32700/-32600`，不 panic（见 [mcp.md](mcp.md)） |
| 未知方法 / 未知工具 | 同上 | 崩溃 | 返回 `-32601` 错误对象，连接保持（有 14 个 stdio 测试钉死） |
| 文件读写 | `apps/tl-desktop` `read_text_file`/`write_text_file` | 越权路径 | 路径只能由用户在系统对话框（`plugin:dialog`）中选择后传入，命令本身不做目录穿越扩展 |
| Tauri IPC | `apps/tl-desktop/src-tauri` | 未授权命令调用 | capabilities 白名单：仅 `core:default` + `dialog:*`，无 shell/fs 通配权限 |
| 依赖 | 全工作区 | 供应链漏洞 | `cargo audit`（见下）；依赖清单见 [attribution.md](attribution.md) |
| 大栈线程 | 三个 App | 栈溢出 | 64 MiB 固定栈（见 [decisions.md](decisions.md) ADR-006），非安全项但影响稳定性 |

## 三、依赖审计

```powershell
# 安装（一次性）
cargo install cargo-audit

# 扫描已知漏洞（RustSec 数据库）
cargo audit
```

- **基线**：v0.1.0 发行前执行过 `cargo audit`，无已知 RUSTSEC 告警（若有会记录于本文件"审计记录"节）。
- **静态链接组件**：`hdf5-metno`（`static` feature）会静态链接 HDF5 C 库，其许可与安全更新由上游维护；见 [attribution.md](attribution.md)。
- **许可合规**：`cargo deny`（如启用）用于检查许可白名单，非强制。

### 审计记录

| 日期 | 版本 | 工具 | 结果 |
|------|------|------|------|
| 2026-10-16 | v0.1.0 | `cargo audit` | 无已知告警 |

## 四、报告漏洞

由于本仓库当前**无公开托管与 issue 系统**（本地开发阶段），漏洞报告请直接联系项目维护者，或在恢复远程仓库后通过私有安全公告（GitHub Security Advisory）提交。

报告请包含：

1. 受影响版本 / commit
2. 复现步骤（最小脚本或 JSON-RPC 报文）
3. 期望行为 vs 实际行为
4. 影响评估（信息泄露 / 崩溃 / 越权）

我们会在确认后于 `CHANGELOG.md` 与本文件的"审计记录"中公开。

## 五、不做的事（明确声明）

- ❌ 不对用户脚本做沙箱隔离（与 MATLAB 一致；如需隔离请用容器/虚拟机）
- ❌ 不实现代码签名（v0.1 无分发渠道；跨平台打包在 v0.2）
- ❌ 不收集遥测、不联网
- ❌ 不在发布产物中内嵌任何密钥

## 六、未来（v0.2+）待办

- 若提供安装包：补充签名与校验和（SHA256）发布流程
- 若 MCP 服务器扩展到非 stdio 传输（HTTP/SSE）：补充认证与限流策略
- 引入 `cargo-deny` 到 CI 检查清单
