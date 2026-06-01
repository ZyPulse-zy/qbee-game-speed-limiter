# 使用说明

这个工具会监控游戏库中的游戏进程：

- 游戏运行时：自动打开 qBittorrent / qBittorrent Enhanced Edition 的备用速度限制。
- 游戏退出后：自动关闭由本工具打开的备用速度限制。

## 首次使用

1. 在 qBittorrent/qBEE 中启用 Web UI。
2. 下载 Release 包，并保持下面三个文件在同一目录：
   - `qbee_limiter_config.exe`
   - `qbee_limiter_monitor.exe`
   - `qbee_game_speed_limiter.json`
3. 双击运行 `qbee_limiter_config.exe`。
4. 浏览器会打开本地配置界面。
5. 填写 qB Web UI 地址、用户名、密码。
6. 点击“自动扫描”查找本机游戏库，也可以手动添加目录。
7. 勾选“保存后自动启动监控”，点击“保存并应用”。

## 两个程序的分工

- `qbee_limiter_monitor.exe`：后台监控程序，无窗口，低内存常驻。
- `qbee_limiter_config.exe`：配置界面，只在需要改配置时打开，用完可以关闭。

配置界面关闭后，后台监控仍会继续运行。

## 开机自启动

勾选“开机启动后台监控”后，程序会写入当前用户的 Windows 启动项，不需要管理员权限。

启动项指向的是 `qbee_limiter_monitor.exe`，不会在开机时弹出配置界面。

## qB Web UI 地址

常见地址：

```text
http://127.0.0.1:8080
```

如果这个地址打开的是 `CEF remote debugging`，可能是 Steam 的 CEF 占用了 IPv4 本机地址。可以尝试：

```text
http://[::1]:8080
```

如果仍然不行，建议在 qBittorrent 设置里把 Web UI 端口改成 `8081` 或其他端口。

## 游戏库扫描

自动扫描会尝试查找：

- Steam `libraryfolders.vdf`
- Epic / GOG / WeGame / XboxGames 常见目录
- Battle.net / EA / Ubisoft 常见目录
- 各磁盘常见目录，例如 `SteamLibrary\steamapps`、`Games`、`Epic Games`、`GOG Games`

Steam 库会读取 `appmanifest_*.acf`，并默认排除 Wallpaper Engine、服务器工具、SDK、运行库、基准测试、编辑器等常见非游戏条目。

## 后台占用

v0.2.0 之后，后台只运行无窗口 monitor。开发机空闲实测约 `5.2 MB` 工作集、`1.1 MB` 私有内存。

`check_interval_seconds` 是后台检测间隔，默认值为 `5` 秒；想进一步降低后台占用可以调大一些，想更快响应游戏启动和退出可以调小一些。

## 状态文件和日志

后台状态文件：

```text
qbee_limiter_status.json
```

日志文件：

```text
qbee_game_speed_limiter.log
```

如果退出游戏后没有自动关闭限制，可以先看配置界面里的“当前检测到”，再查看日志。

## 隐私

`qbee_game_speed_limiter.json` 会在本机明文保存 Web UI 地址、用户名和密码。不要把自己的真实配置上传到 GitHub。
