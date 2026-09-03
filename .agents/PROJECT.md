# Project

`PROJECT.md` is the canonical source for what ClipLingo is, why it exists, committed product behavior, scope, constraints, and material open product questions.

## Purpose

ClipLingo is a Windows-first native translation utility for translating selected text without leaving the user's current application. It runs as a background utility in the Windows notification area/system tray, validates the current selection when the global shortcut is invoked, and only then shows a compact translation popup near the selected text or cursor fallback.

Normal translation must work offline after the required language pack is installed.

## Primary experience

1. Launch ClipLingo; it remains available from the Windows system tray/notification area.
2. Select non-empty English text in a Windows application such as a browser, PDF reader, editor, IDE, or chat application.
3. Invoke the canonical global shortcut: `Ctrl+Alt+T`.
4. Validate/capture the selection before any translation window is shown.
5. If there is no valid selection, stop silently: no popup, no translation request, and no worker startup for that interaction.
6. If the selection is valid, show the popup near the selected text or cursor fallback and translate locally through the direct English -> Indonesian route.
7. The popup is a compact utility surface, can be dragged by its `EN → ID` header, and can be dismissed without terminating ClipLingo.
8. Open Settings from the tray to inspect route, shortcut, model state, install/remove the offline model, or explicitly quit ClipLingo. Closing Settings keeps the tray process running.

## Product invariants

- **Selection-gated UI:** a translation popup must not become visible until a valid, non-empty selection has been captured for the current request.
- **Silent no-selection:** `Ctrl+Alt+T` with no valid selection produces no visible UI and does not call the translator.
- **Direct primary route:** the current production route is English -> Indonesian. Japanese/pivot routing is not part of the active product path.
- **Tray ownership:** ClipLingo is a background Windows utility; the system tray is its persistent application home, while Settings is a control surface rather than a dashboard.
- **Local normal path:** source text and translation remain on-device during normal translation.
- **Restrained UI:** translation content outranks branding/decorative chrome. The product must not default to generic glass/gradient/bento/AI visual treatments.

## Product priorities

When constraints conflict, prefer in this order:

1. Reliable selection detection and predictable popup behavior.
2. Privacy and offline normal operation.
3. Low perceived latency and near-zero idle CPU.
4. Strong English-to-Indonesian translation quality.
5. Native Windows utility behavior, including tray lifecycle and unobtrusive settings.
6. Low memory/disk cost through explicit model lifecycle management.
7. Broader language coverage only after the primary path is proven.

Broad future capability must not expand current scope automatically.

## Committed current scope

- Windows 10/11 desktop, x64 first.
- System tray/notification-area lifecycle with Settings and Quit.
- Canonical global shortcut `Ctrl+Alt+T`.
- Selected real text; OCR is not required for the current product path.
- UI Automation selection capture with safe clipboard fallback where supported.
- Selection validation before popup visibility.
- Draggable compact translation popup.
- Indonesian as the primary target language.
- Direct OPUS-MT English -> Indonesian local CPU inference.
- Model install/remove/version/integrity lifecycle.
- Direct installer and GitHub Releases distribution for alpha builds.

## Data and privacy expectations

- Selected source text and translated text remain on-device in the normal translation path.
- Raw selected/translated content must not appear in normal logs or telemetry.
- Configuration and model metadata are local application data.
- No account or cloud service is required for normal translation.
- Network access is reserved for explicit model/update acquisition workflows.

## Deferred / non-goals

Do not promote these into current scope without new product intent:

- Japanese or other additional production routes.
- Automatic language detection/routing.
- OCR/screen-reading.
- Cloud translation as the normal path.
- Translation history.
- Configurable shortcut UI.
- User accounts, sync, or collaboration.
- GPU requirement.
- macOS/Linux/mobile implementation before Windows behavior is proven.
- Plugin marketplace, telemetry-heavy analytics, or language-count competition.

## UI quality direction

ClipLingo should read as a native Windows utility, not a generic SaaS/AI surface:

- compact spacing and restrained radius;
- semantic typography and information hierarchy;
- native light/dark adaptation;
- modest, functional elevation only where a floating popup needs separation;
- no decorative gradient, glow, glass blur, bento/card stacks, oversized radius, hero copy, or unnecessary iconography;
- popup header communicates `EN → ID`, not redundant product branding;
- Settings uses simple rows/dividers rather than nested cards.

## Quality hypotheses

These are engineering targets to measure, not product guarantees:

- Idle CPU: effectively 0% during normal inactivity.
- Hotkey -> visible popup after a valid selection is captured: target p95 under 100 ms on supported development hardware.
- Warm short-text EN -> ID translation: target p95 under 500 ms after the model is loaded.
- Cold short-text translation: target p95 under 1 second where model size/runtime permit.
- Resident shell including tray/popup: aim under 100 MB, then tighten only from measurements.

Do not change these into release guarantees without benchmark evidence and an explicit maturity decision.

## Material open questions

- Final beta/stable latency and resource budgets.
- Windows code-signing provider/certificate strategy.
