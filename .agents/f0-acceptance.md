# F0 Windows acceptance log — pending execution

This is a narrow manual/native evidence checklist, not a success report.

## Setup
- Windows 11 x64 interactive user session; record OS version, CPU, GPU, RAM and WebView2.
- Build debug Cargo scaffold; open Chrome/Notepad with a focused text field.
- Start `scripts/measure-f0.ps1` with Settings closed, repeat with Settings open and popup preview.

## Evidence required
1. Register `Ctrl+Alt+T`: accepted or explicit conflict status; no generated fake translation.
2. `Ctrl+Alt+P` while Chrome/Notepad is foreground: compare foreground HWND and GUI thread focus before/after. Must remain unchanged. Confirm popup never appears in Alt+Tab.
3. Preview fixed word-length and paragraph-length text (paragraph is future controlled test mode): measure actual width/height and text accessibility. Current GDI F0 is *not* full acceptance.
4. Test 100%, 125%, 150%, 200% displays, monitor edges, negative coordinates; screenshot/measure pixel placement against logical DIPs.
5. Repeated previews, Settings open/close, tray quit, Explorer restart, sleep/lock; check handles/processes do not leak and stale popup disappears.
6. Record private bytes, working set and child WebView2 processes. No arbitrary pass thresholds until representative comparison.
7. Run compiled Windows native test binary from fresh clone, inspect actual failure and fix before merging any shipping claim.

No COM UIA, model, provider, OCR, installer or updater test is meaningful at F0 because those components do not exist.
