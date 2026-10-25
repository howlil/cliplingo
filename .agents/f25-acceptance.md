# F2.5 — Native Windows pipeline and performance acceptance

Status: F2 native headless inference previously passed. F2.5 adds latest-pending work scheduling and a reproducible cold/warm benchmark. **GitHub-hosted CI does not establish interactive GUI correctness.**

## Automated proof matrix

| Invariant | Faithful verification |
| --- | --- |
| Rapid A/B/C/D hotkeys during inference must retain only latest *pending* request | Deterministic blocked-consumer race test in latest_slot.rs |
| Pending sensitive input is discarded when worker closes | LatestSlot close/wakeup tests |
| Native translator still loads the verified CTranslate2 model | F2 Windows native model smoke |
| Distinguish model-loading cold path and warm inference | Headless model smoke --benchmark: 1 cold + 12 warm fixed public fixtures, p50/p95/max |

## Interactive Windows acceptance — NOT yet verified

1. Install verified local EN→ID model; run ClipLingo with the optional offline-ct2 feature and MSVC static CRT.
2. In Notepad, select "Good morning."; press Ctrl+Alt+T; confirm the Indonesian result in the text-only popup. Compare original foreground HWND and keyboard focus before and after. Repeat for Chrome and a real PDF viewer. Deny inaccessible/password fields.
3. Rapidly request text A then B. The latest valid text B must be shown, never A. Measure UIA capture, scheduling, cold/warm engine and actual DWM-visible paint separately.
4. Test 100%, 125%, 150%, 200% DPI, monitor edges, negative-coordinate displays and accessibility.
5. Simulate stalled UIA provider; event-loop/tray must remain responsive. Cancellation of already-running COM work is not guaranteed and may require a helper process.
6. Run process-tree memory samples in Windows PowerShell, with Settings closed/open and model warm/evicted. Record CPU, Windows, toolchain, model revision and comparable activity. The F2.5 script does not record selected text:

~~~powershell
.\scripts\measure-f25.ps1 -Phase idle-tray -Samples 30 -Output f25-idle.json
.\scripts\measure-f25.ps1 -Phase settings-open -Samples 30 -Output f25-settings.json
.\scripts\measure-f25.ps1 -Phase warm-model -Samples 30 -Output f25-warm.json
# Run after at least 90 seconds of inactivity
.\scripts\measure-f25.ps1 -Phase evicted -Samples 30 -Output f25-evicted.json
~~~

**Interpretation:** CI headless cold time includes worker scheduling and model hash verification/loading/inference but excludes hotkey, UIA and DWM paint. Warm percentile is based on twelve fixed sequential English fixtures; it is *not* representative application p95. There are no validated production performance thresholds until measured on representative Windows desktops.
