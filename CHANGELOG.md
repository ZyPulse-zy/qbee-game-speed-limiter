# Changelog

## Unreleased

- Reduced background overhead by caching game folders and process executable paths during monitoring.
- Refreshed the WinForms UI with a cleaner header, card layout, and styled controls.
- Changed the default check interval from 3 seconds to 5 seconds.

## v0.1.0

- Added current-user Windows startup registration.
- Added option to auto-start monitoring when the app opens.
- Added desktop configuration UI.
- Added game library auto-scan.
- Added qBittorrent localhost IPv6 fallback when Steam CEF occupies `127.0.0.1:8080`.
- Added Steam tool/runtime filtering to reduce false positives.
