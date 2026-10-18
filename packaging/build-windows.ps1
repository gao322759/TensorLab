# TensorLab — Windows 打包脚本
# 用法（在仓库根执行）：
#   powershell -ExecutionPolicy Bypass -File packaging\build-windows.ps1
#   powershell -ExecutionPolicy Bypass -File packaging\build-windows.ps1 -Bundle
param(
    [switch]$Bundle
)

$ErrorActionPreference = "Stop"

# VS 2026 自带 CMake（系统 CMake 不识别 VS18 生成器）+ rustup bin
$env:PATH = "D:\Program Files\Microsoft Visual Studio\2026\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"

$root = Split-Path -Parent $PSScriptRoot
$version = "0.2.1"
Push-Location $root
try {
    Write-Host "== 1/6 fmt =="
    cargo fmt --check
    if ($LASTEXITCODE -ne 0) { throw "cargo fmt --check failed" }

    Write-Host "== 2/6 layers =="
    cargo xtask layers
    if ($LASTEXITCODE -ne 0) { throw "layering check failed" }

    Write-Host "== 3/6 tests (exclude tl-desktop; GUI smoke runs via Node) =="
    cargo test --workspace --exclude tl-desktop -j 2
    if ($LASTEXITCODE -ne 0) { throw "tests failed" }
    if (Get-Command node -ErrorAction SilentlyContinue) {
        node "apps\tl-desktop\ui\tests\gui_smoke.mjs"
        if ($LASTEXITCODE -ne 0) { throw "GUI smoke test failed" }
    } else {
        Write-Warning "node 未安装，跳过 GUI smoke；安装 Node 后可补跑"
    }

    Write-Host "== 4/6 release build =="
    cargo build --release -p tl-cli -p tl-mcp-server -p tl-desktop
    if ($LASTEXITCODE -ne 0) { throw "release build failed" }

    Write-Host "== 5/6 assemble portable package =="
    $out = Join-Path $root "dist\tensorlab-$version-windows-x64"
    if (Test-Path $out) { Remove-Item $out -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $out | Out-Null
    Copy-Item "target\release\tensorlab.exe" $out -Force
    Copy-Item "target\release\tl-mcp-server.exe" $out -Force
    Copy-Item "target\release\tensorlab-desktop.exe" $out -Force
    Copy-Item "toolbox" (Join-Path $out "toolbox") -Recurse -Force
    Copy-Item "demos" (Join-Path $out "demos") -Recurse -Force
    Copy-Item "README.md", "CHANGELOG.md", "ROADMAP.md" $out -Force
    Copy-Item "docs\attribution.md" (Join-Path $out "ATTRIBUTION.md") -Force
    $zip = "$out.zip"
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Compress-Archive -Path "$out\*" -DestinationPath $zip -Force
    Write-Host "portable package -> $zip"

    if ($Bundle) {
        Write-Host "== 6/6 tauri installers =="
        if (-not (Get-Command cargo-tauri -ErrorAction SilentlyContinue)) {
            Write-Warning "cargo-tauri 未安装，跳过安装包：cargo install tauri-cli --version '^2'"
        } else {
            Push-Location "apps\tl-desktop"
            try {
                cargo tauri build --bundles msi
                if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }
            } finally {
                Pop-Location
            }
            Write-Host "installer (MSI) -> target\release\bundle\msi\"
            $msi = Join-Path $root "target\release\bundle\msi\TensorLab_${version}_x64_en-US.msi"
            if (Test-Path $msi) {
                Copy-Item $msi (Join-Path $root "dist\tensorlab-$version-windows-x64.msi") -Force
                Write-Host "MSI copied -> dist\tensorlab-$version-windows-x64.msi"
            }
        }
    } else {
        Write-Host "== 6/6 跳过安装包（加 -Bundle 生成 MSI/NSIS）=="
    }
} finally {
    Pop-Location
}
