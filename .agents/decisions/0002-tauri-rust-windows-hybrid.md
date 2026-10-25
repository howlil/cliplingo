# ADR-0002 — Tauri v2 shell with Rust-owned native Windows behavior

Status: **Tauri shell accepted; popup choice and offline-only routing superseded by [ADR-0003](0003-latency-first-dual-mode-translation.md)**; implementation/performance not verified
Date: 2026-10-26
Supersedes: [ADR-0001](0001-rust-first-native-windows.md) **only for frontend/shell framework choice**. Its Rust-first ownership, privacy requirements, archived C++ recovery and unmeasured claims remain valid.

## Context

ClipLingo is a low-resource Windows offline EN→ID translator. The repository is documentation-only after a user-authorized clean-slate reset. The previous ADR preferred Rust + native Win32 without a default WebView, to avoid overhead and complexity. The user's later explicit decision is to research/build with **Tauri + Rust + native Windows integrations**.

Product constraints remain: background tray, Ctrl+Alt+T, user-selected/hover text through Windows UI Automation, offline local inference, above-cursor text-only popup, strict latest-request-wins, no focus theft, six-section Settings and privacy-first capture.

## Decision

1. Adopt **Tauri v2** as the desktop shell and application lifecycle. Settings is rendered using **React + TypeScript + WebView2**; Tauri Rust owns tray, shortcut, startup/shutdown, preferences, and bridges to native Windows behavior.
2. Keep **all privileged system actions in Rust**, with UI Automation via Windows COM/MTA worker, bounded capture policy, native OCR integration and local model/tokenizer ownership. Rust command permissions and per-window capability scopes are mandatory; TypeScript/renderer state is non-authoritative.
3. **Historical proposal, superseded by ADR-0003:** prototype a reusable Tauri WebviewWindow for result output, with Win32 fallback. **Current decision is the reverse:** native Rust-owned Win32/DirectWrite result HWND is the default, and WebView is a comparison prototype only. All actual no-focus, DPI and accessibility proofs are still required.
4. **Refined by ADR-0003:** offline local EN→ID is the default and must remain available without network after model install. Optional official online EN↔ID provider is user-consented; never silently upload selected text. The Rust-compatible native model runtime/FFI remains unresolved.
5. Treat **Windows.Media.Ocr desktop package identity** as a binding constraint; do not select it as the primary unpackaged/NSIS backend without a real package identity proof. Evaluate an unpackaged-compatible local OCR library/backend for explicitly selected regions.
6. Start with a **single-process** Tauri/Rust app. A supervised native capture/inference helper process is permitted only when actual blocked COM providers, native crashes, or ABI isolation require it.
7. Installer candidate is **Windows NSIS** using appropriate WebView2 bootstrapper policy; later updates use Tauri's **signed updater**. No auto-update reliability or signed release is claimed until end-to-end installed verification.

## Alternatives considered

| Option | Benefits | Risks / outcome |
| --- | --- | --- |
| Full Rust + pure native Win32 UI | potentially lower WebView footprint; direct HWND control | custom Settings and updater effort; **previous default superseded** |
| Tauri + Rust + WebView2 for Settings and popup | consistent fast-moving UI, Tauri lifecycle/plugins/installer | popup activation, exact DPI sizing, idle WebView cost; **superseded as default by ADR-0003** |
| Tauri Settings + native Win32 popup | combines Settings productivity with precise popup HWND | two render stacks; **now preferred by ADR-0003, subject to native acceptance** |
| Tauri + remote translation API | offers model-less online mode | **allowed only as explicit user-consented official provider**; silent uploads remain rejected by ADR-0003 |
| Tauri + Windows.Media.Ocr without package identity verification | native OCR integration | Microsoft desktop identity constraint; **not accepted without evidence** |

## Consequences

- Frontend is now justified and part of the selected shell architecture; do not assert that the application is "100% native UI" or "pure Rust." WebView2 has resource and runtime requirements.
- Native Windows COM, API security, DPI, accessibility, model FFI, backend packaging, WebView IPC and version signing remain engineering/test obligations.
- User-facing experience in [DESIGN.md](../../DESIGN.md) and product behavior in [product.md](../product.md) do **not** change. Tauri's windows are still Windows desktop windows; Settings content is rendered by WebView2.
- There is **no source implementation, Cargo project, CI or packaged Windows app** as of this decision. Documentation is not runtime proof.
- Implementation details, critical paths, blockers, acceptance and source citations live in [engineering-design.md](../engineering-design.md), not duplicated here.

## Explicit go/no-go gates

- **G1 popup + idle viability:** native HWND first; measure focus behavior, mixed DPI, accessibility, cold/warm latency and WebView comparison/resource use before making performance claims (ADR-0003).
- **G2 native capture validity:** real interactive Windows selection + foreground restrictions + COM provider stall mitigation.
- **G3 offline translation and installation:** validated model/tokenizer, no network requests from inference, clean-VM CPU/DLL/OCR compatibility.
- **G4 release:** signed NSIS installer + verified GitHub assets/latest metadata + in-app updater test; do not infer success from a CI artifact upload.

No numeric performance targets or win/loss estimates are approved without benchmarking.
