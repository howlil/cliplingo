# ClipLingo (F1 — experimental Windows UIA selection capture)

**Experimental capture scaffold, not a working translator or public beta.** Read [product contract](.agents/product.md), [engineering contract](.agents/engineering-design.md) and [DESIGN.md](DESIGN.md) before adding translation.

## What is implemented

- Tauri v2 Rust tray application and single-instance handler.
- Global hotkey `Ctrl+Alt+T` captures a bounded real UIA selection using a separate COM MTA thread after foreground process authorization. It **does not translate, store, print or display captured text**.
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

Open Settings from the tray. For a native tooltip preview, choose the explicit tray command. On debug builds, hover Chrome/Notepad and press **Ctrl+Alt+P** to see the sample preview without opening the tray. **Ctrl+Alt+T runs F1 selection capture, but does not show a translated result until the local/online translation engine is implemented.**

## Measure

```powershell
# After launching, with Settings closed:
.\scripts\measure-f0.ps1 -Samples 30 -IntervalMs 1000 -Output baseline-idle.json

# Repeat with Settings open, and after pressing Ctrl+Alt+P.
```

This measures ClipLingo + descendant processes' working set and private bytes. The preview's `last_paint_us` value is **dispatch-to-WM_PAINT completion, not DWM visibly presented**. Never label it as full hotkey-to-visible-translation or an inference benchmark. Record the machine, Windows build, DPI and WebView2 version when comparing samples.

## F0 acceptance still pending

- A Windows desktop host test must assert `GetForegroundWindow` and keyboard focus remain in the source app after `Ctrl+Alt+P`, and verify placement at mixed monitor DPIs and negative virtual-screen coordinates.
- Windows UIA selection and default deny policy are implemented. Foreground/focused-control attribution, multi-process browser compatibility, genuine interactive Windows E2E, provider hang containment, configurable exclusions, clipboard, OCR, language models, offline inference, online provider consent, cache, Settings persistence and signed updater remain **unverified or not implemented**.
- DirectWrite/shaping, screen reader readable overflow, high-contrast adaptation, power draw, screenshot-based geometry and native handle leak soak are later work.
- CI passing only establishes a compilable Rust shell and unit test behavior; it does not substitute for interactive Windows/E2E verification.
