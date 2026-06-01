# qBittorrent Game Speed Limiter

[English](README.md) | [简体中文](README.zh-CN.md)

Automatically throttle qBittorrent downloads and uploads while you play games, then restore speeds after exit. Built for Windows users who keep qBittorrent or qBittorrent Enhanced Edition running in the background.

[Download for Windows](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases/latest) · [中文教程](README.zh-CN.md) · [FAQ](README.zh-CN.md#常见问题)

![Configuration UI screenshot](docs/config-ui.png)

## Why Use It?

Background BT / PT downloads can saturate your bandwidth and increase latency in CS2, Valorant, Minecraft, Palworld, voice chat, or other online games.

This tool detects when a game is running, enables qBittorrent alternative speed limits, and restores the previous state after the game exits.

## Highlights

- Supports Steam / Epic / Xbox / Battle.net / EA / Ubisoft / WeGame and common game folders.
- Supports qBittorrent and qBittorrent Enhanced Edition.
- Starts the low-memory monitor with Windows.
- Keeps the configuration UI separate from the background monitor.
- Does not overwrite alternative speed limits that you enabled manually.
- Can auto-scan game library folders or use manually added folders.

## Download

Download `qbee-game-speed-limiter-windows.zip` from the [Releases page](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases).

The zip contains only the end-user files:

```text
qbee_limiter_config.exe
qbee_limiter_monitor.exe
qbee_game_speed_limiter.json
README.zh-CN.md
LICENSE
```

## Quick Start

1. Enable qBittorrent Web UI in `Tools -> Options -> WebUI`.
2. Extract the zip.
3. Run `qbee_limiter_config.exe`.
4. Enter the qB Web UI URL, username, and password.
5. Click `测试连接` to test the connection.
6. Click `自动扫描` to find game library folders, or add folders manually.
7. Enable `保存后自动启动监控`.
8. Click `保存并应用`.

After saving, `qbee_limiter_monitor.exe` runs in the background. You can close the browser configuration page.

## qB Web UI Notes

The common URL is:

```text
http://127.0.0.1:8080
```

If qB allows localhost clients to bypass authentication, you can usually leave username and password empty.

If the URL opens `CEF remote debugging`, Steam may be occupying `127.0.0.1:8080`. Try:

```text
http://[::1]:8080
```

If that still fails, change the qB Web UI port to `8081` and set this app to:

```text
http://127.0.0.1:8081
```

## How It Works

- `qbee_limiter_config.exe` opens the local browser-based configuration UI.
- `qbee_limiter_monitor.exe` is the low-memory background monitor.
- Windows startup launches the monitor, not the configuration UI.
- The monitor only restores speed limits that it changed itself.

## Build

Install Rust and MinGW-w64, then run:

```powershell
.\build.ps1
```

Developer docs, UI design notes, and changelog are kept in the repository instead of the release zip.

## License

MIT License
