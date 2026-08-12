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

No application code exists yet; architecture/performance assumptions remain unproven until Iteration 001 produces evidence.

---

## Current iteration — Iteration 001: Windows interaction

**Status: PLANNED — implementation not started**

**Implementation plan:** `.agents/plans/001-windows-interaction.md`

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
- Development shortcut is `Ctrl+Alt+T`; `Ctrl+Shift+T` is avoided because it conflicts with common browser/application shortcuts.
- UI Automation is primary capture.
- Clipboard fallback must preserve an existing plain-text clipboard and refuse fallback when existing clipboard formats cannot be safely restored.
- A single dedicated interaction thread owns COM/UIA/clipboard native state; it uses a one-slot latest-request-wins pending request rather than spawning a thread per hotkey.
- Popup is pre-created/hidden and shown in `Capturing` state before capture/translation finishes.
- Translation result is deterministic fake output only: `[FAKE] <selected text>`.

### Exit criteria summary

Iteration 001 remains incomplete until:

- frontend and Rust quality gates pass;
- Notepad, a Chromium browser, VS Code, and an available selectable PDF reader are checked on Windows;
- stale requests cannot overwrite newer state;
- safe clipboard restoration/refusal behavior is verified;
- monitor/DPI placement is checked with actual environment evidence;
- no selected/translated text appears in logs;
- initial latency, idle memory, idle CPU, and compatibility evidence is recorded in `docs/benchmarks/iteration-001.md`.

The detailed RED/GREEN task sequence, file paths, interfaces, commands, manual verification, architecture-smell checks, and PR gate live only in the implementation plan; do not duplicate them here.

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