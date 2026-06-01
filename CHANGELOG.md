# Changelog

## Unreleased

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
