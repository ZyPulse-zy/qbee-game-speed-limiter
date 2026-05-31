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
6. 点击“保存配置”，再点击“开始监控”。

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

## 排查

如果游戏退出后没有自动关闭限制，先看界面底部的“当前检测到”。如果这里仍显示某个 exe 路径，说明它还在让程序认为游戏正在运行。

日志文件：

```text
qbee_game_speed_limiter.log
```

## 隐私

`qbee_game_speed_limiter.json` 会在本机明文保存 Web UI 地址、用户名和密码。不要把自己的真实配置上传到 GitHub。
