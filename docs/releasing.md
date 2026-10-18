# 发布流程（Releasing）

> 面向维护者。定义如何从源码产出一个可交付的 TensorLab 版本。参考 PhotoCraft 的 `docs/releasing.md` / `docs/release-playbook.md` 思路，裁剪为本项目实际。

## 一、版本号策略

- 语义化版本 `MAJOR.MINOR.PATCH`，工作区统一版本（`Cargo.toml` → `[workspace.package] version`）。
- v0.x 阶段：每个 MINOR 对应一个路线图里程碑（v0.1 / v0.2 / v0.3），PATCH 用于补丁。
- 版本号同时写入：[Cargo.toml](../Cargo.toml)、[CHANGELOG.md](../CHANGELOG.md)、[docs/releases/](releases/)。

```powershell
# 全工作区版本由 workspace 继承，只需改一处
# f:\软件开发\TensorLab\Cargo.toml
[workspace.package]
version = "0.1.0"
```

## 二、发布前检查清单

发布一个版本前，**全部必须通过**：

- [ ] `cargo fmt --check` 无差异
- [ ] `cargo check --workspace --all-targets` 零新增警告
- [ ] `cargo test --workspace` 全绿（v0.1 基线：94 passed）
- [ ] `cargo xtask layers` 通过（v0.1 基线：43 条依赖边）
- [ ] 5 个 `demos/*.m` 零错误运行
- [ ] `CHANGELOG.md` 已补上本版本条目
- [ ] `docs/releases/vX.Y.md` 已写发布说明
- [ ] 版本号已更新
- [ ] `cargo audit` 无新增告警
- [ ] （v0.2+）跨平台产物已构建并校验

### 一键跑法

```powershell
# 环境前缀（每终端一次）
$env:PATH = "D:\Program Files\Microsoft Visual Studio\2026\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"

cargo fmt --check
cargo xtask layers
cargo test --workspace
```

> 链接期内存不足时用 `cargo test -j 2`；详见 [development.md](development.md) 的环境坑。

## 三、构建发行产物

```powershell
# 全量 release 构建（thin LTO + codegen-units=1，耗时较长）
cargo build --release -p tl-cli -p tl-mcp-server -p tl-desktop
```

产物：

| 产物 | 路径 | 说明 |
|------|------|------|
| CLI | `target/release/tensorlab.exe` | REPL + `run` |
| MCP 服务器 | `target/release/tl-mcp-server.exe` | stdio JSON-RPC |
| 桌面端 | `target/release/tensorlab-desktop.exe` | Tauri 2（需 web 资源同目录） |

> Windows 上产物为 `.exe`；Linux/macOS 无后缀（v0.2 起支持跨平台）。

### 产物自检

```powershell
target\release\tensorlab.exe run demos\fft_demo.m
# 期待：打印脚本输出，退出码 0
```

## 四、冻结（Freeze）

当一个版本的**对外契约**（IPC serde 载荷、分层、命令名、控制台输出语义）确认稳定后，执行冻结：

1. 写 `docs/releases/vX.Y.md`，含 **🔒 冻结声明** 与冻结范围清单。
2. 在 `CHANGELOG.md` 的该版本条目加 `🔒 已冻结` 标记。
3. 冻结后，**契约变更必须走新版本**（破坏性变更记入下一 MINOR）。

> v0.1.0 已按此流程冻结，见 [releases/v0.1.md](releases/v0.1.md)。本仓库当前**无 git**，故冻结以文件标记落地，未打 tag；恢复 git 后应补 `git tag v0.1.0`。

## 五、发布说明模板

```markdown
# TensorLab vX.Y.0 — <一句话主题>

**发布日期**：YYYY-MM-DD
**状态**：🔒 已冻结 / 🚧 开发中

## 交付物
| 产物 | 说明 |

## 验收基线
| 检查 | 结果 |

## 功能范围
## 冻结范围（不可变更契约）
## 已知限制
```

## 六、回滚

若发布后发现阻塞缺陷：

1. 在 `CHANGELOG.md` 新增 PATCH 条目并在 `docs/releases/` 记录。
2. **不修改**已冻结版本的契约文件；修复以新 PATCH 版本发布。
3. 若产物不可用，直接删除 `target/release/` 内对应产物，并在发行说明标注，等待重构建。
