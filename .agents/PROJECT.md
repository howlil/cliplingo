> **Devland migration note:** canonical structured project facts now live in `.devland/project.yaml`. This document is retained as project-local narrative and supporting evidence; if a factual conflict appears, verify repository reality and update the canonical model rather than maintaining two independent truths.

# Project Description

## Product

ClipLingo is a Windows-first native translation utility. A user selects text in another application, presses a global shortcut, and receives a translation in a small popup positioned near the selected text or cursor.

The normal translation path must work offline after the required language pack is installed.

## Primary use case

1. User selects real text in a browser, PDF reader, document editor, IDE, chat application, or other Windows application.
2. User presses the configured global shortcut, initially `Ctrl+Shift+T` unless it conflicts with system/application behavior.
3. ClipLingo captures the selection without stealing workflow focus.
4. A popup appears quickly near the selection/cursor and immediately communicates progress.
5. Translation runs locally.
6. The popup updates with the result; Escape dismisses it and keyboard-first actions remain possible.

## Product priorities

Priority order matters when constraints conflict:

1. Reliable text capture and predictable popup behavior.
2. Privacy and offline normal operation.
3. Low perceived latency and near-zero idle CPU.
4. Strong CJK quality, especially Chinese, Japanese, Korean, plus English and Indonesian.
5. Low memory/disk cost through model lifecycle management.
6. Broad language coverage.
7. Arbitrary language-to-language routing.

Architecture may support future arbitrary languages, but V1 does not need to ship every language.

## V1 scope

- Windows 10/11 desktop.
- Global shortcut.
- Selected real text; no OCR requirement.
- UI Automation capture with safe clipboard fallback.
- Popup and settings UI.
- Indonesian as the initial primary target language; English may be used as a routing pivot where justified by measured quality.
- CJK/English language-pack path designed first.
- Local CPU inference.
- Model installation/removal/version verification.
- Direct installer and GitHub Releases distribution when release quality is reached.

## Explicit non-goals for early iterations

- No Electron.
- No web backend or cloud translation service.
- No user accounts.
- No database until a persistent feature such as searchable history proves the need.
- No OCR/screen-reading pipeline in the first working translation path.
- No GPU requirement.
- No mobile/macOS/Linux implementation before Windows behavior is proven.
- No plugin marketplace, sync, collaboration, telemetry-heavy analytics, or language-count race.

## Quality targets

Initial engineering targets are hypotheses to benchmark, not marketing guarantees:

- Idle CPU: effectively 0% during normal inactivity.
- Hotkey → visible popup: target p95 under 100 ms on supported development hardware.
- Warm short-text translation: target p95 under 500 ms after a suitable model is loaded.
- Cold short-text translation: target p95 under 1 second where model size permits.
- Resident shell including pre-created popup: aim under 100 MB, then tighten using measurements.
- Selected text must never appear in normal logs or telemetry.

Targets may change only with benchmark evidence recorded in the iteration state or a relevant design decision.
