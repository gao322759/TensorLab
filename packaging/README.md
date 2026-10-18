# TensorLab 打包（Packaging）

> v0.2 交付物：Windows / macOS / Linux 的可分发产物。
> 本目录提供**可直接运行**的构建脚本；CI 接入前先在本地/构建机上执行。

## 一、产物

| 平台 | 产物 | 生成方式 |
|------|------|----------|
| Windows | `dist/tensorlab-0.2.1-windows-x64.zip`（便携版：3 个 exe + `toolbox/` + `demos/` + 文档 + `ATTRIBUTION.md`） | `packaging\build-windows.ps1` |
| Windows | `dist/tensorlab-0.2.1-windows-x64.msi`（MSI 安装包，约 84 MiB；同时保留在 `target/release/bundle/msi/`） | 同上，加 `-Bundle` |
| macOS | `dist/tensorlab-0.2.1-macos-<arch>.tar.gz`、`bundle/dmg/*.dmg` | `packaging/build-unix.sh`（在 macOS 上） |
| Linux | `dist/tensorlab-0.2.1-linux-x86_64.tar.gz`、`bundle/appimage/*.AppImage`、`bundle/deb/*.deb` | `packaging/build-unix.sh`（在 Linux 上） |

> 跨平台产物**必须在对应平台上构建**（Tauri 不支持交叉编译安装包）。

## 二、前置条件

- Rust stable（MSVC on Windows / GNU on Linux）
- Windows：VS 2026 自带 CMake（见 [docs/development.md](../docs/development.md) 坑 2）
- 安装包（可选）：`cargo install tauri-cli --version "^2"`；MSI 还需要 WiX 3.14
  （Tauri 首次构建会自动下载；若 GitHub 超时，见下方“WiX 预置”）
- Linux 额外：`libwebkit2gtk-4.1-dev`、`libgtk-3-dev`、`libayatana-appindicator3-dev`、`librsvg2-dev`
- macOS 额外：Xcode Command Line Tools

## 三、Windows

```powershell
# 便携版（推荐先跑这个，最快可用）
powershell -ExecutionPolicy Bypass -File packaging\build-windows.ps1

# 便携版 + MSI/NSIS 安装包
powershell -ExecutionPolicy Bypass -File packaging\build-windows.ps1 -Bundle
```

脚本依次执行：`cargo fmt --check` → `cargo xtask layers`
→ `cargo test --workspace --exclude tl-desktop -j 2`（GUI smoke 直接跑 Node，避免编译 Tauri）
→ `cargo build --release` → 组装 `dist/` → 压缩。

## 四、macOS / Linux

```bash
./packaging/build-unix.sh            # 便携版
./packaging/build-unix.sh --bundle   # 便携版 + DMG/AppImage/deb
```

## 五、WiX 预置（GitHub 下载超时时）

Tauri 的 MSI bundler 首次运行会从 GitHub 下载 WiX 3.14 到
`%LOCALAPPDATA%\tauri\WixTools314`。若网络访问 GitHub 超时，可手动预置并通过 SHA256 校验：

```powershell
$zip = "$env:TEMP\wix314-binaries.zip"
curl.exe -L --fail --max-time 300 -o $zip `
  "https://gh-proxy.com/https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip"
# 期望 SHA256：6ac824e1642d6f7277d0ed7ea09411a508f6116ba6fae0aa5f2c7daa2ff43d31
(Get-FileHash $zip -Algorithm SHA256).Hash

$target = "$env:LOCALAPPDATA\tauri\WixTools314"
if (Test-Path $target) { Remove-Item $target -Recurse -Force }
New-Item -ItemType Directory -Path $target -Force | Out-Null
Expand-Archive -Path $zip -DestinationPath $target -Force
```

预置后重跑 `cargo tauri build --bundles msi` 即可；2026-10-17 在本机实测产出
`TensorLab_0.2.1_x64_en-US.msi`（88,510,464 bytes）。

## 六、发行前检查（Release Checklist）

- [ ] 便携包内含 `ATTRIBUTION.md`（RunMat 为 Apache-2.0/MIT，**必须随发行物**，见 [docs/attribution.md](../docs/attribution.md)）
- [ ] `toolbox/` 随包分发（否则工具箱函数不可用；`Engine` 找不到目录时会静默跳过）
- [ ] 冒烟：`tensorlab run demos/fft_demo.m`、`tl-mcp-server` 的 `initialize` 握手
- [ ] 记录产物体积到 `docs/releases/vX.Y.md`

## 七、已知限制

- release 二进制约 141–149 MB（内核 + Tauri 静态链接），远超 `< 30 MB` 目标 → v0.3 专项优化
- 未做代码签名 / 公证（v0.2 无分发渠道）；Windows SmartScreen 会提示未知发布者
- 便携版不含安装器：需手动解压并双击 `tensorlab-desktop.exe`
