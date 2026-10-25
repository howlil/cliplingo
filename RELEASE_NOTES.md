# ClipLingo v0.1.0 Beta 1

**Windows 11 x64 · Portable early-access pre-release · Offline EN → ID**

This preview packages a native Windows system-tray translation utility. It is **not a signed or production-validated release**. Download the Windows ZIP *and* the offline EN–ID model ZIP; translations do not work until you install the model in Settings.

## Included
- Global **Ctrl + Alt + T**: selected-text translation; optional on-demand word/sentence/paragraph hover when selection is empty and UI Automation supports it.
- Compact cursor-anchored, text-only native translation popup; light/dark and solid/tinted options.
- Manual drag-to-select screen region OCR using **local Tesseract English** (no screen polling).
- Privacy: executable-name exclusion list, applied before UIA capture. For safety, arbitrary screen OCR is disabled whenever exclusions are configured.
- Explicit opt-in **Translate clipboard** tray action; never monitors, overwrites or automatically copies clipboard contents.
- Offline CTranslate2 + SentencePiece EN–ID model inference, model integrity verification/import, single-instance tray host and native performance diagnostic.

## Installation
1. Download **ClipLingo-v0.1.0-beta.1-win-x64.zip** and **ClipLingo-v0.1.0-beta.1-en-id-model.zip**.
2. Extract the Windows ZIP to a normal folder you own. **Keep ClipLingo.exe and its DLLs together**, including the `tessdata` directory. No installer or auto-update is supplied in this version.
3. Run `ClipLingo.exe`. In tray → **Settings → Language Models → Install verified model pack**, select the extracted **en-id** folder from the model ZIP.
4. Select English text in a supported application and press **Ctrl + Alt + T**. For images, use tray → **Capture screen text** and drag a rectangle.
5. To enable manual clipboard translation, opt in under Settings → Text Capture and then use the tray command. Clipboard origin cannot be verified.

Offline means content is translated locally after the model is installed. The **builder** downloads pinned open-source model files when preparing the release; the resident application does not need cloud inference.

## Security, compatibility, and limitations
- **Unsigned preview binary:** Windows SmartScreen may warn. Do not bypass security policy on managed devices; review the source and SHA-256 checksums.
- Windows **11 x64** is the initial target; no claim of ARM64 or Windows 10 support.
- Clipboard may contain material copied from any application; excluded-app policy does not protect historical clipboard contents.
- UI Automation selection and hover are provider-dependent; OCR accuracy varies by content. Secure/protected windows are not bypassed.
- A model pack must pass local integrity checks. SHA-256 manifests are **not publisher signatures**.
- The Win32 portable ZIP includes runtime DLLs and English OCR traineddata; bundled third-party licensing notices are in `THIRD_PARTY_NOTICES`.
- Native CI and offline model inference are verified in the release workflow, but keyboard-only region selection, Narrator, DPI matrices, 8-hour soak, RAM/latency on end-user hardware, signed installer, and automatic updater **have not been accepted**.

**Checksums:** see attached `SHA256SUMS.txt`. The model is sourced from `Helsinki-NLP/opus-mt-en-id` at immutable commit `052fd659487dd0d1a655969050e97950f83dc903` (model license: Apache-2.0). English OCR traineddata is from the official `tesseract-ocr/tessdata_fast` `4.1.0` release.

**Reporting bugs:** Include Windows build/version, affected application, and steps to reproduce. Avoid sharing sensitive captured text or screenshots in public issues.
