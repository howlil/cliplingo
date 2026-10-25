# F1 Windows selection-capture acceptance — not yet run interactively

Scope: `Ctrl+Alt+T` -> foreground provenance -> UIA selection from dedicated MTA -> bounded data validation -> stale generation gate -> non-content Settings diagnostics. **No translation engine, cloud upload, clipboard fallback or result popup is used by this path.**

## Proofs implemented as Rust tests

- Deny self process, unknown PID/path and known password managers; process names compared case-insensitively.
- Reject empty/whitespace selection, NUL and selections over 8,192 UTF-16 units; preserve exact selection for allowed inputs.
- Most recent generation token invalidates earlier capture results; unknown telemetry code fails to idle.
- Existing F0 native geometry tests remain.

## Required interactive Windows host E2E — BLOCKED until performed

1. Launch debug build; open Notepad, type a known sentence, select just part, focus remains in Notepad; press `Ctrl+Alt+T`. In Settings diagnostics observe `selection_verified_no_translation_engine` and incremented count, **but no popup**. Never print selected text to logs/console.
2. Select nothing; press hotkey and observe `no_selection` without popup. Verify original source application's focus remains intact.
3. Open Chrome/Edge and a PDF viewer; verify selection is attributed to the correct foreground window. Some multi-process providers may be denied until origin can be proven. **Do not bypass fail-closed checks to inflate compatibility.**
4. Open a password input and a protected/elevated/unavailable source: deny without reading or logging contents. Exercise a blocklisted password manager and the ClipLingo Settings window.
5. Trigger two different selections A and B quickly; assert only B may update final status. Trigger hotkey repeatedly under slow provider and verify bounded work/no accumulating native threads.
6. Use App/Process Explorer to simulate a hung UIA provider. Verify the UI/tray remains responsive. Note: the worker thread itself may remain blocked. Full fault containment needs a supervised child process if this occurs in the field.
7. Move cursor and change foreground before worker reads: stale/mismatched provenance denies. No clipboard side effects (clipboard sequence number unchanged).
8. Lock/unlock session, Explorer restart, exit while capture is queued; ensure no raw-text persistence, no lingering popup, no new system-wide hook.

## Boundaries and known limitations

- `GetFocusedElement` plus selected `TextPattern` supports accessible text editors, not arbitrary canvas/PDF/hover content. Multi-range selections are currently denied. `CurrentNativeWindowHandle` and root-window attribution may not be provided by browser renderers.
- F1 worker is in process. A UIA provider stuck in a synchronous COM call **cannot be forcibly cancelled** by generation invalidation; queued requests are bounded to 1. A proven hang must drive a separate helper-process architecture.
- No supported user-editable excluded-app list, paused mode, hover capture, OCR, clipboard, model/tokenizer or cloud provider yet. These must not be claimed.
- Unit tests in GitHub-hosted Windows CI cannot prove a foreground app session or focus correctness. This document stays a pending verification checklist, not a test report.

## API basis

- https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading
- https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-understandingperformanceissues
- https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nn-uiautomationclient-iuiautomationtextpattern
