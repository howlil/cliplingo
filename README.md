# ClipLingo (F2 — experimental offline CTranslate2 EN→ID)

**Experimental offline translator path, not a public beta.** Read [product contract](.agents/product.md), [engineering contract](.agents/engineering-design.md) and [DESIGN.md](DESIGN.md) before adding translation.

## What is implemented

- Tauri v2 Rust tray application and single-instance handler.
- Global hotkey `Ctrl+Alt+T` captures a bounded real UIA selection using a separate COM MTA thread after foreground process authorization. When built with the optional `offline-ct2` feature and a verified local model pack, it translates locally and sends **only Indonesian target text** to the native popup; no source text is logged/uploaded.
- Lazily created, six-section read-only Settings UI; it is intentionally static HTML/CSS/JS for F0 to avoid bringing in a frontend build tool before the native feasibility gate.
- Explicit tray **Preview native tooltip (F0 only)** and debug-only `Ctrl+Alt+P`. These display fixed **preview text, never a purported translation**, through one Rust-owned Win32 HWND using `WS_EX_NOACTIVATE` and `SWP_NOACTIVATE`.
- Current F0 text renderer is Win32 GDI with measured wrap. **DirectWrite, accessible overflow, mixed-DPI polish, real focus and E2E acceptance remain unverified.**
- Unit tests for native placement on normal/negative-coordinate monitors.
- Windows GitHub CI compile/check and an on-demand PowerShell whole-process-tree memory diagnostic.

## Build

Requirements: Windows 11 x64, Rust stable (MSVC toolchain), Visual Studio C++ build tools and WebView2 runtime.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo run --manifest-path src-tauri/Cargo.toml
```

Cargo uses the Tauri generated context; there is no frontend bundler or cloud API in F0. `src-tauri/tauri.conf.json` sets `bundle.active=false` because no installer or release has been verified. Future Tauri CLI builds/NSIS packaging require their own release gate.

Open Settings from the tray. For a native tooltip preview, choose the explicit tray command. On debug builds, hover Chrome/Notepad and press **Ctrl+Alt+P** to see the sample preview without opening the tray. **Ctrl+Alt+T uses F1 capture and, when explicitly built with offline-ct2 and a valid installed model, runs local EN→ID inference before showing target text. Without the feature/model, there is no fabricated output.**

## Measure

```powershell
# After launching, with Settings closed:
.\scripts\measure-f0.ps1 -Samples 30 -IntervalMs 1000 -Output baseline-idle.json

# Repeat with Settings open, and after pressing Ctrl+Alt+P.
```

This measures ClipLingo + descendant processes' working set and private bytes. The preview's `last_paint_us` value is **dispatch-to-WM_PAINT completion, not DWM visibly presented**. Never label it as full hotkey-to-visible-translation or an inference benchmark. Record the machine, Windows build, DPI and WebView2 version when comparing samples.

## F0 acceptance still pending

- A Windows desktop host test must assert `GetForegroundWindow` and keyboard focus remain in the source app after `Ctrl+Alt+P`, and verify placement at mixed monitor DPIs and negative virtual-screen coordinates.
- Windows UIA selection and default deny policy are implemented. Foreground/focused-control attribution, multi-process browser compatibility, genuine interactive Windows E2E, provider hang containment, configurable exclusions, clipboard, OCR, Windows live model quality/latency acceptance, online provider consent, cache, Settings persistence and signed updater remain **unverified or not implemented**.
- DirectWrite/shaping, screen reader readable overflow, high-contrast adaptation, power draw, screenshot-based geometry and native handle leak soak are later work.
- CI passing only establishes a compilable Rust shell and unit test behavior; it does not substitute for interactive Windows/E2E verification.


## F2: prepare a verified EN→ID model pack

Default `cargo run` remains a lightweight shell build **without CTranslate2**, and cannot translate. Building the optional offline feature requires MSVC, CMake and time to compile native dependencies. No Python runtime, network service, model download or cloud credential is required by the running app.

Developer machine (one-time model preparation only):

```powershell
python -m pip install ctranslate2==4.8.3 "transformers>=4.49,<5" "huggingface_hub>=0.30,<1" "hf_xet>=1,<2" sentencepiece torch
python scripts/prepare-en-id-pack.py --output "$env:LOCALAPPDATA\ClipLingo\models\en-id"
```

The script converts the Apache-2.0 [Helsinki-NLP OPUS-MT EN→ID model](https://huggingface.co/Helsinki-NLP/opus-mt-en-id) to CTranslate2 INT8 and writes a SHA-256 manifest for the weights and SentencePiece files. It refuses to overwrite an existing pack. The Rust runtime checks the manifest before first load.

Build with real CPU backend:

```powershell
$env:RUSTFLAGS = "-C target-feature=+crt-static"
cargo run --manifest-path src-tauri/Cargo.toml --features offline-ct2
```

Then select English text in a supported Windows editor and press `Ctrl+Alt+T`. The native popup is shown only if foreground policy, model validation, inference and latest-generation checks succeed. No text is sent online. The CPU model is lazy-loaded on first use, uses at most two engine threads per replica and is evicted after roughly 90 seconds idle. Note: model generation/performance on a real Windows desktop and screen-reader/long-text popup behavior still require acceptance testing; Windows Cargo checks cannot prove those behaviors.


### Native F2 verification

A [Windows CI run](https://github.com/howlil/cliplingo/actions/runs/40243341975) successfully built CTranslate2, converted a pinned OPUS-MT EN→ID model, linked the native executable, loaded the model, and produced a nonempty translated output from a fixed test sentence. [Default Rust tests](https://github.com/howlil/cliplingo/actions/runs/40243341796) also passed. `RUSTFLAGS=-C target-feature=+crt-static` is **required for the Windows native CT2/CXX build**, since upstream CTranslate2 uses the static MSVC C runtime. This is headless inference proof; desktop focus, UIA provider compatibility, model quality, latency/RAM and release packaging remain open acceptance gates. See [.agents/f2-acceptance.md](.agents/f2-acceptance.md).


## F2.5: bounded latest-request scheduler and measurement

When many shortcuts arrive while a model inference is running, only the **newest pending** selection is retained; obsolete pending sensitive text is dropped. The already-running native inference is not forcibly interrupted. Source generation and original foreground-window checks remain required before native presentation.

The native Windows model workflow runs one **cold** load/inference and twelve **warm** public English fixtures to report p50/p95/max **inference-only** timing. The result is not hotkey-to-popup latency or an application SLO. On a real Windows session, collect separate process-tree private/working memory samples with `scripts/measure-f25.ps1` and follow [.agents/f25-acceptance.md](.agents/f25-acceptance.md) for Notepad/Chrome/PDF focus/DPI testing. Interactive desktop acceptance has **not** passed merely because GitHub CI is green.


## F2.7 measured model quality and resource footprint

The [F2.7 Windows run](https://github.com/howlil/cliplingo/actions/runs/40251146437) compared the real EN→ID CTranslate2 model on 28 public test cases and measured sampled CPU/working/private memory. Current default `beam=2, threads=2` scored 71.96 chrF vs 69.00 for greedy `beam=1, threads=2`; greedy took 37.891s vs 40.656s for the complete headless test. These are exploratory figures, **not** actual hotkey-to-popup latency, battery impact or a human quality rating.

**Known beta blocker:** both profiles can omit meaningful information when translating multiple sentences; the greedy output was particularly incomplete on one paragraph. See [Issue #28](https://github.com/howlil/cliplingo/issues/28) and [F2.7 evidence](.agents/f27-acceptance.md). No production default change, no online text upload, and no public installer/release should be inferred from these tests.
