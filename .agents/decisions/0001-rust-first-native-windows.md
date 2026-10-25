# ADR-0001 — Rust-first native Windows migration

Status: **Accepted Rust-first direction; clean-slate reset authorized, implementation pending**
Date: 2026-10-26

## Context

ClipLingo is a low-resource, offline, Windows-first translation utility. Its archived historical implementation and CI were C++/Win32/CMake, including COM UI Automation, DirectWrite/Direct2D popup, CTranslate2/SentencePiece inference, optional Tesseract OCR, explicit clipboard and Windows release packaging.

C++ makes application-owned native resource and exception/ABI ownership harder to reason about, especially across the UI message loop, native worker threads, COM and model dependencies. The product behavior and its Windows-native rendering constraints remain unchanged. Language alone does not determine latency, stability or install size.

## Decision

Choose **Rust-first native Windows** for the future application host, request coordinator, settings, capture policy, Win32 UI/geometry and other ClipLingo-owned application logic.

Retain underlying Win32/COM/DirectWrite APIs where appropriate. Native C or C++ libraries may remain as narrowly isolated dependencies via audited FFI if CTranslate2, SentencePiece, Tesseract or required platform support cannot be replaced with proven compatible Rust implementations. Do not mandate Tauri, WebView2, JavaScript, a service or a multi-process topology.

Implement Rust incrementally behind independent acceptance gates in `../engineering-design.md`. **Subsequent user decision (2026-10-26):** reset active repository content to `.agents/`, `AGENTS.md` and `DESIGN.md` only. Former C++ code, tests and release workflow are archived on `archive/pre-rust-clean-slate-20261026` at `24b8e3c30f390e6d571e2b15d1e1a56ba4781846`; no runnable active release baseline exists. This is an explicit exception to the prior incremental preservation plan, not evidence of Rust implementation.

## Why this choice

- Rust's ownership system and explicit `unsafe` boundaries can narrow application-level lifetime and memory-safety risk.
- It preserves the native background tray and text-only popup UX without requiring an additional web renderer.
- The archived C++ code and tests provide historical behavior and measurement references.
- The user explicitly selected Rust to reduce future C++ implementation complexity; this is not a measured performance claim.

## Considered alternatives

1. **Continue full C++:** avoids migration costs and native FFI boundary, but retains application-level memory/exception/lifetime risk and the maintenance concerns driving this decision.
2. **Rust host + selective native dependencies (chosen):** enables Rust ownership for application logic while retaining proven Windows APIs and model assets. Requires disciplined FFI, COM apartment handling, mixed toolchains and real Windows packaging tests.
3. **Tauri/WebView2 UI:** potentially simplifies settings development, but adds a web runtime and a second UI stack with no current product requirement; it is not selected as the default.

## Consequences and explicit non-claims

- Introduces migration effort and potentially complex native model/tokenizer/OCR bindings. No claim of fewer bugs, faster translation, smaller RAM or successful release is made before measurement.
- `unsafe`/FFI Windows adapters need documented ownership, thread affinity, panic/exception isolation and lifetime tests. Rust cannot forcibly cancel a blocked COM provider.
- The current user-visible product contract (`../product.md`) and UI contract (`../../DESIGN.md`) remain authoritative; The follow-up clean-slate reset removes implementation and CI files by explicit request, without implementing any Rust functionality.
- Architecture decisions such as model residency, helper-process topology, OS support and installer/update strategy remain open until evidenced.
- Record Rust cutover separately after end-to-end Windows parity and real GitHub Release verification. The last known releasable C++ artifact must remain recoverable throughout migration.
