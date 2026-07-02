param(
    [switch]$StartConfig
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$configExe = Join-Path $root "download_limiter_config.exe"
$monitorExe = Join-Path $root "download_limiter_monitor.exe"
$config = Join-Path $root "download_client_game_speed_limiter.json"
$legacyConfig = Join-Path $root "qbee_game_speed_limiter.json"
$example = Join-Path $root "download_client_game_speed_limiter.example.json"

if (!(Test-Path $configExe)) { throw "找不到 download_limiter_config.exe，请先完整解压发行包。" }
if (!(Test-Path $monitorExe)) { throw "找不到 download_limiter_monitor.exe，请先完整解压发行包。" }
if (!(Test-Path $config) -and (Test-Path $legacyConfig)) { Copy-Item $legacyConfig $config }
if (!(Test-Path $config) -and (Test-Path $example)) { Copy-Item $example $config }

Get-Process qbee_limiter_monitor,qbee_limiter_config -ErrorAction SilentlyContinue | Stop-Process -Force

function New-UrlShortcut($Path, $Target) {
    $content = "[InternetShortcut]`r`nURL=file:///" + ($Target -replace '\\','/') + "`r`nIconFile=$Target`r`nIconIndex=0`r`n"
    Set-Content -LiteralPath $Path -Value $content -Encoding ASCII
}

$desktop = [Environment]::GetFolderPath("Desktop")
if ($desktop) {
    New-UrlShortcut (Join-Path $desktop "下载器游戏限速助手.url") $configExe
    Remove-Item -LiteralPath (Join-Path $desktop "qbee 游戏限速助手.url") -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $desktop "qBittorrent 游戏限速助手.url") -ErrorAction SilentlyContinue
}

$programs = [Environment]::GetFolderPath("Programs")
if ($programs) {
    $folder = Join-Path $programs "下载器游戏限速助手"
    New-Item -ItemType Directory -Force -Path $folder | Out-Null
    New-UrlShortcut (Join-Path $folder "配置下载器游戏限速助手.url") $configExe
    Remove-Item -LiteralPath (Join-Path $programs "qbee 游戏限速助手") -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $programs "qBittorrent 游戏限速助手") -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "安装完成：已创建快捷入口。"
Write-Host "下一步：双击 download_limiter_config.exe，或从桌面入口打开配置界面。"

if ($StartConfig) {
    Start-Process -FilePath $configExe
}
