# Iteration State

Last updated: 2026-08-13

## Iteration 0 — Engineering foundation

**Status: DONE**

Established before implementation:

- project scope and constraints
- target Tauri/Svelte/Rust/C++ architecture
- dependency and clean-code rules
- TDD and Git workflow
- release/version/package-channel strategy
- role-specific engineering skills

No application code exists yet; therefore no architectural choice below should be treated as benchmark-proven.

## Current iteration — Iteration 1: Windows interaction spike

### Goal

Prove the core interaction with **no real ML dependency yet**:

```text
select text in another Windows app
 -> global hotkey
 -> capture text
 -> position popup near selection/cursor
 -> show a deterministic fake translation
 -> dismiss cleanly
```

### Why this is first

Cross-application selection capture, focus behavior, DPI/multi-monitor positioning, and perceived popup latency are higher product risks than wiring an ML model.

### Intended technical slice

- scaffold Tauri 2 + Svelte 5 + TypeScript + Vite
- establish Rust core boundary separate from Tauri commands
- register the global shortcut
- implement UI Automation selection capture
- add clipboard fallback without corrupting the user's clipboard
- pre-create/show/hide the popup as needed
- fake translator behind a `Translator` boundary
- measure hotkey → popup latency and idle footprint
- integration checks with at least Notepad, one Chromium browser, VS Code, and a selectable PDF reader available in the test environment

### Acceptance evidence

- repeatable demo from external app selection to popup
- no app crash when selection is unavailable
- no selected text in logs
- stale hotkey requests cannot overwrite newer popup state
- initial latency/memory measurements recorded

## Next iteration candidate — Iteration 2: isolated inference worker

Only after Iteration 1 works:

- define minimal versioned Named Pipe protocol
- build worker supervisor in Rust
- build C++ worker skeleton
- first real CTranslate2 model smoke test
- measure cold/warm latency and model memory

Do not begin model routing, multi-language pack management, WinGet/Chocolatey publication, OCR, or history database before the preceding slice proves its boundary.

## Open decisions requiring evidence

- Exact popup WebView residency strategy: pre-created hidden window vs on-demand creation.
- First production-quality translation model/language direction and its license.
- Direct CJK→Indonesian model vs English pivot route quality.
- Final idle-memory and translation-latency release budgets.
- Windows code-signing provider/certificate strategy.
