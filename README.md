# ClipLingo — Native Windows Selection, Hover & Explicit Clipboard Translation

Milestones: M1/M2 merged to `master`; M3 OCR and M4 Privacy remain separate Draft PRs; M5 clipboard on `feat/m5-explicit-clipboard-translation`. C++20 · Win32 · DirectWrite/Direct2D · UI Automation · CTranslate2 · SentencePiece.

## What M1 actually implements

- Windows tray icon (including Explorer taskbar recreation), Settings, Pause/Resume, Quit and a single-instance mutex.
- `Ctrl+Alt+T` request: freeze foreground window/cursor, capture selected text via a *short-lived isolated helper process*, validate selection, run offline EN→ID, update only the latest request's popup.
- The capture helper shares **one bounded memory mapping handle** with the host, runs UIA on COM MTA, exits after capture, and is forcibly terminated after 1.8 s when blocked. This adds a per-request process-startup cost; profile before deciding whether to add a persistent child process.
- Native content-fit popup (DirectWrite/Direct2D), 420×300-DIP maximum by default, placed above cursor without focus. Translation text **only**: no popup controls or header.
- Long results are readable by mouse wheel or `Ctrl+Alt+Up/Down` without activating the popup. Exposes full target text via a native UI Automation live-region *Name* property; actual NVDA/Narrator behavior must be checked on Windows.
- Native six-section Settings shell (General, Text Capture, Translation Popup, Appearance, Language Models, Privacy). Functional: Pause/Resume, launch at login, verified model pack import, max popup bounds, light/dark, solid/tinted, and interactive popup preview. OCR/Glass remain disabled on this branch; hover and clipboard opt-in are available in Text Capture.
- EN→ID model is prepared by a separate developer-side conversion tool and imported into `%LOCALAPPDATA%\ClipLingo\models`. SHA-256 file manifest validates source and staged content. Versioned rename activates pack atomically. Model stays warm between uses and can be unloaded after about two minutes idle.
- No browser processes, network on translation path, accounts, continuous OCR or clipboard hooks. M2 adds shortcut-only hover fallback; M5 adds opt-in tray clipboard reading.

## M2 — Hover fallback (merged)

When Ctrl+Alt+T is pressed, ClipLingo still translates a valid selection **exactly**.
If the UI Automation selection is confirmed empty and hover fallback is enabled,
it reads the word directly under the cursor; Settings → Text Capture allows
Word, Sentence or Paragraph scope. Hover is **on-demand**, not continuous tracking.
The UIA helper uses the existing 1.8-second isolation watchdog and IPC mapping
with CLU2 versioned message fields (frozen cursor + scope). A nearest-word range
without a visible rectangle containing the cursor is rejected. A parent
TextPattern may be used when the element under the cursor is a leaf.
Sentence is a conservative English punctuation/abbreviation segmentation
of a bounded paragraph; it is not a semantic language model.

Default: hover enabled, Word scope; change or disable it in Text Capture.
No clipboard fallback or OCR. Capture limits (8192 UTF-16 units), source-app
foreground checks, latest-request wins, offline inference and popup visual
baseline remain unchanged.

Run hover tests using the existing smoke build and `ctest`; a **real Windows
desktop** compatibility matrix remains mandatory before release, especially
for browser/PDF providers whose UIA text patterns are incomplete.

## M5 — Explicit clipboard translation (opt-in)

This is an **intentional manual action**, not automatic clipboard capture and
not a simulated Ctrl+C. Enable **Settings → Text Capture → Allow explicit
clipboard translation from tray** (disabled by default), then choose
**ClipLingo tray → Translate clipboard (explicit)**. Only the current
`CF_UNICODETEXT` format is read, at most 8192 UTF-16 units, on a bounded
one-shot helper process. ClipLingo does not change clipboard ownership or
contents, create history, or poll the clipboard. A sequence-number check
rejects clipboard content that changed after the tray action.

When valid, the text uses the existing local EN→ID CTranslate2 path and the
same text-only native popup anchored at the trigger-time mouse position.
When the clipboard is busy, empty, over limit, unavailable or changed, no
result popup is shown; the error status can be read in Settings.

**Privacy limitation:** ClipLingo cannot establish which application created
historical clipboard content. Only enable the feature if you accept that the
explicit command may translate text copied from other apps, including text
you did not intend to process. This function is separate from the live
foreground-app exclusions being developed in PR #4 (and OCR PR #3).

Native CI exercises bounded Unicode parsing; an interactive Windows desktop
is still needed to test clipboard owner contention and delayed rendering.

## Prepare a trusted model pack (developer / release packager)

Conversion requires Python >=3.11, `ctranslate2`, `sentencepiece`, `huggingface_hub`, `transformers<5`, and PyTorch on a **builder machine**, never on the resident Windows app. Choose and pin an immutable commit from [Helsinki-NLP/opus-mt-en-id](https://huggingface.co/Helsinki-NLP/opus-mt-en-id/tree/main). Then:

```powershell
python -m pip install --index-url https://download.pytorch.org/whl/cpu "torch==2.6.0"
python -m pip install "ctranslate2==4.8.3" "sentencepiece==0.2.1" "transformers==4.57.3" "huggingface_hub>=0.25,<1"
python tools/build_en_id_pack.py .\pack\en-id --revision 052fd659487dd0d1a655969050e97950f83dc903
python tools/verify_en_id_pack.py .\pack\en-id
```

The tool converts to CPU INT8-FP32, copies source/target SentencePiece files, and creates `cliplingo-model.sha256` covering **every** file. Check the model license (Apache-2.0), tokenizer assets, third-party notices and model provenance before redistributing packs. Keep pack files together in one directory; no ZIP extraction is required in the app.

Run ClipLingo, open Windows tray → Settings → Language Models → **Install verified model pack…** → choose the folder. File hashing/copy is offloaded from the UI thread; only successfully validated/staged packs become the active model. A pack generated with a newer converter might not work with an older inference library; actual inference compatibility is a release acceptance gate.

## Build native application

Requirements: Windows 11 x64 (initial support target), Visual Studio 2022 C++ Desktop workload, CMake >= 3.25, Windows SDK, CTranslate2 **C++ installed package**, SentencePiece installed package. CTranslate2 is not an official vcpkg port in the baseline registry, so build/install its C++ library from an audited pinned tag and point CMake to the install prefix. The CMake config exported targets must include `CTranslate2::ctranslate2` and `sentencepiece::sentencepiece`. Native dependency DLL/CRT distribution must also be verified on a clean VM.

```powershell
cmake -S . -B build -A x64 -DCMAKE_PREFIX_PATH='C:\deps\installed'
cmake --build build --config Release
.\build\Release\ClipLingo.exe
```

With a validated imported pack, execute the production C++ inference path without a
desktop session (returns exit code 0 only if EN->ID output contains the expected
Indonesian word; nonzero otherwise):

```powershell
.\build\Release\ClipLingo.exe --verify-model "C:\path\to\en-id-pack"
if ($LASTEXITCODE -ne 0) { throw "Native C++ EN-ID model acceptance failed" }
```

This command loads and validates the pack using the actual SentencePiece/CTranslate2
C++ implementation and keeps all selected text offline. It is a targeted probe,
not a full screenshot/UIA/GUI acceptance test.

Real model integration is enabled by default. `CLIPLINGO_HOST_SMOKE=ON` **replaces inference with a test stub** to validate Win32 host/capture/popup/model package code on GitHub Actions without downloading heavy libraries; never distribute or benchmark this target as a translator.

## Tests

```powershell
cmake -S . -B build-smoke -A x64 -DCLIPLINGO_HOST_SMOKE=ON
cmake --build build-smoke --config Release
ctest --test-dir build-smoke -C Release --output-on-failure
```

Windows CI runs geometry test, model-package integrity regression test, and an MSVC host build. Production acceptance additionally requires a real CTranslate2 build, real offline EN→ID output, foreground-app UIA experiments, stress tests (rapid hotkeys, repeated model reload, explorer restart, multi-monitor/DPI), no focus stealing, NVDA/Narrator, and native power/idle profiling. **CI smoke success is not proof of production readiness.**

## Known constraints and deliberate M1 scope

- The per-request UIA helper introduces process launch/exit overhead; its benefit is bounded hang/crash containment. Measure hotkey-to-visible-result before optimization.
- Inference still runs inside the main process on a dedicated worker: an unrecoverable native CTranslate2 crash may bring down the app. A persistent *inference* helper requires a separate measured reliability decision.
- Settings have a native accent sidebar and some functional Appearance controls but **not yet complete pixel parity** with `DESIGN.md` (full glass material, responsive DPI/window layout and screen-reader acceptance require Windows validation).
- Importing a local pack is implemented; automatic network model downloading, installer/updater, and signature distribution are separate release concerns.
- Win32 window/UIA contracts differ by target app, elevated UI, OS version and DPI. M2 adds geometry-checked hover but does not claim universal capture.
- An accessible live-region name is provided, but a full `TextPattern` provider and assistive technology acceptance remain unverified.
- The offline pack generator and complete CTranslate2 link/run were not executed in this environment; model conversion may require adjustments for the selected library revision.

## Source ownership

| Source | Responsibility |
| --- | --- |
| `src/main.cpp` | event loop, tray, settings projection, serialized requests, lifecycle |
| `src/capture.cpp` | UI Automation selection and input validation |
| `src/clipboard.cpp` | isolated one-shot Windows clipboard data acquisition |
| `src/clipboard_text.h` | bounded Unicode clipboard parsing without Windows dependencies |
| `src/hover.cpp` | on-demand UIA hover hit-testing and Word/Sentence/Paragraph acquisition |
| `src/scope_text.h` | pure bounds validation and sentence segmentation |
| `src/capture_helper.cpp` | bounded helper process, IPC memory mapping, timeout and crash containment |
| `src/translation.cpp` | local model/tokenizer loading, inference and warm lifecycle |
| `src/model_package.cpp` | SHA-256 package validation, staging and atomic activation |
| `src/popup.cpp` | native DirectWrite text renderer, placement, accessible name |
| `src/core.h` | shared types/contracts, no speculative dependency hierarchy |

Other product/engineering decisions live in `DESIGN.md`, `.agents/product.md`, `.agents/engineering-design.md`, and `AGENTS.md`.