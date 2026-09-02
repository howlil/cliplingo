# Current Milestone — Real Offline Translation Alpha

**Status:** active; the native translation runtime foundation is blocked at MSVC link-time runtime-library consistency.

**Goal:** replace deterministic worker translation with a real distributable offline Japanese-to-Indonesian path while preserving the isolated worker, privacy, popup, and request-correlation contracts.

**Why:** the shell-to-worker path is already proven. The highest-value remaining alpha capability is real local translation through that established path.

## Feature Compass

**Shape:** selected Japanese text -> local `ja -> en` OPUS stage -> local `en -> id` OPUS stage -> existing popup, with no cloud translation.

**Position:** model-pack foundation is merged. PR #16 is the active prerequisite slice. Its latest Windows CI now configures the native CTranslate2/SentencePiece graph successfully with C++17, but fails while linking the runtime probe because the MSVC runtime library is inconsistent across worker/runtime-probe and SentencePiece-bundled Abseil dependencies. PR #19 already contains the downstream worker OPUS runtime work and is stacked on #16; it remains downstream until the native runtime foundation is green and integrated.

**Delta:** establish one consistent MSVC runtime-library policy across the native dependency graph, prove the native runtime probe and existing worker regressions, then integrate the already-defined worker OPUS runtime and continue toward the real offline user path.

**Next Move:** fix the `/MD` versus `/MT` propagation at the native dependency/configuration boundary in PR #16, run only the targeted native build/probe plus affected worker-boundary regressions during iteration, use CI as integration evidence, then merge the coherent prerequisite slice and continue directly to the worker OPUS runtime.

## Milestone scope

### In

- pinned native CTranslate2 + SentencePiece runtime for the isolated worker;
- committed `ja -> en -> id` OPUS model-pack contract;
- production worker execution through the real local model stages;
- preserve protocol v1, process isolation, privacy, and latest-request-wins behavior;
- prove the smallest credible offline selected-text -> translated-popup path for the alpha milestone.

### Out

- new language families or routing expansion;
- cloud fallback;
- OCR;
- accounts/sync;
- broad UI polish unrelated to the real translation path;
- release/signing/package-manager work not required to prove this alpha capability.

## Milestone slices

- [x] **Model-pack foundation** — catalog, pinned source revisions, build/manifest intent, license/integrity metadata.
- [ ] **Native translation runtime foundation** — CTranslate2/SentencePiece builds and executes behind the worker boundary. **ACTIVE / BLOCKED**
- [ ] **Worker OPUS runtime** — real `ja -> en -> id` execution in the production worker. PR #19 already contains this downstream slice.
- [ ] **End-to-end offline translation** — real selected text reaches the popup through the existing shell/worker path.
- [ ] **Alpha milestone gate** — focused correctness/privacy/native evidence for the advertised alpha behavior.

A foundation slice is a prerequisite, not a completed user feature. Do not expand this milestone with nice-to-have work while the core offline translation path is incomplete.

## Current decisions

- Preserve the existing Rust application authority, Svelte presentation boundary, Windows adapters, worker process isolation, Named Pipe transport, and worker protocol v1.
- First real distributable route remains OPUS-MT `ja -> en -> id`, CPU INT8, as recorded in `DECISIONS.md`.
- Native dependency/toolchain work is verified at the boundary it changes. Do not turn every implementation loop into a full repository test ladder.
- Integrate at coherent slice/logical-change boundaries. Do not create additional planning state or branch-per-tweak workflow around this blocker.

## Current evidence

- Model-pack foundation is already merged on `master`.
- PR #16 (`feat: prove native translation runtime foundation`) is open and remains the active prerequisite.
- Latest observed PR #16 Windows CI: run #109 (`33676446605`) failed.
- Frontend checks/tests/build, model-pack dry-run, Rust formatting/Clippy/tests, and native CMake configure passed in that run.
- Abseil's C++17 configure probe passed, so the previous language-standard configure blocker is no longer current.
- Native runtime probe linking failed with repeated `LNK2038` runtime-library mismatches: `MD_DynamicRelease` versus `MT_StaticRelease`, followed by CRT/default-library conflicts and unresolved externals.
- Because native linking failed, the runtime probe and downstream Rust-to-worker integration checks did not execute in that run.
- PR #19 (`feat: run OPUS translation in isolated worker`) is open and intentionally stacked on PR #16.

## Blockers / risks

### Current blocker

The native dependency graph mixes MSVC CRT modes. The fix must make runtime-library selection consistent across ClipLingo targets and dependencies configured by CTranslate2/SentencePiece/Abseil; suppressing linker diagnostics or forcing conflicting default libraries is not acceptable evidence.

### Follow-on risks

After the native runtime foundation is green:

- real OPUS tokenization/model loading can still expose model/runtime compatibility defects;
- the full local route can expose latency/memory issues that are invisible in deterministic worker regressions;
- model acquisition/install lifecycle remains necessary before a distributable user-facing alpha is complete.

Treat these as follow-on slice risks, not reasons to widen the current blocker fix.

## Single next action

Make PR #16 use one coherent MSVC runtime-library policy through the complete native dependency graph, prove the runtime probe and affected worker-boundary regressions, integrate the slice, then continue directly with the already-defined worker OPUS runtime.