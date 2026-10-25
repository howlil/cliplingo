# ClipLingo — Tauri v2 + Rust + Native Windows Engineering Contract

Status: **research-based target with a partial F0 Tauri shell/native preview scaffold** (2026-10-26). Translation and performance are NOT implemented or verified.
Decision: [ADR-0003](decisions/0003-latency-first-dual-mode-translation.md) refines [ADR-0002](decisions/0002-tauri-rust-windows-hybrid.md) for a native-first tooltip and opt-in online translation; Rust-first [ADR-0001](decisions/0001-rust-first-native-windows.md) remains valid. F0 feature branch now contains Cargo/Tauri Windows shell, static WebView Settings, native Win32/GDI preview, and CI. No capture, translation model, official online provider, native E2E evidence or release exists. The documentation-only `master` statement describes the prior baseline.
Source of truth: [Product contract](product.md), root [DESIGN.md](../DESIGN.md), root [AGENTS.md](../AGENTS.md). The archived C++ project is preserved at branch `archive/pre-rust-clean-slate-20261026`, commit `24b8e3c30f390e6d571e2b15d1e1a56ba4781846`. It is a historical reference, not a current runtime.

## 1. Job and constraints (before technology)

A user reading English text in a Windows browser/PDF/editor selects a span or invokes a configured hover target, presses `Ctrl+Alt+T`, receives **only its Indonesian translation** in a compact text-only popup above the **frozen hotkey-time cursor**, and keeps focus in the source application. The utility lives in the tray and works fully offline **after the local model is installed**; optional online translation requires explicit provider-specific user consent. Manual region OCR and opt-in clipboard are separate explicit capture paths, never continuous polling. Settings is an accessible six-section window. Language model/update downloads, when deliberately initiated, are allowed online; **captured text must never go to an external translation service**.

Non-negotiable behavior is owned by `product.md` and `DESIGN.md`, **not** by a preference for Tauri, WebView, React or Rust. Product still requires word/sentence/paragraph scope, no fake result for failed/no capture, latest-request-wins, excluded app policy, mixed-DPI work-area placement, no focus theft and accessibility.

Architecture drivers: reduce application-owned native complexity with Rust, enable polished Settings and straightforward installer/updater via Tauri v2, while retaining low idle usage, low hotkey-to-result latency and OS capture/inference reliability. Tauri does **not** provide UI Automation, desktop OCR or an offline translation engine merely by being adopted. WebView2 costs process/memory; there is **no measured evidence yet** that this hybrid is faster, smaller or more reliable than archived Win32 C++.

## 2. Stack selection and decisions

| Concern | Target | Status, qualification |
| --- | --- | --- |
| Desktop shell, tray, lifecycle | Tauri **v2**, Rust process; Tauri tray/menu and single-instance plugin | Selected; use no persistent main/dashboard window |
| Settings and preview | React + TypeScript + Vite, hosted by **WebView2 on Windows** | Selected rendering candidate for the six approved settings sections; Settings window created hidden/on demand |
| Result popup | **Reuse a Rust-owned Win32/DirectWrite HWND**; WebView2 only for Settings | New default due to low-latency/low-idle goals; Windows focus, DPI, text fitting and accessibility remain acceptance gates |
| Global shortcut | `tauri-plugin-global-shortcut` registered by Rust owner | Selected capability; registration collision/unregister must be observed |
| System text selection/hover | Windows UI Automation COM through Rust `windows` bindings, dedicated MTA worker/helper | Required native boundary; alternate wrapper crates need validation |
| User-drawn OCR region | Rust-managed bounded native screen capture + local OCR | Optional later slice; **Windows.Media.Ocr requires desktop package identity** per Microsoft docs, so **do not assume it works with default NSIS/unpackaged installer**. Prototype compatible Tesseract/alternative with licensing/dependency review; MSIX is a separate deployment decision |
| EN→ID inference | TranslationRouter: **offline local EN→ID** or **user-authorized online provider** | OPUS-MT EN→ID is offline candidate; official Azure/Google provider is online candidate. No runtime, accuracy, price or latency winner is yet proven |
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
    |        +--> TranslationRouter [mode, network consent, bounded memory cache]
    |        |         +-> OfflineEngine [ModelManager, local tokenizer+CPU model]
    |        |         +-> OnlineProvider [documented HTTPS API, explicit consent]
    |        |
    |        +--> PopupOwner [native Win32/DirectWrite text-only HWND; one owner]
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

### 4.1 Selection → configured offline/online translation → popup (critical vertical slice)

1. **Launch:** Tauri boots into tray; register `Ctrl+Alt+T` (or report shortcut conflict), single-instance protection; Settings is available independent of model readiness. Background startup does not eagerly render two hidden WebViews or repeatedly poll input.
2. **Trigger:** Rust shortcut handler captures foreground HWND/process identity, cursor in **physical virtual-screen pixels**, monitor and DPI once; increments monotonic generation. Reject if paused, source is own app, excluded/unresolvable, or secure desktop/permission rules deny.
3. **Capture:** dispatch only to a dedicated initialized COM MTA worker; call UI Automation for actual selected ranges first. A missing selection is `NoSelection`; do not silently read clipboard or synthesize Ctrl+C. Hover fallback is separately configured and must validate text geometry, not simply choose nearest UIA word.
4. **Bound:** validate nonempty text/UTF, max units, scope provenance; snapshot request and current preferences; avoid sending raw source to any JS view or disk.
5. **Translate:** Router checks permitted bounded RAM-only exact cache, then the selected provider: offline model+tokenizer, or an explicitly authorized official HTTPS provider. Online is off by default; an optional user-consented smart fallback is a separate setting. Return typed `ModelMissing | ModelInvalid | NotAuthorized | NetworkUnavailable | RateLimited | Timeout | Failed`; never fabricate output.
6. **Present:** reject if generation stale, locked, settings/policy changed or result empty. Measure final output with actual font/DPI, intrinsic width → cap at max 420 DIP → wrap → cap height at max 300 DIP, position above frozen cursor with default 12 DIP gap, flip below/clamp into monitor work area (including negative coordinates), show **without activating** the original app.
7. **Dismiss:** explicit policy and escape/action; on new hotkey hide/replace old popup per coordinator; no translation history by default.

**Popup constraint:** use one Rust-owned Win32/DirectWrite native result HWND, not one Tauri WebviewWindow per translation. Implement nonactivating display with appropriate Win32 styles and calls, intrinsic measurement, mixed DPI and a readable screen-reader representation. These must be verified on real Windows; native alone does not prove them. Tauri WebView popup remains a *comparison experiment*, not the first shipping implementation.

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
- Do not emit source text or captured OCR pixels in global Tauri events. The Rust-owned native result popup receives only translated target text; Settings frontend receives statuses/settings, not captured content. Document privacy boundaries for devtools/logging and crash reports.
- Settings persisted with versioned schema and atomic replacement; corrupted config produces safe fallback plus recoverable indication; no silently destructive migrations.

## 5. Binding, deployment and performance constraints

| Boundary | Constraint / evidence needed | Implementation consequence |
| --- | --- | --- |
| Tauri/WebView2 | Runtime present on Windows 11; Tauri installer can bootstrap WebView2, even if the app itself performs offline inference | Test clean Windows VM with/without runtime, online installer and chosen offline install scenario; no claim fully offline installer by default |
| Tray resident | Hidden WebViews may have measurable idle memory/CPU, and Tauri worker processes may persist | Measure Settings closed, model cold, model warm, popup hidden/visible; do not claim "zero cost" |
| UIA/COM | Microsoft recommends separate **non-window-owning MTA** thread for desktop-wide UI Automation | Keep UIA away from Tauri main/event loop and JS; test deadlocks, stalls, COM lifetime and privileges |
| Native popup | Raw HWND, DirectWrite and Tauri host must share one UI-thread/window-lifetime contract; mixed DPI and nonactivation are not automatic | Verify focus, accessible text, intrinsic sizing and negative-screen coordinates on real Windows; compare WebView popup only as a performance experiment |
| Rust native model | CTranslate2 and SentencePiece are C++ dependencies; no guaranteed compatible first-party stable Rust ABI | Prototype C ABI wrapper or isolated helper; document allocators, panics, exceptions, DLLs, CPU instruction sets, license and pack format |
| OCR | Microsoft's `Windows.Media.Ocr` desktop package-identity restriction | Avoid assuming NSIS portable exe can invoke it; investigate unpackaged-compatible local OCR backend and MSIX as separate option |
| User safety | Protected/elevated/UAC/secure desktop may block capture legitimately | No forced elevation, driver, hook/injection or bypass; actionable status only |
| Offline/online vs update | Offline-only must work without network after installing a model; online-only requires explicit provider upload permission; updater/model download are separately authorized | Isolate Rust HTTPS online adapter from local translator and updater; no silent fallback or raw-text telemetry |
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
| R1 — shell + native UI proof | Initialize Tauri v2 Cargo + lazy React/TS Settings, tray/single-instance/hotkey, **reusable native Rust Win32 tooltip** | Real Windows tray/hotkey, close-to-tray, no focus steal, mixed-DPI, accessible text and measured process-tree idle RAM; compare a WebView popup only to falsify the native-first choice |
| R2 — selection + policy | Rust coordinator + UIA MTA worker, selection only, excluded-app policy | Latest-wins, exact selection, no-selection silent, foreground focus and provider timeouts |
| R3 — EN→ID offline engine | Proven local model/tokenizer + management and typed errors | Offline valid fixture, cold/warm evidence, crash/failure behavior, model integrity and licensing |
| R4 — result/presentation | Native popup final acceptance + Settings preview/parity; optional Online-only via official provider after consent | Correct translation only; intrinsic sizing, mixed DPI, screen reader, no activation; privacy/network failure cases |
| R5 — explicit enhancements | Geometry-checked hover, opt-in clipboard, manual OCR | Only explicit capture modes, no periodic capture, protected content respected; correct OCR backend for installer |
| R6 — distribution | Windows NSIS, versioning, signed updater, GitHub Release, installed E2E | Verified download/install/update/rollback on clean Windows, runtime dependencies, signed metadata/asset availability |

**Critical path:** R0 → R1 **native popup + idle feasibility** → R2 → R3 offline → R4 → R6. The optional online provider may be implemented after offline proof without delaying the core offline release. R5 additions do not block selection-only release unless product scope changes. Do not add model/update backend before shell+popup viability is shown.

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


## 9. Performance-first online + offline research (2026-10-26)

This section and [ADR-0003](decisions/0003-latency-first-dual-mode-translation.md) supersede the **offline-only translation and WebView-popup-first** assumptions in sections 2–8. Historical R0–R6 milestones remain a research log, not the new implementation priority. **No application, model benchmark or Windows executable exists in the repository yet.**

### 9.1 What the references actually establish

- **Mouse Tooltip Translator** (ttop32): browser DOM/PDF.js/OCR + tooltip + remotely called Google/Bing/DeepL/Papago translation services. The upstream documentation says it requires internet; **do not assume it embeds a local neural model**. Browser extensions have direct page/DOM access; Windows-wide ClipLingo requires UI Automation and manual OCR fallback.
- **Firefox Translations**: a distinct offline pattern, downloading directional on-device Marian/Bergamot models; Indonesian is supported. This validates an architectural option, **not** an SDK automatically usable by a Tauri app.
- **Helsinki-NLP/opus-mt-en-id**: initial offline candidate, Apache-2.0, Marian + SentencePiece. PyTorch weight file is approximately 291 MB; Hugging Face repo includes other weight formats (~586 MB in the repo). Neither size proves the eventually converted/quantized CT2 pack size or installed RAM. Benchmark CPU cold/warm, quality and tokenizer parity; do not mistake an ONNX encoder for a complete autoregressive translator.
- **NLLB-200-distilled-600M** is a much larger alternative; many converted versions carry CC-BY-NC-4.0 from the base weights. Do not ship commercially without resolving original and conversion licenses.
- **Online provider**: start with *one* documented official EN↔ID API (Azure Translator or Google Cloud Translation), not unofficial Google Translate web-scraping endpoints. No API contract, cost or latency is proven until a real integration test.

### 9.2 Required target performance flow

```text
WM_HOTKEY
 -> Rust coordinator increments generation and freezes foreground/cursor/monitor/DPI
 -> exclude/deny policy BEFORE reading text
 -> UIA selection on dedicated no-window COM MTA worker (geometry-check hover separately)
 -> bounded UTF validation
 -> TranslationRouter
       -> RAM-only exact LRU HIT -> target
       -> MISS -> configured local OR explicitly permitted online provider
 -> stale generation and policy revision check
 -> reusable native Rust HWND with DirectWrite measurement
 -> nonactivating target-text-only popup at frozen anchor
```

**No WebView initialization, network model download, OCR, new process, repeated tokenizer/model creation or unbounded thread spawning on the hotkey path.** A cache hit still obeys privacy/capture policy and freshness. Rust owns result display, not a background JavaScript listener. Use Tauri/WebView2 only for lazy-created Settings; close/destroy its renderer when not in use if measured and safe. Count child WebView2 browser/renderer/GPU processes when profiling, not just host process memory.

### 9.3 Mode and consent contract

| Mode | Model requirement | Network behavior | Failure |
| --- | --- | --- | --- |
| **Offline only (default)** | Locally installed verified pack | No translation network traffic | Missing/corrupt model -> repair status in Settings; no fake popup |
| **Online only (explicit)** | No local model needed | Send selected text via TLS to named consented provider | No internet/unauthorized/rate-limited -> no popup + actionable status |
| **Smart (explicit)** | Prefer local | Can use online fallback only if separately opted in for this provider and this class of failure | Never upload for excluded source, denied capture, corrupt model or revoked consent |
| **Exact RAM cache hit** | None on hit | None on hit | Must match mode/provider/model version + language + segmentation + policy context |

Do not activate online merely because the user is connected to the internet. Online operation is a **privacy choice**, not a faster default. Consent copy must make text upload, chosen vendor and possible API cost explicit. Revoking consent invalidates queued generations; it cannot retract already transmitted data. Source-app exclusions and unknown/protected foreground fail closed in **both** modes. Raw OCR screenshots remain local; no web payload beyond minimal selected text and language pair without separate authorization.

### 9.4 Translation model state, cache and native boundaries

- **Model lifecycle:** Unloaded -> Loading -> Ready -> Idle/Evict -> Unloaded. Only one authoritative model/session owner; use bounded workers and measure memory-vs-first-hit latency. Model is *not* loaded on tray boot or Settings open unless benchmarking justifies warm retention. No claim that a cold model starts instantly.
- **Model installation:** default installer remains small; model is an optional separately installed pack (user action, size/license shown). Download and validate manifests, content checksums, paths, tokenizer pairing, CPU feature support, package ABI and atomic activation; never partially replace active pack. Support *optional local import* of verified pack without network.
- **Offline engine:** evaluate CTranslate2 CPU int8 with a small audited C ABI adapter and compatible SentencePiece assets; compare with viable Rust-callable runtime. Do not assume native Rust bindings or converting raw ONNX files supplies generation/tokenizer correctness. Keep FFI panic/exception/allocator ownership, invalid input sizes and DLL trust explicit.
- **Exact cache:** memory-only bounded LRU, disabled for privacy mode, cleared on exit. Key by source normalized text + EN/ID pair + selected/hover scope + provider/model ID/version + context policy; do not cross-serve model/provider versions. Cache successful validated nonempty translations only, never plaintext disk history. No speculative hovered-text prefetch or background page reading.
- **Online engine:** user-selected provider's documented HTTPS endpoint, typed errors, bounded payload/response size, connect/overall timeout, safe redirect/host policy, rate limit and bounded retry for safe cases only. Per-provider circuit health. No auto-switch to another vendor without consent.
- **Credentials:** no service/API key embedded in the public EXE. Evaluate BYOK supported provider auth; store in Windows Credential Manager or equivalent OS-protected credential store, not Settings JSON, JS localStorage, logs or GitHub. A universal vendor-funded online API would require a separately scoped authenticated proxy, rate limits, abuse protection and billing; no such service exists here.

### 9.5 Correctness and native constraints

| Constraint | Proof required |
| --- | --- |
| Native popup focus | GetForegroundWindow/keyboard focus unchanged; test Chrome, Edge, Notepad, PDF, mixed DPI and multi-monitor boundaries |
| UIA hang | A timeout does not cancel COM. Dedicated MTA thread, finite pending work, supervised helper process only if real hangs require crash isolation |
| Native memory | Measure whole process tree (WebView2 and optional helpers), idle closed/open Settings, cold/warm model, 1000+ hotkey cycles and long soak |
| Unicode text | DirectWrite intrinsic wrap/measure, emoji/combining/BiDi support, keyboard-only and screen-reader accessible text |
| Model quality | Curated EN→ID samples: ambiguous words, proper nouns, software/finance terms, idioms, long paragraphs, numbers, mixed language; judge output beyond timing |
| Online privacy | Consent off => zero network text requests, unknown/excluded apps denied, provider auth/errors, no raw-text logging or silent retry/upload |
| Cache | Same input/provider/version/scope yields cache hit; policy change, provider switch and model upgrade cannot use wrong result |
| Windows install | WebView2 runtime provision; NSIS signed updater and GitHub Release verified by installed upgrade, not just CI green |

**Latency accounting:** instrument t0 hotkey received, t1 capture done, t2 cache/route, t3 inference/HTTP done, t4 target visibly presented (and verify DWM/window paint, not only method invocation). Report t4−t0 p50/p95/p99 for cold/warm across word, sentence and paragraph; report component contributions, CPU/power plan/ISA/model hash, model-resident RAM and all WebView2 children. User goals "fast" and "light" are not measured budgets. Set numeric go/no-go budgets only after baseline runs on low- and mid-spec Windows machines.

### 9.6 Milestones and gates

- **F0 — native UI/idle feasibility:** Tauri tray + lazy Settings, Rust native text-only popup (one HWND), no-focus-steal, real Windows mixed-DPI + process-tree RAM; compare Tauri WebView popup and document why one wins.
- **F1 — Windows capture:** MTA UIA selection + exclusions, generation, frozen cursor, responsive tray under stuck provider.
- **F2 — offline baseline:** OPUS-MT model+tokenizer install/validation, CPU inference, offline native E2E, model quality and cold/warm profile.
- **F3 — optional online:** official one provider, explicit consent/BYOK and OS secret storage, network failure/retry and privacy E2E, exact RAM LRU.
- **F4 — feature completion:** Settings six sections, accessible native popup, hover segmentation, opt-in clipboard and manual OCR after capture semantics proven.
- **F5 — production:** one NSIS installer, verified WebView2 runtime, signed updater, model package, real GitHub Release and clean-VM update/recovery proof.

**Critical path:** F0 → F1 → F2 → F4 → F5. Online F3 can be delivered after F2; it must not delay a working offline path or silently introduce service costs.

### 9.7 Sources (revalidate versions/licenses before coding)

- Mouse Tooltip Translator description: https://github.com/ttop32/MouseTooltipTranslator/blob/main/doc/description.md
- Mouse Tooltip Translator offline limitation: https://github.com/ttop32/MouseTooltipTranslator/blob/main/doc/intro.md
- Firefox on-device Bergamot/Marian: https://firefox-source-docs.mozilla.org/toolkit/components/translations/resources/01_overview.html
- OPUS MT EN-ID model/license/tokenizer: https://huggingface.co/Helsinki-NLP/opus-mt-en-id
- NLLB-derived noncommercial weights example: https://huggingface.co/luigi000/nllb-200-distilled-600M-ct2-int8
- Microsoft WebView2 process/memory: https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model and https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance
- Microsoft UIA threading: https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading
- Official cloud language support: https://learn.microsoft.com/en-us/azure/ai-services/translator/language-support and https://docs.cloud.google.com/translate/docs/languages
- Signed Tauri updater: https://v2.tauri.app/plugin/updater/

## 10. F1 implementation delta and verification boundary (2026-10-26)

- `capture_policy.rs`: default deny list and fail-closed source identity; exact selection preservation up to 8,192 UTF-16 units. No configurable list or hover/clipboard/OCR yet.
- `capture.rs`: one named MTA thread initialized with `CoInitializeEx(COINIT_MULTITHREADED)`, COM-owned `IUIAutomation`, focused `TextPattern.GetSelection` with a single contiguous range, foreground root/PID attribution and password-field denial. Pending queue capacity **one**; no unbounded worker spawning.
- `CaptureCoordinator`: atomic generation increments before every trigger; late responses dropped. Captured string never escapes worker. Settings gets only typed status and count via existing `get_status` IPC. Exact foreground HWND and cursor are frozen at hotkey-time; source foreground is revalidated before/after UIA operation.
- **Expected limitations:** Chrome/Edge renderer processes can expose UIA child identity different from top window; if no verifiable root/foreground attribution is available, F1 denies (correctness prioritized over compatibility). Synchronous UIA provider calls can hang a thread and `try_send` only bounds the queue; a helper process is still conditional on real evidence.
- [F1 interactive acceptance](f1-acceptance.md) remains pending; GitHub Windows cargo compile/unit checks do not validate real UIA provider responses or focus semantics.
