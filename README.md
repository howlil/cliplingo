# ClipLingo

ClipLingo is a Windows-first offline translation utility built around one interaction: select text in another application, invoke a global shortcut, and read the translation in a compact popup without leaving the current workflow.

## Install from PowerShell

From Windows PowerShell 5.1 or PowerShell 7, run:

```powershell
$p = Join-Path $env:TEMP 'cliplingo-install.ps1'; Invoke-WebRequest 'https://raw.githubusercontent.com/howlil/cliplingo/master/scripts/install.ps1' -UseBasicParsing -OutFile $p; & $p
```

The bootstrap resolves the newest **published, non-draft GitHub Release**, including alpha/beta prereleases, selects the Windows x64 NSIS installer, verifies its SHA256 from GitHub release metadata (with the release checksum sidecar as fallback), and only then runs the installer. The command is version-independent, so a future `alpha.2`, `alpha.3`, or stable release is picked automatically once it is published.

Optional silent install:

```powershell
& $p -Silent
```

This installs the newest **published release**, not unmerged development code. Until Alpha 2 is actually published, the bootstrap resolves the current Alpha 1 release.

## Current development experience — EN → ID

The active product path is direct English -> Indonesian translation through a local OPUS-MT model:

```text
ClipLingo in Windows system tray
        -> select English text
        -> Ctrl+Alt+T
        -> validate/capture selection
        -> local OPUS en -> id
        -> draggable Indonesian result popup
```

ClipLingo does **not** show a translation window just because the shortcut was pressed. The current selection must be valid and non-empty first. If there is no valid selection, the interaction stops silently and no translation request is started.

The system tray/notification area is ClipLingo's persistent application home. Left-click the tray icon opens Settings. The tray menu exposes Settings and Quit ClipLingo. Closing Settings leaves ClipLingo running in the tray.

## Settings

The Settings surface is intentionally small. It shows:

- primary route: English -> Indonesian;
- shortcut: `Ctrl+Alt+T`;
- offline model install/remove state;
- running status;
- explicit Quit ClipLingo action.

It is a Windows utility control surface, not a dashboard.

## Offline model

The active model pack is `en-id-opus-v1`, built from the pinned `Helsinki-NLP/opus-mt-en-id` model and converted for local CPU INT8 inference with CTranslate2 + SentencePiece.

A fresh release installation can install the model explicitly. ClipLingo downloads the release-owned pack, verifies its SHA256 before activation, and keeps normal translation on-device. Raw selected and translated text is not written to normal diagnostics.

## UI direction

ClipLingo uses restrained Windows utility styling: compact spacing, semantic typography, modest elevation, native light/dark adaptation, and a draggable `EN → ID` result surface. Decorative glass blur, gradients, glow, bento/card stacks, oversized radii, hero copy, and generic AI visual chrome are intentionally excluded. See [`DESIGN.md`](./DESIGN.md).

## Published Alpha 1

`v0.1.0-alpha.1` remains available as the first historical offline alpha. It used the earlier JA -> EN -> ID route and must not be confused with the current EN -> ID development milestone. Published Alpha 1 binaries and checksums remain immutable.

- release: https://github.com/howlil/cliplingo/releases/tag/v0.1.0-alpha.1
- installer: `ClipLingo_0.1.0-alpha.1_x64-setup.exe`
- installer SHA256: `96a26f3985c95553177cb20f120b0bd1abb31040d8ded6af32132438c667e39b`

The actual Alpha 1 shortcut is `Ctrl+Alt+T`; earlier documentation that said `Ctrl+Shift+T` was incorrect.

## Alpha 2 qualification

The next gated prerelease is `v0.1.0-alpha.2`. It is not considered published until the release workflow successfully:

- builds/validates the pinned `en-id-opus-v1` production pack;
- builds and executes the native runtime probe;
- performs a real non-deterministic English -> Indonesian translation through the isolated worker;
- builds the tray-enabled Windows x64 NSIS installer;
- emits installer/model SHA256 checksums;
- publishes the immutable prerelease only after all prior steps pass.

## Current architecture

- **Presentation:** Tauri 2 + Svelte 5 + TypeScript + Vite.
- **Application core:** Rust owns selection/translation workflow, request identity, popup state, tray lifecycle, model lifecycle, and worker lifecycle.
- **Windows integration:** `windows-rs`, Win32/COM/UI Automation, safe clipboard fallback, global shortcut, cursor/selection positioning, and Named Pipe transport.
- **Inference:** isolated `cliplingo-worker.exe` C++ process behind worker protocol v1.
- **Native runtime:** source-pinned CTranslate2 + SentencePiece with one coherent static MSVC CRT policy.
- **Models:** current active pack is direct OPUS-MT `en -> id`, CPU INT8.
- **Normal translation path:** local/offline after the required pack is installed.

## Current scope limits

The current milestone does not include OCR, cloud fallback, GPU inference, translation history, accounts/sync, configurable shortcuts, automatic language detection, Japanese/additional production routes, or a broad settings dashboard.

## Engineering guidance

Start with [`AGENTS.md`](./AGENTS.md). Canonical project knowledge lives in [`.agents/`](./.agents/), current work is in [`.agents/CURRENT_ITERATION.md`](./.agents/CURRENT_ITERATION.md), and product-specific UI rules are in [`DESIGN.md`](./DESIGN.md).
