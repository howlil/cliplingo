# Current Milestone — Real Offline Translation Alpha

**Status:** active; current slice is blocked on a native CMake configure defect.

**Goal:** replace the deterministic worker translation with a real distributable offline Japanese-to-Indonesian path while preserving the established isolated worker, privacy, and popup contracts.

**Why:** the shell-to-worker boundary is proven; the next product capability is actual local translation rather than a deterministic fake response.

## Feature Compass

**Shape:** selected Japanese text -> local `ja -> en` OPUS stage -> local `en -> id` OPUS stage -> existing popup, with no cloud translation dependency.

**Position:** the model-pack foundation is merged. Native CTranslate2/SentencePiece runtime integration is open in PR #16 and has not passed its Windows configure gate.

**Delta:** make the pinned native runtime build and execute in CI, then wire the real OPUS stages into the worker and prove the end-to-end path.

**Next Move:** fix C++17 propagation for SentencePiece/Abseil during CMake configuration on PR #16, rerun its CI, and do not start production model wiring until that native runtime gate is green.

## Scope

### In

- First offline alpha route: Japanese -> English -> Indonesian.
- Apache-2.0 OPUS-MT model-pack sources pinned by revision.
- CTranslate2 CPU INT8 runtime.
- SentencePiece tokenization assets.
- Existing Windows Named Pipe / worker-protocol boundary.
- Real worker inference behind the existing `Translator` port.
- Automated Windows CI evidence for native runtime and worker integration.
- Alpha-level latency/resource qualification after the real path exists.

### Out

- Automatic language detection beyond what this route requires.
- Chinese or Korean packs in this milestone.
- Model download/settings UX.
- OCR.
- GPU inference.
- Installer/signing/package-manager release work.
- Stable-grade compatibility/performance certification.
- Manual Windows interactive testing as a merge blocker unless explicitly reintroduced.

## Slices

- [x] **Model-pack foundation** — define and validate the distributable `ja -> en -> id` OPUS-MT INT8 pack contract.
- [ ] **Native translation runtime foundation** <- ACTIVE — source-pin and execute CTranslate2 + SentencePiece in the Windows C++ toolchain without Python runtime dependency.
- [ ] **Worker OPUS runtime** — load the committed pack stages and perform real worker translation through the existing protocol.
- [ ] **End-to-end offline translation** — prove selected Japanese text reaches the popup as real Indonesian output through `WorkerTranslator`.
- [ ] **Alpha milestone gate** — qualify correctness plus alpha-appropriate latency/resource behavior and close the milestone.

## Current decisions

- Alpha route is `ja -> en -> id`.
- Model family is Helsinki-NLP OPUS-MT with Apache-2.0 license metadata and pinned revisions.
- Runtime baseline is CTranslate2 4.8.2, CPU, INT8, with SentencePiece 0.2.2 assets.
- Production model weights are not downloaded in ordinary CI; deterministic/small fixtures provide native runtime evidence.
- Existing protocol/process isolation remains unchanged while inference implementation evolves.

## Verification / evidence

### Completed model-pack foundation

- PR #15 merged.
- CI #75 (`33538594417`) passed automated model-pack and regression gates.
- Merge commit: `e87a0dd1b2a6286dbc6e73bdc1db45bcc30194e3`.
- Catalog: `models/catalog/ja-id-opus-v1.json`.

### Active native runtime slice

- PR #16: `feat: prove native translation runtime foundation`.
- Branch: `feat/worker-ctranslate-runtime`.
- Observed head: `62284b8ceeac2ccdedce1adcc0b930b7755b1208`.
- CI #81 (`33540440956`) completed with failure.
- Frontend checks/build, model-pack dry-run, Rust formatting, Clippy, and Rust tests passed before the native configure failure.
- Failure occurs while configuring SentencePiece's Abseil dependency: Abseil reports that the compiler is configured for C++ < 17 even though C++17 is required.
- Native build/runtime probe and downstream worker integration gates did not run after that configure failure.

## Blockers / risks

- **Current blocker:** CMake language-standard propagation must make SentencePiece/Abseil configure under C++17 on the hosted Windows runner.
- Real OPUS model quality and latency are not yet qualified through the shipped worker path.
- Final beta/stable performance budgets remain an open product decision, not an alpha blocker.

## Next Action

Fix the PR #16 CMake configuration so Abseil observes C++17, rerun the existing Windows CI, and merge the slice only after native configure/build/runtime-probe plus existing worker regressions are green.
