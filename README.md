# qbee Game Speed Limiter

Windows tool for qBittorrent / qBittorrent Enhanced Edition. It monitors your Steam game libraries and automatically enables qBittorrent's alternative speed limits while a game is running, then disables them after the game closes.

## Features

- Detects running games by executable path under configured game library folders.
- Supports multiple game folders.
- Keeps an optional manual process-name list for games outside the libraries.
- Uses qBittorrent Web UI API.
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
3. Edit `qbee_game_speed_limiter.json` and set your Web UI URL, username, and password.
4. Run `qbee_game_speed_limiter.exe`.

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
python -m pip install pyinstaller
python -m PyInstaller --onefile --console --name qbee_game_speed_limiter qbee_game_speed_limiter.py
```

## Notes

- qBittorrent must be running.
- Web UI credentials must be correct.
- The executable does not bundle your config. Keep the JSON file beside the EXE.
