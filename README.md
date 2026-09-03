# ClipLingo

ClipLingo is a Windows-first, offline translation utility built around one interaction: select text in another application, press a global shortcut, and read the translation in a lightweight popup near the selection.

The product prioritizes reliable text capture, privacy, low perceived latency, near-zero idle CPU, and strong CJK/English-to-Indonesian translation quality before broad language coverage.

## Real Offline Translation Alpha

The first production route is Japanese -> English -> Indonesian and runs locally through OPUS-MT models after the required model pack is installed.

```text
Select Japanese text
        -> Ctrl+Shift+T
        -> Windows selection capture
        -> local OPUS ja -> en
        -> local OPUS en -> id
        -> Indonesian result in the ClipLingo popup
```

A fresh installation does not need developer setup. If the model pack is missing, the popup exposes an explicit **Install offline model** action. ClipLingo downloads the release-owned pack, verifies its SHA256 before activation, and keeps normal translation on-device. Raw selected and translated text is not written to normal diagnostics.

This alpha intentionally covers one route. OCR, cloud fallback, GPU inference, accounts/sync, and additional language routes are not part of the milestone.

## Install and use

ClipLingo 0.1.0 Alpha 1 is distributed as a Windows x64 NSIS installer from GitHub Releases after release qualification succeeds.

1. Install ClipLingo.
2. Select Japanese text in a browser, editor, PDF reader, chat application, or another supported Windows application.
3. Press `Ctrl+Shift+T`.
4. If prompted, choose **Install offline model** once and let ClipLingo download and verify the Japanese -> Indonesian pack.
5. Press the shortcut again to translate; the result appears in the popup near the selection or cursor.
6. Press Escape or use the close action to dismiss the popup.

The alpha installer is currently unsigned, so Windows SmartScreen may show a warning on first install.

## Current architecture

- **Presentation:** Tauri 2 + Svelte 5 + TypeScript + Vite.
- **Application core:** Rust owns workflow, popup state, request identity, model lifecycle, routing, and worker lifecycle.
- **Windows integration:** `windows-rs`, Win32/COM/UI Automation, clipboard fallback, global shortcut, positioning, and Named Pipe transport.
- **Inference:** isolated `cliplingo-worker.exe` C++ process behind worker protocol v1.
- **Native runtime:** source-pinned CTranslate2 + SentencePiece, statically linked with one coherent MSVC CRT policy.
- **Models:** versioned local language packs; the first alpha route is OPUS-MT `ja -> en -> id`, CPU INT8.
- **Normal translation path:** local/offline after the required model pack is installed.

## Release qualification

The alpha tag is release-gated. CI must successfully build the Windows native runtime, execute its runtime probe, preserve deterministic worker-boundary regressions, build the pinned production model pack, execute a real non-deterministic Japanese translation through both OPUS stages, and produce the NSIS installer before `v0.1.0-alpha.1` is published.

Release notes live in [`docs/releases/v0.1.0-alpha.1.md`](./docs/releases/v0.1.0-alpha.1.md). Active milestone evidence remains in [`.agents/CURRENT_ITERATION.md`](./.agents/CURRENT_ITERATION.md) until the milestone gate is closed.

## Engineering guidance

Start with [`AGENTS.md`](./AGENTS.md). Canonical project knowledge lives in [`.agents/`](./.agents/), with one owner per concept for product, architecture, current iteration, code patterns, quality, decisions, and release rules.
