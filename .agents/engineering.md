# Engineering — ClipLingo (Windows)

Status: **proposed system contract**, 2026-11-03. Derived from `.agents/product.md`; no existing repository code validated. Target Windows 10/11, C++20/Win32 candidate. UI toolkit is behind a port; do not claim a Slint non-activating popup without an interactive Windows feasibility check.

## System
One resident **single-process, event-driven modular monolith** (tray + short-lived popup + on-demand settings/vocabulary). No mandatory WebView, server, OCR daemon, polling loop, or offline model. Threads: UI/message-loop, dedicated UIA MTA worker, bounded network worker, serialized storage execution; no uncontrolled per-request threads.

```text
Hotkey / source HWND -> K1 WindowsInput -> K2 Capture -> K3 Session/Policy
                                                      |       |       |
                                                      v       v       v
                                               K4 Translator  K5 Popup K6 Vocabulary
                                                      |       |       |
                                                   DEP1 API  UI port  K7 LocalStore
                                            K3 reads K8 Settings/Consent
K5 Settings/Vocabulary views -> public K8/K6 entries (never direct storage)
```

## Responsibilities (product trace)
- R1 (B1,B7,X8): register/unregister a configurable hotkey, retain last working binding; record source window before opening popup.
- R2 (B2,X1-X3): safely obtain selected text from UIA; reject protected/unsupported surfaces; return explicit capture status and optional bounded context.
- R3 (B3,B5,X4): render popup immediately, preserve source focus on passive display, place/resize on monitor work area, manage copy/dismiss and paragraph scroll.
- R4 (B4,B8,X5,X6,X9): own request generation, validate permission/settings, allow only newest request to affect the popup, classify and recover provider failures.
- R5 (B4,X5,X6): translate with one online provider, keep translation contract independent of vendor JSON; distinguish contextual from context-unavailable results.
- R6 (B6,X7): save/list/delete vocabulary with explicit deduplication and durable acknowledgement.
- R7 (B7,X8,X10): own settings, exclusions, consent and provider credentials; deny outbound requests until consent and policy pass.
- R8 (Q1-Q6): collect content-free stage timing, idle resource and compatibility diagnostic signals for local acceptance; diagnostics off unless requested.

## Structure and dependency direction
- `src/app/` — executable composition root, lifecycle, startup/shutdown and module wiring.
- `src/core/session/` (K3) — controller/use-case orchestration, generation IDs, request state; imports domain ports only.
- `src/core/translation/` (K4) — input/response types, context policy, translation-provider **port**; no HTTP or UI dependencies.
- `src/core/vocabulary/` (K6) — vocabulary rules/repository **port**; no SQLite imports.
- `src/core/preferences/` (K8) — validated settings and consent policies/ports, no Win32 or DB specifics.
- `src/platform/windows/` (K1,K2) — Win32 shortcut, UIA capture, optional explicit clipboard fallback, monitor/focus APIs.
- `src/platform/translation/` — provider adapter and network I/O, implements K4 port.
- `src/platform/storage/` (K7) — one local SQLite owner implementing vocabulary/preferences ports; credentials in Windows Credential Manager (separate secret storage).
- `src/ui/` (K5) — Popup, Settings, Vocabulary surfaces + thin controller; presentation-only view state; Slint candidate; Win32-native fallback.
- `tests/` — contract/logic integration tests, Windows-only acceptance tests; no duplicated test frameworks.
- Entry points: one public header/target per domain module; OS and storage adapters depend inward on core; core must never import platform or UI. Avoid generic `utils/` modules and speculative provider/plugin frameworks.

## Authoritative state
- S1 **Active request** | owner R4/K3 | monotonically increasing `generation`; state Idle/Capturing/Translating/Success/Failed; only current generation may publish result | volatile, never persisted; includes transient selected text, zero history default.
- S2 **Source snapshot** | owner R2/K2 for capture duration | source HWND/process and cursor/selection coordinates captured at trigger; never switch source because focus later moves | volatile, sensitive text.
- S3 **Popup view** | owner R3/K5 | derived only from newest S1, last user dismiss, settings and layout | volatile; no own truth about translation.
- S4 **Preferences** | owner R7/K8 backed by K7 | valid language pair/shortcut/dimensions/exclusions/provider; old hotkey stays bound if new registration fails | local persistent; acknowledged updates survive process restart.
- S5 **Online consent/exclusions** | owner R7/K8 backed by K7 | deny by default until explicit consent; exclusions checked before capture/transmission and again before outbound request | local persistent; not synchronized remotely.
- S6 **Vocabulary** | owner R6/K6 backed by K7 | one record per normalized text + source/target + sense identity; saves/deletes transactional; acknowledgements only after commit | local user content; RPO=0 acknowledged changes for app-process crash; no cloud backup promise.
- S7 **Provider credential** | owner R7/K8 via OS credential vault | never stored in SQLite, UI state, or logs | local secret protected by OS; retention until explicit removal.
- S8 **Diagnostics** | owner R8 | counters/timings/error category only, never raw text or credentials | volatile by default.
- One SQLite database `cliplingo.db` under per-user app-data with `preferences`, `consent`, `vocabulary`, and `schema_version`. One serialized writer; short operations; SQLite rollback journaling initially (WAL only if real concurrent read/write contention). Explicit unique index implements S6; schema migrations use additive expand→migrate→contract and keep previous released code able to open the new schema before breaking migrations.

## Boundary contracts
- K1 → K3: `HotkeyTriggered(source_hwnd, cursor_point, monotonic_id)`; source snapshot collected before popup; registration conflict preserves existing shortcut (X8).
- K3 → K2: `CaptureRequest(source_snapshot, max_chars, deadline, app_policy)` → `{selected_text, optional_context, selection_rect?, provenance, error}`; no selected-text history. Only read accessible OS-provided data; UIA worker MTA, never UI thread. No unbounded document traversal. A deadline bounds caller wait, NOT necessarily the underlying COM call.
- K2 → K3: `CaptureResult`; error types Empty, Unsupported, Protected, Excluded, Timeout, Internal; no clipboard writes in default mode (X1-X3,X10).
- K3 → K4: `TranslateRequest(request_id, text, source_lang, target_lang, context?, mode)` → `{translated_text, detected_lang?, context_used, provider, error}`; source/target validation; HTTPS only; external call requires consent, eligible app, and available credentials (X5,X6,X10).
- K4 → K3: result may arrive late; ignore unless `request_id == active generation`; cancellation best effort but stale suppression mandatory (B8,X9).
- K3 → K5: immutable `PopupViewState(generation, phase, text?, result?, error?, anchor?)`; UI displays loading before capture completes; no capture/network/DB call on UI thread (B3,B5).
- K6 → K7: `SaveVocabulary` is idempotent on normalized language/text/sense key; transactional upsert; result `saved|already_exists|failed`; list/delete only via K6 entry (X7).
- K8 → K1: hotkey change stages proposed binding, registers first, and swaps only if successful; persistence and runtime binding reconciled on rollback (X8).
- K8 → K3/K4: consent changes invalidate previously authorized in-flight work where possible and prevent sending future data; on revocation drop results even if provider cancellation fails (X10).
- DEP1: one external translation API; **buy**; offline/timeout/429/auth/lang errors mapped to typed failures; not in critical local popup feedback path; vendor exit cost isolated to K4 adapter.
- DEP2: Windows UI Automation / Win32; **reuse**; provider support and cross-process responsiveness vary; no elevated bypass; isolation of UIA worker and recoverable capture failure.
- DEP3: UI toolkit (Slint proposed); **reuse**; runtime backend non-activation/window ownership compatibility unverified; UI port allows retained/native popup without rewriting core.
- DEP4: SQLite + Windows Credential Manager; **reuse**; local filesystem failures return explicit write error, without fake success; secrets never persisted as plaintext.

## Critical flows
1. **First vertical slice** (B1-B5,B8): WM_HOTKEY → snapshot source → increment generation → render loading near cursor → check exclusion/consent → async UIA capture → validate selected text → async translate → check generation → render result → explicit copy/dismiss. No raw history persistence.
2. **Capture unavailable** (X1-X3): bounded caller wait → error state; if UIA remains stuck, open circuit for further capture attempts until worker recovery or app restart; offer user-initiated manual paste. `Ctrl+C` fallback is an explicitly enabled *experimental* path only after rich-clipboard risk is disclosed; never claim lossless restoration. Respect clipboard sequence changes and abort on conflict; never synthesize paste into source app.
3. **Consent absent** (X10): show locally rendered explanation; no outbound call; if consent accepted, resume only the still-current request and recheck exclusions.
4. **Provider failure** (X5): network worker maps offline, timeout, quota, credential and unsupported language into recoverable popup state; no fabricated translation and no unbounded retries.
5. **Rapid retrigger** (X9): generation N+1 supersedes N before new capture; late completion N is ignored regardless of whether cancellation succeeds.
6. **Save vocabulary** (B6,X7): user click → K6 normalize/check key → K7 transaction → committed result → popup Saved/Already saved; disk failure leaves unsaved and retryable.
7. **Preferences** (B7,X8): proposed hotkey validated/registered → persists and swaps current binding; on persistence failure restore previous runtime binding. UI settings changes only through K8.
8. **Window placement** (X4): selected-range rectangle if reliable, else snapshot cursor → corresponding monitor `rcWork` → DPI-compatible clamp → popup with `SWP_NOACTIVATE`; long text scrolls inside max-height viewport.
9. **Restart/upgrade**: no unsaved selection/session restored; stored preferences/consent/vocabulary loaded; if DB migration fails, fail storage functions safely rather than overwriting data; prior released version remains usable with compatible schema.

## Guarantees (targets from product.md, not achieved)
- G1 (Q1,B3) [H]: p95 hotkey→visible popup/loading ≤150 ms on specified Windows reference device, measured from WM_HOTKEY dispatch to first visible frame; capture/network excluded from critical UI path.
- G2 (Q2,B4) [H]: short-text p95 final response ≤2 s and 300–500-word p95 ≤4 s under a defined provider/network baseline, measured from hotkey to final rendered response; provider/network SLI recorded separately. Cloud dependency bound means **conditional target**, not unconditional SLO.
- G3 (Q3,C3) [M]: idle 5-minute private working set ≤80 MiB and average CPU ≤1%, with settings closed and no local model, measured on reference PC; report settings-open/process-group cost separately.
- G4 (Q4,B2,X2,X3) [H]: no source mutation, silent clipboard overwrite, or passive focus theft in supported test matrix; SLI = violating sessions / tested sessions, target 0 observed violations.
- G5 (Q5,B4,X6) [M]: ≥90% human-rated context-appropriate translations in agreed EN↔ID evaluation corpus; report word, sentence, paragraph and absent-context results separately, no fabricated certainty.
- G6 (Q6,B2) [M]: actual per-app UIA-only and optional fallback coverage measured across Chrome/Edge, Notepad, Word, VS Code, selectable PDF; no universal compatibility promise.
- G7 (C5,X10) [H]: zero outbound requests with selected text before explicit consent or from excluded apps; SLI = unauthorized requests / network-boundary attempts, target 0, enforced at request boundary.
- G8 (B8,X9) [H]: 100% visible translation results correspond to current generation under reordered capture/provider responses; measured by mismatched result count / synthetic reordered sessions, target 0.
- G9 (B6,B7,X7,X8) [M]: acknowledged saved vocab/settings remain after app restart; RPO 0 for committed operations after process crash, RTO = manual application restart (not timed in product Q).

## Challenge — high-priority guarantees
| Flow/G | Failure/threat | Blast radius | Containment | Detection | Recovery |
| --- | --- | --- | --- | --- | --- |
| Capture→Popup G1 | UIA provider hangs | delayed/blocked capture | dedicated MTA worker; caller deadline; trip circuit breaker if worker remains blocked; never spawn unlimited replacement threads | capture timeout + hotkey-to-visible latency | visible timeout, manual paste, optional user restart; consider supervised helper only if measured |
| Translate G2 | network slow/429/offline | request result only | bounded network worker, deadline, typed errors | stage latency/HTTP category | explicit retry; keep popup responsive |
| Capture G4 | copy fallback overwrites rich clipboard, focus stolen | user data/workflow | UIA default; explicit opt-in fallback; no automatic paste; nonactivate native window | clipboard sequence and focus regression | stop unsafe fallback, manual paste |
| Permission G7 | unconsented/excluded text transmitted | privacy leak | deny-first policy before network; no raw logging; validate on each request | instrumentation at outbound send boundary | block, purge transient content, explain settings |
| Retrigger G8 | response N arrives after N+1 | wrong popup translation | immutable monotonic generation check at publish boundary | mismatched generation invariant | drop stale reply |

## Decisions / trade-offs
- DEC1 [two-way]: one resident process + bounded workers over separate services; favors memory/debuggability, gives less isolation from a pathological UIA provider; revisit if stalled UIA cannot be contained.
- DEC2 [two-way]: UIA first, manual paste as default recovery; opt-in clipboard-copy fallback only after proving safe behavior on compatibility matrix; gives lower coverage than aggressive simulated input.
- DEC3 [two-way]: core contracts independent of renderer, native Win32 window behavior required; Slint favored for iterative declarative UI **only if** no-activate/HiDPI/scroll proof passes; otherwise keep Win32 popup and evaluate Slint Settings separately.
- DEC4 [two-way]: one cloud translation provider via simple translation port; no full plugin system/LLM orchestration; revisit when two real vendors are needed.
- DEC5 [two-way]: SQLite for settings+vocabulary, OS vault for secrets; centralizes persistence and consistency, adds SQLite binary cost.
- DEC6 [two-way]: latest-generation-wins with best-effort cancel instead of strict serial full-request completion; saves perceived latency, may incur charges for already-sent stale cloud requests.
- DEC7 [two-way]: transient selected text and context held only in request memory; no persistent history, cloud cache or raw-content telemetry by default; repeated translations may cost more.
- D8: `src/contracts/` is a data-only layer (standard library only) shared across K1-K8; it exists so core can carry anchor/text/result values without importing Win32 types. Boundary enforced by `tools/check_boundaries.ps1`.
- D9: unit proof uses CTest plus a ~40-line assert harness instead of a fetched test framework; no configure-time download or lockfile yet. Revisit only when a second suite justifies a real framework.
- D10: vocabulary sense identity is not implemented in v1; the dedupe key is (normalized text, source language, target language) with `sense = std::nullopt`, which still satisfies X7 for what the v1 UI can produce.
- D11: cross-boundary async results use a sink callback (`std::function`) invoked on the UI thread instead of futures/coroutines; staleness is handled by the generation check at publish, not by cancellation.
- D12: `CaptureResult.provenance` (UIA vs copy fallback) is added together with the capture adapter that can actually produce it (X2, Q6); until then `CaptureStatus` is the only provenance signal, and `PopupViewState` carries no `already_saved` placeholder before the vocabulary slice produces it.
- D13: the composition root owns shutdown order (stop/cancel in-flight work, join worker threads, destroy adapters, then the session controller). The controller additionally guards its sinks with a shared liveness flag so a callback arriving after destruction is a no-op instead of undefined behaviour.

## Fitness rules (lightweight CI enforcement)
- F1: core targets cannot depend on `platform` or `ui` targets and cannot include Win32/Slint/HTTP/SQLite headers | CMake target graph + include-boundary script | enforce on CI/build.
- F2: UI must not link provider/network or SQLite adapters directly; only call core public interfaces | CMake target dependency graph | enforce on CI.
- F3: only `platform/windows` may call UIA, clipboard or OS hotkey APIs | source path-boundary check | enforce on CI, exceptions via explicit D#/DEC#.
- F4: only `platform/storage` can directly access SQL/vault APIs | source path-boundary check | enforce on CI.

## Verification (test-engineering will own concrete proof plan)
- `iter`: format/build plus logic unit tests for latest-wins, permission gate, dedupe, prefs/hotkey rollback and popup geometry. Commands: `cmake --build --preset windows-msvc`, `ctest --test-dir out/build/windows-msvc -C Debug --output-on-failure` (runs `core_unit` and `layer_boundaries`).
- `pr`: Windows integration for UIA failures/response reorder, SQLite durable save, consent outbound gate and capture time bounds.
- `release`: one real-desktop app compatibility matrix + first-frame/CPU/RAM traces + focus/clipboard/high-DPI exploratory pass.
- Do not add continuous OCR/E2E browser suites or extra CI workflows merely to validate docs.

## Assumptions / risks
- A6 [non-blocking]: C++20 is a proposed continuation of previous Win32 ClipLingo work; actual repository/codebase could not be inspected from available GitHub source, so existing module paths and build tooling are unverified.
- A7 [non-blocking]: Slint can meet UI styling needs, but no-activate rendering, accessibility and native HWND integration are unproven; native Win32 popup is the fallback boundary.
- A8 [non-blocking]: customer-supplied API key vs bundled-provider credential model is not yet fixed; user-facing credential strategy must be selected before production distribution, without altering the core translation contract.
- A9 [non-blocking]: some UIA COM calls may not be cancellable in-process. An unresponsive worker disables UIA for the session (manual paste remains usable); if acceptance reveals frequent hangs, move capture into a supervised helper process rather than leaking threads.
- Risks: (1) app/provider compatibility and hard-to-cancel UIA, (2) safety of clipboard+focus and personal text privacy, (3) unpredictable cloud translation latency and quality. Trade-off: low memory vs UI ease; coverage vs non-invasive access; cloud quality vs privacy/offline availability.