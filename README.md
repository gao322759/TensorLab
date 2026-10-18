# TensorLab

**MATLAB-compatible, Rust-powered** — 一个用 Rust 构建的 m-script 科学计算平台。

TensorLab 提供 m-script 解释器、命令行工具、Tauri 桌面应用与 MCP 服务器（面向 AI Agent），
四个入口共享同一个计算内核。内核采用 [RunMat 0.6.2](https://github.com/runmat-org/runmat)。

![Rust](https://img.shields.io/badge/Rust-2021-orange)
![Tauri](https://img.shields.io/badge/Tauri-2-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/license-MIT-green)

## 特性

- **m-script 兼容**：矩阵字面量、1-based 索引、`end`、隐式广播、控制流、函数文件、多输出、复数运算
- **1000+ 内置函数**（由 RunMat 内核提供）：线性代数、信号处理、统计、绘图等
- **5 个 MVP 工具箱**（纯 m-script，锚定 MATLAB R2026a）：Signal / Image / Optimization / Statistics & ML / Control
- **桌面端**：Tauri 2 + 自研编辑器（overlay 高亮）+ 变量面板 + 控制台 REPL + 自研 SVG 图窗渲染器（离线、无 CDN）
- **MCP-first**：`tl-mcp-server` 让 Claude / Cursor 等 AI Agent 直接运行 m 代码并取回图窗数据
- **MEX 加载桥**：`mex load <path>` 真实加载动态库并探测 `mexFunction` 入口
- **Everything is a command**：命令注册表让 GUI / CLI / MCP 共享同一语义

## 安装

### Windows（预编译）

从 [Releases](../../releases/latest) 下载：

- `tensorlab-0.2.1-windows-x64.msi` — 安装包
- `tensorlab-0.2.1-windows-x64.zip` — 便携包（解压后直接运行）

### 从源码构建

环境要求：Rust stable、VS Build Tools（Windows，含 CMake）、Node.js（可选，仅 GUI 测试）。

```bash
cargo build --release
```

产物：

```
target/release/tensorlab.exe            # CLI + REPL
target/release/tensorlab-desktop.exe    # 桌面端
target/release/tl-mcp-server.exe        # MCP 服务器
```

## 快速开始

```bash
# 运行 demo
cargo run -p tl-cli -- run demos/plot_demo.m

# REPL
cargo run -p tl-cli

# 桌面端
cargo build -p tl-desktop

# MCP 服务器（stdio）
cargo run -p tl-mcp-server
```

REPL 示例：

```matlab
>> A = magic(3);
>> x = A \ ones(3, 1);
>> vars
>> exit
```

## 演示脚本

| 脚本 | 内容 |
|------|------|
| `demos/hello_world.m` | 输出与字符串 |
| `demos/matrix_ops.m` | 矩阵运算 / inv / det / eig |
| `demos/plot_demo.m` | plot / subplot / legend / grid |
| `demos/fft_demo.m` | FFT 频谱分析 |
| `demos/solve_demo.m` | 线性方程组 `A\b` |
| `demos/signal_toolbox.m` | 信号工具箱：xcorr / dct / stft / czt / fir2 |
| `demos/image_toolbox.m` | 图像工具箱：imresize / edge / medfilt2 / bwlabel |
| `demos/stats_toolbox.m` | 统计工具箱：anova1 / mle / glmfit / nlinfit |
| `demos/optim_demo.m` | 优化工具箱：fmincon / lsqlin / intlinprog / ga / PSO |
| `demos/control_demo.m` | 控制工具箱：bode / margin / c2d / dlqr / kalman |

这些脚本同时是验收标准：`cargo test` 会逐个执行并断言关键输出。

## 架构

L0-L8 强制分层（`cargo xtask layers` 校验），RunMat 内核由 `tl-kernel` 单独封装：

```
apps/ (tl-cli, tl-desktop, tl-mcp-server)
  └─ L8 tl-commands → L7 tl-toolbox → L6 tl-mex → L5 tl-jit
     → L4 tl-interpreter → L3 tl-parser → L2 tl-runtime / tl-kernel
     → L1 tl-linalg → L0 tl-types ── RunMat 0.6.2
```

详见 [AGENTS.md](AGENTS.md)（项目宪法）与 [docs/architecture.md](docs/architecture.md)。

## 项目结构

```
apps/          CLI / 桌面端 / MCP 服务器
crates/        L0-L8 分层 crate
toolbox/       5 个 MVP 工具箱（纯 m-script）
demos/         验收用 m-script
docs/          开发文档 + 离线文档站（docs/site）
repro/         内核缺陷最小复现
bench/         TensorLab ↔ MATLAB 基准
packaging/     Windows/macOS/Linux 打包脚本
```

## 构建与测试

```bash
cargo check
cargo test --workspace --exclude tl-desktop -j 2
cargo xtask layers
node apps/tl-desktop/ui/tests/gui_smoke.mjs   # GUI 冒烟（需 Node）
```

打包：

```powershell
powershell -ExecutionPolicy Bypass -File packaging\build-windows.ps1 -Bundle
```

产物写入 `dist/`。详见 [packaging/README.md](packaging/README.md)。

## 文档

| 主题 | 文档 |
|------|------|
| 文档站（教程） | [docs/site/index.html](docs/site/index.html) |
| 文档索引 | [docs/README.md](docs/README.md) |
| 路线图 | [ROADMAP.md](ROADMAP.md) · [docs/roadmap.md](docs/roadmap.md) |
| 架构 / 决策 | [docs/architecture.md](docs/architecture.md) · [docs/decisions.md](docs/decisions.md) |
| 开发 / 测试 | [docs/development.md](docs/development.md) · [docs/testing.md](docs/testing.md) |
| CLI / MCP / 桌面端 | [docs/cli.md](docs/cli.md) · [docs/mcp.md](docs/mcp.md) · [docs/ui-desktop.md](docs/ui-desktop.md) |
| MATLAB 对标 | [docs/parity-checklist.md](docs/parity-checklist.md) · [docs/toolboxes/](docs/toolboxes/) |
| 已知差距 | [docs/gaps.md](docs/gaps.md) · [docs/kernel-notes.md](docs/kernel-notes.md) |
| 性能基准 | [docs/jit-benchmarks.md](docs/jit-benchmarks.md) |
| 变更日志 | [CHANGELOG.md](CHANGELOG.md) |

## 参与贡献

- 提交前请阅读 [docs/contributing.md](docs/contributing.md) 与 [AGENTS.md](AGENTS.md)；
- 任何改动必须保持 `cargo xtask layers` 通过；
- 新增内置函数需同时提供测试与文档。

## 许可

TensorLab 以 **MIT License** 发布，见 [LICENSE](LICENSE)。

第三方组件（RunMat、Tauri、HDF5 等）的许可与分发义务见 [docs/attribution.md](docs/attribution.md)。
RunMat 采用 Apache-2.0 / MIT 双许可，分发二进制时需随附其许可文本。
