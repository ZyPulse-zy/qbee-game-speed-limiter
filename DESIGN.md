# Design Guide

This project follows the `DESIGN.md`-first workflow inspired by [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md): keep a small, explicit design contract next to the code so future UI changes stay consistent.

## Product Feel

- Quiet Windows utility, not a marketing app.
- Fast to scan, low visual noise, and safe for long-running background use.
- Modern dark interface with a Linear/Tailwind-like rhythm: compact spacing, subtle borders, clear primary action.

## Layout

- One window, two main sections:
  - Connection settings
  - Game library detection
- Primary actions stay in the lower-right action row.
- Status stays visible in the header and detection result stays near the library list.

## Visual System

- Background: near-black `#080A0F`
- Surface: dark panel `#11131C`
- Input/list background: `#0B0D14`
- Border: muted slate `#272D3D`
- Text: `#E5E7EB`
- Muted text: `#9CA3AF`
- Primary action: indigo `#6366F1`

## Components

- Panels use a thin border and soft radius.
- Buttons are owner-drawn native Win32 controls:
  - Primary: filled indigo
  - Secondary: dark filled button with border
  - Disabled: low-contrast dark fill
- Native edit/list controls are kept for low memory usage, with dark colors applied through Win32 color messages.

## Constraints

- Keep the app native Rust + Win32 to preserve low idle memory.
- Avoid adding a browser runtime, webview, or large UI framework.
- Prefer a small number of stable colors over decorative gradients or heavy effects.
