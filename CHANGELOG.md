# Changelog

## Unreleased

## v0.3.4

- Added automatic global speed-limit control for BitComet through its newer WebUI connection configuration API.
- Added BitComet address, username/password, diagnostics, test-connection, game-mode, and restore support across the config UI and monitor.
- Simplified the configuration UI so only fields relevant to the selected download client are shown.
- Added a small transition for client-specific fields to keep the browser configuration page feeling responsive without adding runtime weight.
- Updated user-facing docs to explain that qB/Transmission use their own alternative limit settings while aria2/µTorrent/Deluge/BitComet use the shared game-speed values.

## v0.3.3

- Added double-clickable `install.cmd` and `uninstall.cmd` wrappers for easier first-time setup and cleanup on Windows.
- Included the CMD wrappers in Build and Release packages.
- Updated the beginner setup guide to prefer `install.cmd` while keeping the PowerShell scripts for advanced use.

## v0.3.2

- Added a multi-size Windows `.ico` derived from the project icon.
- Embedded the app icon into both Windows executables during MinGW release builds when `windres` is available.
- Updated shortcut naming to use the broader qbee Game Speed Limiter branding while keeping uninstall cleanup for old shortcut names.

## v0.3.1

- Added automatic global speed-limit control for µTorrent / BitTorrent Classic through the Web UI API.
- Added automatic global speed-limit control for Deluge through the Web JSON-RPC API.
- Added a configuration self-check tool that explains missing monitor files, invalid URLs, missing game folders, stale monitor status, and startup settings.
- Added portable install and uninstall scripts for desktop and Start Menu shortcuts without requiring administrator permissions.
- Included the install scripts in the Windows release package and documented the simpler setup flow.
- Updated the configuration UI and user guides to present qBittorrent/qBEE, Transmission, aria2/Motrix, µTorrent/BitTorrent Classic, Deluge, and BitComet status clearly.
## v0.3.0

- Added a multi-downloader configuration model for qBittorrent/qBEE, Transmission, aria2/Motrix, and BitComet visibility.
- Added real automatic control for Transmission alternative speed mode and aria2 global speed limits.
- Added clearer BitComet status messaging instead of pretending automatic control is reliable without a stable public API.
- Improved the configuration UI with downloader selection, subtle interaction animations, aria2 game-speed fields, and desktop shortcut creation.
- Added icon assets and installation guidance for a more complete end-user package.
## v0.2.5

- Fixed the Start Monitor action reporting success before the background monitor actually became responsive.
- The config UI now shows a clear failure message if the monitor starts and immediately exits or another old monitor instance is blocking it.

## v0.2.4

- Reworked the README first screen as a user-facing software landing page.
- Added clearer positioning for gaming latency, BT/PT downloads, qBittorrent alternative speed limits, and multi-launcher game library support.
- Added a configuration UI screenshot and a user-facing Release description template.

## v0.2.3

- Rewrote the Chinese README as a beginner-friendly setup guide.
- Corrected the release package file list in both README files.
- Added clearer qB Web UI setup, password, localhost bypass, CEF remote debugging, and game library folder guidance.

## v0.2.2

- Slimmed the Windows release package to only include the two executables, the default config file, Chinese quick-start notes, and the license.
- Kept developer docs, design notes, and changelog in the GitHub repository instead of shipping them in the end-user zip.

## v0.2.1

- Fixed GitHub Actions Release builds on newer `windows-latest` runners by letting `msys2/setup-msys2` provide the active MinGW path instead of forcing a stale `C:\msys64` path.
- Keeps the v0.2.x split architecture: browser-based config UI plus a low-memory Rust background monitor.

## v0.2.0

- Split the app into `qbee_limiter_config.exe` and `qbee_limiter_monitor.exe`.
- Replaced the always-on native configuration window with a browser-based local configuration UI.
- Moved long-running game detection and qBittorrent control into a no-window monitor process for lower background memory usage.
- Saving config can now automatically start the monitor process.
- Fixed GitHub Actions release builds by calling Cargo through a stable executable path in `build.ps1`.

## v0.1.9

- Moved status text from the narrow top-right corner to a full-width status row under the title to prevent overlapping.
- Added direct button feedback: Save changes to `已保存`, Start changes to `监控中`, and Stop changes to `停止中` while actions are running.

## v0.1.8

- Made the native UI DPI-aware to avoid blurry Windows bitmap scaling on high-DPI displays.
- Increased window width and adjusted form spacing so Chinese labels, checkboxes, and action buttons have more room.
- Changed font creation to negative-height Segoe UI fonts for clearer text rendering.

## v0.1.7

- Restyled the Rust Win32 UI using a `DESIGN.md`-first workflow inspired by `awesome-design-md`.
- Added a dark Linear/Tailwind-like visual system with native owner-drawn buttons, dark inputs, bordered panels, and lower-noise status presentation.
- Documented the project's UI rules in `DESIGN.md` so future interface work stays consistent without adding a heavy UI framework.

## v0.1.6

- Fixed a Rust UI re-entrant borrow crash that could close the app after clicking `Test connection`.

## v0.1.5

- Rewrote the desktop app in Rust while keeping native Win32 controls for low idle memory usage.
- Restyled the UI with a Tailwind-inspired minimal card and form layout.
- Added Cargo-based local and GitHub Actions builds.

## v0.1.4

- Rewrote the desktop app as a native C++/Win32 program to lower idle memory usage.
- Replaced the WinForms layout with a cleaner minimal native UI.
- Updated local and GitHub Actions builds to use MinGW-w64 g++.

## v0.1.3

- Stop monitoring from a background worker so the UI no longer freezes while qBittorrent API calls finish.
- Shorten qBittorrent Web API timeout from 10 seconds to 5 seconds.

## v0.1.2

- Preserve user-enabled qBittorrent alternative speed limits instead of disabling them after games exit.
- Keep UI controls in sync when the monitor exits because of an error.
- Run game library auto-scan in the background so the window stays responsive.
- Prevent multiple app instances from running at the same time.
- Validate qB Web UI URL and game folder configuration before saving or starting monitoring.

## v0.1.1

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
