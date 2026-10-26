# F2.6 — Offline CPU inference profile experiment

Status: **Windows native model benchmark PASS** ([GitHub Actions run #40248108153](https://github.com/howlil/cliplingo/actions/runs/40248108153)); default app stays `beam_size=2` / 2 threads because output identity on 12 short fixtures does not prove broad EN→ID translation quality or battery/CPU impact. Profile implementation merged via [PR #25](https://github.com/howlil/cliplingo/pull/25).

## Decision and experiment scope

[CTranslate2 performance guidance](https://opennmt.net/CTranslate2/performance.html) recommends trying greedy decoding (`beam_size=1`) to reduce CPU translation latency, while warning implicitly that search behavior changes. No user-facing accuracy/latency improvement is presumed. The CI experiment uses the **same pinned Helsinki-NLP EN→ID INT8 pack**, fixed public English fixtures and independent processes for each profile:

| ID | Beam | CPU threads | Meaning |
| --- | ---: | ---: | --- |
| A — production baseline | 2 | 2 | Existing F2.5 behavior |
| B — greedy baseline threads | 1 | 2 | Search optimization alone |
| C — greedy fewer threads | 1 | 1 | Reduce scheduling/contention |
| D — greedy more threads | 1 | 4 | CPU-scaling counterexample |

Each run executes one **cold** model load + translation followed by 12 sequential **warm** translations, reporting warm p50/p95/max in ms. Outputs are nonempty and a combined SHA-256 target digest is printed per profile to detect differences without logging source-user content or model results. This is **not** a translation quality grade; differing hashes require a separate quality review and matching hashes prove only output identity for the 12 fixtures. Run order, shared CI hardware, filesystem cache, OS power policy and small sample sizes confound comparing cold and warm timings: never promote a profile solely from these values.

## Hard requirements

- Maintain latest-only pending work, source-exclusion policy, native foreground check, offline-only privacy and bounded model lifecycle.
- No runtime auto-detection that changes model quality unpredictably. Benchmark-only profiles are explicitly validated (beam 1/2; CPU 1/2/4) and not exposed in Settings or normal hotkeys.
- No default beam-size change until validated real model output, a curated EN→ID quality suite, practical Windows memory/CPU/p95 evidence and interactive `Ctrl+Alt+T` → UIA → popup acceptance.
- Default Cargo feature-off build cannot fabricate translation. No model download at tray startup.

## Evidence to capture

- Workflow URL, complete success status, each profile's cold + warm p50/p95/max, SHA-256 output digest, model revision, Windows machine/CPU information where available.
- A native CI success proves that *headless CTranslate2 and tokenizer* run; it does not prove browser UIA, mixed-DPI/native accessibility, focus preservation or battery use.
- Interactive acceptance remains in [F2.5 acceptance](f25-acceptance.md). Do not label a cold/warm CI metric as actual hotkey-to-visible p95.

## Follow-up gate

If greedy decoding is materially faster, evaluate actual translated outputs on a representative EN→ID suite (short/long sentences, domain terms, proper nouns, numbers and ambiguity), RAM/CPU and desktop focus. Change production defaults in a separate measured commit only after that evidence.

## Measured Windows CI results — 2026-10-26

| Profile | Cold first request | Warm p50 | Warm p95/max | 12-target output SHA-256 |
| --- | ---: | ---: | ---: | --- |
| A: beam 2, 2 threads | 3697 ms | 654 ms | 901 ms | `6b80be24f637…` |
| B: beam 1, 2 threads | 3648 ms | 603 ms | 727 ms | `6b80be24f637…` |
| C: beam 1, 1 thread | 4008 ms | 1083 ms | 1300 ms | `6b80be24f637…` |
| D: beam 1, 4 threads | 3548 ms | 478 ms | 566 ms | `6b80be24f637…` |

All four aggregate target SHA-256 values were exactly `6b80be24f637bdcc38f69b6adfb3e2a38ab9a7469c2cf56760bb6601073f22a9` for the 12 fixed public fixtures. The CI output also confirmed a nonempty real EN→ID translation for the cold fixture. This is **output identity**, not human-rated quality. The 12 short inputs are not representative of long paragraphs, professional terms, ambiguous language or production traffic.

- Profile B improved warm p95 by **19.3%**, with unchanged thread count and identical output on these fixtures.
- Profile D improved warm p95 by **37.2%**, but four active CPU threads may increase contention, energy use and idle/warm resource footprint on low-spec Windows devices.
- Profile C was slower on this runner. Single run, 12 sequential warm fixtures, no hardware/CPU controls or process-tree RAM measurements: these are **exploratory** performance observations.
- **Decision:** Keep A as the active production default. Revisit B as an efficient potential default only after representative EN→ID quality test cases, repeat runs on low-/mid-tier CPUs and interactive Windows UIA→popup latency; D remains an opt-in benchmark comparison until CPU/RAM/power costs are proven.

### Evidence and code review

- [Default Rust Windows unit and release check — PASS](https://github.com/howlil/cliplingo/actions/runs/40248108254)
- [Optional CTranslate2 build, model conversion and all four native profiles — PASS](https://github.com/howlil/cliplingo/actions/runs/40248108153)
- [Merged implementation PR #25](https://github.com/howlil/cliplingo/pull/25)
- The benchmark uses `TranslationEngine::start_with_profile` with a strict profile allowlist; standard `TranslationEngine::start` retains the previous default. The CLI harness logs timings and a digest, not user-selected text.

No hotkey-to-visible p95, focus-preservation, signed installer, RAM/power or full model quality evidence was produced by this run.
