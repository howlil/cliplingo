# ADR-0001 — Rust-first native Windows migration

Status: **Accepted as migration direction, not implemented**
Date: 2026-10-26

## Context

ClipLingo is a low-resource, offline, Windows-first translation utility. Its current implementation and CI are C++/Win32/CMake, including COM UI Automation, DirectWrite/Direct2D popup, CTranslate2/SentencePiece inference, optional Tesseract OCR, explicit clipboard and Windows release packaging.

C++ makes application-owned native resource and exception/ABI ownership harder to reason about, especially across the UI message loop, native worker threads, COM and model dependencies. The product behavior and its Windows-native rendering constraints remain unchanged. Language alone does not determine latency, stability or install size.

## Decision

Choose **Rust-first native Windows** for the future application host, request coordinator, settings, capture policy, Win32 UI/geometry and other ClipLingo-owned application logic.

Retain underlying Win32/COM/DirectWrite APIs where appropriate. Native C or C++ libraries may remain as narrowly isolated dependencies via audited FFI if CTranslate2, SentencePiece, Tesseract or required platform support cannot be replaced with proven compatible Rust implementations. Do not mandate Tauri, WebView2, JavaScript, a service or a multi-process topology.

Implement incrementally behind independent migration acceptance gates in `../engineering-design.md`. Current C++ code and CI remain the release baseline until a Rust build has passed equivalent real-Windows functional, security, accessibility, reliability, performance and packaging checks. Do **not** interpret this ADR as approval to remove C++ sources today.

## Why this choice

- Rust's ownership system and explicit `unsafe` boundaries can narrow application-level lifetime and memory-safety risk.
- It preserves the native background tray and text-only popup UX without requiring an additional web renderer.
- The current code and tests provide concrete behavior and performance comparison baselines.
- The user explicitly selected Rust to reduce future C++ implementation complexity; this is not a measured performance claim.

## Considered alternatives

1. **Continue full C++:** avoids migration costs and native FFI boundary, but retains application-level memory/exception/lifetime risk and the maintenance concerns driving this decision.
2. **Rust host + selective native dependencies (chosen):** enables Rust ownership for application logic while retaining proven Windows APIs and model assets. Requires disciplined FFI, COM apartment handling, mixed toolchains and real Windows packaging tests.
3. **Tauri/WebView2 UI:** potentially simplifies settings development, but adds a web runtime and a second UI stack with no current product requirement; it is not selected as the default.

## Consequences and explicit non-claims

- Introduces migration effort and potentially complex native model/tokenizer/OCR bindings. No claim of fewer bugs, faster translation, smaller RAM or successful release is made before measurement.
- `unsafe`/FFI Windows adapters need documented ownership, thread affinity, panic/exception isolation and lifetime tests. Rust cannot forcibly cancel a blocked COM provider.
- The current user-visible product contract (`../product.md`) and UI contract (`../../DESIGN.md`) remain authoritative; M0 changes documentation only.
- Architecture decisions such as model residency, helper-process topology, OS support and installer/update strategy remain open until evidenced.
- Record Rust cutover separately after end-to-end Windows parity and real GitHub Release verification. The last known releasable C++ artifact must remain recoverable throughout migration.
