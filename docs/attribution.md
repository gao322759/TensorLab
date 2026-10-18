# 第三方归属与许可

TensorLab 自身以 MIT 许可发布（见根目录 [LICENSE](../LICENSE)）。本文件列出**直接依赖中影响分发的重要组件**
及其许可要求。

> ⚠️ 发布二进制前请复核：本表随依赖变化需更新。可用 `cargo tree` / `cargo license` 生成完整清单。

## 一、核心运行时依赖

| 组件 | 版本 | 许可 | 说明与义务 |
|---|---|---|---|
| **RunMat**（`runmat-core` / `runmat-runtime` / `runmat-value`，[runmat-org/runmat](https://github.com/runmat-org/runmat)） | 0.6.2 | **Apache-2.0 / MIT** 双许可 | **m-script 运行时本体**。分发需保留版权与许可声明（Apache-2.0 还要求保留 `NOTICE` 文件如有）。这是本项目最关键的第三方组件，必须随发行物提供其许可文本。兼容目标声明为 R2026a。 |
| Tauri | 2.x | Apache-2.0 / MIT | 桌面端外壳（WebView + IPC） |
| `tauri-plugin-dialog` | 2.x | Apache-2.0 / MIT | 原生打开/保存对话框 |
| `tauri-plugin-fs` | 2.x | Apache-2.0 / MIT | 由 dialog 插件间接引入（本项目不直接使用其 scope 权限） |
| `inventory` | 0.3 | MIT / Apache-2.0 | 编译期命令注册表（PhotoCraft 哲学的实现手段） |
| `libloading` | 0.8 | ISC | `tl-mex` 的动态库加载（`mex load`） |

## 二、构建期依赖（不进入发行物，但影响构建环境）

| 组件 | 许可 | 说明 |
|---|---|---|
| **HDF5**（经 `hdf5-metno` + `hdf5-metno-src` 源码构建） | HDF5 License（BSD 风格，允许再分发） | RunMat 在 Windows 上的非可选依赖。本项目启用 `features = ["static"]`，**静态链接进二进制**，因此分发时需随附 HDF5 许可声明。 |
| Rust 标准库 | MIT / Apache-2.0 | |
| Cranelift（经 RunMat） | Apache-2.0 WITH LLVM-exception | JIT 后端 |
| Visual Studio / MSVC 运行库 | Microsoft 许可 | 动态链接（默认），分发需符合其再分发条款 |

## 三、设计参考（无代码衍生，仅理念与公开文档）

| 来源 | 使用方式 | 义务 |
|---|---|---|
| **PhotoCraft**（[storytold/photocraft](https://github.com/storytold/photocraft), Apache-2.0） | **仅借鉴**：项目宪法结构（`AGENTS.md`）、"Everything is a command" 哲学、`cargo xtask layers` 式的分层强制、文档目录组织。**未复制其代码**。 | 无（无代码衍生） |
| MATLAB 公开文档 | 语义描述、函数签名、行为对标（clean-room 依据：仅公开 API 与语义描述） | 无（外部 API 兼容属正当行为） |
| LAPACK / NumPy 等公开论文与教科书 | `tl-linalg` 的 LU 分解等算法依据 | 无 |

### clean-room 红线（[AGENTS.md](../AGENTS.md) §九）

- ✅ 可参考：MATLAB 公开文档中的语义描述、教科书算法（LAPACK/NumPy 论文）、通用函数名
- ❌ 不可参考：MathWorks 源码、其测试数据文件、文档原文摘抄

## 四、发行物需要包含的内容

发布二进制时，至少需提供：

1. 本项目的 `LICENSE`（MIT）
2. **RunMat 的 Apache-2.0 / MIT 许可文本**（及其 `NOTICE`，如上游提供）
3. **HDF5 的许可文本**（因静态链接）
4. 本文件（`docs/attribution.md`）或等价的第三方声明清单
5. 完整依赖清单（建议用 `cargo metadata` / `cargo about` 生成）

> v0.1 为源码发布为主：使用者自行 `cargo build`，依赖许可由 `Cargo.lock` 追溯。
> **v0.2 开始分发安装包时，必须补齐上述 1-5 项。**（见 [gaps.md](gaps.md) B7）
