# qbee Game Speed Limiter

[English](README.md) | 简体中文

这是一个 Windows 小工具，用于在玩游戏时自动打开 qBittorrent / qBittorrent Enhanced Edition 的“备用速度限制”，游戏退出后再自动关闭。

最新版下载：[v0.1.2](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases/tag/v0.1.2)

## 功能

- 根据游戏库中的运行进程自动判断是否正在玩游戏。
- 支持多个游戏库目录。
- 支持自动扫描本机 Steam、Epic、GOG、WeGame、XboxGames 和常见游戏目录。
- 支持手动补充进程名。
- 界面会显示当前检测到的 exe 路径，方便排查误判。
- 默认忽略 Steamworks、运行库、安装器、启动器等常见非游戏目录。
- 会读取 Steam `appmanifest_*.acf`，并过滤 Wallpaper Engine、服务器工具、SDK、运行库、基准测试、编辑器等常见非游戏条目。
- 通过 qBittorrent Web UI API 控制备用速度限制。
- 提供 Windows 桌面配置界面。
- 支持当前用户开机自启动，并可在启动后自动开始监控。
- 如果备用速度限制原本就是打开的，游戏退出后会保持打开，不会误关。
- 阻止重复打开多个程序实例，避免互相抢状态。
- 使用缓存式进程检测，降低后台 CPU 占用。
- 游戏库自动扫描会在后台执行，扫描时界面不会卡住。
- GitHub Actions 会自动构建 Windows exe。

## 下载

请从 [Releases](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases) 下载最新版 Windows 包。

打包内容包括：

- `qbee_game_speed_limiter.exe`
- `qbee_game_speed_limiter.json`
- `README.md`
- `USER_GUIDE.zh-CN.md`
- `LICENSE`

更详细的中文使用说明见：[docs/USER_GUIDE.zh-CN.md](docs/USER_GUIDE.zh-CN.md)

## 快速开始

1. 在 qBittorrent/qBEE 中启用 Web UI。
2. 保持 `qbee_game_speed_limiter.exe` 和 `qbee_game_speed_limiter.json` 在同一目录。
3. 双击运行 `qbee_game_speed_limiter.exe`。
4. 在界面中填写 qB Web UI 地址、用户名、密码和游戏库目录。
5. 点击“自动扫描”查找本机游戏库，也可以手动添加目录。
6. 如果需要，勾选“开机自启动”和“启动后自动开始监控”。
7. 点击“保存配置”，再点击“开始监控”。

## 行为说明

- 程序只会关闭由它自己打开的备用速度限制。
- 如果游戏启动前备用速度限制已经是打开状态，程序会认为这是你的手动设置，游戏退出后会保持打开。
- 程序会阻止重复打开多个实例，避免多个窗口互相抢状态。
- 默认检测间隔是 5 秒。想进一步降低后台占用，可以把 `check_interval_seconds` 调大；想更快响应游戏启动和退出，可以调小。

## qB Web UI 地址

常见地址：

```text
http://127.0.0.1:8080
```

如果 `127.0.0.1:8080` 打开的是 `CEF remote debugging`，可能是 Steam 的 CEF helper 占用了 IPv4 本机地址。可以尝试：

```text
http://[::1]:8080
```

程序检测到这种情况时，也会自动尝试切换到 IPv6 本机地址。

## 配置

仓库中提交的是示例配置：

```text
qbee_game_speed_limiter.example.json
```

你的本地真实配置是：

```text
qbee_game_speed_limiter.json
```

真实配置文件会被 Git 忽略，因为里面可能包含 Web UI 用户名和密码。

示例：

```json
{
  "qbee_url": "http://127.0.0.1:8080",
  "username": "admin",
  "password": "",
  "game_folders": [
    "C:\\Program Files (x86)\\Steam\\steamapps",
    "D:\\SteamLibrary\\steamapps"
  ],
  "game_processes": [],
  "check_interval_seconds": 5,
  "restore_on_exit": true,
  "start_with_windows": false,
  "auto_start_monitor": false,
  "log_file": "qbee_game_speed_limiter.log"
}
```

## 构建

在 Windows 上运行：

```powershell
.\build.ps1
```

主程序是 .NET Framework WinForms 应用，源码为：

```text
QbeeGameSpeedLimiter.cs
```

`qbee_game_speed_limiter.py` 保留为脚本友好的旧版实现。

## 排查

如果退出游戏后没有自动关闭限制：

1. 查看界面底部“当前检测到”的 exe 路径。
2. 查看日志：

```text
qbee_game_speed_limiter.log
```

如果某个工具类程序被误判，可以把它加入配置中的 `exclude_processes`、`exclude_path_keywords` 或 `exclude_steam_app_keywords`。

## 隐私

`qbee_game_speed_limiter.json` 会在本机明文保存 qB Web UI 地址、用户名和密码。不要把自己的真实配置上传到 GitHub。

## 许可证

MIT License
