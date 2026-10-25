# ADR-0003 — Latency-first tooltip and explicit dual-mode translation

Status: **Accepted direction; experimental F0 Tauri/native GDI preview code exists, but no translation implementation or measured performance evidence**
Date: 2026-10-26
Refines: [ADR-0002](0002-tauri-rust-windows-hybrid.md); retains [ADR-0001](0001-rust-first-native-windows.md) Rust-first language choice.

## Context

ClipLingo targets a low-resource, low-latency Windows translation experience. Mouse Tooltip Translator (browser extension) uses a tooltip UI and cloud translation services; Firefox Translations demonstrates local downloaded models. The user explicitly wants ClipLingo to support both offline and online translation while remaining light and fast. Tauri v2 WebView2 is useful for Settings but introduces browser/renderer processes whose resource impact is relevant to a tiny persistent tooltip.

## Decision

1. **Render the result through a reusable Rust-owned native Win32/DirectWrite text-only HWND by default.** Keep Tauri v2 for tray, lifecycle, React/TS Settings and signed updater. A Tauri Webview popup is now a benchmark comparison, not the preferred first implementation. Real Windows focus, DPI, accessibility and latency proof remains mandatory.
2. **Route every authorized capture through a single Rust TranslationRouter**. Modes are `Offline only` (default, no translation-network), `Online only` (explicit provider upload consent), and optional `Smart` (offline-preferred, **explicitly opted-in per-provider automatic network fallback**). Network being available is never consent.
3. **Offline candidate:** Helsinki-NLP OPUS-MT EN→ID with compatible SentencePiece preprocessing. Test CPU native runtime + CTranslate2 narrow ABI vs a genuinely compatible Rust-loadable runtime; license, quantization, accuracy and memory must be proven. Never download/load model at tray startup by default. Install a separately verifiable model pack on user action, with offline pack import as an option.
4. **Online candidate:** one documented official provider (Azure Translator or Google Cloud Translation) with user-chosen supported auth; no scraped consumer endpoints or embedded app-wide API keys. BYOK secrets belong in OS-protected credentials; a developer-financed universal endpoint would require a separate backend/abuse/cost contract.
5. **Fast path:** selection/cursor snapshot + fail-closed policy -> bounded exact RAM-only cache -> authorized offline model or online request -> newest-generation gate -> native popup. No synchronous COM on shell event loop, no WebView startup/inference-model installation/new process on a hotkey.
6. **Privacy:** never upload raw text without consent, even if local model absent or corrupt. Excluded/unknown/protected app content fails closed in both modes; Settings indicates provider and costs. No capture-history or disk translation cache by default.
7. **Performance:** prefer correct translation, privacy and no-focus-steal before speed; measure end-to-end hotkey-to-visible p50/p95/p99, complete process-tree idle RAM, cold/warm model, CPU and long-run reliability. No fabricated numerical guarantees.

## Tradeoffs and risks

- Native popup means two UI technologies, but keeps WebView2 out of the frequent hotkey-to-tooltip path and avoids a second persistent renderer. A native HWND still needs accessibility, Unicode, DPI and focus work.
- A loaded CPU model increases RAM; evicting saves RAM but adds cold latency. Model residency is a measured policy, not a universal best value.
- Online may produce different translations, incurs request/network latency, transmits selected text and possibly provider charges. Defaults/consent must make this explicit.
- Tauri remains a real WebView2 dependency for Settings; lazy creation doesn't imply zero child-process memory.
- Offline weights, converted artifacts and inference runtime can have different licenses; model family and weights must be vetted before distribution.

## Rejected defaults

- Always-online/offline-to-cloud silent fallback; no consent.
- Always-resident large multilingual LLM/NLLB model; not justified by EN→ID low-memory baseline and license concerns.
- WebView popup as first shipping renderer for a native cursor tooltip; compare in a measured spike instead.
- Hidden clipboard copy or background OCR prefetch; violates capture/privacy semantics.
- Browser-extension unofficial Google/Bing endpoint scraping; no stable supported API contract.

## Acceptance

- Running in Offline-only with network disconnected after model install translates a selected EN→ID fixture, with no translation traffic and no stolen focus.
- Online-only can operate without local model only after selecting provider and authorizing upload; no automatic traffic before consent.
- Smart mode falls back only after per-provider consent and documented allowed errors; never after app-exclusion/permission denial.
- Cache hit still validates privacy and latest-generation; cache is in-memory, bounded, cleared as documented.
- Popup remains target-text-only and supports word/paragraph, mixed-DPI and accessible text.
- Compare full Windows process-tree memory and hotkey-to-actually-visible result with WebView popup prototype on representative machines.
- No model or native executable exists until created and tested; this ADR is a design choice, not an implementation claim.

Full runtime flow, provider constraints, model source/licensing, native proof matrix and milestones live in [engineering-design.md](../engineering-design.md).
