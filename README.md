# ClipLingo

ClipLingo is a Windows-first, offline-first translation utility built around one interaction: select text in another application, press a global shortcut, and read the translation in a lightweight popup near the selection.

The product is intentionally optimized for reliability, low perceived latency, low idle resource use, privacy, and high-quality Chinese/Japanese/Korean translation before broad language-count marketing.

## Planned architecture

- **UI:** Tauri 2 + Svelte 5 + TypeScript + Vite
- **Native application core:** Rust
- **Windows integration:** `windows-rs`, Win32, COM, UI Automation, clipboard fallback
- **Inference worker:** isolated C++ process using CTranslate2 + SentencePiece
- **IPC:** Windows Named Pipe
- **Models:** downloadable, versioned, verified INT8 language packs
- **Normal translation path:** fully local; no translation server required

The repository is currently in engineering-foundation stage. Implementation starts with the Windows interaction spike before integrating real ML inference.

## Engineering guidance

Start with [`AGENTS.md`](./AGENTS.md). Detailed project constraints, design, SDLC, release rules, iteration state, and role-specific skills live under [`.agents/`](./.agents/README.md).
