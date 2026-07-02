param(
    [switch]$KeepConfig
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path

Get-Process download_limiter_monitor,download_limiter_config,qbee_limiter_monitor,qbee_limiter_config -ErrorAction SilentlyContinue | Stop-Process -Force

try {
    Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "DownloadClientGameLimiterMonitor" -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "QbeeLimiterMonitor" -ErrorAction SilentlyContinue
} catch {}

$desktop = [Environment]::GetFolderPath("Desktop")
if ($desktop) {
    Remove-Item -LiteralPath (Join-Path $desktop "下载器游戏限速助手.url") -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $desktop "qbee 游戏限速助手.url") -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $desktop "qBittorrent 游戏限速助手.url") -ErrorAction SilentlyContinue
}

$programs = [Environment]::GetFolderPath("Programs")
if ($programs) {
    Remove-Item -LiteralPath (Join-Path $programs "下载器游戏限速助手") -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $programs "qbee 游戏限速助手") -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $programs "qBittorrent 游戏限速助手") -Recurse -Force -ErrorAction SilentlyContinue
}

Remove-Item -LiteralPath (Join-Path $root "download_limiter_monitor.stop") -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $root "download_limiter_status.json") -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $root "qbee_limiter_monitor.stop") -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $root "qbee_limiter_status.json") -ErrorAction SilentlyContinue

if ($KeepConfig) {
    Write-Host "已按 KeepConfig 要求保留配置文件。"
} else {
    Write-Host "已保留 download_client_game_speed_limiter.json 和旧版 qbee_game_speed_limiter.json。若要彻底删除，请手动删除解压目录。"
}

Write-Host "卸载清理完成。现在可以删除整个解压文件夹。"
