# Architecture

This file is the canonical source for ClipLingo system boundaries, responsibility ownership, runtime topology, and architecture invariants.

## Architecture thesis

Keep the interaction hot path native and small, isolate inference from the shell, and keep presentation replaceable.

```text
Windows application
  -> selected text + global shortcut
  -> Rust/Tauri shell
  -> InteractionCoordinator
  -> WorkerTranslator
  -> cliplingo-worker.exe
  -> local translation model pack
  -> Rust popup state
  -> Svelte presentation
```

## Current verified runtime

The integrated runtime currently uses:

```text
WindowsSelectionProvider
  -> InteractionCoordinator
  -> WorkerTranslator
  -> lazy-spawned cliplingo-worker.exe
  -> Windows Named Pipe \\.\pipe\cliplingo-worker-v1
  -> worker protocol v1
  -> C++ worker
  -> correlated response
  -> popup state/rendering
```

The worker boundary, process lifecycle, Named Pipe transport, request correlation, and shell integration are established. The current milestone is replacing the deterministic worker response with real local CTranslate2/SentencePiece inference; see `CURRENT_ITERATION.md` for current status.

## Responsibility ownership

### Rust application core

Owns:
- application workflow and state;
- selected-text orchestration;
- request identity and latest-request-wins semantics;
- popup state;
- translation routing decisions;
- worker lifecycle/supervision;
- configuration and model metadata orchestration.

Rust application logic must not depend on Svelte component structure.

### Windows platform adapters

Own Win32/COM/UI Automation, clipboard fallback, global shortcut integration, native positioning, and Named Pipe connection details. Raw handles, COM lifetimes, and `unsafe` stay inside focused Windows modules and are converted into safe application/domain types before crossing the boundary.

### Tauri

Tauri is the desktop host and transport bridge. Commands/events remain narrow adapters; product workflows do not live in command handlers. Do not introduce localhost HTTP for UI/native communication.

### Svelte

Svelte owns presentation only. It renders Rust-provided view models and emits narrow user intents. Durable application state remains in Rust.

### C++ inference worker

The worker owns inference-runtime concerns only: model load/use/unload, translation execution, health, and process-local runtime state. It must not own windows, hotkeys, clipboard, Tauri state, or user-interface behavior. Worker failure must not terminate the shell.

## Selection capture

Primary path:

```text
UI Automation TextPattern/GetSelection
  -> selected text + bounds when available
```

Fallback:

```text
safe clipboard copy
  -> preserve/restore clipboard as safely as practical
  -> cursor/known bounds for positioning when selection bounds are unavailable
```

Do not add application-specific capture hacks without a reproducible compatibility failure.

## Interaction semantics

Interactive translation is latest-request-wins. Each request has an identity; stale completions cannot overwrite a newer popup request. Do not create an unbounded hotkey/request queue.

Popup state is explicit rather than boolean combinations:

```text
Hidden -> Capturing -> Translating -> Ready
                         \-> Error
```

Worker lifecycle is explicit and restart-bounded:

```text
Stopped -> Starting -> Ready <-> Busy
                     \-> Failed
```

## Worker protocol and transport

The shell/worker contract is `docs/protocol/worker-v1.md`.

Protocol v1 is a bounded, language-agnostic binary frame with:
- `CLNG` magic;
- explicit protocol version and message type;
- little-endian `u64` request ID;
- little-endian `u32` payload length;
- maximum 1 MiB payload;
- UTF-8 request/translation bodies.

Windows Named Pipe is the current local transport. The protocol must not depend on Tauri, selection capture, popup state, or model implementation details.

## Translation and model boundary

The model/runtime implementation remains behind the worker boundary. The first committed offline alpha pack is `ja -> en -> id`, using CTranslate2 CPU INT8 conversion and SentencePiece assets as declared in `models/catalog/ja-id-opus-v1.json`.

A model pack is a versioned dependency. Its manifest must preserve model identity/version, source/target direction, runtime compatibility, quantization, checksum/integrity data, and redistribution/license metadata.

Partial or unverified model artifacts must never appear as installed. Installation uses temporary storage, verification, then atomic promotion into the final versioned location.

## Storage and network

Start with versioned local configuration and a filesystem model registry. Do not introduce a database until a real queryable persistence requirement exists.

Normal translation is offline. Network access is limited to explicit model/update acquisition workflows. Selected or translated text must not be sent to a cloud service in the normal path.

## Architecture invariants

1. Rust owns product workflow/application state.
2. Svelte owns presentation, not business state.
3. Tauri handlers stay thin.
4. Windows-specific code stays behind Windows adapters.
5. Inference remains isolated from the shell behind the worker contract.
6. Worker failure cannot take down the shell.
7. Traits/interfaces exist only at real external or replacement-worthy seams.
8. Selected/translated content stays local in the normal path and is not emitted to normal logs/telemetry.
9. Model/update artifacts are verified before activation.
10. Windows 10/11 is the V1 product platform; do not build speculative macOS/Linux abstraction layers before a port is planned.
