# F2.7 — EN→ID quality, CPU/memory and Windows interactive proof

**Status:** F2.7 Windows native model/quality/resource CI **PASSED** (2026-10-26). The default remains `beam=2, cpu_threads=2`; the comparative results and qualitative paragraph omission make switching to greedy unsafe without more work. **Full desktop Windows UIA-to-popup E2E remains unverified.**

## What is actually tested

- **Real backend**: Windows `offline_model_smoke.exe`, converted from the pinned OPUS-MT EN→ID INT8 pack. The test compares the existing `beam=2, 2 threads` against `beam=1, 2 threads` in separate native processes; no simulated translator.
- **Quality dataset**: `tests/fixtures/en-id-quality-v1.json`, 28 manually authored English prompts and independent Indonesian *candidate references*: simple requests, software and privacy terms, numbers/dates, proper nouns, ambiguity, negation/conditionals, multi-sentence text and punctuation. Single-reference chrF is computed by SacreBLEU. This is a narrow **diagnostic**, not measured human translation quality.
- **Output parity**: per-fixture exact target differences, category chrF and digit-preservation *warnings*, never a misleading pass/fail threshold on a single reference.
- **Resource sampling**: independent Python/psutil native process-tree memory, Windows working set and private committed memory, approximate cumulative CPU seconds/core-equivalents sampled every 100ms. These are **headless inference worker** metrics, not ClipLingo tray + Settings WebView2 + UIA, not power/battery consumption, and not hard real-time peaks.
- **Existing latency**: F2.6 `beam=2/2` 12-fixture warm p95=901ms; `beam=1/2` warm p95=727ms, identical fixed-fixture target hash. That small-sample result **does not automatically justify switching the default**.

## Automated acceptance gates

| Invariant | Boundary |
| --- | --- |
| Schema/fixture ID consistency; no blank target or reordering | Real native Rust output parsed in Python |
| Both profile runs truly use model-loaded `ct2rs` | Windows optional CT2 build, converted model, headless binary |
| Baseline vs greedy output parity and chrF observations | SacreBLEU + independent translated results on 28 varied examples |
| Sampled process private/working set and CPU seconds recorded | psutil against spawned Windows native process, no estimation from thread count |
| Production default is unchanged | `TranslationEngine::start` uses `OfflineProfile::default` beam=2/threads=2 |
| No selection source telemetry or network upload | native CLI test fixtures only; runtime still has no online provider |

## Windows desktop acceptance — requires real interactive session

CI runs on a Windows runner, **not** an asserted foreground Notepad/Chrome/PDF desktop. In an interactive Windows session, install the local model, build with static MSVC CRT and activate ClipLingo. Then:

1. Focus Notepad, select `Good morning.` and press `Ctrl+Alt+T`. Confirm actual Indonesian result, no fake output, no clipboard side effects and original text-input keyboard focus retained.
2. Repeat in Chrome and an actual PDF viewer. Failure to authenticate UIA focused-element ancestry must **fail closed**. No fallback to clipboard or OCR is authorized in F2.7.
3. Try rapidly selecting A then B: only B may appear. Test a password input, elevated/protected app, no selection and blocked source app. UIA hangs must not freeze the Tauri UI thread.
4. On a mixed-DPI display with negative monitor coordinates, check popup placement, accessibility and no focus theft. Native GDI is not an accepted final DirectWrite/accessible renderer.
5. For an observational foreground transition log (without source text/title), run `scripts/probe-f27-foreground.ps1 -Seconds 15`, then switch to Notepad and trigger translation. **Transition absence does not prove focused control or no activation**; separate GetGUIThreadInfo/native UI test and visual inspection are needed.
6. Measure ClipLingo **entire process tree** with `scripts/measure-f25.ps1` for tray idle, Settings open, warm model, and after model eviction. On lower-spec CPUs, repeat the model p95 experiment while controlling background load.

## Promotion decision

Do **not** change default beam or thread count based solely on chrF, single-reference output identity, one GitHub runner, sampled peak RSS, or synthetic short sentences. Promote only after relevant EN→ID human-rated sentence meaning, terminology, negation, numbers and paragraphs plus CPU contention, memory and full Windows GUI behavior are acceptable.

Evidence is linked through the [F2.6 baseline](f26-acceptance.md) and [F2.5 native GUI acceptance](f25-acceptance.md). New F2.7 CI output should be captured in this document after it finishes.

## Observed Windows CI results — 2026-10-26

**Evidence:** [CI run #40251146437](https://github.com/howlil/cliplingo/actions/runs/40251146437) (CTranslate2 model download/conversion, four-profile regression, quality+resource evaluator PASSED), `f27-public-model-evaluation` artifact (raw fixed-public-fixture model outputs, not user data), [implementation PR #27](https://github.com/howlil/cliplingo/pull/27).

| Measurement | Beam 2 / 2 threads (current default) | Beam 1 / 2 threads (greedy) |
| --- | ---: | ---: |
| Curated 28-item single-reference corpus chrF | **71.96** | **69.00** |
| Total quality-suite process wall time | 40.656 s | 37.891 s |
| Sampled process CPU time | 76.875 s | 70.031 s |
| Approximate CPU core equivalents | 1.891 | 1.848 |
| Peak sampled Windows working set | 137.2 MiB | 135.7 MiB |
| Peak sampled Windows private commit | 132.4 MiB | 131.1 MiB |
| Public fixture target changes vs default | — | **5 of 28** |
| Digit-preservation diagnostics | No flagged cases | No flagged cases |

Greedy was **6.8% faster in total wall time** and used ~**8.9% less sampled CPU time**, with only ~1–2 MiB less sampled peak memory. These are one-run headless process measurements with a 100-ms sampler, not process-tree ClipLingo tray/Settings footprint, GPU/battery power, or production latency. Differences are not statistically stable without repeated controlled runs.

The output changes occurred in `apology`, `ambiguous_bank`, `conditional`, `paragraph`, and `question_mark`. The category diagnostics show a large fall in chrF for the 2-item paragraph category (baseline **70.37**, greedy **51.07**), and ambiguity (baseline **59.60**, greedy **49.65**). Corpus scores are only clues: there is one human-authored reference per case, so valid paraphrases may be unfairly penalized.

### Semantic regression found (beta blocker)

Public test source: `The report summarizes the experiment. We tested several settings, recorded the response time, and compared the results.`

- Baseline beam=2 result: `Kami menguji beberapa pengaturan, mencatat waktu respon, dan membandingkan hasilnya.` **Omits the opening sentence** about the report summarizing the experiment.
- Greedy beam=1 result: `Laporan itu meringkaskan percobaannya. dan membandingkan hasilnya.` **Omits tested settings and recorded response time** and has unnatural syntax.

Therefore **neither profile is yet trustworthy for all multi-sentence selections**. The problem is tracked in [Issue #28](https://github.com/howlil/cliplingo/issues/28). Sentence-aware segmentation and full semantic coverage must be verified on real model output before beta; raising model decoding speed does not solve omission.

### Decision

- **KEEP default:** `beam=2, cpu_threads=2` (no runtime behavior changed in PR #27).
- **DO NOT promote greedy yet:** five changed outputs; important omission in a paragraph; only modest CPU/wall improvement on this suite.
- **NEXT:** fix multi-sentence completeness and run an independent qualitative review, then interactive Windows `Ctrl+Alt+T → UIA capture → inference → native popup` acceptance, mixed DPI/foreground focus and whole-app RAM profile. A normal GitHub Actions Windows runner does not prove GUI behavior.
