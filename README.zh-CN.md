# qbee 游戏限速助手

[English](README.md) | 简体中文

这是一个 Windows 小工具，用于在玩游戏时自动打开 qBittorrent / qBittorrent Enhanced Edition 的“备用速度限制”，游戏退出后再自动关闭限制。

最新版下载：[v0.2.1](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases/tag/v0.2.1)

## v0.2.1 变化

- 拆分为两个程序：
  - `qbee_limiter_monitor.exe`：无窗口后台监控，负责检测游戏和控制 qB。
  - `qbee_limiter_config.exe`：本地网页配置界面，只在需要配置时打开。
- 保存配置后可以自动启动后台监控程序。
- 开机自启动现在启动后台监控程序，而不是配置界面。
- 修复 GitHub Actions Release 构建中 Cargo 路径调用失败的问题。

## 功能

- 根据游戏库中的运行进程自动判断是否正在玩游戏。
- 支持多个游戏库目录。
- 支持自动扫描本机 Steam、Epic、GOG、WeGame、XboxGames、Battle.net、EA、Ubisoft 和常见游戏目录。
- 会读取 Steam `appmanifest_*.acf`，并过滤 Wallpaper Engine、服务器工具、SDK、运行库、基准测试、编辑器等常见非游戏条目。
- 通过 qBittorrent Web UI API 控制备用速度限制。
- 如果备用速度限制原本就是打开的，游戏退出后会保持打开，不会误关。
- 阻止重复打开多个后台监控实例，避免互相抢状态。
- 使用缓存式进程检测，降低后台 CPU 占用。
- 配置界面按 Linear 风格设计，设计说明见 [`docs/CONFIG_UI_DESIGN.md`](docs/CONFIG_UI_DESIGN.md)。

## 下载

请从 [Releases](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases) 下载最新版 Windows 包。

打包内容包括：

- `qbee_limiter_config.exe`
- `qbee_limiter_monitor.exe`
- `qbee_game_speed_limiter.json`
- `README.md`
- `README.zh-CN.md`
- `USER_GUIDE.zh-CN.md`
- `DESIGN.md`
- `CONFIG_UI_DESIGN.md`
- `LICENSE`

## 快速开始

1. 在 qBittorrent/qBEE 中启用 Web UI。
2. 保持两个 exe 和 `qbee_game_speed_limiter.json` 在同一目录。
3. 双击运行 `qbee_limiter_config.exe`。
4. 在浏览器打开的配置界面中填写 qB Web UI 地址、用户名、密码和游戏库目录。
5. 点击“自动扫描”查找本机游戏库，也可以手动添加目录。
6. 勾选“保存后自动启动监控”。
7. 点击“保存并应用”。

## 行为说明

- 平时只需要后台 `qbee_limiter_monitor.exe` 常驻。
- 配置界面可以用完就关，不影响后台监控。
- 程序只会关闭由它自己打开的备用速度限制。
- 如果游戏启动前备用速度限制已经是打开状态，程序会认为这是你的手动设置，游戏退出后会保持打开。
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

如果仍然不行，建议在 qBittorrent 设置里把 Web UI 端口改成 `8081` 或其他端口，然后在本工具里填写：

```text
http://127.0.0.1:8081
```

## 构建

先安装 Rust 和 MinGW-w64，然后在 Windows 中运行：

```powershell
.\build.ps1
```

构建输出：

```text
qbee_limiter_config.exe
qbee_limiter_monitor.exe
```

旧的 C++、C#、Python 和 Win32 单窗口实现仅保留为参考。

## 隐私

`qbee_game_speed_limiter.json` 会在本机明文保存 qB Web UI 地址、用户名和密码。不要把自己的真实配置上传到 GitHub。

## 许可证

MIT License
