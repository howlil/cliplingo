# ClipLingo — Product Research & Contract

Status: **approved product behavior baseline + explicit open research** (2026-10-25). This file owns the product problem, user job, observable behavior, scope, edge cases and acceptance. Refer to root `DESIGN.md` for the approved Windows UI, and `.agents/engineering-design.md` for architecture.

## 1. Context, evidence and assumptions

**Actor:** someone reading English text within a Windows application who needs the Indonesian meaning without breaking their reading context.

**Trigger:** user selects text or intentionally invokes translation using the ClipLingo shortcut; OCR is an on-demand fallback for non-selectable content.

**Problem:** copy → switch to translator → paste → switch back is disruptive. A conventional full translation window adds unnecessary controls and attention cost.

**Evidence:** user-approved interactive iterations established a text-only result, above-cursor placement, intrinsic sizing, tray entry point, and separate Settings. Earlier research established the practical need for multiple capture strategies; Windows APIs have availability and permission limitations.

**Assumptions to validate:** what percentage of target apps expose selection/hover via accessibility; actual perceived latency threshold; how often word/sentence/paragraph is used; whether users need OCR in the first publicly released version. Do not present these as proven usage statistics.

## 2. Job / intended outcome (J)

> While reading in a Windows app, I can understand a selected English word, sentence or paragraph in Indonesian quickly, offline, without switching context or manipulating a large translation UI.

Additional job: configure the utility once via Windows system tray, then let it stay unobtrusive with minimal idle resource usage.

## 3. Observable behavior graph (B)

```text
Reading inside source app
  → select text OR hover over a readable text target
  → press global shortcut (default Ctrl+Alt+T)
  → ClipLingo captures a valid target and the trigger cursor location
       ├─ no text/unsupported → no translation popup
       └─ valid
            → local EN→ID translation
                 ├─ failure → no fake result; actionable state via tray/Settings
                 └─ successful result
                      → intrinsic text-only container above frozen cursor
                      → user reads target meaning
                      → dismisses/continues reading
```

**Selection semantics:** translate precisely the selected span, even if it includes multiple words, sentences or paragraphs. Do not automatically expand a manual selection. **Hover semantics:** use configured Word/Sentence/Paragraph extraction, with hit-testing and context. **OCR semantics:** user explicitly chooses a screen region; no perpetual screen scanning. Default direction EN→ID; reversed direction is optional and requires its valid model. No network request for selected content during translation.

## 4. Failure and edge-state behavior

| Trigger/state | Required user-visible behavior |
| --- | --- |
| Shortcut with no accessible selected text | No result popup; no empty loading card |
| Capture provider unavailable / protected content | Do not bypass OS security; surface explanation in diagnostics when needed |
| UIA returns nearest text not actually under cursor | Reject incorrect hover target rather than translate unrelated text |
| Offline model missing/corrupt | Settings shows status and recovery/install action; no result popup |
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
- Offline local translation after model assets are present.
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
- Full C++ migration cost if an existing Rust/Tauri implementation is later brought into this repo.

## 6. Smallest coherent release scope

### M1 — core reading loop

- Tray icon, pause/resume, Settings, Quit, shortcut registration/conflict status.
- EN→ID model installation/status; offline translation.
- Explicit selection capture; request ordering, empty-selection silent stop.
- Text-only auto-fit popup above trigger-time cursor; sensible dismiss; mixed-DPI readiness.
- Native UI consistent with `DESIGN.md`; no mandatory model download for merely opening Settings.
- Basic privacy settings and excluded-app handling.

### After M1 — additional compatibility where research justifies it

- Hover Word/Sentence/Paragraph using geometry-validated text ranges and segmentation.
- Manual region OCR for inaccessible text; clipboard fallback **opt-in**.
- Optional ID→EN model and language direction.
- Accessibility and rendering refinements that preserve text-only popup contract.

### Explicitly out of scope

Cloud accounts/sync, chat assistant, translation history and usage tracking by default, constant OCR/background screen surveillance, forced elevation, on-popup toolbars, arbitrary plugin architecture, in-app web backend, speculative multi-model marketplaces.

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

## 8. Research questions and falsification

- Test a matrix of Chrome/Edge, PDF viewers, text editors, and custom/canvas UI targets with UIA selection/hover.
- Benchmark cold and warm end-to-end time, CPU/RAM for idle and model-warm, and user-perceived disruption (not just model inference).
- Evaluate manually triggered OCR viability on supported Windows versions *before* promising a specific OCR implementation.
- Validate one-word disambiguation using optional local context without changing displayed selection.
- Validate text-only popup with screen readers and keyboard-only workflows; no extra visual controls added.
- Compare native Win32 C++ and existing host approaches on actual Windows hardware before claiming measured superiority.

## 9. Product change policy

A proposed feature must trace to the reading job and its observable proof. If it adds attention, idle work, privacy risk, or a new window without strengthening the job, defer it. Preserve `DESIGN.md` as the accepted visual baseline; update product behavior here before architecture.
