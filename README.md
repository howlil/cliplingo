# ClipLingo M1 — Windows-native reading loop

**Branch:** `feat/m1-native-translation`. C++20 / Win32 / DirectWrite / Direct2D / UI Automation / CTranslate2 / SentencePiece. No Tauri, WebView2, external server, account or internet call during inference.

## Implemented in M1

1. Windows notification-area icon/menu (Pause, Settings, Quit); hotkey `Ctrl+Alt+T` with registration-conflict status.
2. UI Automation selection capture on worker COM MTA thread. No selected text -> no popup or inference.
3. Select an existing **converted EN→ID CTranslate2 model folder** from Settings → Language Models. Model must contain `model.bin`, `source.spm`, `target.spm` (SentencePiece files manually added to converted folder). M1 does **not** download models.
4. On-demand local inference via CTranslate2 C++ API and SentencePiece; result is returned to the UI message pump with generation ordering.
5. Native no-activation popup: DirectWrite measurement + Direct2D rendering, content-fit until configurable width/height, anchor captured at hotkey, flip/clamp to monitor work area.
6. Native six-section Settings shell. Currently functional: pause/resume, model directory selection, maximum popup dimensions. Other rows are labeled as M1-limited instead of pretending unavailable features work.

## Build (Windows 11 x64, Visual Studio 2022, CMake)

**Requirements:** Visual Studio 2022 with C++ Desktop Development and Windows SDK; x64 binaries/headers for CTranslate2 and SentencePiece discoverable via `CMAKE_PREFIX_PATH` or a package manager exporting `ctranslate2` and `sentencepiece` CMake config targets. The package manager and exact binary distribution are not yet pinned.

```powershell
cmake -S . -B build -A x64 -DCMAKE_PREFIX_PATH='C:\deps\installed'
cmake --build build --config Release
.\build\Release\ClipLingo.exe
```

For a suitable model, convert an EN→ID Marian/OPUS model with CTranslate2's transformer conversion utility and stage **matching** SentencePiece source/target tokenizers in that folder. Do not assume a download/conversion script has been validated by this M1. Check model licensing and supported tokenizer format before distribution.

On launch, look at the tray icon (or the hidden `^` menu) → Settings → Language Models → choose folder. In any application exposing an accessible text-selection pattern, highlight English text and press `Ctrl+Alt+T`. Popup contains only Indonesian translation. Closing Settings does not quit the tray app.

## Native Windows host compile smoke

The CI workflow builds **all Win32 host/UI/capture sources** with a deliberately nonfunctional inference stub using `-DCLIPLINGO_HOST_SMOKE=ON`. This checks Windows SDK/MSVC symbols and linking without bundling model/toolkit dependencies. The normal `ClipLingo` build uses real CTranslate2. **A green smoke build does not prove offline translation or runtime compatibility.**

## M1 limitations, not silent promises

- Native build/runtime **not verified yet on Windows in this session**. UI Automation compatibility needs real-application tests.
- The single worker keeps one model warm between requests and unloads it after approximately 2 minutes idle. The memory/latency trade-off still needs real benchmarking.
- UIA calls can still hang; host/UI thread stays responsive, but worker shutdown can block until provider returns. A separate capture helper process is an open production decision.
- Standard Settings controls are functionally native but are not yet visually parity-tested against the approved `DESIGN.md` reference. App startup, configurable hotkey, installer, excluded apps, ID→EN and polished model status are not implemented.
- Only manual text selection; hover, OCR and clipboard fallback are deferred; some rich text controls do not expose UIA selection.
- Solid light UI only. Native accessibility provider/keyboard reading for custom popup and richer scrollbar behavior require real Windows acceptance. Wheel scroll is supported when popup text exceeds configured height.
- Incomplete runtime lifecycle handling (Explorer restart, single-instance, power/session notifications) is a release gate, not evidence M1 is production-ready.

## Source boundaries

- `src/main.cpp`: Win32 host, tray, lifecycle, native Settings, request generation and single MTA worker.
- `src/capture.cpp`: UIA selection capture only.
- `src/translation.cpp`: CTranslate2 + SentencePiece integration only.
- `src/popup.cpp`: native popup text layout, rendering and placement only.
- `src/core.h`: smallest contracts crossing boundaries.

Do not introduce generic service/interface trees, a browser shell, multi-process supervision or any plugin framework without demonstrated need. See `AGENTS.md`, `DESIGN.md` and `.agents/{product,engineering-design}.md`.