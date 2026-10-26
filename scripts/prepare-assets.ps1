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
$ffmpegVersion = '7.1.1'
$ffmpegUrl = "https://github.com/GyanD/codexffmpeg/releases/download/$ffmpegVersion/ffmpeg-$ffmpegVersion-essentials_build.zip"
$expectedHashes = @{
    ffmpeg = 'B90225987BDD042CCA09A1EFB5E34E9848F2D1DBF5FBCD388753A44145522997'
    ffprobe = '05E8FA639450F8191635192871AE37A3EC3E4638FA12F3B7D49C6522BA16A8ED'
}

New-Item -ItemType Directory -Force -Path $assetTemp, $binaryDir, $resourceDir | Out-Null

$ffmpegTarget = Join-Path $binaryDir "ffmpeg-$target.exe"
$ffprobeTarget = Join-Path $binaryDir "ffprobe-$target.exe"
$ffmpegDir = Join-Path $assetTemp "ffmpeg-$ffmpegVersion"
$matchesPinned = (Test-Path $ffmpegTarget) -and (Test-Path $ffprobeTarget)
if ($matchesPinned) {
    $matchesPinned = ((Get-Sha256 $ffmpegTarget) -eq $expectedHashes.ffmpeg) -and ((Get-Sha256 $ffprobeTarget) -eq $expectedHashes.ffprobe)
}
if (-not $matchesPinned) {
    $ffmpegZip = Join-Path $assetTemp "ffmpeg-$ffmpegVersion-essentials_build.zip"
    if (-not (Test-Path $ffmpegZip) -or (Get-Item $ffmpegZip).Length -lt 50MB) {
        curl.exe -L --fail --retry 5 --retry-delay 3 -o $ffmpegZip $ffmpegUrl
        if ($LASTEXITCODE) { throw 'FFmpeg 下载失败' }
    }
    if (-not (Test-Path $ffmpegDir)) {
        Expand-Archive -LiteralPath $ffmpegZip -DestinationPath $ffmpegDir
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
