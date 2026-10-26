# F2.8 — Preserve complete EN→ID sentence coverage

Scope: [Issue #28](https://github.com/howlil/cliplingo/issues/28). The existing Marian/OPUS-MT translator can omit a whole sentence in one-shot paragraph inference. **This milestone segments sentence boundaries and translates each nonempty segment independently; it does not claim semantic infallibility.** The production beam=2/2 CPU threads default is unchanged.

## Changed execution contract

1. Windows UIA capture remains privacy gated, exactly one latest pending selection.
2. Native translator uses Unicode Annex #29 `split_sentence_bounds` plus limited abbreviation merging (honorifics). Unlike `unicode_sentences`, the source segments reassemble to **exactly the original bytes**, including punctuation-only ranges and whitespace. English-oriented abbreviation heuristics remain a known edge case.
3. Reject more than **8 source sentences** before inference. Independently validate each sentence against the Marian 480-token budget. No silent CTranslate2 truncation or unbounded concurrent sentence workers.
4. Translate each segment serially using the **same lazy-loaded CTranslate2 model worker**, in original order. Build the target locally; if any segment fails/returns empty, exceeds 4,096 UTF-16 units combined, or a newer queued request replaces it, **no partial translation may reach the popup callback**.
5. Native GDI popup checks actual measured text dimensions and monitor work-area **before** displaying target text. If height exceeds 300 DIP, width 420 DIP, or monitor bounds, fail closed instead of clamping and hiding translated lines. A proper accessible scrolling/expansion surface is separately required before beta.
6. Tauri Settings still sees status/counters, not source content. No cloud, clipboard or OCR fallback. Existing generation/source-window guards remain in place.

## Required executable evidence

| Property | Faithful boundary | State |
| --- | --- | --- |
| Lossless source segmentation; decimals, honorifics, quotation; 8-sentence limit | Rust sentence_pipeline unit tests | Await Windows CI |
| Error in sentence N never yields partial result from sentence 1…N−1 | Native pure orchestration test with controlled failure | Await Windows CI |
| New hotkey between sentences suppresses stale translation | Native orchestration test and latest-slot unit tests | Await Windows CI |
| Complete source paragraphs translated with same real OPUS-MT model | Windows CTranslate2/CPU feature-run, 28 public fixtures | Await Windows CI |
| Original Issue #28 paragraph retains report + settings + response time + results | `scripts/check-f28-coverage.py` run against baseline model output, **not mocks** | Await Windows CI |
| Existing short text output regression and latency/RAM comparison | Headless benchmark and F2.7 artifact; before/after comparison | Await Windows CI |
| Correct native focus, mixed DPI, long target presentation and actual DWM-visible time | **Interactive Windows desktop** (Notepad/Chrome/PDF) | Not run |

## Important limitations and next work

- Segment-level translation improves the odds of retaining every sentence but does **not** formally prove all clauses/meaning survive inside a sentence. The regression guard checks a narrow fixed public example, not broad translation quality.
- The additional sentence inference calls can increase paragraph latency and keep CPU busy. Obsolete requests can be cancelled **between** model calls, not during an active synchronous CTranslate2 invocation.
- A full result larger than the native text-only popup is **rejected**, not scrolled. Without UI error presentation, users may see no popup. A subsequent milestone must provide a non-focus-stealing, accessible full-text presentation rather than relaxing the 300-DIP cap or silently clipping.
- Native UI Automation ancestry/foreground correctness, protected apps, focus-preservation, accessibility and mixed DPI remain unverified on a real interactive desktop.
- Do **not** close Issue #28 or publish a beta until broader human translation coverage and interactive GUI acceptance have been demonstrated. A CI success proves a bounded regression, not general correctness.

Background: [UAX#29 sentence boundaries](https://docs.rs/unicode-segmentation/latest/unicode_segmentation/trait.UnicodeSegmentation.html), [F2.7 measured regression](f27-acceptance.md), [F2.5 GUI acceptance](f25-acceptance.md).
