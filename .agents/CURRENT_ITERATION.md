# Current Milestone — Real Offline Translation Alpha

**Status:** active; the coherent product capability is implemented on PR #23 and is now in qualification. The previous MSVC CRT link blocker is resolved; the remaining native gate is bounded runtime execution evidence.

**Goal:** ship a distributable Windows alpha where selected Japanese text is translated locally through the pinned OPUS `ja -> en -> id` route and returned through the existing ClipLingo popup, with a fresh-install model acquisition path and no cloud translation dependency.

**Why:** shell capture, popup behavior, request correlation, worker isolation, and model-pack definition already exist. This milestone turns those foundations into the first user-usable real offline translation capability.

## Feature Compass

**Shape:** select Japanese text -> `Ctrl+Shift+T` -> Windows selection capture -> local SentencePiece/CTranslate2 `ja -> en` -> local SentencePiece/CTranslate2 `en -> id` -> Indonesian popup result. If the model pack is absent, the popup offers a verified one-time install action.

**Position:** PR #23 (`feat: ship real offline translation alpha`) contains the native runtime fix, real worker OPUS execution, verified model-pack lifecycle, popup install state, real translation smoke, NSIS release configuration, release automation, README, and release notes. The native C++ graph now configures and links successfully on Windows after aligning CTranslate2/SentencePiece/bundled Abseil on one static MSVC CRT policy. The earlier unbounded runtime probe exposed a hang, so the capability branch now uses a single-thread/greedy diagnostic probe with an explicit CI timeout rather than allowing verification to stall indefinitely.

**Delta:** get PR #23 through Rust/native qualification, prove the real production model path in the release workflow, merge to `master`, create the gated `v0.1.0-alpha.1` GitHub prerelease, then submit that exact installer asset and SHA256 to WinGet.

**Next Move:** qualify the latest PR #23 head. Fix only evidence-backed failures. When its integration checks are green, merge the coherent milestone implementation and trigger the gated alpha release branch.

## Milestone scope

### In

- pinned native CTranslate2 4.8.2 + SentencePiece 0.2.2 runtime;
- OPUS-MT `ja -> en -> id` CPU INT8 production route;
- real production inference inside the isolated C++ worker;
- existing protocol v1, request correlation, latest-request-wins, popup, and Windows capture path;
- installed/missing model-pack state;
- explicit model download/install with SHA256 verification and staged activation;
- model removal and stable local model-pack location;
- missing/corrupt model behavior that does not crash the shell;
- real non-deterministic worker smoke and measured cold/warm translation timing during release qualification;
- Windows x64 NSIS alpha installer containing the worker;
- README + release documentation;
- immutable GitHub prerelease/tag `v0.1.0-alpha.1` only after qualification succeeds;
- downstream WinGet submission using the canonical GitHub Release installer and SHA256.

### Out

- additional language routes;
- OCR;
- cloud fallback;
- GPU inference;
- accounts/sync;
- broad unrelated UI redesign;
- advanced model marketplace/automatic route selection;
- stable-grade signing or compatibility certification.

WinGet/release packaging entered this milestone because the user explicitly requested distribution after the product capability is complete. That scope amendment does not add unrelated product functionality.

## Milestone slices

- [x] **Model-pack foundation** — catalog, pinned revisions, manifest/license/integrity intent.
- [~] **Native translation runtime** — configure + compile + link are proven after coherent MSVC CRT propagation; bounded runtime execution is still being qualified.
- [~] **Worker OPUS runtime** — real two-stage SentencePiece/CTranslate2 execution is implemented on PR #23; production smoke still gates release.
- [~] **Model-pack lifecycle** — installed/missing detection, verified download, staged activation, removal, worker path injection, and popup install action are implemented; Rust qualification is in progress.
- [~] **End-to-end offline translation** — existing selected-text/popup path is wired to the real worker; automated production model smoke and release packaging evidence remain.
- [ ] **Alpha release gate** — merge verified capability, build production model pack, run real translation smoke, build NSIS installer, publish `v0.1.0-alpha.1` prerelease/tag.
- [ ] **WinGet distribution** — submit the released installer URL + SHA256 to `microsoft/winget-pkgs`; publication remains pending until upstream validation/merge.

A technical slice is not reported as a completed user feature until the required user path has credible evidence.

## Current decisions

- Rust remains application authority; Svelte remains presentation only.
- Inference remains isolated in `cliplingo-worker.exe` over Windows Named Pipe protocol v1.
- First production route remains OPUS-MT `ja -> en -> id`, CPU INT8.
- CTranslate2/SentencePiece are source-pinned and linked into the worker; the native graph uses one static MSVC CRT policy.
- Deterministic worker output exists only under explicit `CLIPLINGO_WORKER_TEST_MODE=deterministic` for protocol/regression CI.
- Production model weights are built only for release qualification, not downloaded by ordinary CI.
- Release binary source is GitHub Releases. WinGet references that immutable asset; it does not rebuild ClipLingo.
- Alpha may be unsigned; signing is not allowed to silently become a stable-grade blocker for this alpha.

## Current evidence

- Model-pack foundation is already merged on `master`.
- PR #16's latest native build reached successful CMake configure and successful C++ worker/runtime-probe linking after `ABSL_MSVC_STATIC_RUNTIME` and related CRT propagation were aligned. This proves the previous `/MD` vs `/MT` link blocker is resolved.
- The old PR #16 runtime probe then remained in progress abnormally long; it is not accepted as pass evidence.
- PR #23 is the coherent replacement/integration branch for the complete alpha capability.
- Observed PR #23 CI #119: frontend check, frontend tests, frontend build, and model-pack dry-run all passed. Rust formatting failed only on two mechanical formatting diffs; those diffs were corrected on the branch before continuing qualification.
- The runtime probe on PR #23 now emits component/phase markers, uses one computation thread and greedy decode, and has a 3-minute CI timeout so a native hang becomes an actionable failure instead of an indefinite gate.
- Release workflow requires a pinned production model pack, native runtime probe, real non-deterministic translation smoke, NSIS installer build, and emitted model/installer SHA256 before it can create `v0.1.0-alpha.1`.

## Blockers / risks

### Current blocker

No known source-level linker blocker remains. The immediate gate is the latest PR #23 qualification, especially Rust compile/tests and bounded native runtime execution.

### Follow-on release risks

- production OPUS conversion/model loading may expose incompatibility not visible in the tiny native fixture;
- real two-stage latency may be higher than target hypotheses and must be measured rather than guessed;
- NSIS/Tauri resource packaging must prove the worker is present at the runtime resource location;
- the alpha is unsigned and may trigger SmartScreen;
- WinGet submission can be prepared and opened after the GitHub Release exists, but final catalog availability depends on upstream validation/merge.

Do not widen the milestone for these risks. Fix only failures that block the advertised alpha path or its requested distribution.

## Single next action

Run the latest PR #23 qualification to completion; resolve any concrete Rust/native failure, merge the coherent capability when green, then trigger the gated alpha release and use its actual installer URL/SHA256 for WinGet submission.
