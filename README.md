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

ClipLingo `v0.1.0-alpha.1` is published as a Windows x64 NSIS installer on GitHub Releases:

- release: https://github.com/howlil/cliplingo/releases/tag/v0.1.0-alpha.1
- installer: `ClipLingo_0.1.0-alpha.1_x64-setup.exe`
- installer SHA256: `96a26f3985c95553177cb20f120b0bd1abb31040d8ded6af32132438c667e39b`
- model pack: `cliplingo-ja-id-opus-v1.zip`
- model-pack SHA256: `e3e6873d688d4ba3860ce20b6e2539481fc6dc1f8ae2396fc037e7329c754e30`

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

`v0.1.0-alpha.1` was release-gated and published only after the release workflow successfully:

- built and validated the pinned production OPUS model pack;
- built the Windows native worker/runtime probe;
- executed the native runtime probe;
- executed a real non-deterministic Japanese translation through both OPUS stages;
- produced the Windows x64 NSIS installer;
- emitted SHA256 checksums for the installer and model pack.

Release notes live in [`docs/releases/v0.1.0-alpha.1.md`](./docs/releases/v0.1.0-alpha.1.md). Final milestone evidence is recorded in [`.agents/CURRENT_ITERATION.md`](./.agents/CURRENT_ITERATION.md).

## WinGet

The WinGet `1.12.0` multi-file manifest set for `Howlil.ClipLingo` `0.1.0-alpha.1` is prepared in the `howlil/winget-pkgs` fork and references the exact GitHub Release installer and SHA256. Catalog availability is not claimed until the manifest PR is accepted and merged upstream by `microsoft/winget-pkgs`.

## Engineering guidance

Start with [`AGENTS.md`](./AGENTS.md). Canonical project knowledge lives in [`.agents/`](./.agents/), with one owner per concept for product, architecture, current iteration, code patterns, quality, decisions, and release rules.
