$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()

$projectRoot = Split-Path -Parent $PSScriptRoot
$assetTemp = Join-Path ([System.IO.Path]::GetTempPath()) 'framemark-release-assets'
$binaryDir = Join-Path $projectRoot 'src-tauri\binaries'
$resourceDir = Join-Path $projectRoot 'src-tauri\resources'
$targets = 'x86_64-pc-windows-msvc', 'x86_64-pc-windows-gnullvm'

New-Item -ItemType Directory -Force -Path $assetTemp, $binaryDir, $resourceDir | Out-Null

$ffmpegZip = Join-Path $assetTemp 'ffmpeg-release-essentials.zip'
if (-not (Test-Path $ffmpegZip) -or (Get-Item $ffmpegZip).Length -lt 50MB) {
    curl.exe -L --fail --retry 5 --retry-delay 3 -o $ffmpegZip 'https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip'
}

$ffmpegDir = Join-Path $assetTemp 'ffmpeg'
if (-not (Test-Path $ffmpegDir)) {
    Expand-Archive -LiteralPath $ffmpegZip -DestinationPath $ffmpegDir
}

foreach ($name in 'ffmpeg', 'ffprobe') {
    $source = Get-ChildItem -LiteralPath $ffmpegDir -Recurse -Filter "$name.exe" | Select-Object -First 1
    if (-not $source) { throw "FFmpeg 发行包缺少 $name.exe" }
    foreach ($target in $targets) {
        Copy-Item -LiteralPath $source.FullName -Destination (Join-Path $binaryDir "$name-$target.exe") -Force
    }
}

$font = Join-Path $resourceDir 'NotoSansSC-Regular.ttf'
if (-not (Test-Path $font) -or (Get-Item $font).Length -lt 10MB) {
    curl.exe -L --fail --retry 5 -o $font 'https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/Variable/TTF/Subset/NotoSansSC-VF.ttf'
}

Write-Host 'Windows x64 FFmpeg sidecar 与 Noto Sans SC 字体已准备完成。'
