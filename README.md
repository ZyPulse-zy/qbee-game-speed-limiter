# qbee Game Speed Limiter

[English](README.md) | [简体中文](README.zh-CN.md)

Windows tool for qBittorrent / qBittorrent Enhanced Edition. It monitors your game libraries and automatically enables qBittorrent's alternative speed limits while a game is running, then disables them after the game closes.

## Features

- Detects running games by executable path under configured game library folders.
- Supports multiple game folders.
- Can auto-scan local Steam, Epic, GOG, WeGame, XboxGames, and common game folders.
- Keeps an optional manual process-name list for games outside the libraries.
- Shows the currently detected executable so stuck monitoring can be diagnosed.
- Ignores common runtime/launcher/installer folders inside game libraries.
- Reads Steam app manifests and filters common non-game app names such as tools, servers, SDKs, runtimes, benchmarks, and Wallpaper Engine.
- Can register itself for current-user Windows startup and auto-start monitoring when launched.
- Uses qBittorrent Web UI API.
- Includes a desktop UI for choosing game folders and editing qBittorrent Web UI credentials.
- GitHub Actions builds a Windows executable artifact.

## Download

Download the latest Windows build from the repository's GitHub Actions artifacts or Releases.
Tagged versions such as `v1.0.0` are packaged automatically by the Release workflow.

The packaged artifact contains:

- `qbee_game_speed_limiter.exe`
- `qbee_game_speed_limiter.json`
- `README.md`
- `USER_GUIDE.zh-CN.md`

Chinese guide: [`docs/USER_GUIDE.zh-CN.md`](docs/USER_GUIDE.zh-CN.md)

## Default Game Libraries

The included config is set to:

```json
"game_folders": [
  "C:\\Program Files (x86)\\Steam\\steamapps",
  "D:\\SteamLibrary\\steamapps"
]
```

## Setup

1. Enable qBittorrent Web UI.
2. Keep `qbee_game_speed_limiter.exe` and `qbee_game_speed_limiter.json` in the same folder.
3. Run `qbee_game_speed_limiter.exe`.
4. In the desktop UI, set your Web UI URL, username, password, and game library folders.
5. Click `Save` and then `Start monitoring`.

## Config

The repository tracks `qbee_game_speed_limiter.example.json`. Your local `qbee_game_speed_limiter.json` is ignored by Git because it may contain private Web UI credentials.

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
  "exclude_processes": [
    "steam.exe",
    "steamservice.exe",
    "steamwebhelper.exe",
    "epicgameslauncher.exe",
    "goggalaxy.exe",
    "wegame.exe",
    "battle.net.exe"
  ],
  "check_interval_seconds": 3,
  "restore_on_exit": true,
  "start_with_windows": false,
  "auto_start_monitor": false,
  "log_file": "qbee_game_speed_limiter.log"
}
```

## Build

```powershell
.\build.ps1
```

The main desktop app is implemented with .NET Framework WinForms in `QbeeGameSpeedLimiter.cs`.
`qbee_game_speed_limiter.py` is kept as a script-friendly legacy version.

The build writes `qbee_game_speed_limiter.exe` in the project folder. The executable is ignored by Git; publish it through GitHub Releases or Actions artifacts instead of committing it.

## Notes

- qBittorrent must be running.
- Web UI credentials must be correct.
- If `http://127.0.0.1:8080` opens `CEF remote debugging`, Steam's CEF helper may be occupying the IPv4 loopback address. Try `http://[::1]:8080` first. If that still fails, change qBittorrent Web UI to another port such as `8081`, then set the app URL to `http://127.0.0.1:8081`.
- If alternative speed limits stay enabled after closing a game, check `qbee_game_speed_limiter.log`; it records the detected executable path that is keeping monitor mode active.
- The executable does not bundle your config. Keep the JSON file beside the EXE.
- Passwords are stored in the local JSON config file as plain text.
