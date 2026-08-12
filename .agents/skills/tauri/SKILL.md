---
name: tauri
description: Use when changing the Tauri shell, windows, tray, capabilities, commands/events, updater, or packaging configuration.
---

# Tauri Skill

## Rules

- Tauri is a desktop host/presentation bridge, not the application core.
- Keep command handlers thin and delegate workflows to Rust core types.
- Prefer official Tauri capabilities/plugins when they solve the requirement cleanly; do not add a plugin automatically.
- Minimize WebView capabilities. Never expose arbitrary shell/process/filesystem access.
- Use commands for UI→Rust requests and narrow events/channels for Rust→UI state/progress.
- Do not introduce localhost HTTP for UI/native communication.
- Treat popup focus, show/hide, startup and memory behavior as benchmarkable desktop behavior, not ordinary web-page behavior.
- Settings can be on demand; popup may be pre-created if measurements justify residency.
- Keep release/updater settings consistent with `.agents/RELEASE.md`.

## Testing

Test Rust logic below Tauri independently. Use integration/E2E tests only for window lifecycle, IPC exposure, capabilities and actual Tauri behavior.

## References

- https://v2.tauri.app/
- https://v2.tauri.app/develop/calling-rust/
- https://v2.tauri.app/distribute/windows-installer/
- https://v2.tauri.app/plugin/updater/
