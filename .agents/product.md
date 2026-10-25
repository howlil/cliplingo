# ClipLingo — Product Research & Contract

Status: **approved product behavior baseline + explicit open research** (2026-10-25). This file owns the product problem, user job, observable behavior, scope, edge cases and acceptance. Refer to root `DESIGN.md` for the approved Windows UI, and `.agents/engineering-design.md` for architecture.

## 1. Context, evidence and assumptions

**Actor:** someone reading English text within a Windows application who needs the Indonesian meaning without breaking their reading context.

**Trigger:** user selects text or intentionally invokes translation using the ClipLingo shortcut; OCR is an on-demand fallback for non-selectable content.

**Problem:** copy → switch to translator → paste → switch back is disruptive. A conventional full translation window adds unnecessary controls and attention cost.

**Evidence:** user-approved interactive iterations established a text-only result, above-cursor placement, intrinsic sizing, tray entry point, and separate Settings. Earlier research established the practical need for multiple capture strategies; Windows APIs have availability and permission limitations.

**Assumptions to validate:** what percentage of target apps expose selection/hover via accessibility; actual perceived latency threshold; how often word/sentence/paragraph is used; whether users need OCR in the first publicly released version. Do not present these as proven usage statistics.

## 2. Job / intended outcome (J)

> While reading in a Windows app, I can understand a selected English word, sentence or paragraph in Indonesian quickly, **offline by default or through a consented online provider**, without switching context or a large UI.

Additional job: configure the utility once via Windows system tray, then let it stay unobtrusive with minimal idle resource usage.

## 3. Observable behavior graph (B)

```text
Reading inside source app
  → select text OR hover over a readable text target
  → press global shortcut (default Ctrl+Alt+T)
  → ClipLingo captures a valid target and the trigger cursor location
       ├─ no text/unsupported → no translation popup
       └─ valid
            → configured EN→ID route: offline model or explicitly consented online provider
                 ├─ failure → no fake result; actionable state via tray/Settings
                 └─ successful result
                      → intrinsic text-only container above frozen cursor
                      → user reads target meaning
                      → dismisses/continues reading
```

**Selection semantics:** translate precisely the selected span, even if it includes multiple words, sentences or paragraphs. Do not automatically expand a manual selection. **Hover semantics:** use configured Word/Sentence/Paragraph extraction, with hit-testing and context. **OCR semantics:** user explicitly chooses a screen region; no perpetual screen scanning. Default direction EN→ID; reversed direction is optional and requires its valid model. Offline-only never sends source text to a network. Online-only and explicit Smart fallback require user permission for the chosen provider before uploading selected text.

## 4. Failure and edge-state behavior

| Trigger/state | Required user-visible behavior |
| --- | --- |
| Shortcut with no accessible selected text | No result popup; no empty loading card |
| Capture provider unavailable / protected content | Do not bypass OS security; surface explanation in diagnostics when needed |
| UIA returns nearest text not actually under cursor | Reject incorrect hover target rather than translate unrelated text |
| Offline model missing/corrupt | Offline-only: Settings shows repair/install action, no fake popup and no automatic cloud upload |
| Translation failed or returned empty | No fabricated translation; non-intrusive actionable status outside the popup |
| Repeated hotkeys / moving cursor during inference | Most recent request wins; popup anchors to its trigger-time cursor snapshot |
| Word/Sentence/Paragraph long result | Content-fit until maximum width/height; accessible overflow afterward |
| Cursor at screen edge, mixed-DPI monitor | Popup stays within correct monitor work area, above or below anchor |
| App paused | Hotkey does not start new translation; Settings/tray remain reachable |
| Session locked / app exited | Popup hidden; no background capture |
| OCR permission denied / restricted app | Respect restriction and offer supported alternatives, not privileged bypass |
| Screen reader, high-contrast, text-scale | Translation and Settings remain accessible without extra popup buttons |

Errors classified as silent (no selection), recoverable/actionable (model missing, shortcut conflict), or hard failure (component crash); products should not expose implementation exceptions.

## 5. Product constraints

### Accepted

- Windows-first, single-user desktop utility, no account, no cloud inference requirement.
- Offline-only is the default after local model installation. Optional online translation explicitly uploads selected text over TLS to a user-chosen official provider after informed consent. Model installation and signed updates use separate network paths.
- Background tray app rather than a constantly visible main window.
- Approved Apple-inspired design tokens applied in native Windows UI (not copied Apple resources).
- Translation popup contains **only translated target text**, no original, buttons, language badges, or settings controls.
- Content-driven popup size with max-width/max-height limits; above-cursor snapshot anchoring.
- Scope distinction word/sentence/paragraph; local EN→ID is the core language direction.
- Low idle resource consumption and low **hotkey-to-visible-result** latency are primary quality goals.
- Privacy by default: no content history, analytics, or raw capture text logs unless explicitly added later with consent.

### Constraints still requiring investigation

- Minimum OS/CPU requirements (Windows 11 x64 proposed first target; Windows 10 support not promised).
- OCR backend and packaging identity requirements.
- Supported target apps, privileged/elevated app limitations, accessibility variants.
- Numeric latency/RAM thresholds: define only after representative benchmarks; don't invent performance claims.
- Tauri v2 desktop shell with Rust-owned native Windows integrations is the **accepted future engineering direction** ([ADR-0002](decisions/0002-tauri-rust-windows-hybrid.md)); Settings uses lazy WebView2, while a Rust-owned native Win32/DirectWrite result popup is the new low-latency implementation direction, subject to real Windows no-focus-steal, DPI and accessibility proof ([ADR-0003](decisions/0003-latency-first-dual-mode-translation.md)). The tracked codebase remains documentation-only. Historical C++/CMake functionality is archived on `archive/pre-rust-clean-slate-20261026`; no current executable is supplied by this branch. A new Rust implementation must meet all listed acceptance behavior; native Windows and model integration remain verification gates. See [engineering contract](engineering-design.md) and [ADR-0001](decisions/0001-rust-first-native-windows.md).

## 6. Smallest coherent release scope

### M1 — core reading loop

- Tray icon, pause/resume, Settings, Quit, shortcut registration/conflict status.
- EN→ID model installation/status; offline translation.
- Explicit selection capture; request ordering, empty-selection silent stop.
- Text-only auto-fit popup above trigger-time cursor; sensible dismiss; mixed-DPI readiness.
- Windows desktop UI consistent with `DESIGN.md` (Tauri WebView2 Settings, popup backend after native acceptance); no mandatory model download for merely opening Settings.
- Basic privacy settings and excluded-app handling.

### After M1 — additional compatibility where research justifies it

- Hover Word/Sentence/Paragraph using geometry-validated text ranges and segmentation.
- Manual region OCR for inaccessible text; clipboard fallback **opt-in**.
- Optional ID→EN model and language direction.
- Optional official online EN↔ID provider for consenting users (BYOK/auth compatible); no offline model is needed in Online-only mode.
- Accessibility and rendering refinements that preserve text-only popup contract.

### Explicitly out of scope

Cloud accounts/sync, chat assistant, translation history and usage tracking by default, constant OCR/background screen surveillance, forced elevation, on-popup toolbars, arbitrary plugin architecture, *unconsented* cloud fallback, scraped consumer translation endpoints, in-app web backend and speculative multi-model marketplaces.

## 7. Proof (P) — acceptance traces

1. **Given** a browser selection of one English word **when** the shortcut is invoked **then** a compact, accurately placed, Indonesian text-only popup appears.
2. **Given** a long selected paragraph **when** translation completes **then** the popup wraps and grows to configured max bounds, with readable overflow; it is not stretched to max dimensions for short content.
3. **Given** no selection **when** the shortcut is invoked **then** no result popup appears and no inference is unnecessarily invoked.
4. **Given** request A followed quickly by request B **when** A finishes last **then** only B may update the popup.
5. **Given** a non-selectable/protected target **when** capture is attempted **then** ClipLingo doesn't elevate or leak content; Settings remains usable.
6. **Given** a missing offline model **when** translation is requested **then** the user can repair the situation through Settings, not by interpreting a fake result.
7. **Given** the cursor at edge of a secondary mixed-DPI monitor **when** translation succeeds **then** it fits the correct work area without stealing source-app focus.
8. **Given** ClipLingo is idle/paused **when** there is no trigger **then** it performs no continuous OCR/inference or continuous input polling.
9. **Given** the user opens the tray **when** Settings is selected **then** the six-section window is accessible regardless of model state.
10. **Given** Windows is offline after the model is installed **when** a valid selection is translated **then** translation succeeds without a network service.

## 7A. New online/offline acceptance cases

1. Fresh install without model: Offline-only is default, tray/Settings works, no cloud text request and there is a model-install action.
2. Installed verified model with network disabled: EN→ID succeeds without network.
3. Online-only: after choosing official vendor, explicit disclosure/consent and supported user credentials, selected text can go over TLS to that vendor; no local model needed.
4. No consent, excluded/unknown/protected window or no selection: zero captured text leaves device.
5. Smart: local-first; explicit provider-specific network fallback only for allowed failure classes, not permission denial/corrupt model.
6. Online timeout/401/429/5xx/revoked consent: typed failure/diagnostics outside popup, no fake output/no hidden provider switching. Already transmitted text cannot be recalled.
7. Cache hit or out-of-order requests: validate privacy/policy and newest generation; only latest authorized target text may show.
8. Native popup stays target-text-only with no focus theft; benchmark its real Windows latency/idle/process-tree against WebView prototype before claiming performance.

## 8. Research questions and falsification

- Test a matrix of Chrome/Edge, PDF viewers, text editors, and custom/canvas UI targets with UIA selection/hover.
- Benchmark cold and warm end-to-end time, CPU/RAM for idle and model-warm, and user-perceived disruption (not just model inference).
- Evaluate manually triggered OCR viability on supported Windows versions *before* promising a specific OCR implementation.
- Validate one-word disambiguation using optional local context without changing displayed selection.
- Validate text-only popup with screen readers and keyboard-only workflows; no extra visual controls added.
- Compare the archived C++ Windows baseline with the future Rust Windows implementation on representative hardware before claiming any performance, memory, crash-safety or packaging improvement.

## 9. Product change policy

A proposed feature must trace to the reading job and its observable proof. If it adds attention, idle work, privacy risk, or a new window without strengthening the job, defer it. Preserve `DESIGN.md` as the accepted visual baseline; update product behavior here before architecture.

## M3 product acceptance — Manual region OCR

When selected text/hover cannot access an image or canvas, the user can
explicitly choose **Capture screen text** in the system tray. A temporary
overlay lets the user drag a rectangular portion of the desktop; Escape or
right-click cancels without translation. A successfully chosen region is
read locally using an optional CPU OCR backend and passed to the same EN→ID
translator and target-text-only popup. The popup anchors to the frozen
drag-release cursor. No full-screen background scanning, clipboard
interception, image history or network text upload is introduced.

Additional acceptance traces:
- Given OCR is unavailable, selecting OCR shows actionable status in Settings
  and does not display a fake success popup.
- Given no meaningful region/recognizable text, there is no result popup.
- Given a huge rectangle, reject before allocating a screenshot; ROI is capped
  at 12 megapixels.
- Given selection cancellation, capture buffers and the overlay do not persist.
- Given a failed OCR request and a subsequent valid hotkey, only the latest
  generation can update the popup.
- Given a protected/secure desktop, do not attempt to bypass capture restrictions.

OCR accuracy on photographed text, multi-monitor DPI, accessibility of the
manual selector and production pack installation remain **unverified**.

## M4 — Privacy and excluded application behavior

Purpose: users must be able to prevent ClipLingo from reading text in an
explicitly excluded Windows application. The check happens **before**
selection/hover capture, not after the translation result exists.

**Behavior proof:** Settings → Privacy accepts one exact executable basename
per line (e.g. `vault.exe`). The list is deduplicated and saved locally.
On a user hotkey, ClipLingo resolves the foreground process identity once;
a matching basename, the app's own HWND, an unknown process identity or an
invalid/corrupted policy denies capture and produces no result popup. The
diagnostic state is available in Settings. An unexcluded, resolvable process
continues through the existing M1/M2 selection-first path without extra input
polling, network traffic or raw-text logging. Changes invalidate pending
requests so a stale result cannot appear after a privacy update.

This list is **not a security sandbox**, and does not grant access to
elevated/protected applications. Executable basename matching intentionally
affects all installations with the same name; users cannot enter paths or
wildcards. If the process cannot be identified, deny rather than guess.

Acceptance: blocked app, case-insensitive match, corrupted saved list,
unqueryable process, settings persistence, and repeated hotkeys must behave
without reading restricted source text. Live-Windows acceptance is pending.

## M5 — Explicit clipboard translation, opt-in

The clipboard fallback is **explicit**, not a hidden `Ctrl+C` simulation.
To protect clipboard ownership and avoid translating stale unrelated text,
the feature is disabled by default in Settings → Text Capture. When enabled,
the system tray offers **Translate clipboard (explicit)**. Choosing it
captures only `CF_UNICODETEXT` currently on the clipboard and passes that
text to the same local EN→ID engine and target-only popup, anchored to the
trigger-time cursor. The selection/hover hotkey remains unchanged.

Behavior proofs:
- Off by default; ordinary selection/hover NEVER reads clipboard.
- One explicit tray click generates at most one read (no monitoring/listener).
- Clipboard memory is read-only; never cleared, overwritten or re-copied,
  and no keystrokes or clipboard APIs that mutate content are invoked.
- Clipboard busy, unsupported format, empty/whitespace, changed content
  or text exceeding 8192 UTF-16 units cannot create a translation popup.
- Turning the setting off invalidates all previous pending display results.
- Selected text never gets silently replaced by unrelated old clipboard data.

Clipboard content has **no reliable application-of-origin identity**.
This explicit manual action is not equivalent to source selection and may
contain sensitive data from any program. Privacy explanations must state
this directly; when the separate M4 app-exclusion work is integrated, its
policy must not be misrepresented as protecting arbitrary historical
clipboard content. No clipboard history, network upload, logging or account
is introduced.

Live Win11 clipboard owner/delayed-rendering and UI validation remain pending.

## M6 — Native performance proof (benchmark milestone)

ClipLingo's low-latency/low-resource positioning needs repeatable evidence, not
guessed thresholds. M6 adds a diagnostic command that runs the **same native
CTranslate2 EN→ID inference code as the app** against public, fixed test inputs
and saves only numerical timings and memory counters. No selected text,
clipboard text, OCR images, or user file paths are included in output.

Success proof: a Windows build accepting
`--benchmark-model <validated-model-pack> <report.json> [iterations]`
checks model loading and generates a versioned JSON report containing:
cold translation time (includes model load), 3 warm workloads
(word/sentence/paragraph) with p50/p95/p99, private bytes/working set,
peak working set and process CPU time. Inputs are fixed literals; there is
no telemetry, automatic network request or background service.

Fail safely: missing/invalid model, invalid iteration count, missing OS
counters, translation failure and report write errors return a nonzero exit
code rather than generating fabricated numbers. Iterations are bounded
(5–200 per workload). The command exits without starting the tray/UIA
helper/popup and unloads the translation model.

Caution: one-off model benchmark does **not** measure user-visible
hotkey-to-result (UIA helper startup and popup rendering), energy, idle tray
RAM, 8-hour soak, or model quality across natural reading tasks. Those remain
separate native Windows acceptance gates. No fixed latency/RAM SLO is claimed
without representative device and workload measurements.

