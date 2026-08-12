---
name: rust
description: Use for the native application core, concurrency, state machines, Windows adapters, worker supervision, and Rust API design.
---

# Rust Skill

## Style

- Prefer explicit domain types and enums over string flags.
- Use `Result` and structured errors; do not panic for recoverable runtime conditions.
- Keep ownership simple. Avoid `Arc<Mutex<...>>` as a default architecture; introduce shared synchronization only where lifecycle requires it.
- Keep native/unsafe details inside focused adapter modules.
- Document the safety assumption for every non-trivial `unsafe` block.
- Prefer standard library/existing crates before adding dependencies.
- Avoid macros/generics when straightforward concrete code is easier to understand.

## Core design

Rust is the source of truth for application state, request identity, popup state, translation routing, worker lifecycle, model metadata and configuration.

Use traits only at meaningful external seams. Test the core with fakes and keep Windows/Tauri details out of pure workflow tests.

## Concurrency

Interactive work uses cancellation identity/latest-request-wins. Do not create unbounded task queues. Long native/worker operations must not block the UI loop.

## Windows

Prefer Microsoft `windows-rs` bindings for Win32/COM access. Keep raw handle/COM lifetime logic inside Windows modules and convert results into safe domain structures before returning to core code.
