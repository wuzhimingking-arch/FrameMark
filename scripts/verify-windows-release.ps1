$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()

$projectRoot = Split-Path -Parent $PSScriptRoot
$target = 'x86_64-pc-windows-msvc'
$tauriConfig = Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') -Encoding UTF8 | ConvertFrom-Json
$version = $tauriConfig.version
$releaseDir = Join-Path $projectRoot "src-tauri\target\$target\release"
$installer = Join-Path $projectRoot "artifacts\windows\帧印_${version}_x64-setup.exe"
$appExe = Join-Path $releaseDir 'frame-mark.exe'
$readobj = Get-Command llvm-readobj.exe -ErrorAction SilentlyContinue
if (-not $readobj) { throw '缺少 llvm-readobj.exe，无法审计发行程序动态依赖' }
if (-not (Test-Path -LiteralPath $installer)) { throw "安装包不存在：$installer" }
if (-not (Test-Path -LiteralPath $appExe)) { throw "发行目录缺少：$appExe" }
$headers = @(& $readobj.Source --file-headers $appExe)
if (-not ($headers -match 'IMAGE_SUBSYSTEM_WINDOWS_GUI')) { throw 'frame-mark.exe 未使用 Windows GUI 子系统，会显示伴生控制台窗口' }

$forbiddenNames = 'webview2loader.dll', 'libunwind.dll', 'libgcc_s_seh-1.dll', 'libstdc++-6.dll', 'vcruntime140.dll', 'msvcp140.dll'
$audits = @{}
foreach ($name in 'frame-mark.exe', 'ffmpeg.exe', 'ffprobe.exe') {
    $path = Join-Path $releaseDir $name
    if (-not (Test-Path -LiteralPath $path)) { throw "发行目录缺少：$name" }
    $audits[$name] = @(& $readobj.Source --coff-imports $path | Select-String -Pattern '^\s+Name:' | ForEach-Object { ($_ -split 'Name:')[1].Trim().ToLowerInvariant() })
    $forbidden = $audits[$name] | Where-Object { $_ -in $forbiddenNames }
    if ($forbidden) { throw "$name 仍依赖不应外置的 DLL：$($forbidden -join ', ')" }
    $nonSystem = $audits[$name] | Where-Object { $_ -notlike 'api-ms-win-*' -and -not (Test-Path -LiteralPath (Join-Path $env:SystemRoot "System32\$_")) }
    if ($nonSystem) { throw "$name 存在发行目录未包含的非系统 DLL：$($nonSystem -join ', ')" }
}

$nsisScript = Join-Path $releaseDir 'nsis\x64\installer.nsi'
$nsisText = Get-Content -LiteralPath $nsisScript -Encoding UTF8 -Raw
$mode = $tauriConfig.bundle.windows.webviewInstallMode.type
if ($mode -ne 'embedBootstrapper' -or $nsisText -notmatch 'WEBVIEW2BOOTSTRAPPERPATH') { throw 'NSIS 未配置内嵌 WebView2 Bootstrapper' }

$verificationRoot = Join-Path ([System.IO.Path]::GetTempPath()) "framemark-clean-$([guid]::NewGuid().ToString('N'))"
$installDir = Join-Path $verificationRoot 'installed'
$process = $null
New-Item -ItemType Directory -Force -Path $verificationRoot | Out-Null
try {
    $isolatedInstaller = Join-Path $verificationRoot (Split-Path -Leaf $installer)
    Copy-Item -LiteralPath $installer -Destination $isolatedInstaller
    $install = Start-Process -FilePath $isolatedInstaller -ArgumentList @('/S', "/D=$installDir") -PassThru -Wait
    if ($install.ExitCode) { throw "安装程序退出码：$($install.ExitCode)" }

    foreach ($relative in 'frame-mark.exe', 'ffmpeg.exe', 'ffprobe.exe', 'resources\NotoSansSC-Regular.ttf', 'resources\licenses\FFmpeg-GPLv3.txt', 'resources\licenses\Noto-OFL.txt', 'uninstall.exe') {
        if (-not (Test-Path -LiteralPath (Join-Path $installDir $relative))) { throw "安装目录缺少：$relative" }
    }

    $startMenu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
    if (-not (Get-ChildItem -LiteralPath $startMenu -Recurse -Filter '帧印.lnk' -ErrorAction SilentlyContinue)) { throw '开始菜单缺少帧印快捷方式' }

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = Join-Path $installDir 'frame-mark.exe'
    $startInfo.WorkingDirectory = $installDir
    $startInfo.UseShellExecute = $false
    $startInfo.Environment['PATH'] = "$env:SystemRoot\System32;$env:SystemRoot"
    $process = [System.Diagnostics.Process]::Start($startInfo)
    $started = $false
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 250
        $process.Refresh()
        if ($process.HasExited) { throw "已安装程序异常退出，退出码：$($process.ExitCode)" }
        if ($process.MainWindowTitle -like '帧印*') { $started = $true; break }
    }
    if (-not $started) { throw '已安装程序未能打开主界面' }
    Stop-Process -Id $process.Id -Force
    $process.WaitForExit()

    $uninstall = Start-Process -FilePath (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -PassThru -Wait
    if ($uninstall.ExitCode) { throw "卸载程序退出码：$($uninstall.ExitCode)" }
    Start-Sleep -Seconds 2
    if (Test-Path -LiteralPath $installDir) { throw '卸载后安装目录仍存在' }

    Write-Host "frame-mark.exe 动态依赖审计通过：$($audits['frame-mark.exe'] -join ', ')"
    Write-Host 'frame-mark.exe Windows GUI 子系统审计通过。'
    Write-Host "NSIS WebView2 安装模式：$mode"
    Write-Host '安装包副本、干净 PATH、独立临时安装目录启动验证通过。'
} finally {
    if ($process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue }
    $uninstaller = Join-Path $installDir 'uninstall.exe'
    if (Test-Path -LiteralPath $uninstaller) { Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait }
    if (Test-Path -LiteralPath $verificationRoot) { Remove-Item -LiteralPath $verificationRoot -Recurse -Force }
}
