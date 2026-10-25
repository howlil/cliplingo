# ClipLingo — Tauri v2 + Rust + Native Windows Engineering Contract

Status: **research-based target architecture, NOT implemented or benchmarked** (2026-10-26).
Decision: [ADR-0002 — Tauri v2 shell, Rust-owned native behavior](decisions/0002-tauri-rust-windows-hybrid.md) supersedes the "avoid Tauri by default" portion of [ADR-0001](decisions/0001-rust-first-native-windows.md). Rust-first and the original product acceptance remain. No new executable, Cargo project, WebView UI, model package or CI currently exists on `master`.
Source of truth: [Product contract](product.md), root [DESIGN.md](../DESIGN.md), root [AGENTS.md](../AGENTS.md). The archived C++ project is preserved at branch `archive/pre-rust-clean-slate-20261026`, commit `24b8e3c30f390e6d571e2b15d1e1a56ba4781846`. It is a historical reference, not a current runtime.

## 1. Job and constraints (before technology)

A user reading English text in a Windows browser/PDF/editor selects a span or invokes a configured hover target, presses `Ctrl+Alt+T`, receives **only its Indonesian translation** in a compact text-only popup above the **frozen hotkey-time cursor**, and keeps focus in the source application. The utility lives in the tray and works fully offline **after the local model is installed**. Manual region OCR and opt-in clipboard are separate explicit capture paths, never continuous polling. Settings is an accessible six-section window. Language model/update downloads, when deliberately initiated, are allowed online; **captured text must never go to an external translation service**.

Non-negotiable behavior is owned by `product.md` and `DESIGN.md`, **not** by a preference for Tauri, WebView, React or Rust. Product still requires word/sentence/paragraph scope, no fake result for failed/no capture, latest-request-wins, excluded app policy, mixed-DPI work-area placement, no focus theft and accessibility.

Architecture drivers: reduce application-owned native complexity with Rust, enable polished Settings and straightforward installer/updater via Tauri v2, while retaining low idle usage, low hotkey-to-result latency and OS capture/inference reliability. Tauri does **not** provide UI Automation, desktop OCR or an offline translation engine merely by being adopted. WebView2 costs process/memory; there is **no measured evidence yet** that this hybrid is faster, smaller or more reliable than archived Win32 C++.

## 2. Stack selection and decisions

| Concern | Target | Status, qualification |
| --- | --- | --- |
| Desktop shell, tray, lifecycle | Tauri **v2**, Rust process; Tauri tray/menu and single-instance plugin | Selected; use no persistent main/dashboard window |
| Settings and preview | React + TypeScript + Vite, hosted by **WebView2 on Windows** | Selected rendering candidate for the six approved settings sections; Settings window created hidden/on demand |
| Result popup | One reusable **frameless Tauri WebviewWindow** (HTML/CSS text only), with Windows native HWND adjustments if necessary | **Conditional prototype**, NOT approved as proven non-activating. If it fails focus/performance/accessibility proof, use a small Rust-owned Win32/DirectWrite result HWND instead; keep WebView2 for Settings only |
| Global shortcut | `tauri-plugin-global-shortcut` registered by Rust owner | Selected capability; registration collision/unregister must be observed |
| System text selection/hover | Windows UI Automation COM through Rust `windows` bindings, dedicated MTA worker/helper | Required native boundary; alternate wrapper crates need validation |
| User-drawn OCR region | Rust-managed bounded native screen capture + local OCR | Optional later slice; **Windows.Media.Ocr requires desktop package identity** per Microsoft docs, so **do not assume it works with default NSIS/unpackaged installer**. Prototype compatible Tesseract/alternative with licensing/dependency review; MSIX is a separate deployment decision |
| EN→ID inference | Rust model orchestration, local compatible model/tokenizer | Engine unresolved; short-list a *narrow, audited* CTranslate2/SentencePiece C ABI wrapper/sidecar and a proven Rust-consumable runtime only after accuracy, ABI, RAM, Windows packaging proof. Do not promise Rust-native inference/ONNX compatibility without demonstrating model parity |
| Local preferences | One versioned Rust SettingsStore, atomic write to user config directory; view is read-only projection | Selected ownership; plugin-store can be evaluated but must not create a second authoritative store |
| Distribution | Tauri Windows **NSIS** installer first candidate; verified WebView2 runtime provision | Candidate; only one user-facing installer format initially; MSI/MSIX not assumed |
| Updates | `tauri-plugin-updater`, **signed artifacts**, controlled GitHub Release or signed metadata endpoint | Later release feature; independent from offline inference; version, signature, install mode and recovery checks required |
| Security | Tauri v2 per-window capabilities, narrow Rust commands, CSP, no remote scripts | Mandatory; Tauri permissions do NOT sandbox Rust itself |

Do not introduce a remote HTTP backend, WebSocket daemon, telemetry SDK, plugin ecosystem, Redux/global JS state copy of domain truth, separate crate per concept or service locator. A helper **process** is admitted only when it solves a demonstrated UIA hang or native inference crash/ABI isolation failure; Tauri sidecars need separately versioned packaging and threat boundaries.

## 3. Responsibility and ownership graph

```text
User's other Windows app (browser / PDF / editor)
    | Ctrl+Alt+T / explicit OCR or clipboard tray command
    v
Tauri Rust Shell [one process, event loop, tray, global shortcut, lifecycle]
    |
    +--> RequestCoordinator [ONLY owner: generation, pause, trigger snapshot,
    |                        bounded in-flight work, cancellation/presentation gate]
    |        |
    |        +--> Policy + CaptureService [Rust: allow/deny before read]
    |        |         -> UIA dedicated MTA worker [Win32/COM, foreign provider]
    |        |         -> optional opt-in clipboard read
    |        |         -> optional region selector -> screen capture -> local OCR
    |        |
    |        +--> ModelManager [model manifest/version/integrity/local availability]
    |        |         -> Translator [one local EN→ID engine+tokenizer/session]
    |        |
    |        +--> PopupOwner [single authoritative shown/hidden text, frozen anchor]
    |                   -> A: reusable Tauri WebviewWindow (conditional)
    |                   -> B: Rust Win32/DirectWrite HWND if A fails acceptance
    |
    +--> SettingsStore [ONLY owner: validated, versioned preferences/exclusions]
    |        ^                  |
    |        | Rust commands    v
    |        +-- Settings WebviewWindow [React/TS projection + input; no direct capture]
    |
    +--> Updater/Installer integration [opt-in/version check; not translation hot path]
```

**WebView frontend is presentation and user input only.** Rust owns capture policy, model access, snapshot, concurrency and OS operations. No capture text goes through the Settings frontend. The result view receives only target translation, generation and *validated presentation metadata*. To avoid native UI failure from JS-event races, Rust must perform authoritative latest-generation checks immediately before showing the popup, regardless of renderer.

State owner / invariants:

| Mutable state | Sole owner | Must remain true |
| --- | --- | --- |
| Active generation, request queue, paused state | RequestCoordinator | Newer valid requests win; no stale display even if worker finishes |
| Cursor, foreground HWND, monitor and DPI at trigger | Immutable coordinator snapshot | No resampling cursor on completion; no wrong-monitor anchor |
| Excluded apps, capture toggles and settings | SettingsStore + validated policy projection | Fail closed for unqueryable live app identity; settings changes invalidate pending work |
| Model activated version & readiness | ModelManager | Verified immutable active model, never partial/corrupt files |
| Translation model/session memory | Translator worker | One bounded session owner; shutdown/upgrade cannot race with inference |
| Popup visibility/text/geometry | PopupOwner | No focus theft, taskbar entry or obsolete generation; only translated text |
| Native COM/HWND/FFI handles | Native adapter/owning worker | Correct thread/apartment lifetime and teardown |
| App/update install version | Bundle/updater | Signed manifest and installed version consistent, no downgrade by accident |

## 4. End-to-end (E2E) runtime contracts

### 4.1 Selection → offline translation → popup (critical vertical slice)

1. **Launch:** Tauri boots into tray; register `Ctrl+Alt+T` (or report shortcut conflict), single-instance protection; Settings is available independent of model readiness. Background startup does not eagerly render two hidden WebViews or repeatedly poll input.
2. **Trigger:** Rust shortcut handler captures foreground HWND/process identity, cursor in **physical virtual-screen pixels**, monitor and DPI once; increments monotonic generation. Reject if paused, source is own app, excluded/unresolvable, or secure desktop/permission rules deny.
3. **Capture:** dispatch only to a dedicated initialized COM MTA worker; call UI Automation for actual selected ranges first. A missing selection is `NoSelection`; do not silently read clipboard or synthesize Ctrl+C. Hover fallback is separately configured and must validate text geometry, not simply choose nearest UIA word.
4. **Bound:** validate nonempty text/UTF, max units, scope provenance; snapshot request and current preferences; avoid sending raw source to any JS view or disk.
5. **Translate:** ModelManager verifies installed pack and tokenizer compatibility, then translator performs offline EN→ID inference on bounded worker(s). Return typed `ModelMissing | ModelInvalid | Unsupported | Timeout | Failed` rather than fake translated output.
6. **Present:** reject if generation stale, locked, settings/policy changed or result empty. Measure final output with actual font/DPI, intrinsic width → cap at max 420 DIP → wrap → cap height at max 300 DIP, position above frozen cursor with default 12 DIP gap, flip below/clamp into monitor work area (including negative coordinates), show **without activating** the original app.
7. **Dismiss:** explicit policy and escape/action; on new hotkey hide/replace old popup per coordinator; no translation history by default.

**Popup implementation risk:** `alwaysOnTop`, `decorations:false`, `transparent`, `skipTaskbar`, `visible:false` are available Tauri window knobs, but **none alone proves `WS_EX_NOACTIVATE`/focus or correct accessibility** on Windows. Prototype Windows HWND semantics using `SWP_NOACTIVATE`/Win32 APIs if supported by Tauri's raw window handle; verify against Edge, Chrome, Notepad and PDF with input focus and keyboard selection. Avoid window-per-hotkey creation; precreate only if justified by measured latency versus resident memory. A native HWND fallback must not duplicate settings or capture logic.

### 4.2 Failure, concurrency and recovery

- Two hotkeys A then B: B becomes generation N+1; if A completes last, A is dropped at **Rust-owned presentation boundary**, not just hidden by JS. No unbounded new OS thread per trigger; bounded queue/backpressure, deterministic latest-wins.
- UIA provider hung: caller timeout cancels *interest* in the answer, **not necessarily the underlying COM call**. If provider hangs make process-unresponsive or exhaust capacity, escalate capture to a separately supervised helper process with timeouts and restart; never claim `tokio::timeout` can terminate COM.
- Missing/corrupt model: Settings displays an actionable install/repair state, result popup stays hidden. Installation stages files in temp area, verifies SHA-256 + tokenizer/model manifest/license, then atomically activates. Interrupted downloads never replace a working model.
- Region OCR: explicit tray action → dedicated selector overlay → freeze ROI and release cursor anchor → bound pixels and permissions → capture in memory → local OCR → same translation/presentation pipeline. Escape/right-click cancels; protected/secure content is denied. Never continuous OCR or persistent screenshots.
- Clipboard: separate user-initiated tray action only after opt-in, `CF_UNICODETEXT` read-only, sequence validation, bounded length, no source-app provenance claims. Ordinary hotkey must not access clipboard.
- Shutdown/restart: invalidate generations, unregister shortcut, hide/destroy popup on proper thread, close WebViews, stop native workers/helper, release COM/FFI resources. Restore tray after Explorer taskbar restart; session lock invalidates pending presentation. Closing Settings hides/destroys *Settings*, not the tray app; Quit is explicit.

### 4.3 Settings and frontend IPC

- UI: exactly six sections from `DESIGN.md` (General, Text Capture, Translation Popup, Appearance, Language Models, Privacy); keyboard/focus/high-contrast/text scaling.
- Commands are deliberately narrow: `get_settings`, `update_settings(validated_patch)`, `get_model_status`, `install_model(user_intent)`, `get_app_status`, `check_update(user_intent)`. Never expose `read_screen(x,y)`, arbitrary `exec`, filesystem paths, or `translate_selected_text` to a compromised renderer without actual authorization policy.
- Tauri command arguments are untrusted regardless of TypeScript typing. Backend validates constraints and source window label/role. Apply per-window v2 capabilities and deliberate command manifest restrictions; an empty/unspecified capabilities list is **not proof that all app commands are inaccessible**. Keep CSP restrictive, no CDN/remote JS injection.
- Do not emit source text or captured OCR pixels in global Tauri events. Result WebView gets only translated target text. Document privacy boundaries for devtools/logging and crash reports.
- Settings persisted with versioned schema and atomic replacement; corrupted config produces safe fallback plus recoverable indication; no silently destructive migrations.

## 5. Binding, deployment and performance constraints

| Boundary | Constraint / evidence needed | Implementation consequence |
| --- | --- | --- |
| Tauri/WebView2 | Runtime present on Windows 11; Tauri installer can bootstrap WebView2, even if the app itself performs offline inference | Test clean Windows VM with/without runtime, online installer and chosen offline install scenario; no claim fully offline installer by default |
| Tray resident | Hidden WebViews may have measurable idle memory/CPU, and Tauri worker processes may persist | Measure Settings closed, model cold, model warm, popup hidden/visible; do not claim "zero cost" |
| UIA/COM | Microsoft recommends separate **non-window-owning MTA** thread for desktop-wide UI Automation | Keep UIA away from Tauri main/event loop and JS; test deadlocks, stalls, COM lifetime and privileges |
| Tauri popup | Window visibility/coordinates differ between physical pixels and logical DIPs; `alwaysOnTop` does not prove no activation | Native focus/window-style/DPI/negative-coordinate proof required **before final popup choice** |
| Rust native model | CTranslate2 and SentencePiece are C++ dependencies; no guaranteed compatible first-party stable Rust ABI | Prototype C ABI wrapper or isolated helper; document allocators, panics, exceptions, DLLs, CPU instruction sets, license and pack format |
| OCR | Microsoft's `Windows.Media.Ocr` desktop package-identity restriction | Avoid assuming NSIS portable exe can invoke it; investigate unpackaged-compatible local OCR backend and MSIX as separate option |
| User safety | Protected/elevated/UAC/secure desktop may block capture legitimately | No forced elevation, driver, hook/injection or bypass; actionable status only |
| Offline vs update | Translation must work without network after installation; updater/model installation naturally requires permitted network | Separate network controls from translation path; deny raw text telemetry |
| Release | Tauri updater validates signature (cannot disable), latest JSON requires correct URL/signature/target; old repo had GitHub release 403 | Use constrained `contents:write` release job + confirmed token permissions, signed Windows NSIS asset, JSON metadata and fresh-VM upgrade/rollback proof |
| Performance | No numeric latency, RAM, package size or battery SLO has been measured in Rust/Tauri | Define p50/p95/p99 hotkey-to-visible-result for cold/warm, separate model warm/cold, idle private bytes and WebView2 child processes before budgets |
| Update mechanism | Installer/update artifact names differ from portable ZIP and require signing/private key protection | Pick only one initial installer channel, keep signing key in protected CI secret; never include key in source |

**Security threat surfaces:** untrusted selection/clipboard/OCR input; COM foreign provider; model archives/DLL loading; updater metadata; WebView XSS/IPC privilege boundary; unsafe Win32 handles. Validate and bound every crossing; keep telemetry off and avoid storing raw text or screenshot. Distinguish Tauri IPC restrictions from Rust command implementation authorization.

## 6. Verification strategy: smallest faithful proofs

**Pure tests (Rust unit / property):**
- latest-generation arbitration under A/B reorder; pause/privacy updates invalidate results
- word/sentence/paragraph segmentation and selected-text exactness
- exclusion policy fail-closed on unknown process, normalization and invalid config
- UTF conversion/length, ROI pixel bounds, monitor geometry clamping and negative coordinates
- model pack manifest/hash/version atomic staging/recovery; no false success

**Windows native integration (real Windows, not mocks):**
- MTA init, selection/hit-test across Chrome/Edge, Notepad, PDF reader; denied/protected window
- concurrent hotkeys with injected stall; source focus preserved, no bogus popup, deterministic shutdown
- popup Win32 `GetForegroundWindow` before/after, keyboard focus, top/bottom/sides, 100/125/150/200% mixed-DPI, screen reader and text overflow
- foreground excludes and clipboard origin caveat; region OCR cancellation/permissions
- native tokenizer/model on supported CPU, DLL load on clean VM, process/handle leak soak

**WebView/Settings E2E:** use `@wdio/tauri-service` / WebdriverIO for six-section navigation, persistence, model status, update UI, keyboard/high contrast and IPC guards. WDIO DOM checks **cannot prove** native no-focus-steal, real UIA capture, system tray restoration, install/update or COM hang containment. Pair these with targeted Windows-native host assertions and real app journey.

**Critical all-the-way user journey on an interactive Windows desktop:**
```text
Install app (clean VM)
 -> launch tray with Settings closed
 -> install/validate EN-ID model
 -> open Chrome/PDF/Notepad fixture
 -> select a known English sentence
 -> trigger Ctrl+Alt+T
 -> observe no focus change, no source text in popup
 -> verify offline Indonesian output + cursor-position geometry
 -> repeat on second monitor/scale
 -> disable network and repeat
 -> simulate fast A/B requests, model corruption, blocked UIA
 -> quit/restart/reopen settings; verify preferences
 -> install signed newer version and verify actual new version/assets
```

For E2E flakiness, use deterministic local fixtures and observable readiness rather than sleeps. Keep sensitive captured content out of screenshots/artifacts/logs. Do not promote browser-mock tests to real-native acceptance.

## 7. Incremental decision/proof gates (not existing historical product milestones)

| Phase | Scope | Acceptance and go/no-go |
| --- | --- | --- |
| R0 — research/contract | Tauri v2 hybrid decision, explicit constraints, ADR, proof plan | Docs consistent; no pretend runnable app |
| R1 — shell + UI proof | Initialize Tauri v2 Cargo + React/TS; tray/single-instance/hotkey/settings; prototype reusable Tauri popup | Real Windows tray/hotkey, close-to-tray, no focus steal and measurable idle/WebView cost; if popup fails, choose native HWND for result |
| R2 — selection + policy | Rust coordinator + UIA MTA worker, selection only, excluded-app policy | Latest-wins, exact selection, no-selection silent, foreground focus and provider timeouts |
| R3 — EN→ID offline engine | Proven local model/tokenizer + management and typed errors | Offline valid fixture, cold/warm evidence, crash/failure behavior, model integrity and licensing |
| R4 — result/presentation | Final chosen popup backend + settings preview/parity | Correct translation only; intrinsic sizing, mixed DPI, screen reader, no activation |
| R5 — explicit enhancements | Geometry-checked hover, opt-in clipboard, manual OCR | Only explicit capture modes, no periodic capture, protected content respected; correct OCR backend for installer |
| R6 — distribution | Windows NSIS, versioning, signed updater, GitHub Release, installed E2E | Verified download/install/update/rollback on clean Windows, runtime dependencies, signed metadata/asset availability |

**Critical path:** R0 → R1 **popup + idle feasibility gate** → R2 → R3 → R4 → R6. R5 additions do not block the minimal selection-only first release unless product scope changes. Do not add model/update backend before the shell+popup viability is shown.

## 8. Sources and decision validity (consulted 2026-10-26)

- Tauri v2 capabilities and command restrictions: https://v2.tauri.app/security/capabilities/
- Tauri IPC/commands (Rust ↔ WebView): https://v2.tauri.app/develop/calling-rust/
- Tauri config/window options: https://v2.tauri.app/reference/config/
- Tauri shortcut plugin: https://v2.tauri.app/plugin/global-shortcut/
- Tauri Windows installer & WebView2 options: https://v2.tauri.app/distribute/windows-installer/
- Tauri updater signing, artifacts & JSON: https://v2.tauri.app/plugin/updater/
- Tauri WebdriverIO testing: https://v2.tauri.app/develop/tests/webdriver/
- Microsoft UIA threading (MTA): https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading
- Microsoft UIA overview: https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-uiautomationoverview
- Microsoft Windows.Media.Ocr namespace/package identity: https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr
- Microsoft screen capture security/picker: https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture

Treat these as **documentation evidence**, not evidence the design compiles or performs. Before coding, pin and verify the exact Tauri/Windows/Rust plugin versions, Windows support matrix, native dependency licenses and model files. Do not copy these research assumptions into runtime claims.
