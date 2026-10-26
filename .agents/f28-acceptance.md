# F2.8 — Preserve complete EN→ID sentence coverage

Scope: [Issue #28](https://github.com/howlil/cliplingo/issues/28). The existing Marian/OPUS-MT translator can omit a whole sentence in one-shot paragraph inference. **This milestone segments sentence boundaries and translates each nonempty segment independently; Windows native model regression PASSED, but semantic infallibility and desktop acceptance are not claimed.** The production beam=2/2 CPU threads default is unchanged.

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
| Lossless source segmentation; decimals, honorifics, quotation; 8-sentence limit | Rust sentence_pipeline unit tests | **PASS — Windows CI #40254834535 / #40254834448** |
| Error in sentence N never yields partial result from sentence 1…N−1 | Native pure orchestration test with controlled failure | **PASS — Windows CI #40254834535 / #40254834448** |
| New hotkey between sentences suppresses stale translation | Native orchestration test and latest-slot unit tests | **PASS — Windows CI #40254834535 / #40254834448** |
| Complete source paragraphs translated with same real OPUS-MT model | Windows CTranslate2/CPU feature-run, 28 public fixtures | **PASS — Windows CI #40254834535 / #40254834448** |
| Original Issue #28 paragraph retains report + settings + response time + results | `scripts/check-f28-coverage.py` run against baseline model output, **not mocks** | **PASS — Windows CI #40254834535 / #40254834448** |
| Existing short text output regression and latency/RAM comparison | Headless benchmark and F2.7 artifact; before/after comparison | **PASS — Windows CI #40254834535 / #40254834448** |
| Correct native focus, mixed DPI, long target presentation and actual DWM-visible time | **Interactive Windows desktop** (Notepad/Chrome/PDF) | Not run |

## Important limitations and next work

- Segment-level translation improves the odds of retaining every sentence but does **not** formally prove all clauses/meaning survive inside a sentence. The regression guard checks a narrow fixed public example, not broad translation quality.
- The additional sentence inference calls can increase paragraph latency and keep CPU busy. Obsolete requests can be cancelled **between** model calls, not during an active synchronous CTranslate2 invocation.
- A full result larger than the native text-only popup is **rejected**, not scrolled. Without UI error presentation, users may see no popup. A subsequent milestone must provide a non-focus-stealing, accessible full-text presentation rather than relaxing the 300-DIP cap or silently clipping.
- Native UI Automation ancestry/foreground correctness, protected apps, focus-preservation, accessibility and mixed DPI remain unverified on a real interactive desktop.
- Do **not** close Issue #28 or publish a beta until broader human translation coverage and interactive GUI acceptance have been demonstrated. A CI success proves a bounded regression, not general correctness.

Background: [UAX#29 sentence boundaries](https://docs.rs/unicode-segmentation/latest/unicode_segmentation/trait.UnicodeSegmentation.html), [F2.7 measured regression](f27-acceptance.md), [F2.5 GUI acceptance](f25-acceptance.md).

## Measured F2.8 Windows evidence — 2026-10-26

- [Windows Rust unit tests and release-check](https://github.com/howlil/cliplingo/actions/runs/40254834535): **PASS**, 21/21 tests, including two-sentence Issue #28 split/byte preservation, abbreviation and decimal boundaries, all-or-nothing on failure and latest-request cancellation.
- [Optional native CTranslate2 Windows run](https://github.com/howlil/cliplingo/actions/runs/40254834448): **PASS**. Verified pinned OPUS-MT INT8 pack, model smoke, four native CPU profiles, 28-case quality/resource evaluation, and `scripts/check-f28-coverage.py` on **real model** outputs.
- The Issue #28 regression now emits baseline: `Laporan itu meringkaskan eksperimen. Kami menguji beberapa pengaturan, mencatat waktu respon, dan membandingkan hasilnya.` This contains the formerly omitted report summary **and** the independently translated settings, response-time and comparison clauses. Greedy also preserved these four public-case anchors. There is **no user-selected content** in the fixture outputs.

| Headless metric | F2.7 before segmentation | F2.8 with bounded sentence segmentation |
| --- | ---: | ---: |
| Beam2/2 baseline 28-item single-reference chrF | 71.96 | **72.92** |
| Beam1/2 greedy chrF | 69.00 | **72.68** |
| Baseline suite wall time | 40.656 s | **42.328 s** |
| Greedy suite wall time | 37.891 s | **38.891 s** |
| Baseline sampled CPU time | 76.875 s | 80.000 s |
| Greedy sampled CPU time | 70.031 s | 72.719 s |
| Baseline sampled peak working set | 137.2 MiB | 136.9 MiB |
| Greedy sampled peak working set | 135.7 MiB | 135.9 MiB |

These are **single-run headless observations**, not statistically supported quality, battery, whole-app footprint, or actual perceived hotkey-to-popup p95. Multi-sentence model inference adds sequential work: the baseline full-suite wall time increased by about 4.1%, while the 12 short-sentence warm smoke profile was similar to the earlier run (beam2/2 p95 889ms). Do not extrapolate from the fixed public sample to production traffic.

### Acceptance decision and remaining blocker

- **Code path / known Issue #28 reproduction:** PASS. The exact regression no longer drops the opening sentence; source span and full-result logic are tested.
- **Human semantic quality, difficult abbreviations, single-sentence clause omissions:** NOT PROVEN. Model can still mistranslate or omit meaning *inside* an individual source segment.
- **Oversized UI:** fail-closed rather than clipping, but currently offers **no readable full-text alternative or clear user-visible error**; retain product/UI blocker before beta.
- **Actual Windows UIA → inference → native popup focus/DPI/accessible text:** NOT RUN on an interactive desktop.

Therefore Issue #28 remains OPEN as the broader beta-quality and UX acceptance item. Do not treat a narrowly passing model regression as a public release gate being complete.
