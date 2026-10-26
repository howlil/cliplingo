# F2.6 — Offline CPU inference profile experiment

Status: **Experiment pending Windows CI**. Default shipping configuration stays `beam_size=2` and `num_threads_per_replica=2` until measured quality/latency comparisons support any change.

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
