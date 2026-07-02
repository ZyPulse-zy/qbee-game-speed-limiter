# Contributing

Thanks for helping improve Download Client Game Speed Limiter.

## Development

Build on Windows:

```powershell
.\build.ps1
```

The active app is the Rust split build in `src/`. The older C#, Python, and native prototypes are kept as historical references.

## Pull Requests

- Keep changes focused.
- Do not commit personal `download_client_game_speed_limiter.json` files, legacy `qbee_game_speed_limiter.json` files, or logs.
- Update `README.md` and `使用说明.md` when behavior changes.
- Include the exact detected process path when reporting false positives.
