# ClipLingo — Engineering Contract

Status: **Rust-first migration target accepted; migration NOT implemented** (2026-10-26).
Current production-oriented code on `master` is C++/CMake. This document describes **the target architecture**, not the language/runtime currently shipping. The existing C++ implementation remains the behavior and regression baseline until Rust passes equivalent Windows verification.

Sources of authority: `.agents/product.md` (observable behavior), `DESIGN.md` (approved UI), `AGENTS.md` (agent workflow), and [ADR-0001](decisions/0001-rust-first-native-windows.md) (consequential language decision). Do not add a competing `.agents/engineering.md`: this existing file is the repository's single engineering contract.

## 1. Current system and migration boundary

Current repository evidence (2026-10-26):
- `src/main.cpp`: Win32 host/tray/settings, hotkey handling, request coordination and lifecycle.
- `src/capture.cpp`, `src/hover.cpp`, `src/capture_helper.cpp`: UI Automation and bounded helper capture; `src/capture_policy.cpp`: excluded-app policy.
- `src/clipboard.cpp`: explicit opt-in clipboard reading; `src/ocr.cpp` and `src/region_selector.cpp`: on-demand region OCR.
- `src/translation.cpp` and `src/model_package.cpp`: local CTranslate2/SentencePiece inference and model management.
- `src/popup.cpp` and geometry headers: native popup, sizing, position and accessible output.
- `src/model_benchmark.cpp`: opt-in headless native performance diagnostic.
- `CMakeLists.txt`, `tests/*.cpp`, `.github/workflows/*.yml`: active C++ build/test/distribution mechanisms.

These are observations about **existing code paths**, not claims that all requirements are proven on real Windows or that releases currently work. The existing benchmark is not a validated end-to-end latency SLO. In particular, GitHub release authorization must be verified separately.

Target: migrate **new application-owned native Windows host, coordinator, capture policy, settings, popup and other application logic to Rust**. Preserve Win32/COM/DirectWrite where they meet behavior; do not introduce Tauri/WebView2 or a local HTTP service by default. A narrowly scoped native C/C++ inference/OCR dependency may remain behind FFI when required by CTranslate2, SentencePiece, Tesseract or packaging realities. Calling this "Rust-first" must not be misrepresented as "zero unsafe code" or "100% Rust".

The migration changes implementation ownership, not the user-facing product. No new runtime, binary, or release is approved solely because this document changed.

## 2. System graph — responsibilities and state owners

```text
Windows message loop / tray / global hotkey (Rust WindowsHost)
  ├── SettingsStore: authoritative local versioned preferences
  ├── RequestCoordinator: generation, trigger-time foreground/cursor/DPI snapshot
  │    ├── Capture: selection first, geometry-checked hover, opt-in clipboard,
  │    │            user-initiated region OCR; excluded-source policy before capture
  │    │            UIA worker/helper isolation where provider hangs demand it
  │    └── TranslationEngine: model/tokenizer integrity, offline EN→ID inference
  │                         (Rust orchestration; isolated native FFI if needed)
  └── PopupController: single UI-thread-owned nonactivating native text HWND,
                       intrinsic measurement, mixed-DPI placement, accessibility
```

Arrows express responsibility, not a mandatory crate/class/thread per box. Prefer one process and one Rust application crate initially; use a real helper process only where current timeout/crash isolation requirements justify it. **Do not assume Rust threads can interrupt a blocking COM call.** Do not create a permanent OCR or updater service. An existing helper protocol must be deliberately migrated or replaced rather than accidentally dropped.

| Mutable state | Authoritative owner | Invariant |
| --- | --- | --- |
| Active translation request and generation | RequestCoordinator | Only newest valid generation may present a result |
| Trigger anchor and foreground identity | Immutable request snapshot from coordinator | Cursor is not resampled after inference |
| Window and visible text | PopupController on UI thread | Native HWND is not mutated from worker threads and cannot steal source focus |
| Capture allow/deny decision | Capture policy using SettingsStore snapshot | Excluded, unknown or protected sources are never implicitly permitted |
| Preferences and exclusions | SettingsStore | Single serialized versioned source of truth; privacy changes invalidate pending results |
| Model installation and activated version | ModelManager/TranslationEngine with one agreed lifecycle owner | Corrupt/partial packs are never reported ready or used for fake success |
| COM and native handle lifetime | Owning Windows adapter/worker | No use past shutdown, no leaking foreign handles/COM references |

No duplicate "current request", "model ready", or mutable preferences in UI/controller/worker. A copied snapshot is read-only.

## 3. Boundary contracts

- `CaptureRequest`: `generation + trigger + foreground + physical cursor + scope + privacy snapshot` -> `TextWithProvenance` or typed `NoSelection | Unsupported | Denied | Timeout | InvalidInput | Failed`. Manual region OCR carries bounded ROI; explicit clipboard carries trigger-time sequence number.
- `TranslationRequest`: `generation + validated captured UTF text + activated model` -> nonempty translated target text or typed `ModelMissing | ModelInvalid | Timeout | Failed`. Inference must be offline after model installation.
- `PopupRequest`: `generation + nonempty target text + frozen cursor/monitor snapshot + display preferences` -> show/reuse measured popup or no display. Always recheck latest generation before showing.
- `SettingsChange`: validated, persisted preference update -> coherent snapshot and invalidation of affected pending requests.
- Native boundary: Win32/COM return values and HRESULT map to Rust error outcomes; Windows adapters own raw pointers/handles, thread/apartment affinity, UTF-8/UTF-16 conversion and teardown. `unsafe` code must be locally encapsulated and its safety invariants documented.
- FFI boundary (if needed): define ABI-compatible ownership, allocator/free side, panic/exception barriers, thread-affinity, lengths, model lifetime and error mapping. Neither Rust panic nor C++ exception may unwind across the ABI. Crash containment cannot be promised by an in-process FFI wrapper.

Capture text, clipboard, images, model manifests, files and tokens are untrusted inputs; validate sizes and structure. Never log or retain raw captured content by default. Do not bypass protected/elevated content restrictions.

## 4. Critical runtime flows

**Normal translation:** `WM_HOTKEY` -> snapshot cursor/foreground/monitor and increment generation -> reject excluded/unknown source -> capture selection (or configured hover) on bounded COM MTA worker/helper -> no popup for absent/invalid text -> offline inference -> reject stale or failed results -> measure actual target text and wrap to maximum DIPs -> place above frozen cursor, flip/clamp to monitor work area -> present without activating source window.

**Competing hotkeys:** request B advancing the generation invalidates A. A may finish its native call but may never display a result after B. A timeout is *not* evidence that the underlying COM call stopped; a stuck provider must not block the tray/message pump.

**Explicit clipboard:** only user-initiated tray action when enabled; read `CF_UNICODETEXT` without modification; fail for unavailable, overlong or changed sequence. Clipboard text has no reliable source-app provenance and is not covered by foreground excluded-app policy.

**Manual OCR:** only the user-drawn bounded region, no background capture; cancel/empty/oversized/denied -> no popup; release images promptly. OCR implementation and model dependencies remain subject to real Windows/package validation.

**Model failure/upgrade:** absent/corrupt/incompatible/incomplete assets never produce translation success. Activate only verified model; failure leaves a recoverable Settings state. The model pack and file format must remain compatible with installed user data unless a tested migration exists.

**Shutdown/session changes:** invalidate pending work; stop accepting triggers, hide popup, release HWND on owning thread, stop/join bounded workers and dispose COM/native resources on the correct apartments. Sleep, lock, Explorer restart and disconnected monitors must not resurrect a stale popup.

## 5. Guarantees and constraints

1. **Behavior:** preserve all tests and acceptance traces in `.agents/product.md` and appearance in `DESIGN.md`; especially selection exactness, latest-request-wins, no fake success and target-text-only popup.
2. **Resource/latency:** low idle footprint and low hotkey-to-visible-result time are objectives **without invented numeric budgets**. Measure cold and warm full-path latency, private bytes, working set, CPU, handles, threads and wakeups on representative Windows machines.
3. **Accessibility:** text-only visual popup still exposes readable accessible text; DPI, high contrast, text scaling, mixed monitors, Unicode/BiDi, keyboard-only settings and no-focus-steal require native acceptance.
4. **Privacy/security:** deny excluded/unknown live sources; no default capture history, analytics, background OCR, clipboard rewriting, or cloud text upload; validate packages and restrict DLL load paths when native dependencies are shipped.
5. **Reliability:** responsive tray/settings during provider/inference failure; bounded concurrent work, typed errors, deterministic shutdown, fault isolation where needed.
6. **Distribution:** existing CMake/MSVC Windows build, models, executable packaging and release workflow remain authoritative until a separately proven Rust replacement is ready. CI green is not proof that a GitHub Release was actually published.

## 6. Rust migration execution gates

| Gate | Change | Acceptance before proceeding |
| --- | --- | --- |
| M0 — contract | Rust target documented; ADR; repository rules aligned | Docs cross-reference correctly; current C++ reality and pending verification are explicit; **no app behavior changes** |
| M1 — host | Build a small Rust Windows tray/hotkey/settings shell | Real Windows launch, hotkey conflict, tray restart, pause/quit and resource teardown; no UI regression |
| M2 — capture & coordination | Migrate latest-generation coordinator, selection/hover, excluded-app policy and explicit clipboard | Deterministic race tests plus live UIA COM/apartment/permission/hang/timeout evidence |
| M3 — local inference & OCR | Integrate existing model packs through tested Rust-native or FFI boundary; port explicit region OCR | EN→ID offline parity, tokenizer integrity, invalid model/ROI tests, privacy, crashes/hangs and native DLL packaging |
| M4 — popup | Native text window, geometry, accessible representation, settings parity | Real Windows word/sentence/paragraph intrinsic fit; mixed-DPI edges; screen reader; no activation |
| M5 — release cutover | Replace build/test/packaging workflow only after Rust acceptance; retire C++ paths | Repeatable Windows CI, full installed-package smoke on clean VM, actual GitHub Release asset verified and rollback route |

These migration M-numbers are **migration gates**, not the existing product M1–M6 feature history. Preserve the prior C++ build and the last known releasable artifact until all required parity checks pass. Do not merge a release-cutover or delete `src/`, CMake or model tooling during M0. No performance improvement is claimed without side-by-side measurements.

## 7. Consequential decisions and open questions

**Accepted:** [ADR-0001 — Rust-first native Windows migration](decisions/0001-rust-first-native-windows.md). This is a user-directed maintainability/safety/engineering decision, not a demonstrated performance win.

**Open until implementation evidence:** exact Windows Rust bindings and native UI wrapper; Rust-to-CTranslate2/SentencePiece ABI packaging, optional Tesseract/OCR boundary, helper IPC protocol/versioning, Windows minimum version, model residency policy, installer/update topology and signed dependency availability. Avoid premature crate frameworks, interface hierarchies, thread pools, services, package/runtime promises, or a claim of zero `unsafe`.

## 8. Verification boundary

At M0 review the docs, existing code inventory and GitHub diff. No app compilation or native test is possible from doc changes alone. For later migration gates use Rust format/lint/unit checks **and** real-Windows integration/soak tests; mock Win32/UIA is not a substitute for genuine COM/apartment, HWND, focus and DPI tests.

The recorded C++ implementation details remain recoverable in Git history; this file deliberately stops treating those implementation-specific techniques as instructions for *new* Rust code.
