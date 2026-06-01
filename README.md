# qbee Game Speed Limiter

[English](README.md) | [简体中文](README.zh-CN.md)

Windows tool for qBittorrent / qBittorrent Enhanced Edition. It monitors your game libraries and automatically enables qBittorrent alternative speed limits while a game is running, then disables them after the game closes.

Latest release: [v0.2.0](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases/tag/v0.2.0)

## What Changed in v0.2.0

- `qbee_limiter_monitor.exe` is the low-memory background monitor.
- `qbee_limiter_config.exe` is a modern browser-based local configuration UI.
- Saving configuration can automatically start the monitor.
- Windows startup now launches the monitor, not the configuration UI.
- GitHub release builds now call Cargo through a stable executable path.

## Features

- Detects running games by executable path under configured game library folders.
- Supports multiple game folders and auto-scans local Steam, Epic, GOG, WeGame, XboxGames, Battle.net, EA, Ubisoft, and common game folders.
- Reads Steam app manifests and filters common non-game entries such as tools, servers, SDKs, runtimes, benchmarks, and Wallpaper Engine.
- Preserves alternative speed limits if they were already enabled before a game started.
- Prevents duplicate monitor instances from competing with each other.
- Uses cached process detection to keep background CPU usage low.
- Uses qBittorrent Web UI API.
- Includes a local browser configuration UI documented in [`docs/CONFIG_UI_DESIGN.md`](docs/CONFIG_UI_DESIGN.md).

## Download

Download the latest Windows package from [Releases](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases).
Tagged versions such as `v0.2.0` are packaged automatically by the Release workflow.

The packaged artifact contains:

- `qbee_limiter_config.exe`
- `qbee_limiter_monitor.exe`
- `qbee_game_speed_limiter.json`
- `README.md`
- `README.zh-CN.md`
- `USER_GUIDE.zh-CN.md`
- `DESIGN.md`
- `CONFIG_UI_DESIGN.md`

Chinese guide: [`docs/USER_GUIDE.zh-CN.md`](docs/USER_GUIDE.zh-CN.md)

## Setup

1. Enable qBittorrent Web UI.
2. Keep both exe files and `qbee_game_speed_limiter.json` in the same folder.
3. Run `qbee_limiter_config.exe`.
4. Set the Web UI URL, username, password, monitor interval, and game library folders.
5. Click `自动扫描` to find local game libraries, or add folders manually.
6. Enable `保存后自动启动监控` if you want the monitor to start after saving.
7. Click `保存并应用`.

## Build

Install Rust and MinGW-w64 first, then run:

```powershell
.\build.ps1
```

The build writes `qbee_limiter_config.exe` and `qbee_limiter_monitor.exe` in the project folder.
Legacy C++, C#, Python, and Win32 single-window implementations are kept as reference material only.

## Notes

- qBittorrent must be running.
- Web UI credentials must be correct.
- If `http://127.0.0.1:8080` opens `CEF remote debugging`, Steam CEF may be occupying the IPv4 loopback address. Try `http://[::1]:8080` first. If that still fails, change qBittorrent Web UI to another port such as `8081`, then set the app URL to `http://127.0.0.1:8081`.
