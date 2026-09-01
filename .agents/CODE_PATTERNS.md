# Code Patterns

This file owns ClipLingo-specific implementation conventions that are expensive or error-prone to reconstruct from individual files.

## Boundary-first ports, concrete internals

Use traits only at real external or replacement-worthy seams such as selection capture and translation execution. Internal helpers/services remain concrete until substitution is actually needed.

Current meaningful seams include:

```text
application core -> SelectionProvider -> Windows implementation
application core -> Translator -> WorkerTranslator -> C++ worker
Svelte -> narrow Tauri commands/view models -> Rust core
```

Do not add parallel implementations merely to prove replaceability.

## Workflow ownership

Keep the selected-text workflow cohesive. `InteractionCoordinator`/application workflow owns orchestration rather than a chain of tiny service/use-case types.

Conceptually:

```text
capture -> normalize/prepare -> translate -> present
```

As language routing/model management grows, keep those decisions in Rust application/core ownership rather than Tauri or Svelte.

## Explicit state machines

Prefer explicit states over boolean combinations.

Popup:

```text
Hidden -> Capturing -> Translating -> Ready
                         \-> Error
```

Worker:

```text
Stopped -> Starting -> Ready <-> Busy
                     \-> Failed
```

Invalid transitions should fail explicitly without silently mutating state.

## Latest request wins

Interactive requests carry identity. Once request `N+1` is current, completion from `N` cannot replace its popup state. Do not introduce an unbounded queue for hotkey-triggered translation.

## Thin Tauri boundary

Tauri command/event code validates transport input, delegates to application/core code, and maps results. It must not directly own selection capture, language/model routing, worker lifecycle, or configuration workflow.

## View models at the UI boundary

Svelte receives presentation data rather than Rust internals. Rust remains the durable application-state authority; UI reload/recreation must not redefine product state ownership.

## Windows/native isolation

Keep Win32/COM/UI Automation/clipboard/raw handle details in focused Windows modules. Document non-obvious safety/lifetime assumptions near `unsafe` code. Return safe structures/errors to the application layer.

## Worker protocol v1

`docs/protocol/worker-v1.md` is the wire contract.

A frame is exactly an 18-byte header plus payload:

```text
0..4   ASCII magic CLNG
4      protocol version = 1
5      message type
6..14  request ID, u64 little-endian
14..18 payload length, u32 little-endian
18..   payload bytes
```

Maximum payload is 1 MiB. Reject oversized declarations before allocating/copying the body. Translate request/response payloads are UTF-8; error payload is the protocol-defined one-byte code.

Reuse the existing bounded framed read/write implementation. Windows Named Pipe is transport, not a reason to create a second protocol. Always validate request-ID correlation and never log raw payloads.

## Structured errors

Use actionable typed errors at application/platform/protocol boundaries. Map them to user-facing state at the presentation boundary. Recoverable runtime conditions must not crash the shell.

## Privacy-safe diagnostics

Allowed diagnostics include request ID, character/byte count, capture source, route identifier, worker state, duration, and non-sensitive status/error code.

Never emit selected source text, translated text, or raw worker payloads in normal logs/telemetry.

## Configuration and model installation

Start with schema/versioned local configuration and filesystem model registry. Do not add SQLite for simple preferences.

Model installation follows:

```text
download/acquire -> temporary location -> verify manifest/hash/license/runtime compatibility -> atomic promotion
```

A partial or unverified artifact is not installed.

## Dependency placement

Agnosticism lives at meaningful seams, not wrapper layers. CTranslate2/SentencePiece stay behind the worker boundary; Windows bindings stay behind Windows adapters; Tauri stays at the UI/native transport boundary. Replacing one should not require rewriting the application workflow.
