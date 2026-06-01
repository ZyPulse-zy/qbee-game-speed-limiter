# qbee Limiter Config UI v0

This design follows the installed `web-design-engineer` workflow and uses the Linear modern-tool recipe as the anchor.

## Positioning

- Narrative role: operational control surface for a background utility.
- Viewing distance: laptop desktop, 1 m, repeated use.
- Visual temperature: quiet, precise, technical, low-noise.
- Capacity check: two-column desktop layout, stacked mobile layout, no status text in narrow corners.

## Design Decisions

- Anchor / recipe: Linear modern builder tool.
- Palette: near-black `#08090A`, panel `#16171C`, raised `#1E1F25`, text `#F7F8F8`, muted `#9CA3AF`, accent `#5E6AD2`.
- Typography: Segoe UI / Inter-style system sans for UI, Consolas / JetBrains Mono style for paths.
- Spacing: 4 / 8 / 12 / 16 / 24 / 40, with 14px card radius.
- Borders: 1px hairlines in low-opacity white, no heavy shadows.
- Motion: simple 150ms button/status transitions in the browser UI.

## v0 Layout

- Header: product title, one-line purpose, compact monitor status panel.
- Main left card: qB Web UI connection, credentials, interval, process list, startup toggles, primary actions.
- Main right card: live monitor state, detected executable, action log.
- Full-width library card: auto-scan, manual add, removable path rows.

## State Model

- Idle: status dot muted, `后台监控未运行`.
- Saving: header says `正在保存配置`.
- Testing: header says `正在测试连接`.
- Scan: header says `正在扫描游戏库`.
- Running: green status dot, message from `qbee_limiter_status.json`.
- Stopping: amber status dot and log message.
- Error: red status dot and error text in the log area.

## Implementation Notes

- Config UI runs as `qbee_limiter_config.exe`, serving a local HTML app on `127.0.0.1`.
- Background monitor runs as `qbee_limiter_monitor.exe` with no window.
- Saving config can automatically launch the monitor when `auto_start_monitor` is enabled.
- Startup registration points to the monitor exe, not the config UI.
