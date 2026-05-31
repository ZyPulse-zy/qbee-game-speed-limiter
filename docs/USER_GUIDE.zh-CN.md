# 使用说明

这个工具会监控游戏库中的游戏进程：

- 游戏运行时：自动打开 qBittorrent / qBittorrent Enhanced Edition 的备用速度限制
- 游戏退出后：自动关闭备用速度限制

## 首次使用

1. 在 qBittorrent/qBEE 中启用 Web UI。
2. 下载构建产物，并保持 `qbee_game_speed_limiter.exe` 和 `qbee_game_speed_limiter.json` 在同一目录。
3. 双击运行 `qbee_game_speed_limiter.exe`。
4. 填写 qB Web UI 地址、用户名、密码。
5. 点击“自动扫描”查找本机游戏库，也可以手动添加目录。
6. 如果需要，勾选“开机自启动”和“启动后自动开始监控”。
7. 点击“保存配置”，再点击“开始监控”。

## 开机自启动

勾选“开机自启动”后，程序会写入当前用户的 Windows 启动项，不需要管理员权限。

如果还勾选“启动后自动开始监控”，程序打开后会自动进入监控状态。

## qB Web UI 地址

常见地址：

```text
http://127.0.0.1:8080
```

如果这个地址打开的是 `CEF remote debugging`，可能是 Steam 的 CEF 占用了 IPv4 本机地址。可以尝试：

```text
http://[::1]:8080
```

程序也会在检测到这种情况时自动尝试 IPv6 本机地址。

## 游戏库扫描

自动扫描会尝试查找：

- Steam `libraryfolders.vdf`
- Epic manifests
- 各磁盘常见目录，例如 `SteamLibrary\steamapps`、`Games`、`Epic Games`、`GOG Games`、`WeGameApps`、`XboxGames`

Steam 库会读取 `appmanifest_*.acf`，并默认排除 Wallpaper Engine、服务器工具、SDK、运行库、基准测试、编辑器等常见非游戏条目。

## 后台占用

程序会缓存游戏目录和进程路径，避免每次检测都完整扫描系统信息。`check_interval_seconds` 是后台检测间隔，默认值为 `5` 秒；想进一步降低后台占用可以调大一些，想更快响应游戏启动和退出可以调小一些。

## 备用速度限制状态

如果游戏启动前备用速度限制已经是打开状态，程序会认为这是你的手动设置。游戏退出后它会保持打开，不会替你关闭。

## 排查

如果游戏退出后没有自动关闭限制，先看界面底部的“当前检测到”。如果这里仍显示某个 exe 路径，说明它还在让程序认为游戏正在运行。

日志文件：

```text
qbee_game_speed_limiter.log
```

## 隐私

`qbee_game_speed_limiter.json` 会在本机明文保存 Web UI 地址、用户名和密码。不要把自己的真实配置上传到 GitHub。
