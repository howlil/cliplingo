---
name: windows-desktop
description: Use for Win32, COM/UI Automation, global hotkeys, clipboard behavior, focus, DPI, multi-monitor positioning, and Windows compatibility.
---

# Windows Desktop Engineering Skill

## Selection capture

- UI Automation selection is primary.
- Clipboard copy is compatibility fallback, not the architectural default.
- Clipboard fallback must preserve/restore user clipboard state as safely as practical and handle contention/failure.
- Return capture provenance and optional bounds so failures are diagnosable.
- Do not add app-specific hacks until a reproducible compatibility failure exists.

## Window behavior

- Popup must appear near selection bounds where available; cursor is fallback.
- Account for per-monitor DPI and multiple monitors.
- Avoid stealing focus unless an action explicitly needs focus.
- Escape/dismiss behavior must be deterministic.
- Verify always-on-top/z-order behavior without blocking the source application.

## Hotkey

Prefer OS-supported global shortcut registration over polling/hooks. Detect and report registration conflicts.

## Compatibility evidence

Maintain representative integration checks as behavior matures: Notepad, Chromium browser, VS Code, a selectable PDF reader, and other high-value applications discovered by users.

## Safety

Keep COM initialization/lifetime and raw HWND handling inside focused modules. Convert native results into safe Rust types before crossing the adapter boundary.
