# Iteration 001 — Windows Interaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove ClipLingo's core Windows interaction without real ML: select text in another application, trigger a global shortcut, capture the selection, show a popup near the selection/cursor immediately, render a deterministic fake translation, and dismiss cleanly.

**Architecture:** Tauri 2 owns the desktop/WebView shell and Svelte 5 renders the popup only. Rust owns state and the workflow. A single dedicated interaction thread owns Windows UI Automation/clipboard work and sleeps on a one-slot latest-request-wins queue when idle; this keeps COM/Win32 lifetime on one thread and prevents hotkey spam from creating an unbounded backlog. Real C++/CTranslate2 inference is deferred to Iteration 002.

**Tech Stack:** Tauri 2, Rust stable, Svelte 5, TypeScript, Vite, Node.js 24 LTS + npm, `windows`/windows-rs, official Tauri global-shortcut plugin, Vitest + Testing Library, Cargo test.

## Global Constraints

- Execute this iteration on Windows 11 x64; avoid Windows 11-only APIs when a stable Windows 10+ API is sufficient.
- Use Node.js 24 LTS. Node 26 is Current, not the project baseline for this iteration.
- Tauri 2 + Svelte 5 + TypeScript + Vite. Do not add SvelteKit.
- Svelte is presentation only. Rust is the source of truth.
- Tauri command/event handlers are thin transport adapters.
- Windows API code stays under `app/src-tauri/src/platform/windows/`.
- No server, localhost HTTP, database, OCR, GPU stack, real translation model, C++ worker, model routing, history, updater, installer/release channel work in Iteration 001.
- Never log selected source text or translated text.
- Apply RED → GREEN → REFACTOR to behavior we control. OS integration that is not meaningfully unit-testable requires explicit Windows verification evidence.
- One iteration = one working branch. Small fixes remain on that branch. Final merge is squash.
- Add abstractions only at meaningful replacement/test seams. Iteration 001 has exactly three architectural ports: `SelectionProvider`, `Translator`, `PopupPort`.
- No background polling. The interaction thread sleeps on a `Condvar`; clipboard completion uses Windows clipboard notifications.
- Development shortcut is `Ctrl+Alt+T`. `Ctrl+Shift+T` is intentionally not the default because it conflicts with common browser/app behavior. Final configurable hotkeys are later scope.
- Popup must become visible in `Capturing` state before selection capture/translation completes.
- Performance numbers from this iteration are measurements, not release promises.

---

## Scope

```text
selected text in external Windows app
        ↓
Ctrl+Alt+T
        ↓
show popup immediately: Capturing…
        ↓
UI Automation selection capture
        ↓ if provider unsupported
safe clipboard fallback
        ↓
selection bounds OR cursor fallback
        ↓
monitor-aware popup placement
        ↓
FakeTranslator
        ↓
[FAKE] <selected text>
```

### Explicitly not in scope

- language detection
- real translation quality
- CTranslate2 / NLLB / Marian / other inference runtime
- C++ sidecar
- language packs/model downloads
- OCR
- translation history
- settings application
- autostart/updater/installers/package managers

## Exit criteria

Iteration 001 is DONE only when all are satisfied:

1. Frontend: `npm run check`, `npm test`, `npm run build` pass.
2. Rust: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all` pass.
3. Manual interaction succeeds with Notepad, one Chromium browser, VS Code, and one selectable PDF reader available on the test machine.
4. Older requests cannot overwrite a newer popup state.
5. The queue is bounded to one latest pending interaction; no thread is spawned per hotkey.
6. UIA is primary capture; fallback does not destroy an existing unsupported clipboard payload.
7. A plain-text clipboard is restored exactly after clipboard fallback.
8. Popup stays within monitor work area and is checked at 100% DPI plus one non-100% scaling environment if available.
9. No selected/translated text appears in logs.
10. `docs/benchmarks/iteration-001.md` records actual compatibility, latency, idle memory and idle CPU evidence.
11. `.agents/ITERATION_STATE.md` records actual outcome before Iteration 002 begins.

---

## Target repository shape

```text
cliplingo/
├── .nvmrc
├── .github/workflows/ci.yml
├── app/
│   ├── package.json
│   ├── package-lock.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── App.svelte
│   │   └── lib/popup/
│   │       ├── model.ts
│   │       ├── PopupView.svelte
│   │       └── PopupView.test.ts
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
└── docs/benchmarks/iteration-001.md
```

Do not create Rust workspace crates or a `worker/` directory yet. There is not enough code to justify them.

---

# Task 1 — Scaffold the minimal Tauri/Svelte application

**Files:**
- Create: `.nvmrc`
- Create: `app/**` from Vite/Tauri
- Modify: `app/package.json`
- Modify: `app/vite.config.ts`
- Modify: `app/src-tauri/tauri.conf.json`

**Produces:** runnable Tauri 2 + Svelte 5 application and repeatable frontend/Rust verification commands.

- [ ] **Step 1: pin Node LTS**

Create `.nvmrc`:

```text
24
```

- [ ] **Step 2: scaffold Svelte/TypeScript under `app/`**

From repository root:

```powershell
npm create vite@latest app -- --template svelte-ts
cd app
npm install
npm install @tauri-apps/api
npm install -D @tauri-apps/cli@latest vitest jsdom @testing-library/svelte @testing-library/jest-dom svelte-check
```

- [ ] **Step 3: initialize Tauri**

From `app/`:

```powershell
npx tauri init
```

Use:

```text
App name: ClipLingo
Window title: ClipLingo
Web assets: ../dist
Dev server URL: http://localhost:5173
Frontend dev command: npm run dev
Frontend build command: npm run build
```

- [ ] **Step 4: add explicit verification scripts**

`package.json` must provide at least:

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

Keep scaffold-required scripts if they differ; do not duplicate equivalent scripts.

- [ ] **Step 5: configure Vitest + ignore `src-tauri` in Vite watcher**

Use one `vite.config.ts`:

```ts
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  test: { environment: 'jsdom' },
  server: {
    watch: { ignored: ['**/src-tauri/**'] },
  },
});
```

- [ ] **Step 6: remove demo UI**

For now `app/src/App.svelte` is only:

```svelte
<main>ClipLingo</main>
```

No router, CSS framework, icon framework, state library or settings UI.

- [ ] **Step 7: verify scaffold**

```powershell
npm run check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all
npx tauri dev
```

Expected: checks pass and one basic Tauri window opens.

- [ ] **Step 8: commit intermediate GREEN state**

```powershell
git add .nvmrc app
git commit -m "chore: scaffold Tauri Svelte application"
```

---

# Task 2 — Core contracts, state machine and placement policy

**Files:**
- Create: `app/src-tauri/src/core/mod.rs`
- Create: `app/src-tauri/src/core/types.rs`
- Create: `app/src-tauri/src/core/ports.rs`
- Create: `app/src-tauri/src/core/popup.rs`
- Create: `app/src-tauri/src/core/positioning.rs`
- Modify: `app/src-tauri/src/lib.rs`

**Produces:** platform-neutral state/types and the three allowed architectural ports.

## Interfaces

```rust
pub trait SelectionProvider: Send {
    fn capture(&mut self) -> Result<Selection, CaptureError>;
}

pub trait Translator: Send {
    fn translate(&mut self, request: &TranslationRequest)
        -> Result<Translation, TranslationError>;
}

pub trait PopupPort: Send + Sync {
    fn show(&self, state: PopupViewModel);
    fn update(&self, state: PopupViewModel);
    fn move_to(&self, x: f64, y: f64);
    fn hide(&self);
}
```

`SelectionProvider` and `Translator` are moved into the dedicated interaction thread and therefore do not need `Sync`. This prevents future COM/native state from being shared across threads merely to satisfy an unnecessary trait bound. `PopupPort` is callable by both trigger and worker paths and remains `Send + Sync`.

- [ ] **Step 1: write RED state-machine tests**

In `core/popup.rs`:

```rust
#[test]
fn new_session_is_hidden() {
    let session = PopupSession::default();
    assert_eq!(session.snapshot(), PopupState::Hidden);
}

#[test]
fn stale_completion_cannot_replace_new_request() {
    let mut session = PopupSession::default();
    let first = session.begin_request();
    let second = session.begin_request();

    assert!(matches!(
        session.complete(first, Translation { text: "old".into() }),
        ApplyResult::Stale
    ));

    session.mark_translating(second, selection("new")).unwrap();
    assert!(matches!(
        session.complete(second, Translation { text: "new-result".into() }),
        ApplyResult::Applied(PopupState::Ready { .. })
    ));
}
```

Run:

```powershell
cargo test --manifest-path app/src-tauri/Cargo.toml core::popup
```

Expected RED: types do not exist yet.

- [ ] **Step 2: implement minimal value types**

`core/types.rs`:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
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

Errors:

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

No user-facing strings in native/domain errors.

- [ ] **Step 3: implement popup state as an enum**

Required states:

```rust
pub enum PopupState {
    Hidden,
    Capturing { request_id: u64 },
    Translating { request_id: u64, source_text: String },
    Ready {
        request_id: u64,
        source_text: String,
        translated_text: String,
    },
    Error { request_id: u64, code: PopupErrorCode },
}
```

`PopupSession` owns monotonically increasing request IDs and rejects state changes from a non-current request.

- [ ] **Step 4: create a serializable UI projection**

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

Svelte receives this view model, not `PopupSession` or native errors.

- [ ] **Step 5: write RED placement tests**

Test normal, right-edge clamp, bottom flip, and negative-coordinate secondary-monitor cases.

Example:

```rust
#[test]
fn flips_above_when_bottom_would_overflow() {
    let result = place_popup(
        rect(100.0, 900.0, 80.0, 20.0),
        size(420.0, 180.0),
        rect(0.0, 0.0, 1920.0, 1040.0),
        8.0,
    );
    assert_eq!(result.y, 712.0);
}
```

- [ ] **Step 6: implement placement without geometry dependency**

Policy:

```text
x = anchor.right
below = anchor.bottom + margin
if below + popup.height <= work_area.bottom:
    y = below
else:
    y = anchor.top - popup.height - margin
x = clamp(x, work_area.left, work_area.right - popup.width)
y = clamp(y, work_area.top, work_area.bottom - popup.height)
```

- [ ] **Step 7: GREEN gate**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo test --manifest-path app/src-tauri/Cargo.toml core
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

- [ ] **Step 8: commit**

```powershell
git add app/src-tauri/src/core app/src-tauri/src/lib.rs
git commit -m "feat: define interaction core"
```

---

# Task 3 — Svelte popup presentation

**Files:**
- Create: `app/src/lib/popup/model.ts`
- Create: `app/src/lib/popup/PopupView.svelte`
- Create: `app/src/lib/popup/PopupView.test.ts`
- Modify: `app/src/App.svelte`

**Consumes:** `PopupViewModel` only.

- [ ] **Step 1: create the TS contract**

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

- [ ] **Step 2: write RED component tests**

```ts
it('shows capture feedback immediately', () => {
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

it('shows ready translation', () => {
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
```

Run:

```powershell
cd app
npm test -- PopupView.test.ts
```

Expected RED: component missing.

- [ ] **Step 3: implement `PopupView.svelte` using Svelte 5 props**

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
      <div class="error" data-error-code={model.errorCode}>
        Unable to translate this selection.
      </div>
    {/if}
  </section>
{/if}
```

Use local CSS only: rounded surface, border/shadow, readable typography, sane max width and dark-compatible variables. Do not add Tailwind just for one popup.

- [ ] **Step 4: make `App.svelte` a thin Tauri bridge**

On mount:

1. subscribe to `popup-state`;
2. call `get_popup_state` after subscription;
3. assign event/command result to one local `$state<PopupViewModel>`;
4. render `PopupView`.

No Svelte store library, Redux, Zustand or TanStack Query.

- [ ] **Step 5: GREEN gate**

```powershell
npm run check
npm test
npm run build
```

- [ ] **Step 6: commit**

```powershell
git add app/src app/package.json app/package-lock.json app/vite.config.ts
git commit -m "feat: add popup presentation"
```

---

# Task 4 — Bounded interaction coordinator and fake translator

**Files:**
- Create: `app/src-tauri/src/application/mod.rs`
- Create: `app/src-tauri/src/application/coordinator.rs`
- Modify: `app/src-tauri/src/lib.rs`

**Consumes:** the three ports and `PopupSession`.

**Produces:** `InteractionCoordinator::trigger()` / `dismiss()` and one long-lived worker thread.

- [ ] **Step 1: RED test for one-slot queue**

Implement/test a small `PendingSlot<T>` backed by `Mutex<Option<T>> + Condvar`:

```rust
#[test]
fn pending_slot_keeps_only_latest() {
    let slot = PendingSlot::default();
    slot.submit(1);
    slot.submit(2);
    slot.submit(3);
    assert_eq!(slot.take_now(), Some(3));
    assert_eq!(slot.take_now(), None);
}
```

This is deliberate coalescing. Do not introduce a queue crate.

- [ ] **Step 2: RED test end-to-end workflow with fakes**

Use fakes for all ports. The translator output is exactly:

```text
[FAKE] <source text>
```

Verify request 1 cannot overwrite request 2 if request 2 becomes current first.

- [ ] **Step 3: implement `FakeTranslator`**

```rust
pub struct FakeTranslator;

impl Translator for FakeTranslator {
    fn translate(
        &mut self,
        request: &TranslationRequest,
    ) -> Result<Translation, TranslationError> {
        Ok(Translation {
            text: format!("[FAKE] {}", request.text),
        })
    }
}
```

No fake framework dependency.

- [ ] **Step 4: implement worker ownership correctly**

`InteractionCoordinator` should keep `PopupSession`, pending slot and an `Arc<dyn PopupPort>`. `SelectionProvider` and `Translator` are moved into the dedicated worker thread when it starts.

Workflow:

```text
trigger()
  -> begin new request id
  -> immediately popup.show(Capturing)
  -> overwrite pending slot with newest request
  -> Condvar.notify_one()

worker thread
  -> waits on Condvar while no request
  -> capture selected text
  -> if stale: discard
  -> mark Translating
  -> translate with FakeTranslator
  -> if stale: discard
  -> Ready/Error
```

No thread per hotkey. No unbounded channel.

- [ ] **Step 5: structured error mapping**

Map native/domain errors to stable UI codes:

```text
NoSelection -> no_selection
ClipboardPreservationUnsupported -> clipboard_preservation_unsupported
other capture failure -> capture_unavailable
translation failure -> translation_failed
```

- [ ] **Step 6: GREEN gate**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo test --manifest-path app/src-tauri/Cargo.toml application
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

- [ ] **Step 7: commit**

```powershell
git add app/src-tauri/src/application app/src-tauri/src/lib.rs
git commit -m "feat: add bounded interaction coordinator"
```

---

# Task 5 — Tauri popup port, hotkey and cursor-first placement

**Files:**
- Create: `app/src-tauri/src/presentation/mod.rs`
- Create: `app/src-tauri/src/presentation/tauri_popup.rs`
- Create: `app/src-tauri/src/platform/mod.rs`
- Create: `app/src-tauri/src/platform/windows/mod.rs`
- Create: `app/src-tauri/src/platform/windows/hotkey.rs`
- Create: `app/src-tauri/src/platform/windows/cursor.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/tauri.conf.json`
- Modify: `app/src-tauri/Cargo.toml`

- [ ] **Step 1: configure one pre-created hidden popup window**

Required behavior:

```text
label: popup
visible: false
resizable: false
decorations: false
always-on-top: true
skip taskbar: true
initial size: 420 x 180
```

No settings/main dashboard window.

- [ ] **Step 2: implement `TauriPopupPort`**

Only:

```text
show -> emit PopupViewModel + show window
update -> emit PopupViewModel
move_to -> set window position
hide -> hide window
```

No capture/translation logic here.

- [ ] **Step 3: add narrow commands**

```text
get_popup_state() -> PopupViewModel
dismiss_popup() -> ()
```

Capabilities must not enable arbitrary shell/process/filesystem/network access for the WebView.

- [ ] **Step 4: add official global shortcut plugin**

From `app/`:

```powershell
npm run tauri -- add global-shortcut
```

Register in Rust only:

```rust
pub const TRANSLATE_SHORTCUT: &str = "Ctrl+Alt+T";
```

Registration failure is visible in metadata logs; do not silently start a broken interaction.

- [ ] **Step 5: hotkey toggle behavior**

```text
Popup Hidden -> coordinator.trigger()
Popup non-Hidden -> coordinator.dismiss()
```

- [ ] **Step 6: implement cursor fallback through Win32**

Use `GetCursorPos` in `cursor.rs`. Raw Win32/unsafe stays inside that file with a safety comment next to each unsafe block.

Before capture finishes, place the popup from cursor coordinates so feedback is immediate.

- [ ] **Step 7: manual hotkey smoke test**

1. Run `npm run tauri dev`.
2. Focus Notepad.
3. Press `Ctrl+Alt+T`.
4. Popup must appear immediately even before real selection capture exists.
5. Press again; popup hides.

The OS registration itself is an integration boundary, not a useful unit-test target.

- [ ] **Step 8: GREEN gate + commit**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path app/src-tauri/Cargo.toml --all
git add app
git commit -m "feat: add popup shell and global hotkey"
```

---

# Task 6 — Windows UI Automation selection capture

**Files:**
- Create: `app/src-tauri/src/platform/windows/selection.rs`
- Modify: `app/src-tauri/src/platform/windows/mod.rs`
- Modify: `app/src-tauri/Cargo.toml`

**Produces:** `WindowsSelectionProvider`, primary UIA path.

- [ ] **Step 1: add focused windows-rs features only**

Enable only feature groups required by compiled APIs, expected to include equivalents of:

```text
Win32_Foundation
Win32_System_Com
Win32_UI_Accessibility
Win32_UI_WindowsAndMessaging
```

Do not enable broad Win32 feature sets for convenience.

- [ ] **Step 2: initialize COM once on the interaction thread**

The coordinator's worker thread initializes its COM apartment before constructing/using `WindowsSelectionProvider`, and uninitializes on worker shutdown. Do not initialize/uninitialize COM in every capture call and do not share COM objects across unrelated Tauri threads.

- [ ] **Step 3: implement primary capture**

Algorithm:

```text
GetFocusedElement
  -> get TextPattern
  -> GetSelection
  -> choose first non-empty selected range
  -> GetText(-1)
  -> GetBoundingRectangles
  -> choose last non-empty visible rectangle as popup anchor
```

A degenerate insertion range/empty text is `CaptureError::NoSelection`.

If focused element does not expose the pattern, return `Unsupported`; Task 7 may attempt clipboard fallback. Do not recursively search the whole UIA tree or add browser-specific hacks without measured compatibility evidence.

- [ ] **Step 4: RED/GREEN pure-helper tests**

Extract/test only non-COM selection logic:

```text
ignore empty candidate text
choose usable selected text
choose final non-empty rectangle
return NoSelection when nothing usable exists
```

Do not mock COM interfaces for coverage theater.

- [ ] **Step 5: privacy-safe metadata only**

Allowed:

```text
request=17 capture=uia chars=14 bounds=true duration_ms=11 status=ok
```

Forbidden: selected text, translation text, `Debug` dump of `Selection`, serialized popup state containing user content.

- [ ] **Step 6: reposition after UIA capture**

Immediate popup position remains cursor-based. Once UIA bounds are available, resolve monitor work area and call `place_popup()` before Translating/Ready presentation.

Keep one physical desktop coordinate convention from UIA bounds through monitor work-area calculation to final Tauri window placement. Document conversions at the adapter boundary.

- [ ] **Step 7: manual UIA compatibility evidence**

Check available versions of:

```text
Notepad
Chrome or Edge
VS Code
selectable PDF reader/browser PDF viewer
```

Record only: app/version, capture source, bounds available yes/no, outcome. Do not record selected text.

- [ ] **Step 8: GREEN gate + commit**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path app/src-tauri/Cargo.toml --all
git add app/src-tauri
git commit -m "feat: capture selection with Windows UI Automation"
```

---

# Task 7 — Safe clipboard fallback

**Files:**
- Create: `app/src-tauri/src/platform/windows/clipboard.rs`
- Modify: `app/src-tauri/src/platform/windows/selection.rs`
- Modify: `app/src-tauri/Cargo.toml`

## Safety policy

Iteration 001 does not pretend it can safely clone every arbitrary Windows clipboard format. Fallback is allowed only when the old clipboard is:

```text
empty
OR
plain Unicode text only
```

If additional/custom formats exist, return `ClipboardPreservationUnsupported` without sending `Ctrl+C`. Failing translation is better than destroying the user's clipboard.

- [ ] **Step 1: RED tests for the preservation policy**

```rust
#[test]
fn empty_clipboard_is_safe() {
    assert!(clipboard_is_safely_restorable(&[]));
}

#[test]
fn unicode_text_only_is_safe() {
    assert!(clipboard_is_safely_restorable(&[CF_UNICODETEXT_ID]));
}

#[test]
fn mixed_formats_are_refused() {
    assert!(!clipboard_is_safely_restorable(&[
        CF_UNICODETEXT_ID,
        49_301,
    ]));
}
```

- [ ] **Step 2: inspect/snapshot before modifying**

1. enumerate formats;
2. refuse unsupported format sets;
3. snapshot old Unicode text into owned memory, or remember that clipboard was empty;
4. only then begin synthetic copy.

- [ ] **Step 3: use Windows notification, not a background polling loop**

On the same long-lived native interaction thread:

1. create a message-only window;
2. register `AddClipboardFormatListener`;
3. record current clipboard sequence;
4. send `Ctrl+C` via `SendInput`;
5. wait for `WM_CLIPBOARDUPDATE` with a hard 250 ms deadline while pumping the thread's native message queue;
6. verify sequence changed;
7. read `CF_UNICODETEXT`;
8. restore previous allowed clipboard state;
9. ignore the restoration update for capture purposes.

- [ ] **Step 4: fallback rules**

```text
UIA success -> return it
UIA NoSelection -> return NoSelection; do not synthesize copy
UIA Unsupported/provider unavailable -> safe clipboard fallback
unsafe existing clipboard -> ClipboardPreservationUnsupported
```

Do not blindly `Ctrl+C` after every UIA failure.

- [ ] **Step 5: restoration failure is a real error**

Do not return successful translation if ClipLingo failed to restore the plain-text clipboard it replaced. Return/log structured metadata-only failure.

- [ ] **Step 6: manual preservation test**

Plain text:

1. copy `CLIPBOARD_SENTINEL_001`;
2. trigger a path that uses clipboard fallback;
3. paste into Notepad afterward;
4. pasted value must still be exactly `CLIPBOARD_SENTINEL_001`.

Unsupported payload:

1. place an image/rich object on clipboard;
2. force/encounter fallback-required capture through a test-only integration harness;
3. fallback must refuse before overwriting it.

Do not commit a production debug switch just to force fallback.

- [ ] **Step 7: GREEN gate + commit**

```powershell
cargo fmt --manifest-path app/src-tauri/Cargo.toml
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path app/src-tauri/Cargo.toml --all
git add app/src-tauri
git commit -m "feat: add safe clipboard capture fallback"
```

---

# Task 8 — Focus, DPI/multi-monitor compatibility and performance evidence

**Files:**
- Modify: `app/src-tauri/src/presentation/tauri_popup.rs`
- Modify: `app/src-tauri/src/platform/windows/cursor.rs`
- Create: `docs/benchmarks/iteration-001.md`

- [ ] **Step 1: preserve source-app focus**

Normal translation display shows/moves the popup without explicitly focusing it. Do not call focus as part of `show()`.

Dismiss through the same global shortcut and visible popup close action. Esc-only dismissal is deferred because a non-focus-stealing popup cannot rely on receiving keyboard events.

- [ ] **Step 2: verify monitor work-area behavior**

Test:

```text
100% primary monitor
one non-100% scaling setting if available
secondary monitor if available
right edge
bottom edge
top-left
negative-coordinate secondary monitor if available
```

If an environment is unavailable, say so in evidence; do not record a fake pass.

- [ ] **Step 3: instrument metadata-only timings**

Use `std::time::Instant` for:

```text
hotkey_received
popup_show_requested
capture_completed
fake_translation_completed
ready_show_requested
```

No source/translation content in timing logs.

- [ ] **Step 4: record at least 20 warm interactions**

Summarize p50/p95 for:

```text
hotkey -> popup show request
capture duration
hotkey -> ready show request
```

Be explicit that `popup_show_requested` is application-side timing, not guaranteed WebView pixel-present timing.

- [ ] **Step 5: record idle resource evidence**

After 60 seconds idle with popup hidden and no devtools:

```powershell
Get-Process cliplingo | Select-Object ProcessName, Id, WorkingSet64, PrivateMemorySize64, CPU
```

Record dev build and a non-debug local build separately if dev mode materially changes memory.

- [ ] **Step 6: write the evidence document with observed values only**

Create `docs/benchmarks/iteration-001.md` during execution. It must contain these sections populated directly with actual observations; do not commit placeholder values:

```markdown
# Iteration 001 Windows Interaction Evidence

## Environment
Record execution date, Windows version/build, CPU, RAM, display/DPI configuration, and tested ClipLingo commit SHA as explicit bullet values.

## Compatibility
| Application | Version | Capture path | Bounds | Placement | Result |
|---|---|---|---|---|---|

## Latency
| Metric | Samples | p50 | p95 |
|---|---:|---:|---:|

## Idle resources
| Build | Working set | Private memory | CPU observation |
|---|---:|---:|---|

## Known gaps
List only limitations actually observed in this iteration.
```

If an observation cannot be collected, state why it was unavailable; do not invent a value.

- [ ] **Step 7: commit evidence-producing code and the populated evidence**

```powershell
git add app docs/benchmarks/iteration-001.md
git commit -m "test: measure Windows interaction behavior"
```

---

# Task 9 — CI, privacy review, iteration-state closeout

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `.agents/ITERATION_STATE.md`
- Modify: `README.md` only if verified development instructions now exist and materially help contributors

- [ ] **Step 1: add one Windows CI workflow**

Trigger on PR and pushes to `master`. Use `windows-latest` and Node 24.

Required gate:

```text
checkout
setup Node 24 with npm cache from app/package-lock.json
npm ci
npm run check
npm test
npm run build
install/use Rust stable
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

No release/signing/install/package-manager workflow yet.

- [ ] **Step 2: run the same gate locally**

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

- [ ] **Step 3: privacy log review**

Inspect every log call in coordinator/capture/popup adapters. Reject any logging of:

```text
selected text
translated text
Selection Debug dump
PopupViewModel Debug/JSON dump containing text
clipboard contents
```

Metadata such as request id, char count, capture strategy, error code and duration is allowed.

- [ ] **Step 4: repository hygiene**

From root:

```powershell
git status --short
git diff master...HEAD --check
```

`node_modules/`, `target/`, `dist/`, local logs and unneeded benchmark raw data must not be committed.

- [ ] **Step 5: update iteration state from evidence**

If every exit criterion passes:

```text
Iteration 1 -> DONE
Evidence -> docs/benchmarks/iteration-001.md
Record actual compatibility gaps and measured numbers
Iteration 2 isolated inference worker becomes next/current candidate
```

If any exit criterion fails, status remains `IN PROGRESS` and the specific blocker is listed. Do not start Iteration 002 merely because most of Iteration 001 works.

- [ ] **Step 6: final commit on the same branch**

```powershell
git add .github .agents/ITERATION_STATE.md app docs
git commit -m "test: verify Windows interaction iteration"
```

If verified README development instructions changed, include `README.md` in the same commit; otherwise leave it untouched.

---

# Final review gate

Before PR/merge, all boxes below must be true.

## Behavior

- [ ] `Ctrl+Alt+T` triggers from another Windows application.
- [ ] Popup appears immediately in Capturing state.
- [ ] UIA is primary selected-text capture.
- [ ] Clipboard fallback refuses unsafe-to-restore clipboard states.
- [ ] Plain-text clipboard restoration is verified.
- [ ] Popup uses selection bounds when available and cursor otherwise.
- [ ] Popup stays inside monitor work area.
- [ ] Fake translator output is deterministic.
- [ ] Second hotkey/dismiss hides popup.
- [ ] Older work cannot overwrite a newer request.

## Architecture

- [ ] Svelte contains presentation logic only.
- [ ] Rust owns authoritative popup/request state.
- [ ] Tauri handlers do not contain capture/translation workflows.
- [ ] Windows code is contained under the Windows adapter.
- [ ] Exactly one interaction worker thread owns UIA/clipboard native state.
- [ ] Provider/translator native state is not forced to `Sync` across threads.
- [ ] Pending requests are a one-slot latest-wins buffer.
- [ ] No localhost server or internal REST/gRPC.
- [ ] No C++/ML/model code entered Iteration 001.

## Testing/performance/security

- [ ] Rust/frontend automated checks pass.
- [ ] Windows compatibility evidence is actual, not assumed.
- [ ] DPI/monitor limitations are documented honestly.
- [ ] Initial latency/memory/CPU measurements exist.
- [ ] No selected/translated text appears in logs.
- [ ] Tauri WebView capabilities remain narrow.

## Smell rejection list

Refactor before merge if any appear:

```text
thread per hotkey
unbounded request queue
COM/UIA object shared across arbitrary runtime threads
background clipboard polling
blind Ctrl+C overwriting arbitrary clipboard formats
Tauri command doing the whole workflow
Svelte as source of truth
Redux/Zustand/TanStack Query for popup state
SvelteKit/router for this single popup
many Rust workspace crates
HTTP localhost IPC
selected text in logs
real inference dependency
```

## PR/merge discipline

1. Use the same Iteration 001 branch for test/review fixes.
2. Open one PR containing implementation + tests + evidence + docs.
3. Resolve mandatory CI/review blockers.
4. Squash merge so `master` receives one coherent Iteration 001 commit.
5. Delete the branch when tooling permits.
6. Do not create a stable release/tag from this engineering spike.

---

# Iteration 002 handoff

Only after Iteration 001 is DONE, the next plan may introduce:

```text
Rust WorkerSupervisor
  -> minimal versioned Windows Named Pipe protocol
  -> isolated C++ worker
  -> CTranslate2
  -> one real translation direction
  -> cold/warm latency + memory measurement
```

Do not broaden that next iteration into all-language routing, OCR, history, installer ecosystems or other unrelated features.