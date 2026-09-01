# Project

`PROJECT.md` is the canonical source for what ClipLingo is, why it exists, committed product behavior, scope, constraints, and material open product questions.

## Purpose

ClipLingo is a Windows-first native translation utility for translating selected text without leaving the user's current application. The user selects text, invokes a global shortcut, and receives a small translation popup near the selection or cursor.

Normal translation must work offline after the required language pack is installed.

## Primary experience

1. Select real text in a Windows application such as a browser, PDF reader, editor, IDE, or chat application.
2. Invoke the configured global shortcut; the initial default is `Ctrl+Shift+T` unless it conflicts.
3. Capture the selection without unnecessarily stealing focus.
4. Show a popup quickly near the selection/cursor and communicate progress immediately.
5. Translate locally.
6. Replace progress with the result; Escape dismisses the popup and keyboard-first actions remain possible.

## Product priorities

When constraints conflict, prefer in this order:

1. Reliable text capture and predictable popup behavior.
2. Privacy and offline normal operation.
3. Low perceived latency and near-zero idle CPU.
4. Strong CJK translation quality, especially Chinese/Japanese/Korean plus English and Indonesian.
5. Low memory/disk cost through explicit model lifecycle management.
6. Broader language coverage.
7. Arbitrary language-to-language routing.

Broad future capability must not expand current scope automatically.

## Committed V1 scope

- Windows 10/11 desktop.
- Global shortcut invocation.
- Selected real text; OCR is not required for the first product path.
- UI Automation selection capture with safe clipboard fallback.
- Popup and settings surfaces.
- Indonesian as the initial primary target language.
- English may be used as a routing pivot where quality/licensing/runtime evidence justifies it.
- CJK/English language-pack path first.
- Local CPU inference.
- Model install/remove/version/integrity lifecycle.
- Direct installer and GitHub Releases distribution when release maturity requires it.

## Data and privacy expectations

- Selected source text and translated text remain on-device in the normal translation path.
- Raw selected/translated content must not appear in normal logs or telemetry.
- Configuration and model metadata are local application data.
- No account or cloud service is required for normal translation.
- Network access is reserved for explicit model/update acquisition workflows.

## Deferred / non-goals

Do not promote these into current scope without new product intent:

- Electron.
- Cloud translation as the normal path.
- User accounts.
- Database storage before a real queryable persistent feature proves the need.
- OCR/screen-reading in the first working translation path.
- GPU requirement.
- macOS/Linux/mobile implementation before Windows behavior is proven.
- Plugin marketplace, sync, collaboration, telemetry-heavy analytics, or language-count competition.

## Quality hypotheses

These are engineering targets to measure, not product guarantees:

- Idle CPU: effectively 0% during normal inactivity.
- Hotkey -> visible popup: target p95 under 100 ms on supported development hardware.
- Warm short-text translation: target p95 under 500 ms after a suitable model is loaded.
- Cold short-text translation: target p95 under 1 second where model size/runtime permit.
- Resident shell including popup: aim under 100 MB, then tighten only from measurements.

Do not change these into release guarantees without benchmark evidence and an explicit maturity decision.

## Material open questions

- Final beta/stable latency and resource budgets.
- Windows code-signing provider/certificate strategy.
