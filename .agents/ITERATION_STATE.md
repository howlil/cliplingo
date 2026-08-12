# Iteration State

Last updated: 2026-08-13

## Iteration 0 — Engineering foundation

**Status: DONE**

Established before implementation:

- project scope and constraints
- target Tauri/Svelte/Rust/C++ architecture
- dependency and clean-code rules
- TDD and Git workflow
- release/version/package-channel strategy
- role-specific engineering skills

Iteration 001 is now exercising these boundaries with real Windows-facing application code; performance and compatibility claims still require observed evidence.

---

## Current iteration — Iteration 001: Windows interaction

**Status: IN PROGRESS — implementation present; final reproducible CI and interactive Windows validation pending**

**Implementation plan:** `.agents/plans/001-windows-interaction.md`

**Evidence record:** `docs/benchmarks/iteration-001.md`

### Goal

Prove the smallest real ClipLingo interaction without ML:

```text
select text in another Windows application
 -> Ctrl+Alt+T
 -> popup appears immediately
 -> capture selected text
 -> position popup near selection/cursor
 -> deterministic fake translation
 -> dismiss cleanly
```

### Implemented on the current iteration branch

- Tauri 2 + Svelte 5 + TypeScript + Vite desktop/popup scaffold.
- Rust-owned popup state machine and latest-request-wins interaction coordinator.
- One dedicated interaction thread with a one-slot pending queue; no thread per hotkey and no polling loop.
- Pre-created hidden popup shown in `Capturing` state before selection capture finishes.
- Global `Ctrl+Alt+T` shortcut and toggle dismissal.
- Win32 cursor/monitor work-area positioning plus pure clamping/flipping tests.
- Windows UI Automation focused-element `TextPattern` selection capture and selection bounds when exposed by the provider.
- Clipboard fallback only when UIA reports an unsupported provider.
- Conservative clipboard preservation: unsupported existing clipboard formats are refused rather than intentionally discarded.
- Deterministic fake translation only: `[FAKE] <selected text>`.
- Privacy-safe timing metadata for popup request, capture, fake translation, and ready-state request.
- Windows CI for frontend checks/tests/build plus Rust formatting, Clippy, and tests.

### Why this iteration comes first

The highest early risks are Windows integration and UX behavior, not model inference: selected-text compatibility across applications, UI Automation behavior, clipboard safety, popup focus, DPI/multi-monitor placement, stale request handling, perceived latency, and idle resource cost.

### Architectural slice under test

```text
Svelte 5 popup presentation
        ↓ / ↑ narrow Tauri bridge
Rust interaction core
        ↓
latest-request-wins dedicated interaction thread
        ↓
Windows UI Automation primary capture
        ↓ fallback only when safe
clipboard capture
```

The real C++/CTranslate2 worker is intentionally absent.

### Development decisions for this iteration

- Tauri 2 + Svelte 5 + TypeScript + Vite.
- Rust owns application state/workflow.
- Node.js 24 LTS is the frontend tooling baseline.
- Development shortcut is `Ctrl+Alt+T`; final user-configurable hotkeys are later scope.
- UI Automation is primary capture.
- Clipboard fallback must preserve an existing plain-text clipboard and refuse fallback when existing clipboard formats cannot be safely restored.
- A single dedicated interaction thread owns COM/UIA/clipboard native state; it uses a one-slot latest-request-wins pending request rather than spawning a thread per hotkey.
- Popup is pre-created/hidden and shown in `Capturing` state before capture/translation finishes.
- Translation result is deterministic fake output only: `[FAKE] <selected text>`.

### Remaining exit evidence

Iteration 001 is not DONE until:

- the final lockfile-backed, read-only Windows CI workflow passes;
- Notepad, a Chromium browser, VS Code, and an available selectable PDF reader are checked in a real interactive Windows session;
- plain Unicode clipboard restoration is observed through a real fallback path;
- monitor/DPI placement is checked with actual environment evidence;
- runtime logs are inspected and contain no selected/translated text;
- at least 20 warm interactions are measured for p50/p95 timing;
- idle working set, private memory, and CPU are measured.

The exact unverified items and collection procedure live in `docs/benchmarks/iteration-001.md`. Do not convert PENDING evidence into a pass without an observed run.

---

## Next iteration candidate — Iteration 002: isolated inference worker

Iteration 002 may start only after Iteration 001 is marked DONE from evidence.

Expected focus:

```text
Rust WorkerSupervisor
 -> minimal versioned Windows Named Pipe protocol
 -> isolated C++ worker
 -> CTranslate2
 -> one real translation direction
 -> cold/warm latency + memory evidence
```

Do not broaden Iteration 002 into all-language routing, OCR, history, model marketplace/package-manager distribution, or unrelated refactors.

---

## Open decisions requiring evidence

- Whether the pre-created hidden WebView popup meets the actual idle-memory and hotkey→visible UX budget.
- Which applications expose reliable focused-element UI Automation selection and which require fallback/another justified strategy.
- Whether clipboard fallback needs broader format preservation after real compatibility testing.
- First production-quality translation model/language direction and its redistribution/commercial license.
- Direct CJK→Indonesian models versus English-pivot routes.
- Final release SLOs for idle memory, CPU, popup-visible latency, and translation latency.
- Windows code-signing provider/certificate strategy.
