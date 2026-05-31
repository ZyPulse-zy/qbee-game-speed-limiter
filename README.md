# qbee Game Speed Limiter

Windows tool for qBittorrent / qBittorrent Enhanced Edition. It monitors your game libraries and automatically enables qBittorrent's alternative speed limits while a game is running, then disables them after the game closes.

## Features

- Detects running games by executable path under configured game library folders.
- Supports multiple game folders.
- Keeps an optional manual process-name list for games outside the libraries.
- Uses qBittorrent Web UI API.
- Includes a desktop UI for choosing game folders and editing qBittorrent Web UI credentials.
- Includes a prebuilt Windows executable.

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

```json
{
  "qbee_url": "http://127.0.0.1:8080",
  "username": "admin",
  "password": "adminadmin",
  "game_folders": [
    "C:\\Program Files (x86)\\Steam\\steamapps",
    "D:\\SteamLibrary\\steamapps"
  ],
  "game_processes": [],
  "exclude_processes": [
    "steam.exe",
    "epicgameslauncher.exe",
    "goggalaxy.exe",
    "wegame.exe",
    "battle.net.exe"
  ],
  "check_interval_seconds": 3,
  "restore_on_exit": true,
  "log_file": "qbee_game_speed_limiter.log"
}
```

## Build

```powershell
.\build.ps1
```

The main desktop app is implemented with .NET Framework WinForms in `QbeeGameSpeedLimiter.cs`.
`qbee_game_speed_limiter.py` is kept as a script-friendly legacy version.

## Notes

- qBittorrent must be running.
- Web UI credentials must be correct.
- The executable does not bundle your config. Keep the JSON file beside the EXE.
- Passwords are stored in the local JSON config file as plain text.
