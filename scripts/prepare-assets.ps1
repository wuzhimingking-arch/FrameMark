$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()

function Get-Sha256([string]$FilePath) {
    $stream = [System.IO.File]::OpenRead($FilePath)
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try { [System.BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '') }
    finally { $stream.Dispose(); $algorithm.Dispose() }
}

$projectRoot = Split-Path -Parent $PSScriptRoot
$assetTemp = Join-Path ([System.IO.Path]::GetTempPath()) 'framemark-release-assets'
$binaryDir = Join-Path $projectRoot 'src-tauri\binaries'
$resourceDir = Join-Path $projectRoot 'src-tauri\resources'
$target = 'x86_64-pc-windows-msvc'
$manifestPath = Join-Path $projectRoot 'docs\compliance\ffmpeg-selfbuild\binaries.json'
if (-not (Test-Path $manifestPath)) { throw '缺少自建 FFmpeg 发布清单，请先完成 scripts/build-ffmpeg.sh 及二进制审计' }
$manifest = Get-Content -Raw -Encoding UTF8 $manifestPath | ConvertFrom-Json
$expectedHashes = @{ ffmpeg = $manifest.ffmpeg; ffprobe = $manifest.ffprobe }

New-Item -ItemType Directory -Force -Path $assetTemp, $binaryDir, $resourceDir | Out-Null

$ffmpegTarget = Join-Path $binaryDir "ffmpeg-$target.exe"
$ffprobeTarget = Join-Path $binaryDir "ffprobe-$target.exe"
$ffmpegDir = Join-Path $assetTemp 'ffmpeg-selfbuild'
$matchesPinned = (Test-Path $ffmpegTarget) -and (Test-Path $ffprobeTarget)
if ($matchesPinned) {
    $matchesPinned = ((Get-Sha256 $ffmpegTarget) -eq $expectedHashes.ffmpeg) -and ((Get-Sha256 $ffprobeTarget) -eq $expectedHashes.ffprobe)
}
if (-not $matchesPinned) {
    if ($env:FRAMEMARK_FFMPEG_DIR) {
        $ffmpegDir = $env:FRAMEMARK_FFMPEG_DIR
    } else {
        $ffmpegZip = Join-Path $assetTemp 'ffmpeg-selfbuild.zip'
        curl.exe -L --fail --retry 5 -o $ffmpegZip $manifest.url
        if ($LASTEXITCODE) { throw '自建 FFmpeg 下载失败' }
        Expand-Archive -LiteralPath $ffmpegZip -DestinationPath $ffmpegDir -Force
    }
    foreach ($name in 'ffmpeg', 'ffprobe') {
        $source = Get-ChildItem -LiteralPath $ffmpegDir -Recurse -Filter "$name.exe" | Select-Object -First 1
        if (-not $source) { throw "FFmpeg 发行包缺少 $name.exe" }
        Copy-Item -LiteralPath $source.FullName -Destination (Join-Path $binaryDir "$name-$target.exe") -Force
        if ((Get-Sha256 (Join-Path $binaryDir "$name-$target.exe")) -ne $expectedHashes[$name]) { throw "FFmpeg 固定版本校验失败：$name" }
    }
}

$licenseDir = Join-Path $resourceDir 'licenses'
New-Item -ItemType Directory -Force -Path $licenseDir | Out-Null
$ffmpegLicenseTarget = Join-Path $licenseDir 'FFmpeg-GPLv3.txt'
if (-not (Test-Path $ffmpegLicenseTarget)) {
    $ffmpegLicense = Get-ChildItem -LiteralPath $ffmpegDir -Recurse -Filter 'LICENSE' | Select-Object -First 1
    if (-not $ffmpegLicense) { throw 'FFmpeg 发行包缺少 GPL 许可证文本' }
    Copy-Item -LiteralPath $ffmpegLicense.FullName -Destination $ffmpegLicenseTarget -Force
}

$font = Join-Path $resourceDir 'NotoSansSC-Regular.ttf'
if (-not (Test-Path $font) -or (Get-Item $font).Length -lt 10MB) {
    curl.exe -L --fail --retry 5 -o $font 'https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/Variable/TTF/Subset/NotoSansSC-VF.ttf'
}
$fontLicense = Join-Path $licenseDir 'Noto-OFL.txt'
if (-not (Test-Path $fontLicense)) {
    curl.exe -L --fail --retry 5 -o $fontLicense 'https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/LICENSE'
}

Write-Host 'Windows x64 FFmpeg sidecar 与 Noto Sans SC 字体已准备完成。'
& node (Join-Path $PSScriptRoot 'collect-licenses.mjs')
if ($LASTEXITCODE) { throw '第三方依赖许可证收集失败' }
