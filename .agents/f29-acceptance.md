# F2.9 — Native overflow viewport and interactive Windows acceptance

**Scope:** Keep native text-only translation popup, 420×300 DIP design cap, non-activating Windows foreground behavior and offline-only content privacy. No new WebView, buttons, badges, title bar or source text.

## Change and ownership

- `native_popup.rs` owns one HWND and the translated target only. Text is measured with GDI using the **trigger cursor's monitor DPI** (also reused at paint time) and monitor work area, with bounded popup geometry and separately maintained **complete translated output** (up to existing 4,096 UTF-16 units). Full translated text is never shortened to fit the viewport.
- If the content is taller than the 300-DIP popup, the window shows a viewport of the original text and handles `WM_MOUSEWHEEL` using a bounded scroll position. Each mouse-wheel notch advances three native text lines. Fine-grained wheel deltas accumulate. A newly arrived result resets offset to 0; no extra model inference or threads are started.
- For long targets, the automatic dismissal interval is 20 seconds, refreshed when the user scrolls. Short translations preserve the previous timeout. The popup stays `WS_EX_NOACTIVATE` and uses `SWP_NOACTIVATE`; `WM_MOUSEACTIVATE` explicitly returns `MA_NOACTIVATE`.
- On timeout, explicit hide, HWND destruction or failed replacement, the native thread-local displayed translation text and scroll state are cleared, avoiding keeping a translation history. A failed wheel timer rearm hides the popup rather than leaving a stale result indefinitely.
- The popup is **translated target only**. No hidden web app, clipboard writes, cloud request, active content selection or new Windows model behavior.

## Automated evidence (CI)

| Property | Verification |
| --- | --- |
| Geometry stays inside available work area, including negative monitor coordinates | `native_popup.rs` existing tests; target-monitor DPI correction requires real mixed-scale monitor verification |
| Large target has finite scroll range, reaches final line, clamps at both ends | New deterministic `ScrollState` tests |
| High-resolution wheel partial notches are accumulated | New deterministic `ScrollState` test |
| Every new translation starts at the first line | New `ScrollState` reset test |
| Existing UIA, model, latest request, F2.8 sentence correctness remain compatible | Windows Rust compile/unit tests + production code check |
| Real CTranslate2 model regression | No translation-engine source touched; existing F2.8 evidence. Rerun if backend changes. |

## Still required on *real interactive Windows desktop*

1. Build ClipLingo with `--features offline-ct2`, verified EN→ID pack, MSVC static CRT. Focus Notepad, select a short English phrase and press `Ctrl+Alt+T`; observe Indonesian target above frozen cursor with no focus loss.
2. Repeat with long paragraph, Chrome and a PDF viewer. Hover **the native popup** and scroll to the final line and back. Test with Settings → Bluetooth & devices → Mouse → **Scroll inactive windows when hovering over them** both enabled and disabled. Windows may route mouse-wheel messages to the source application instead of an unfocused popup; in that case **F2.9 is not accepted** for that configuration, and a different explicit nonfocus scrolling contract is required.
3. Record live foreground and focused HWND transitions with `scripts/probe-f29-focus.ps1`, and verify the original editor caret remains. This is sampling-based; it can miss short focus theft, so add a real native GUI test as a future step.
4. Test mixed-DPI 100%/125%/150%/200%, monitors with negative coordinates and tiny monitor work areas. Explicitly inspect that no line is cut horizontally and last line can be read. Long unbroken tokens and per-monitor DPI placement need human examination.
5. Use Narrator/NVDA and Windows high-contrast/text scaling. **GDI text drawn without UIA TextPattern does not constitute screen-reader acceptance.** This implementation is a *mouse-wheel readable viewport*, not an accessible final renderer. A real native UIA/MSAA provider or focus-neutral alternative must be designed/tested before beta.
6. Measure cold/warm hotkey-to-visible, idle whole-process-tree RSS/private memory and long-paragraph CPU using the existing F2.5 and F2.7 scripts. No numerical production SLO is claimed from CI.
7. Rapid requests A→B when the first result is scrolled: popup must reset to the start of B; no stale partial result should remain visible.

**Decision:** This increment fixes the prior all-or-nothing overflow refusal for ordinary long paragraphs when the OS routes scrolling. It **does not close** the product accessibility, screen-reader, mouse-wheel routing, GUI focus, or full Windows E2E gates. Do not publish beta or close Issue #28 based solely on CI.

Related: [F2.8 regression](f28-acceptance.md), [F2.5 GUI acceptance](f25-acceptance.md), [Product contract](product.md).
