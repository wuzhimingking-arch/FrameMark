$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()

$projectRoot = Split-Path -Parent $PSScriptRoot
$target = 'x86_64-pc-windows-msvc'
$version = (Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') -Encoding UTF8 | ConvertFrom-Json).version
$releaseDir = Join-Path $projectRoot "src-tauri\target\$target\release"
$bundle = Join-Path $releaseDir "bundle\nsis\帧印_${version}_x64-setup.exe"
$distDir = Join-Path $projectRoot 'artifacts\windows'
$env:XWIN_ARCH = 'x86_64'
$env:XWIN_CROSS_COMPILER = 'clang'
if (-not (Get-Command cargo-xwin.exe -ErrorAction SilentlyContinue)) { throw '缺少 cargo-xwin，请先执行 cargo install --locked cargo-xwin' }
$lldToolDir = Join-Path $env:LOCALAPPDATA 'frame-mark-build-tools'
New-Item -ItemType Directory -Force -Path $lldToolDir | Out-Null
if (-not (Get-Command lld-link.exe -ErrorAction SilentlyContinue)) {
    $ldLld = Get-Command ld.lld.exe -ErrorAction SilentlyContinue
    if (-not $ldLld) { throw '缺少 LLVM LLD 链接器' }
    Copy-Item -LiteralPath $ldLld.Source -Destination (Join-Path $lldToolDir 'lld-link.exe') -Force
    $env:PATH = "$lldToolDir;$env:PATH"
}
$msvcLib = Join-Path $env:LOCALAPPDATA 'cargo-xwin\windows-msvc-sysroot\windows-msvc-sysroot\lib\x86_64-unknown-windows-msvc'
if (-not (Test-Path -LiteralPath (Join-Path $msvcLib 'kernel32.lib'))) {
    & cargo xwin cache windows-msvc-sysroot
    if ($LASTEXITCODE -or -not (Test-Path -LiteralPath (Join-Path $msvcLib 'kernel32.lib'))) { throw 'cargo-xwin MSVC sysroot 准备失败' }
}
$env:LIB = $msvcLib
$llvmRc = Get-Command llvm-rc.exe -ErrorAction SilentlyContinue
if (-not $llvmRc) { throw '缺少 LLVM 资源编译器' }
$rcWrapper = Join-Path $lldToolDir 'rc.cmd'
Set-Content -LiteralPath $rcWrapper -Encoding ASCII -Value "@echo off`r`n`"$($llvmRc.Source)`" /C 65001 %*"
$env:RC = $rcWrapper

& (Join-Path $projectRoot 'scripts\prepare-assets.ps1')
if ($LASTEXITCODE) { throw '发行资源准备失败' }

Push-Location $projectRoot
try {
    & npm run tauri -- build --runner cargo-xwin --target $target
    if ($LASTEXITCODE) { throw 'Tauri Windows MSVC 构建失败' }
} finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $bundle)) { throw "安装包未生成：$bundle" }
New-Item -ItemType Directory -Force -Path $distDir | Out-Null
Copy-Item -LiteralPath $bundle -Destination $distDir -Force
Write-Host "Windows 安装包：$(Join-Path $distDir (Split-Path -Leaf $bundle))"
