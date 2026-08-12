# System Design

## Architectural thesis

Optimize the hot path, isolate the heavy/risky path, and keep the UI disposable.

- **Hot path:** shortcut, selection capture, window positioning and orchestration stay native in Rust/Windows APIs.
- **Heavy/risky path:** ML inference runs in an isolated C++ worker process.
- **Fast-changing path:** presentation is Svelte inside Tauri/WebView2.

## Target architecture

```text
Windows applications
      │ selected text + global shortcut
      ▼
┌─────────────────────────────────────────┐
│ Tauri 2 process / Rust application core│
│                                         │
│ hotkey                                  │
│ selection capture                      │
│ popup controller                       │
│ language detection                     │
│ translation routing                    │
│ model registry/manager                 │
│ worker supervisor                      │
│ config                                 │
└───────────────┬─────────────────────────┘
                │ narrow Tauri commands/events
                ▼
      ┌─────────────────────┐
      │ Svelte 5 UI         │
      │ popup/settings only │
      └─────────────────────┘

Rust worker client
      │ Windows Named Pipe
      ▼
┌─────────────────────────────┐
│ cliplingo-worker.exe        │
│ C++                         │
│ CTranslate2 + SentencePiece │
│ 1–2 loaded INT8 models     │
└──────────────┬──────────────┘
               ▼
       versioned model packs
```

## Core invariants

1. The Rust core does not depend on Svelte component structure.
2. Domain/application logic must not live in Tauri command handlers; handlers adapt transport to core use cases.
3. Selected text is processed locally in the normal path.
4. The inference worker does not know about windows, hotkeys, clipboard, UI state, or Tauri.
5. Worker failure must not take down the shell.
6. Only external or genuinely volatile boundaries receive interfaces/traits. Do not create an interface for every concrete type.
7. A model is a versioned dependency with license, checksum and runtime compatibility metadata.
8. Popup rendering must never block on model load/inference.

## Selection capture

Use a strategy pipeline:

```text
UI Automation TextPattern/GetSelection
        │ success -> text + bounding rectangles
        ▼ unsupported/failure
safe clipboard copy fallback
        │ success -> text; position from cursor/known bounds
        ▼ failure
structured capture error
```

Represent provenance so compatibility issues are diagnosable:

```text
Selection {
  text,
  source: UIAutomation | Clipboard,
  bounds: optional,
  confidence
}
```

Do not implement application-specific hacks until compatibility evidence justifies one.

## Translation workflow

`TranslateSelection` owns the user workflow:

```text
capture → normalize → detect → resolve route → request worker → present
```

Use monotonically increasing request IDs or equivalent cancellation identity. Interactive behavior is **latest request wins**; stale results are dropped. Do not queue unbounded hotkey requests.

## Translation routing

Represent installed translation directions as a small graph. Prefer a good direct route; allow a measured pivot route such as `ja -> en -> id` when a direct model is unavailable or lower quality.

Route selection may consider quality, installed state, model-load cost and pivot penalty. Start simple; use a small explicit scorer before introducing a generic graph library.

## Worker lifecycle

- Worker starts on first inference demand, not application boot unless benchmarks prove prewarming is required.
- Keep required model(s) warm for a bounded inactivity window.
- Keep at most what one active route needs, initially one or two models.
- Shut the worker down after inactivity so Windows can reclaim the process address space.
- Supervisor states: `Stopped`, `Starting`, `Ready`, `Busy`, `Failed`.
- Restart on unexpected failure at most a bounded number of times; never create an infinite restart loop.

## Model packs

A pack manifest must carry at least:

- model ID and version
- source/target language
- architecture/runtime compatibility
- quantization
- size
- SHA-256
- license/redistribution metadata

Installation flow: download to temporary location → verify → atomically move into final versioned directory. A partial download must never appear as installed.

## UI boundary

Rust is the source of truth. Svelte receives view models such as `PopupViewModel` and `SettingsViewModel` and emits narrow user intents.

Popup state is a state machine, not a bag of booleans:

```text
Hidden
Capturing
Translating
Ready
Error
```

Pre-create and hide the popup WebView if benchmarking shows this is needed for instant presentation. Settings may be created on demand.

## Storage and network

Start with versioned JSON configuration and the filesystem model registry. Introduce SQLite only when a real queryable persistence requirement exists.

Network access is limited to explicit update/model-download workflows. Translation itself must not require network access.

## Platform strategy

Windows is the product platform for V1. Keep translation core interfaces reasonably platform-agnostic, but do not degrade Windows integration or build speculative macOS/Linux abstraction layers before a port is planned.
