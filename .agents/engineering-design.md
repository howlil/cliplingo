# ClipLingo — Engineering Research & System Contract

Status: **proposed native C++ system architecture**, not implemented/benchmarked (2026-10-25).
Inputs: `.agents/product.md` (behavior), root `DESIGN.md` (approved UI), root `AGENTS.md` (SDLC rules).

**No existing code in this newly initialized GitHub repository was available to audit at document creation.** Do not describe an architecture as measured or migrated simply because it appears here.

## 1. Decision drivers and non-negotiable guarantees

- Windows-first resident tray utility; tiny idle host, event-driven hotkey, low hotkey-to-visible-translation latency.
- Local EN→ID offline inference using CTranslate2; no server/network during translation.
- Popup is native, text-only, content-sized, cursor-anchored, non-activating and DPI-correct.
- UI stays responsive when target apps/COM providers stall or inference fails.
- No stale request may overwrite newer output.
- No unsafe resource lifetime, fake success result, unconsented content logs, or protected-content bypass.
- Follow the existing product contract; **SOLID as ownership guidance, not a reason to add layers**.
- Numeric idle-memory, latency, package-size, CPU, and battery budgets are **unmeasured**; benchmark first.

## 2. Candidate architecture and decision status

**Candidate:** C++20/23 application host; Win32 message loop, notification icon, RegisterHotKey; COM UI Automation for selectable text; DirectWrite/Direct2D or equivalent minimal native renderer for the single target-text HWND; local CTranslate2 integration via its C++ API. Native Settings window conforms to `DESIGN.md`. Keep the language target and actual process topology conditional on benchmark/safety findings; neither C++ nor a single EXE is automatically faster.

```text
User hotkey / tray
    │
    ▼
Native Windows Host (message pump / one UI-thread owner)
    ├── Tray + settings lifecycle
    ├── Settings persistence (authoritative preferences)
    ├── Request Coordinator (authoritative generation ID + cursor snapshot)
    │     ├── Capture boundary
    │     │     ├── UI Automation selection / geometry-checked hover [MTA]
    │     │     ├── Clipboard fallback [opt-in]
    │     │     └── OCR region capture [on-demand]
    │     └── Translation boundary
    │           └── CTranslate2 model/tokenizer inference [CPU-first]
    └── Popup owner
          ├── DirectWrite text measurement
          ├── work-area / per-monitor DPI placement
          └── non-activating, target-text-only renderer
```

Arrows represent owned contracts, not mandatory files/classes/threads. Start with the smallest host; isolate UIA/inference into separate processes **only** if hang/crash containment warrants IPC/extra memory.

### Why full-native C++ is being evaluated

Advantages: direct Win32/COM/DirectWrite calls, CTranslate2 C++ API, no WebView2/JavaScript renderer, deterministic lifetime with RAII. Costs: undefined behavior, COM apartment complexity, accessibility providers for custom-drawn UI, ABI/exception safety, toolchain/DLL distribution, packaging and native UI development. Direct C++ integration improves simplicity but does **not** prove better latency than Rust; capture + model are usually substantial parts of the critical path.

## 3. Responsibilities and state ownership (R/S)

| Responsibility / owner | Owns | Must not own |
| --- | --- | --- |
| WindowsHost | HWND message pump, tray icon, shortcut registration, shutdown/session lifecycle | Translation rules or model state |
| RequestCoordinator | active generation, cursor/foreground snapshot, cancellation/late-result gating | COM internals, drawing |
| CapturePort and concrete provider | capture operation and transient provider handles/results | Popup state, settings persistence |
| TextValidator/Scope | nonempty text, hit test and Word/Sentence/Paragraph extraction semantics | Win32 focus, model loading |
| TranslationEngine | model lifecycle, tokenizer compatibility, inference results | Tray/settings controls |
| PopupController | visible text, measurement, popup HWND/geometry | Capture or inference |
| SettingsStore | versioned local preferences, exclusions | duplicate feature state in UI controls |
| ModelManager | downloaded/staged model packages, integrity/version activation | presentation or raw text history |

**Invariant:** each mutable state has one source of truth. Avoid two independent 'current request', 'model ready', 'last cursor' or 'current settings' values that can diverge. The UI may cache a rendered projection but cannot commit authoritative worker state.

### Boundary interfaces — conceptual, not mandatory abstract base classes

```text
CaptureRequest { generation, trigger, target_window, cursor_px, scope }
  -> CaptureResult = Text+geometry | NoSelection | Unsupported | Denied | Timeout | InternalError

TranslationRequest { generation, captured_text, model_id }
  -> TranslationResult = TargetText | ModelMissing | ModelInvalid | Timeout | Failed

PopupRequest { generation, target_text, cursor_snapshot, appearance }
  -> measured/window-clamped NativePopup or NotShown
```

Represent results with explicit error types (`std::expected` if using compatible C++23 tooling; otherwise a single simple local result type). Only introduce interfaces if there are two genuine implementations, an independent failure boundary, or useful deterministic tests. No generic `IManager` hierarchy, broad service locator, repository pattern, event bus, or one-use factories.

## 4. Critical runtime ordering (F)

### Normal path

```text
WM_HOTKEY
  → freeze cursor HWND/monitor/DPI and generation number
  → capture selection on UIA MTA worker (no popup before valid capture)
  → reject empty/denied or expired generation
  → local translation using installed model
  → reject error/empty/expired generation
  → measure final text with actual font/DPI & max constraints
  → compute above-cursor geometry; flip below if needed; clamp to work area
  → update/reuse HWND on UI thread without activating source app
  → dismiss according to user action / settings
```

### Concurrency and lifetime

- UI thread is the only owner of popup HWND and visible geometry.
- Only coordinator advances the current generation. Late capture/inference responses are discarded.
- Capture is run on correctly initialized COM **MTA** worker(s), not in the UI message pump.
- Snapshot original foreground window/cursor before dispatch; never recapture anchor after inference.
- Use bounded work in flight and a deterministic shutdown protocol. Do not spawn a thread on each hotkey.
- Waiting with a timeout does **not** guarantee cancellation of a blocked COM call. If UIA hang containment is mandatory, evaluate a separate helper process with watchdog and bounded restart.
- Failures during sleep/lock/explorer restart or monitor reconfiguration invalidate stale snapshots; do not display at disconnected monitors.
- Prefer a reusable hidden native popup HWND and renderer state; validate against native focus semantics (`WS_EX_NOACTIVATE`, tool-window behavior, `SWP_NOACTIVATE`).

### Popup measurement / DPI

- Dimension settings are DIPs; cursor from Win32 is in screen pixels. Use monitor-aware conversions.
- Measure intrinsic target text first; constrain to configured **maximum**, wrap, remeasure height; never use maximum as initial/fixed/minimum size.
- Clamp to monitor work area, including negative virtual-screen coordinates and taskbar. Prefer 12-DIP offset above cursor; flip below when required.
- Keep a readable strategy for long text beyond max height; accessibility must not rely on visual scrolling alone.
- Custom drawing must handle Unicode, BiDi, font scale, high contrast, reduced motion and screen readers.

## 5. Native C++ safety and error policy (E/G)

### Error classes

| Class | Representative cases | Handling |
| --- | --- | --- |
| Normal absence | no selection, no hit-tested text | explicit result, no popup |
| Recoverable platform | `HRESULT` failure, shortcut already registered, permission denied | map to typed error, keep host responsive, show actionable status in Settings/tray |
| Dependency failure | model unavailable/corrupt, OCR unavailable, inference exception | preserve existing valid state, cleanup, report, retry only where safe |
| Timeout/unresponsive dependency | UIA call blocked, inference worker unavailable | mark generation expired; bound waiting; consider helper process isolation |
| Fatal invariant/memory corruption | use-after-free, access violation, corrupted process state | fail fast; never pretend `catch (...)` repairs undefined behavior; recover via separate process only |

**Rules:**
- RAII is mandatory for HWND/HANDLE, COM interfaces, GDI/DirectWrite resources, memory buffers, mapping handles and subprocess handles. Use smart COM pointers and Windows Implementation Library (WIL) when appropriate.
- No raw owning `new`/`delete`, `malloc`/`free` in application-level flows without documented ownership; prefer `vector`, `string`, `span`, `unique_ptr`.
- Check Windows return values / HRESULT, normalize into one domain error language; do not log sensitive capture text.
- Catch C++ exceptions at actual boundaries. Never propagate C++ exceptions across Win32 callbacks, COM/ABI or C-language interfaces; do not use SEH to hide memory corruption.
- Be explicit about thread/apartment affinity and cleanup sequence. Avoid shared mutable state and lock inversions across UI/worker threads.
- Treat captured text, OCR buffers, manifests, tokenizers and downloaded model files as untrusted input with size limits and validation.
- Fault injection is required for crashes, timeouts and partial model downloads; ASan + static analysis in verification builds. MSVC ASan does not prove race freedom.

## 6. Low resource / low latency policy

- Event-driven registration, not global input polling or always-on OCR.
- No permanent webview, browser engine, local HTTP service, telemetry SDK or updater daemon by default.
- Measure **hotkey-to-visible-result** (including capture, waiting, tokenizer, inference, rendering) at p50/p95/p99.
- Separately track first/cold model load vs repeated/warm translation, capture latency, render time, idle private bytes/working set, handle/thread count and CPU wakeups.
- CTranslate2 CPU-first with empirically chosen quantization/thread count/beam size; no assumption that smallest INT8 model is always good enough linguistically.
- **Model residency is a trade-off:** warm lowers repeated latency but raises idle memory; balanced TTL and memory saver remain candidate configurations pending measurement.
- Process isolation adds memory and IPC but improves crash containment; benchmark before selecting topology.
- No fake hard numeric SLOs or claims that C++ itself is faster than Rust.

## 7. Windows platform and distribution constraints

| Area | Design implication / experiment |
| --- | --- |
| Windows versions | Proposed Windows 11 x64 first; Windows 10/ARM64 support requires explicit compatibility matrix |
| CPU | Validate required instruction sets for chosen CTranslate2 binary; feature-check/fail gracefully |
| OCR | Verify Windows.Media.Ocr packaging/package-identity and language availability *before* committing to backend; consider an unpackaged-compatible local engine if needed |
| Packaging | Compare unpackaged EXE/MSI, MSIX and package-with-external-location against capture, OCR, model paths, startup and update needs |
| Win32 runtime | Select coherent /MD vs /MT strategy with CTranslate2 DLLs and redistribution; test on fresh Windows VM |
| DLL loading | Avoid unsafe DLL search order; restrict trusted load dirs; pin, sign or hash-check packages |
| Distribution | Code sign release binaries/installers; SmartScreen publisher/reputation and publisher continuity |
| Updates | Stage/validate versioned assets, then atomically activate; preserve prior model if download is interrupted |
| OS lifecycle | Handle Explorer `TaskbarCreated`, session lock/unlock, display/DPI changes, shutdown and single instance |
| Security | Default least privilege: do not elevate or bypass UIAccess/UAC/protected content for capture |
| Privacy | No raw text logging; secure local model package downloads; dumps can contain sensitive text |
| Accessibility | Custom-drawn text needs UIA exposure, text scaling, keyboard path and contrast fallback |

Important links for implementation verification:
- [Microsoft Windows UI Automation](https://learn.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32)
- [Win32 extended window styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
- [DirectWrite text layout metrics](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextlayout-getmetrics)
- [Microsoft WIL](https://github.com/microsoft/wil)
- [C++ Core Guidelines](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines)
- [CTranslate2 performance](https://opennmt.net/CTranslate2/performance.html)
- [Windows packaging](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/)

Links are reference material, not claims of completed platform compatibility testing.

## 8. SOLID without over-abstraction

- **Single responsibility:** cluster code by state/lifecycle owner (host, capture, translation, popup, settings/model). Do not make a separate class for every method.
- **Open/closed:** add alternate capture providers or rendering strategies only when a second real use case exists; avoid speculation-driven plugin architectures.
- **Liskov:** if there is a provider interface, every implementation must preserve success/error/timeout semantics; don't invent substitutability where it isn't needed.
- **Interface segregation:** narrow contracts at OS/worker boundaries, not a mega-interface controlling UI, OCR and models.
- **Dependency inversion:** coordinator depends on small capture/inference contracts when the boundary is real, not implementation-specific OS details; simple direct calls within one owner are acceptable.
- No circular dependencies, duplicated source-of-truth, universal `BaseService`, `ManagerFactory`, singleton service locators, unnecessary async plumbing, wrapper chains or more than one layer that only forwards parameters.

**Review question for each new type, module or thread:** which current invariant, resource lifetime or externally observable responsibility requires its existence? Remove it if there is no concrete answer.

## 9. Proof gates

### Prototype gate
1. Win32 host + tray + hotkey runs without WebView2.
2. Text-only DirectWrite popup: intrinsic word/sentence/paragraph size and frozen cursor placement across mixed DPI.
3. UIA capture on MTA worker, no-selection silent stop, deterministic latest-request-wins.
4. CTranslate2 EN→ID works offline with tokenizer/model validated.
5. Model missing/corrupt and UIA timeout don't lock Settings/tray.

### Production gate
- MSVC static analysis, useful warnings-as-errors policy for project code, ASan on test builds; fault-injection coverage for worker crashes, UIA hangs, race scenarios, model corruption and interrupted installation.
- Long-running idle/soak run measuring RAM, handles, threads, CPU; repeated hotkey and load/unload cycles; compare cold/warm TTTR.
- Clean Windows install/package identity/DLL dependencies/code signing; OS/session/multi-monitor acceptance.
- Accessibility, high-contrast and user privacy verification; no claim of universal app capture.
- Main must stay green; no release without real Windows proof of selected failure modes.

## 10. Consequential decisions / open ADR candidates

| Decision | Status | Decision evidence needed |
| --- | --- | --- |
| Full native C++ Win32 vs native Rust | **Candidate C++** per current product direction | existing code audit/migration cost and Windows measurements |
| UIA helper process vs in-process MTA | **Open** | provider hang tests, idle footprint, restart reliability |
| Inference process vs same process | **Open** | crash isolation need vs model memory/IPC footprint |
| OCR backend/packaging | **Open** | Windows support, licensing, package identity, latency |
| Model residency policy | **Balanced candidate** | idle RAM vs cold/warm TTTR and battery |
| Windows minimum version | **Windows 11 x64 candidate** | app audience and compatibility tests |

Record an ADR under `.agents/decisions/` only when a consequential alternative is actually chosen with evidence. Do not implement all candidates or pre-create folders/modules because they appear in this document.

## M4 — Privacy capture-policy boundary

`capture_policy.h` owns only a validated, bounded executable-name list.
`capture_policy.cpp` owns Win32 process identity lookup via
`GetWindowThreadProcessId` and `QueryFullProcessImageNameW` with
`PROCESS_QUERY_LIMITED_INFORMATION`. The source window is checked at the
shortcut/coordinator boundary and again immediately before the isolated
UIA helper. The check never runs as a background poll, never enumerates all
processes and never persists executable paths or captured text.

The `Preferences` snapshot contains a single authoritative serialized
policy (max 32 names / 4096 characters). Invalid policy or inaccessible
foreground-process identity fails closed. Saving a new policy invalidates
in-flight presentation generations and queued requests. Active native capture
calls already in progress may continue until their bounded timeout; do not
claim retroactive cancellation.

The user-facing settings design remains a single Privacy section. No service
registry, policy engine, generic capture-provider abstraction, Windows
administrative privilege, clipboard injection, or global mouse hooks are added.

Follow-up when integrating M3: ensure the separately developed OCR tray
entrypoint respects the policy if it operates on content from the currently
foreground application; never merge two independent routes that disagree about
privacy checks.
