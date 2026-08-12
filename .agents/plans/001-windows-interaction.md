# Iteration 001 — Windows Interaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the smallest end-to-end Windows interaction slice: select text in another application, trigger a global shortcut, capture the selected text, show a popup near the selection/cursor, render a deterministic fake translation, and dismiss cleanly without adding real ML inference.

**Architecture:** Tauri 2 hosts a pre-created hidden WebView popup rendered by Svelte 5. Rust owns all application state and the interaction workflow. Windows-specific capture/cursor/clipboard code is isolated under a Windows adapter. A single background interaction worker uses a one-slot latest-request-wins queue so repeated hotkeys cannot create an unbounded backlog. Real C++/CTranslate2 inference is explicitly deferred to Iteration 002.

**Tech Stack:** Tauri 2, Rust stable, Svelte 5, TypeScript, Vite, Node.js 24 LTS + npm, `windows`/windows-rs for Win32 and UI Automation, official Tauri global-shortcut plugin, Vitest + Testing Library for UI tests, Cargo test for Rust.

## Global Constraints

- Windows-first; execute and validate this iteration on Windows 11 x64. Do not intentionally depend on Windows 11-only behavior where a stable Windows 10+ API is already sufficient.
- Use Node.js 24 LTS for reproducible frontend tooling. Node 26 is still Current as of this plan and is not the baseline.
- Tauri 2 and Svelte 5 are the UI/runtime baseline. Do not add SvelteKit.
- Svelte is presentation only. Rust owns state and workflows.
- Tauri commands/events are transport adapters, not business-logic containers.
- Windows APIs remain inside `src-tauri/src/platform/windows/`.
- No server, localhost HTTP API, database, OCR, GPU stack, translation model, C++ worker, model routing, history, updater, WinGet, or Chocolatey work in this iteration.
- Selected source text and fake translated text must never be written to logs or telemetry.
- Use RED → GREEN → REFACTOR for behavior we own. Native Windows integration that cannot be meaningfully unit-tested must have a documented manual/integration verification step.
- Use only one task branch for the entire iteration. Intermediate commits are allowed; merge the completed iteration with squash merge.
- Prefer standard library and official Tauri/Windows facilities. Add a dependency only when it removes real complexity or risk.
- Dependency agnosticism is required only at meaningful seams: `SelectionProvider`, `Translator`, and `PopupPort`. Do not add interfaces for ordinary helpers.
- No background polling. The interaction worker sleeps on a condition variable; clipboard completion uses Windows clipboard notifications rather than a permanent polling loop.
- Development shortcut for this iteration is `Ctrl+Alt+T`. Do not use `Ctrl+Shift+T` as the default because it collides with common browser/application shortcuts. Final user-configurable shortcut selection is a later product task.
- Popup must be shown before capture/translation completes so perceived latency is independent from native capture latency.
- Initial performance numbers are evidence, not release promises. Record hardware and methodology with every measurement.

---

## Scope and exit criteria

### In scope

```text
external selectable text
    -> Ctrl+Alt+T
    -> immediate popup: Capturing…
    -> Windows UI Automation selection attempt
    -> safe clipboard fallback when allowed
    -> popup repositioned near selection or cursor
    -> deterministic fake translator
    -> popup Ready/Error state
    -> second shortcut or close action dismisses popup
```

### Explicitly out of scope

- real language detection
- CTranslate2 or any other ML runtime
- C++ sidecar/worker
- model downloads or language packs
- arbitrary language routing
- OCR/image translation
- translation history
- cloud APIs
- autostart/updater/release packaging beyond a debug/dev build
- final visual design system or settings application

### Exit evidence

Iteration 001 is DONE only when all are true:

1. `npm run check`, `npm test`, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all` pass.
2. The interaction works manually with at least Notepad, one Chromium browser, VS Code, and one selectable PDF reader available on the test machine.
3. UI Automation success/failure and clipboard fallback behavior are recorded by **metadata only**; no selected/translated text appears in logs.
4. Repeated shortcut presses cannot let an older request overwrite a newer popup state.
5. Clipboard fallback restores a pre-existing plain-text clipboard exactly; if the existing clipboard contains formats the iteration cannot safely restore, fallback refuses rather than destroying them.
6. Popup positioning is clamped to the monitor work area and is manually checked at normal DPI plus one non-100% scaling configuration if available.
7. Initial hotkey→popup-visible, capture, total fake-translation latency, idle memory, and idle CPU evidence is recorded in `docs/benchmarks/iteration-001.md`.
8. `.agents/ITERATION_STATE.md` is updated with actual evidence and the next iteration decision.

---

## Target repository shape after this iteration

```text
cliplingo/
├── .nvmrc
├── .github/
│   └── workflows/
│       └── ci.yml
├── app/
│   ├── package.json
│   ├── package-lock.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── App.svelte
│   │   ├── main.ts
│   │   └── lib/
│   │       └── popup/
│   │           ├── model.ts
│   │           ├── PopupView.svelte
│   │           └── PopupView.test.ts
│   └── src-tauri/
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── capabilities/
│       └── src/
│           ├── lib.rs
│           ├── main.rs
│           ├── application/
│           │   ├── coordinator.rs
│           │   └── mod.rs
│           ├── core/
│           │   ├── mod.rs
│           │   ├── ports.rs
│           │   ├── popup.rs
│           │   ├── positioning.rs
│           │   └── types.rs
│           ├── platform/
│           │   ├── mod.rs
│           │   └── windows/
│           │       ├── clipboard.rs
│           │       ├── cursor.rs
│           │       ├── hotkey.rs
│           │       ├── mod.rs
│           │       └── selection.rs
│           └── presentation/
│               ├── mod.rs
│               └── tauri_popup.rs
└── docs/
    └── benchmarks/
        └── iteration-001.md
```

Do not create workspace crates or a `worker/` directory in this iteration. The current codebase is too small to justify those boundaries yet.

---

## Task 1: Scaffold the smallest Tauri + Svelte application and verification harness

**Files:**
- Create: `.nvmrc`
- Create: `app/**` from Vite/Tauri scaffolding
- Modify: `app/package.json`
- Modify: `app/vite.config.ts`
- Modify: `app/src-tauri/tauri.conf.json`
- Create: `.github/workflows/ci.yml` later in Task 9, not during scaffold

**Interfaces:**
- Consumes: repository engineering rules under `.agents/`
- Produces: a runnable Tauri 2 application with Svelte 5/TypeScript frontend and Rust backend; `npm run tauri dev`, frontend checks, and Rust checks become available to all later tasks.

- [ ] **Step 1: Pin the frontend toolchain to Node 24 LTS**

Create `.nvmrc`:

```text
24
```

Do not pin npm to a patch release in repository policy; `package-lock.json` is the dependency reproducibility mechanism.

- [ ] **Step 2: Scaffold Svelte + TypeScript under `app/`**

From repository root on Windows PowerShell:

```powershell
npm create vite@latest app -- --template svelte-ts
cd app
npm install
npm install @tauri-apps/api
npm install -D @tauri-apps/cli@latest vitest jsdom @testing-library/svelte @testing-library/jest-dom svelte-check
```

Expected: `app/package.json`, `app/src/`, `app/vite.config.ts`, and `app/package-lock.json` exist.

- [ ] **Step 3: Initialize Tauri in the existing Vite app**

Run from `app/`:

```powershell
npx tauri init
```

Use these answers:

```text
App name: ClipLingo
Window title: ClipLingo
Web assets: ../dist
Dev server URL: http://localhost:5173
Frontend dev command: npm run dev
Frontend build command: npm run build
```

Expected: `app/src-tauri/` exists and `npx tauri dev` compiles on the Windows development machine.

- [ ] **Step 4: Make verification commands explicit in `package.json`**

Add scripts equivalent to:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "check": "svelte-check --tsconfig ./tsconfig.app.json",
    "test": "vitest run",
    "test:watch": "vitest",
    "tauri": "tauri"
  }
}
```

Keep whatever extra scripts the scaffold genuinely needs; remove demo-only scripts/assets once they are no longer referenced.

- [ ] **Step 5: Configure Vitest for DOM component tests**

In `app/vite.config.ts`, preserve the Svelte Vite plugin and add a test environment using `jsdom`:

```ts
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: 'jsdom',
  },
  server: {
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
```

If the scaffold already uses a compatible `defineConfig` import, keep one config rather than duplicating files.

- [ ] **Step 6: Remove scaffold demo UI and keep one neutral root component**

`app/src/App.svelte` should temporarily contain only:

```svelte
<main>ClipLingo</main>
```

Do not add settings, history, language selector, theme framework, icon library, router, or state library.

- [ ] **Step 7: Verify the clean scaffold**

Run:

```powershell
npm run check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all
npx tauri dev
```

Expected: checks/build/tests succeed and one basic desktop window opens.

- [ ] **Step 8: Commit the scaffold as an intermediate iteration commit**

```powershell
git add .nvmrc app
ngit commit -m "chore: scaffold Tauri Svelte application"
```

Use `git commit`, not `ngit commit`; the intended command is:

```powershell
git commit -m "chore: scaffold Tauri Svelte application"
```

The branch may contain intermediate commits; the final merge remains squash.

---

## Task 2: Define the core contracts and popup state machine

**Files:**
- Create: `app/src-tauri/src/core/mod.rs`
- Create: `app/src-tauri/src/core/types.rs`
- Create: `app/src-tauri/src/core/ports.rs`
- Create: `app/src-tauri/src/core/popup.rs`
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Produces:
  - `SelectionProvider::capture() -> Result<Selection, CaptureError>`
  - `Translator::translate(&TranslationRequest) -> Result<Translation, TranslationError>`
  - `PopupPort::{show, update, move_to, hide}`
  - `PopupSession::{begin_request, mark_translating, complete, fail, hide, snapshot}`
- Consumes: no Tauri/Win32 types. Core types are platform-neutral serializable/value types.

- [ ] **Step 1: Write RED tests for request identity and stale-result rejection**

In `app/src-tauri/src/core/popup.rs`, create tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Selection, SelectionSource, Translation};

    fn selection(text: &str) -> Selection {
        Selection {
            text: text.to_owned(),
            source: SelectionSource::UiAutomation,
            bounds: None,
        }
    }

    #[test]
    fn new_session_is_hidden() {
        let session = PopupSession::default();
        assert_eq!(session.snapshot(), PopupState::Hidden);
    }

    #[test]
    fn newer_request_makes_older_completion_stale() {
        let mut session = PopupSession::default();
        let first = session.begin_request();
        let second = session.begin_request();

        assert!(matches!(session.complete(
            first,
            Translation { text: "old".into() }
        ), ApplyResult::Stale));

        session.mark_translating(second, selection("new")).unwrap();
        assert!(matches!(session.complete(
            second,
            Translation { text: "new-result".into() }
        ), ApplyResult::Applied(PopupState::Ready { .. })));
    }
}
```

- [ ] **Step 2: Run the focused Rust test and verify RED**

```powershell
cargo test --manifest-path app/src-tauri/Cargo.toml core::popup::tests -- --nocapture
```

Expected: compile failure because `PopupSession`, states, and value types do not exist yet.

- [ ] **Step 3: Implement the minimal value types**

In `core/types.rs` define platform-neutral types with `serde::Serialize`/`Deserialize` only where they cross the UI boundary:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectionSource {
    UiAutomation,
    Clipboard,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    pub text: String,
    pub source: SelectionSource,
    pub bounds: Option<ScreenRect>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TranslationRequest {
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Translation {
    pub text: String,
}
```

Define structured errors, not arbitrary user-facing strings:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum CaptureError {
    NoSelection,
    Unsupported,
    ClipboardUnavailable,
    ClipboardPreservationUnsupported,
    Timeout,
    NativeFailure { operation: &'static str, code: i32 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TranslationError {
    Failed,
}
```

- [ ] **Step 4: Define only the three meaningful ports**

In `core/ports.rs`:

```rust
use crate::core::types::*;

pub trait SelectionProvider: Send + Sync {
    fn capture(&self) -> Result<Selection, CaptureError>;
}

pub trait Translator: Send + Sync {
    fn translate(&self, request: &TranslationRequest)
        -> Result<Translation, TranslationError>;
}

pub trait PopupPort: Send + Sync {
    fn show(&self, state: PopupViewModel);
    fn update(&self, state: PopupViewModel);
    fn move_to(&self, x: f64, y: f64);
    fn hide(&self);
}
```

Do not add repository/service/factory abstractions.

- [ ] **Step 5: Implement popup state as an enum**

In `core/popup.rs` implement:

```rust
pub type RequestId = u64;

#[derive(Clone, Debug, PartialEq)]
pub enum PopupState {
    Hidden,
    Capturing { request_id: RequestId },
    Translating {
        request_id: RequestId,
        source_text: String,
    },
    Ready {
        request_id: RequestId,
        source_text: String,
        translated_text: String,
    },
    Error {
        request_id: RequestId,
        code: PopupErrorCode,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PopupErrorCode {
    NoSelection,
    CaptureUnavailable,
    ClipboardPreservationUnsupported,
    TranslationFailed,
}
```

`PopupSession` owns `next_request_id`, `current_request_id`, and `PopupState`. Completion/failure for non-current request IDs returns `ApplyResult::Stale` without modifying state.

- [ ] **Step 6: Add a serializable `PopupViewModel` projection**

The Svelte boundary receives only:

```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupViewModel {
    pub status: &'static str,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
    pub error_code: Option<&'static str>,
}
```

Implement `From<&PopupState> for PopupViewModel`. Do not expose request IDs unless frontend behavior requires them; stale-response handling belongs in Rust.

- [ ] **Step 7: Run GREEN tests and full Rust checks**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo test --manifest-path app/src-tauri/Cargo.toml core::popup::tests
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: all pass.

- [ ] **Step 8: Commit**

```powershell
git add app/src-tauri/src
ngit commit -m "feat: define interaction core state"
```

Use:

```powershell
git commit -m "feat: define interaction core state"
```

---

## Task 3: Build popup placement as deterministic pure logic

**Files:**
- Create: `app/src-tauri/src/core/positioning.rs`
- Modify: `app/src-tauri/src/core/mod.rs`

**Interfaces:**
- Consumes: `ScreenRect`
- Produces: `place_popup(anchor, popup, work_area, margin) -> ScreenPoint`

- [ ] **Step 1: Write RED placement tests**

Tests must cover normal placement, right-edge clamp, bottom-edge flip-above, and final work-area clamp:

```rust
#[test]
fn places_below_and_to_the_right_when_space_exists() {
    let p = place_popup(
        rect(100.0, 100.0, 80.0, 20.0),
        size(420.0, 180.0),
        rect(0.0, 0.0, 1920.0, 1040.0),
        8.0,
    );
    assert_eq!(p, point(180.0, 128.0));
}

#[test]
fn flips_above_when_bottom_would_overflow() {
    let p = place_popup(
        rect(100.0, 900.0, 80.0, 20.0),
        size(420.0, 180.0),
        rect(0.0, 0.0, 1920.0, 1040.0),
        8.0,
    );
    assert_eq!(p.y, 712.0);
}

#[test]
fn clamps_x_inside_work_area() {
    let p = place_popup(
        rect(1850.0, 100.0, 50.0, 20.0),
        size(420.0, 180.0),
        rect(0.0, 0.0, 1920.0, 1040.0),
        8.0,
    );
    assert_eq!(p.x, 1500.0);
}
```

- [ ] **Step 2: Run RED**

```powershell
cargo test --manifest-path app/src-tauri/Cargo.toml positioning
```

Expected: compile failure because placement types/functions do not exist.

- [ ] **Step 3: Implement the smallest placement function**

Algorithm:

```text
x = anchor.right
below_y = anchor.bottom + margin
if below_y + popup.height <= work_area.bottom:
    y = below_y
else:
    y = anchor.top - popup.height - margin
x = clamp(x, work_area.left, work_area.right - popup.width)
y = clamp(y, work_area.top, work_area.bottom - popup.height)
```

Do not add a geometry dependency.

- [ ] **Step 4: Run GREEN and refactor naming only if needed**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo test --manifest-path app/src-tauri/Cargo.toml positioning
```

- [ ] **Step 5: Commit**

```powershell
git add app/src-tauri/src/core
git commit -m "feat: add popup placement policy"
```

---

## Task 4: Build the Svelte popup as a disposable presentation layer

**Files:**
- Create: `app/src/lib/popup/model.ts`
- Create: `app/src/lib/popup/PopupView.svelte`
- Create: `app/src/lib/popup/PopupView.test.ts`
- Modify: `app/src/App.svelte`
- Modify: `app/src/main.ts` only if scaffold wiring requires it

**Interfaces:**
- Consumes: `PopupViewModel` from Rust through Tauri events/commands
- Produces: visual states for `hidden`, `capturing`, `translating`, `ready`, `error`; emits only user intent such as dismiss/copy.

- [ ] **Step 1: Define the TypeScript view-model contract**

`model.ts`:

```ts
export type PopupStatus =
  | 'hidden'
  | 'capturing'
  | 'translating'
  | 'ready'
  | 'error';

export interface PopupViewModel {
  status: PopupStatus;
  sourceText: string | null;
  translatedText: string | null;
  errorCode: string | null;
}
```

Keep the TS shape exactly aligned with Rust serde names.

- [ ] **Step 2: Write RED component tests**

`PopupView.test.ts` must assert behavior, not CSS implementation:

```ts
import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import PopupView from './PopupView.svelte';

describe('PopupView', () => {
  it('shows immediate capturing feedback', () => {
    render(PopupView, {
      props: {
        model: {
          status: 'capturing',
          sourceText: null,
          translatedText: null,
          errorCode: null,
        },
      },
    });
    expect(screen.getByText('Capturing…')).toBeTruthy();
  });

  it('renders the fake translation when ready', () => {
    render(PopupView, {
      props: {
        model: {
          status: 'ready',
          sourceText: 'こんにちは',
          translatedText: '[FAKE] こんにちは',
          errorCode: null,
        },
      },
    });
    expect(screen.getByText('[FAKE] こんにちは')).toBeTruthy();
  });
});
```

- [ ] **Step 3: Run RED**

```powershell
cd app
npm test -- PopupView.test.ts
```

Expected: fail because the component does not exist.

- [ ] **Step 4: Implement `PopupView.svelte` with Svelte 5 props**

Use a direct prop and simple conditional rendering. No global store:

```svelte
<script lang="ts">
  import type { PopupViewModel } from './model';
  let { model }: { model: PopupViewModel } = $props();
</script>

{#if model.status !== 'hidden'}
  <section class="popup" aria-live="polite">
    {#if model.status === 'capturing'}
      <div class="status">Capturing…</div>
    {:else if model.status === 'translating'}
      <div class="source">{model.sourceText}</div>
      <div class="status">Translating…</div>
    {:else if model.status === 'ready'}
      <div class="source">{model.sourceText}</div>
      <div class="translation">{model.translatedText}</div>
    {:else if model.status === 'error'}
      <div class="error" data-error-code={model.errorCode}>Unable to translate this selection.</div>
    {/if}
  </section>
{/if}
```

Add local CSS for a compact, premium neutral popup: rounded surface, subtle border/shadow, readable typography, max-width, and dark-mode-compatible variables. Do not add Tailwind or an animation library in this iteration; plain component CSS is sufficient for one popup.

- [ ] **Step 5: Make `App.svelte` the Tauri transport adapter**

On mount:

1. subscribe to a single `popup-state` event;
2. call `get_popup_state` after subscribing to recover any state emitted before listener registration;
3. assign returned/event payload to local Svelte state;
4. render `PopupView`.

Conceptual wiring:

```ts
let model = $state<PopupViewModel>({
  status: 'hidden',
  sourceText: null,
  translatedText: null,
  errorCode: null,
});
```

Do not add Zustand/Redux/TanStack Query/Svelte stores for this state.

- [ ] **Step 6: Run GREEN frontend checks**

```powershell
npm run check
npm test
npm run build
```

- [ ] **Step 7: Commit**

```powershell
git add app/src app/package.json app/package-lock.json app/vite.config.ts
git commit -m "feat: add popup presentation"
```

---

## Task 5: Implement the bounded interaction coordinator and fake translator

**Files:**
- Create: `app/src-tauri/src/application/mod.rs`
- Create: `app/src-tauri/src/application/coordinator.rs`
- Modify: `app/src-tauri/src/core/ports.rs`
- Modify: `app/src-tauri/src/core/types.rs`
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `SelectionProvider`, `Translator`, `PopupPort`, `PopupSession`
- Produces: `InteractionCoordinator::trigger()` and `InteractionCoordinator::dismiss()`; a single background worker sleeps when idle and keeps only the latest pending request.

- [ ] **Step 1: Write RED tests for latest-pending coalescing**

Create a pure `PendingSlot<T>` inside `coordinator.rs` backed by `Mutex<Option<T>> + Condvar`. Test replacement semantics:

```rust
#[test]
fn pending_slot_keeps_only_latest_request() {
    let slot = PendingSlot::default();
    slot.submit(1);
    slot.submit(2);
    slot.submit(3);
    assert_eq!(slot.take_now(), Some(3));
    assert_eq!(slot.take_now(), None);
}
```

- [ ] **Step 2: Write RED tests for stale workflow completion**

Use fakes for the three boundaries. The fake popup stores received view models; the fake selection provider returns deterministic text; the fake translator returns `[FAKE] {source}`. Trigger request 1, trigger request 2 before request 1 completes, and assert the final popup is from request 2 only.

The deterministic translator behavior for this iteration is exactly:

```text
input:  こんにちは
output: [FAKE] こんにちは
```

No language detection is performed.

- [ ] **Step 3: Run RED**

```powershell
cargo test --manifest-path app/src-tauri/Cargo.toml application::coordinator::tests -- --nocapture
```

Expected: missing coordinator types/functions.

- [ ] **Step 4: Implement `FakeTranslator` without an extra dependency**

A concrete implementation may live in `application/coordinator.rs` for this iteration or a focused `application/fake_translator.rs` if the file becomes difficult to read:

```rust
pub struct FakeTranslator;

impl Translator for FakeTranslator {
    fn translate(&self, request: &TranslationRequest)
        -> Result<Translation, TranslationError>
    {
        Ok(Translation {
            text: format!("[FAKE] {}", request.text),
        })
    }
}
```

Do not create a mock framework.

- [ ] **Step 5: Implement the interaction worker lifecycle**

`InteractionCoordinator` must:

1. own `Arc<dyn SelectionProvider>`, `Arc<dyn Translator>`, `Arc<dyn PopupPort>`;
2. own a `PopupSession` behind a mutex;
3. own a `PendingSlot<RequestId>`;
4. start exactly one background worker thread;
5. block that worker on `Condvar` while idle, consuming zero polling CPU;
6. on `trigger()`, create a new request ID, immediately `show()` a Capturing view model, replace any pending request with the new request, and wake the worker;
7. worker captures selection, checks request is still current, moves popup when bounds exist, updates Translating, calls translator, and applies Ready/Error only if still current;
8. on `dismiss()`, set state Hidden and hide popup; any later result for the dismissed request is stale.

Do not start one thread per hotkey.

- [ ] **Step 6: Map errors to stable popup error codes**

Use a focused mapping function:

```text
CaptureError::NoSelection -> no_selection
CaptureError::ClipboardPreservationUnsupported -> clipboard_preservation_unsupported
all other capture failures -> capture_unavailable
TranslationError::* -> translation_failed
```

No native HRESULT/error details are exposed directly to Svelte.

- [ ] **Step 7: Run GREEN**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo test --manifest-path app/src-tauri/Cargo.toml application::coordinator::tests
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

- [ ] **Step 8: Commit**

```powershell
git add app/src-tauri/src/application app/src-tauri/src/core app/src-tauri/src/lib.rs
git commit -m "feat: add bounded interaction coordinator"
```

---

## Task 6: Implement the Tauri popup port and pre-created hidden popup

**Files:**
- Create: `app/src-tauri/src/presentation/mod.rs`
- Create: `app/src-tauri/src/presentation/tauri_popup.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/tauri.conf.json`
- Modify: `app/src-tauri/capabilities/default.json` or the generated capability file actually used by the scaffold

**Interfaces:**
- Consumes: `PopupPort`, `PopupViewModel`, pure placement result
- Produces: Tauri implementation that emits `popup-state`, controls visibility/position, and exposes narrow `get_popup_state`/`dismiss_popup` commands.

- [ ] **Step 1: Configure one popup WebView window**

Keep a single pre-created window labelled `popup` with these behaviors:

```text
visible: false
resizable: false
decorations: false
always-on-top: true
skip taskbar: true
initial size: 420 x 180
```

Do not create a settings/main dashboard window in Iteration 001.

- [ ] **Step 2: Implement `TauriPopupPort` as a thin adapter**

Responsibilities only:

- map `PopupViewModel` to one `popup-state` event;
- `show()` -> emit model + show window;
- `update()` -> emit model;
- `move_to(x, y)` -> set physical/logical window position using one consistent coordinate space;
- `hide()` -> hide window.

It must not call UI Automation or translation logic.

- [ ] **Step 3: Add narrow Tauri commands**

Commands:

```text
get_popup_state() -> PopupViewModel
dismiss_popup() -> ()
```

`dismiss_popup` calls `InteractionCoordinator::dismiss()`. `get_popup_state` returns the Rust session projection. No generic filesystem/process/shell command is exposed.

- [ ] **Step 4: Minimize capabilities**

The popup needs only the Tauri APIs actually called by the frontend. Do not enable shell, unrestricted filesystem, process execution, HTTP client, or broad window-management permissions for JavaScript.

- [ ] **Step 5: Manual smoke test the presentation boundary**

Temporarily invoke coordinator trigger through a dev-only Rust startup call or focused unit harness, then verify:

```text
hidden -> Capturing -> Ready
```

and confirm Svelte displays `[FAKE] ...` from Rust state. Remove any startup auto-trigger before commit.

This temporary manual harness is not committed; committed behavior is reachable through the hotkey added next.

- [ ] **Step 6: Run checks**

```powershell
cd app
npm run check
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all
```

- [ ] **Step 7: Commit**

```powershell
git add src-tauri src
git commit -m "feat: connect popup to Rust state"
```

---

## Task 7: Register the global hotkey and cursor fallback using focused Windows adapters

**Files:**
- Create: `app/src-tauri/src/platform/mod.rs`
- Create: `app/src-tauri/src/platform/windows/mod.rs`
- Create: `app/src-tauri/src/platform/windows/hotkey.rs`
- Create: `app/src-tauri/src/platform/windows/cursor.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: `InteractionCoordinator::trigger()` / `dismiss()` and popup position adapter
- Produces: Windows global shortcut registration and cursor-position fallback.

- [ ] **Step 1: Add the official Tauri global shortcut plugin**

From `app/`:

```powershell
npm run tauri add global-shortcut
```

Expected: plugin dependency/configuration is added by Tauri tooling. Do not expose JS global-shortcut permission unless frontend code actually needs it; registration stays Rust-side.

- [ ] **Step 2: Keep the shortcut string in one Windows adapter constant**

```rust
pub const TRANSLATE_SHORTCUT: &str = "Ctrl+Alt+T";
```

No shortcut abstraction/registry is needed yet.

- [ ] **Step 3: Register hotkey on setup and fail visibly if unavailable**

`hotkey.rs` should register once. Registration failure is logged as metadata such as:

```text
event=hotkey_registration shortcut=Ctrl+Alt+T status=failed
```

Do not silently continue with a non-functional app.

Hotkey behavior:

- if popup is visible/current session is not Hidden: second press dismisses;
- otherwise: call `InteractionCoordinator::trigger()`.

- [ ] **Step 4: Implement cursor position fallback through windows-rs/Win32**

Use `GetCursorPos` in `cursor.rs` and return a core `ScreenRect` with zero width/height at the physical cursor coordinate. Keep raw Win32 types and `unsafe` inside this module, with safety comments immediately above each unsafe block.

- [ ] **Step 5: Native integration verification**

Because global shortcut registration is an OS integration boundary, unit testing the OS registration itself is not useful. Manually verify on Windows:

1. launch `npm run tauri dev`;
2. focus Notepad;
3. press `Ctrl+Alt+T`;
4. verify popup becomes visible immediately;
5. press `Ctrl+Alt+T` again;
6. verify popup hides;
7. verify the source application did not crash and the hotkey does not type characters into it.

Record failure details in the iteration PR if shortcut registration conflicts on the test machine.

- [ ] **Step 6: Run checks and commit**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path app/src-tauri/Cargo.toml --all
git add app
git commit -m "feat: add Windows translation hotkey"
```

---

## Task 8: Implement Windows UI Automation selection capture first

**Files:**
- Create: `app/src-tauri/src/platform/windows/selection.rs`
- Modify: `app/src-tauri/src/platform/windows/mod.rs`
- Modify: `app/src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: `SelectionProvider`, `Selection`, `CaptureError`, `ScreenRect`
- Produces: `WindowsSelectionProvider` whose primary path uses UI Automation `TextPattern/GetSelection`; fallback is added in Task 9.

- [ ] **Step 1: Add only the windows-rs features required by the code**

Use the `windows` crate and enable focused feature groups for the APIs actually compiled, expected to include equivalents of:

```text
Win32_Foundation
Win32_System_Com
Win32_UI_Accessibility
Win32_UI_WindowsAndMessaging
```

Task 9 may add clipboard/input/memory feature groups. Do not enable the entire Win32 surface.

- [ ] **Step 2: Keep COM initialization on the coordinator worker thread**

Initialize the COM apartment once for the long-lived interaction worker before UI Automation objects are used, and uninitialize when the worker exits. Do not initialize/uninitialize COM around every selected-text call.

If the chosen apartment model conflicts with actual UI Automation behavior on the target machine, document the evidence in the PR and adjust the dedicated worker thread; do not scatter COM initialization across Tauri handlers.

- [ ] **Step 3: Implement focused-element UIA capture**

Primary algorithm:

```text
GetFocusedElement
 -> request UIA TextPattern
 -> GetSelection
 -> choose first non-empty selected range
 -> GetText(-1)
 -> trim only terminal NUL/control artifacts introduced by the API; do not normalize user content aggressively
 -> GetBoundingRectangles
 -> choose the last non-empty visible selection rectangle as popup anchor
```

Important behavior from the Windows API: an insertion point with no real selection may produce a degenerate/empty range. Treat empty selected text as `CaptureError::NoSelection`, not success.

Do not recursively search the entire UI Automation tree or add browser-specific hacks in this iteration. If focused-element TextPattern is insufficient in a representative app, record it as compatibility evidence before adding another strategy.

- [ ] **Step 4: Ensure logs contain metadata only**

Allowed example:

```text
request=17 capture=uia chars=14 bounds=true duration_ms=11 status=ok
```

Forbidden:

```text
text="selected confidential text"
```

- [ ] **Step 5: Add narrow pure tests around range selection/rectangle choice**

Extract only the logic that does not require COM into small functions. Given candidate text ranges/rectangles, tests must verify:

- empty ranges are ignored;
- final non-empty selection is returned;
- last non-empty rectangle is selected as anchor;
- missing usable text becomes `NoSelection`.

Do not mock COM interfaces merely to increase coverage.

- [ ] **Step 6: Wire UIA capture into the coordinator and reposition popup after capture**

Before capture, popup appears at cursor fallback. After successful UIA capture with bounds, use `place_popup()` + current monitor work area and move the popup. Keep capture and positioning off the frontend.

- [ ] **Step 7: Manual compatibility check for UIA path**

For each available application, select a short unique string and press the shortcut:

```text
Notepad
Chromium browser (Chrome or Edge)
VS Code
selectable PDF reader/browser PDF viewer
```

Record whether capture source is UIA, whether bounds are available, and whether popup placement is plausible. Record application/version in the benchmark/compatibility document. Do not record the selected text.

- [ ] **Step 8: Run checks and commit**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path app/src-tauri/Cargo.toml --all
git add app/src-tauri
git commit -m "feat: capture selection with Windows UI Automation"
```

---

## Task 9: Add a deliberately safe clipboard fallback

**Files:**
- Create: `app/src-tauri/src/platform/windows/clipboard.rs`
- Modify: `app/src-tauri/src/platform/windows/selection.rs`
- Modify: `app/src-tauri/src/platform/windows/mod.rs`
- Modify: `app/src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: UIA primary result and `CaptureError`
- Produces: fallback only for clipboard states the iteration can safely restore.

### Safety decision

Iteration 001 does **not** attempt to snapshot/restore every arbitrary Windows clipboard format. That becomes complex because delayed-rendered/custom formats may have external ownership and lifetime semantics. The safe initial policy is:

```text
clipboard empty -> fallback allowed
clipboard contains only plain Unicode text -> snapshot text, fallback allowed
clipboard contains any additional/unsupported format -> fallback refused with ClipboardPreservationUnsupported
```

This intentionally prefers a failed translation over destroying a user's image/HTML/custom clipboard data.

- [ ] **Step 1: Write RED tests for clipboard preservation policy**

Extract a pure policy function operating on enumerated format IDs:

```rust
#[test]
fn fallback_allows_empty_clipboard() {
    assert!(clipboard_is_safely_restorable(&[]));
}

#[test]
fn fallback_allows_unicode_text_only() {
    assert!(clipboard_is_safely_restorable(&[CF_UNICODETEXT_ID]));
}

#[test]
fn fallback_refuses_mixed_or_unknown_formats() {
    assert!(!clipboard_is_safely_restorable(&[CF_UNICODETEXT_ID, 49301]));
}
```

- [ ] **Step 2: Run RED**

```powershell
cargo test --manifest-path app/src-tauri/Cargo.toml clipboard
```

- [ ] **Step 3: Implement clipboard inspection/snapshot**

Use Win32 clipboard APIs in one focused module. Before sending `Ctrl+C`:

1. enumerate current clipboard formats;
2. if unsupported formats exist, return `ClipboardPreservationUnsupported` without modifying the clipboard;
3. if Unicode text exists, snapshot it into owned Rust memory;
4. remember whether clipboard was originally empty.

Retry opening a temporarily busy clipboard only with a short bounded attempt associated with the user action; never run a background polling loop.

- [ ] **Step 4: Use Windows clipboard-update notification for copy completion**

The long-lived native interaction thread may host a message-only window registered with `AddClipboardFormatListener`. For fallback capture:

1. record current clipboard sequence number;
2. send a synthetic `Ctrl+C` using `SendInput` from the user-triggered action;
3. wait for `WM_CLIPBOARDUPDATE` with a hard timeout of 250 ms while pumping that native thread's message queue;
4. verify sequence changed;
5. read `CF_UNICODETEXT` into owned memory;
6. restore the original allowed clipboard state;
7. ignore the restoration notification for capture purposes.

Do not use an endless `GetClipboardSequenceNumber` polling loop.

- [ ] **Step 5: Make restoration failure explicit**

If restoration fails, return/log a structured metadata-only native failure. Do not claim success while leaving the user's plain-text clipboard overwritten.

- [ ] **Step 6: Add fallback policy to `WindowsSelectionProvider`**

```text
try UIA
  success -> return UIA selection
  NoSelection -> do not synthesize copy if the app clearly reports no selected text
  Unsupported/native pattern unavailable -> try safe clipboard fallback
  clipboard unsafe to preserve -> return ClipboardPreservationUnsupported
```

Do not fallback on every UIA error blindly; distinguish “there is no selection” from “this provider does not expose selection.”

- [ ] **Step 7: Verify plain-text clipboard restoration manually**

Manual scenario:

1. copy a known marker such as `CLIPBOARD_SENTINEL_001`;
2. select different text in an application whose path exercises clipboard fallback;
3. trigger ClipLingo;
4. paste into Notepad;
5. verify pasted value is exactly `CLIPBOARD_SENTINEL_001`.

For unsupported clipboard state:

1. copy an image or rich clipboard object that exposes non-text formats;
2. trigger a fallback-required capture;
3. verify ClipLingo reports the preservation error and does not replace the clipboard.

If no readily available application forces fallback, expose the fallback implementation to a `#[cfg(test)]` Windows integration test rather than committing a hidden production debug switch.

- [ ] **Step 8: Run checks and commit**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path app/src-tauri/Cargo.toml --all
git add app/src-tauri
git commit -m "feat: add safe clipboard selection fallback"
```

---

## Task 10: Add monitor-aware positioning and focus-behavior verification

**Files:**
- Modify: `app/src-tauri/src/presentation/tauri_popup.rs`
- Modify: `app/src-tauri/src/platform/windows/cursor.rs`
- Modify: `app/src-tauri/src/platform/windows/mod.rs`
- Modify: `app/src-tauri/Cargo.toml` only if additional focused Windows features are required

**Interfaces:**
- Consumes: `ScreenRect` anchor and pure `place_popup()`
- Produces: work-area selection for the monitor containing the anchor and consistent physical-coordinate placement.

- [ ] **Step 1: Add pure tests for negative-coordinate monitors**

Extend positioning tests to cover a secondary monitor to the left of primary:

```text
work area x = -1920 .. 0
anchor x = -400
```

Assert the popup remains inside that negative-coordinate work area.

- [ ] **Step 2: Resolve the monitor work area containing the anchor**

Use Windows monitor APIs or Tauri's monitor APIs only if they preserve the same physical coordinate space used by UIA bounds. Do not mix logical CSS pixels with physical desktop pixels in the placement calculation.

Document the chosen coordinate convention in a code comment at the adapter boundary.

- [ ] **Step 3: Keep source focus stable**

Popup display must call show/move without explicitly focusing the popup. Do not call a focus API as part of normal translation display. Close/dismiss is available through the global shortcut and a visible close action; Esc-only dismissal is deferred because a non-focus-stealing popup cannot depend on receiving keyboard focus.

- [ ] **Step 4: Manual DPI/multi-monitor matrix**

Verify at minimum:

```text
100% scaling on primary monitor
one non-100% scaling setting if available
secondary monitor if available
selection near right edge
selection near bottom edge
selection near top-left
```

Record actual environment limitations rather than claiming untested configurations.

- [ ] **Step 5: Run full checks and commit**

```powershell
cd app
npm run check
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all
git add src-tauri
git commit -m "fix: make popup placement monitor aware"
```

---

## Task 11: Add Windows CI and iteration evidence

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `docs/benchmarks/iteration-001.md`
- Modify: `.agents/ITERATION_STATE.md`
- Modify: `README.md` only if a real development/run instruction now exists and is verified

**Interfaces:**
- Consumes: complete iteration implementation
- Produces: repeatable static/unit verification on GitHub Actions plus recorded native compatibility/performance evidence from a real Windows machine.

- [ ] **Step 1: Add one non-release Windows CI workflow**

Use `windows-latest`. CI should run on pull requests and pushes to `master` and must perform:

```text
checkout
setup Node 24 with npm cache using app/package-lock.json
install frontend dependencies with npm ci
install Rust stable
npm run check
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

Do not add release publishing, signing, installers, WinGet, Chocolatey, or updater steps to this workflow.

Keep working directories explicit so frontend commands execute in `app/` and Rust commands against `app/src-tauri/Cargo.toml`.

- [ ] **Step 2: Run the same gate locally before relying on CI**

From repository root:

```powershell
cd app
npm ci
npm run check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all
```

All must pass.

- [ ] **Step 3: Record compatibility evidence without source text**

`docs/benchmarks/iteration-001.md` begins with:

```markdown
# Iteration 001 Windows Interaction Evidence

## Environment
- Date:
- Windows version/build:
- CPU:
- RAM:
- Display/DPI configuration:
- ClipLingo commit:

## Compatibility matrix
| Application | Version | UIA/Clipboard | Bounds | Popup placement | Result |
|---|---|---|---|---|---|
```

Fill every field with actual observed data before marking the iteration done. If a target application is unavailable, write `Not available in test environment`; do not invent a pass.

- [ ] **Step 4: Record latency methodology and samples**

Instrument metadata-only `std::time::Instant` timestamps for:

```text
hotkey_received
popup_show_requested
capture_completed
fake_translation_completed
ready_render_requested
```

Never include text. Record at least 20 manual interactions after one warm launch and summarize p50/p95 for:

```text
hotkey -> popup show request
capture duration
hotkey -> ready request
```

The popup-show event is an application-side approximation; label it honestly rather than claiming it measures actual pixels rendered by WebView2.

- [ ] **Step 5: Record idle resource evidence**

After 60 seconds idle with the popup hidden and no devtools, record process memory and CPU using a reproducible Windows command such as:

```powershell
Get-Process cliplingo | Select-Object ProcessName, Id, WorkingSet64, PrivateMemorySize64, CPU
```

If Tauri dev mode materially inflates the value, also record a non-debug local build and label both measurements.

- [ ] **Step 6: Self-review privacy before completion**

Search repository source/log statements for accidental selected-text logging patterns. Review every log call in capture/coordinator code. There must be no `%text`, `{:?}` of `Selection`, or serialized `PopupViewModel` logs containing source/translation content.

- [ ] **Step 7: Update `ITERATION_STATE.md` using evidence**

If all exit criteria pass:

```text
Iteration 1 status -> DONE
include evidence path -> docs/benchmarks/iteration-001.md
record key findings/known compatibility gaps
promote Iteration 2 isolated inference worker to current iteration
```

If criteria do not pass, keep status `IN PROGRESS` and list the concrete blocker; do not start Iteration 2 merely because most work is complete.

- [ ] **Step 8: Final branch verification**

Run:

```powershell
git status --short
git diff master...HEAD --check
```

Expected: no accidental generated/build files and no whitespace errors. Verify `target/`, `node_modules/`, Vite output, logs, and local benchmark raw artifacts are ignored unless intentionally committed.

- [ ] **Step 9: Commit final iteration evidence**

```powershell
git add .github README.md .agents/ITERATION_STATE.md docs/benchmarks/iteration-001.md app
git commit -m "test: verify Windows interaction iteration"
```

If README did not need a change, omit it from `git add` rather than touching it unnecessarily.

---

## Final review gate before PR/merge

The implementer must perform this review before claiming Iteration 001 complete.

### Spec coverage

- [ ] Hotkey works globally with development shortcut `Ctrl+Alt+T`.
- [ ] Popup becomes visible immediately in Capturing state.
- [ ] UIA selection capture is primary.
- [ ] Clipboard fallback is safe-by-refusal for unsupported existing formats.
- [ ] Popup repositions near UIA bounds or cursor and remains inside monitor work area.
- [ ] Fake translator is deterministic and behind `Translator`.
- [ ] Rust owns state; Svelte only renders/dispatches narrow intent.
- [ ] Latest request wins and pending work is bounded to one slot.
- [ ] No selected/translated content in logs.
- [ ] No real ML/runtime/model dependency exists.
- [ ] Compatibility and performance evidence is real and recorded.

### Architecture smell scan

Reject/refactor before merge if any are present:

```text
Tauri command directly performs UIA + translation workflow
Svelte owns authoritative application state
HTTP localhost IPC
Redux/Zustand/TanStack Query for popup state
SvelteKit/router for the single popup
workspace split into many Rust crates
thread spawned per hotkey
unbounded queue of interaction requests
background clipboard polling
blind Ctrl+C that destroys arbitrary clipboard formats
selected text in logs
C++/CTranslate2/model code in Iteration 001
```

### Dependency scan

For every newly added dependency, the PR description must answer in one sentence why standard library / existing official facilities were insufficient. Expected external additions are small: Tauri/Svelte scaffold dependencies, official global-shortcut plugin, `windows`, and test tooling.

### PR and merge

1. Continue using the same Iteration 001 feature branch for all fixes found by tests/review.
2. Open one PR describing behavior, evidence, known compatibility limitations, performance measurements, and dependency additions.
3. Resolve mandatory CI/review blockers.
4. Squash merge so `master` receives one coherent Iteration 001 commit.
5. Delete the feature branch when tooling permits.
6. Do not tag a stable release from this iteration; it is an engineering/dev milestone, not the first production release.

---

## Iteration 002 handoff condition

Iteration 002 may begin only after Iteration 001 is marked DONE. The next plan should introduce the smallest isolated inference boundary:

```text
Rust WorkerSupervisor
    -> versioned Named Pipe protocol
    -> minimal C++ worker
    -> one real CTranslate2 smoke-model direction
    -> cold/warm latency + memory evidence
```

Do not carry UI Automation or popup refactoring into Iteration 002 unless Iteration 001 evidence shows a concrete blocker. Keep the next iteration focused on inference isolation, not broad language support.