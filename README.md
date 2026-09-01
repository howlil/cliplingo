# ClipLingo

ClipLingo is a Windows-first, offline translation utility built around one interaction: select text in another application, press a global shortcut, and read the translation in a lightweight popup near the selection.

The product prioritizes reliable text capture, privacy, low perceived latency, near-zero idle CPU, and strong CJK/English-to-Indonesian translation quality before broad language coverage.

## Current architecture

- **Presentation:** Tauri 2 + Svelte 5 + TypeScript + Vite.
- **Application core:** Rust owns workflow, popup state, request identity, routing, and worker lifecycle.
- **Windows integration:** `windows-rs`, Win32/COM/UI Automation, clipboard fallback, global shortcut, positioning, and Named Pipe transport.
- **Inference:** isolated `cliplingo-worker.exe` C++ process behind worker protocol v1.
- **Models:** versioned local language packs; the first committed alpha route is Japanese -> English -> Indonesian using CTranslate2 CPU INT8 + SentencePiece assets.
- **Normal translation path:** local/offline after the required model pack is installed.

The Windows selection -> Rust coordinator -> worker -> popup boundary is integrated. Real offline model inference is the current milestone; see [`.agents/CURRENT_ITERATION.md`](./.agents/CURRENT_ITERATION.md) for the active slice and evidence.

## Engineering guidance

Start with [`AGENTS.md`](./AGENTS.md). Canonical project knowledge lives in [`.agents/`](./.agents/), with one owner per concept for product, architecture, current iteration, code patterns, quality, decisions, and release rules.
