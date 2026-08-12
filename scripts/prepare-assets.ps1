$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()

$projectRoot = Split-Path -Parent $PSScriptRoot
$assetTemp = Join-Path ([System.IO.Path]::GetTempPath()) 'framemark-release-assets'
$binaryDir = Join-Path $projectRoot 'src-tauri\binaries'
$resourceDir = Join-Path $projectRoot 'src-tauri\resources'
$target = 'x86_64-pc-windows-msvc'

New-Item -ItemType Directory -Force -Path $assetTemp, $binaryDir, $resourceDir | Out-Null

$ffmpegTarget = Join-Path $binaryDir "ffmpeg-$target.exe"
$ffprobeTarget = Join-Path $binaryDir "ffprobe-$target.exe"
$ffmpegDir = Join-Path $assetTemp 'ffmpeg'
if (-not (Test-Path $ffmpegTarget) -or (Get-Item $ffmpegTarget).Length -lt 50MB -or -not (Test-Path $ffprobeTarget)) {
    $ffmpegZip = Join-Path $assetTemp 'ffmpeg-release-essentials.zip'
    if (-not (Test-Path $ffmpegZip) -or (Get-Item $ffmpegZip).Length -lt 50MB) {
        curl.exe -L --fail --retry 5 --retry-delay 3 -o $ffmpegZip 'https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip'
    }
    if (-not (Test-Path $ffmpegDir)) {
        Expand-Archive -LiteralPath $ffmpegZip -DestinationPath $ffmpegDir
    }
    foreach ($name in 'ffmpeg', 'ffprobe') {
        $source = Get-ChildItem -LiteralPath $ffmpegDir -Recurse -Filter "$name.exe" | Select-Object -First 1
        if (-not $source) { throw "FFmpeg 发行包缺少 $name.exe" }
        Copy-Item -LiteralPath $source.FullName -Destination (Join-Path $binaryDir "$name-$target.exe") -Force
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
