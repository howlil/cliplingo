# Completed Milestone — Real Offline Translation Alpha

**Status:** product milestone and requested GitHub alpha release are complete. `v0.1.0-alpha.1` was published from verified `master`. The only remaining distribution follow-up is opening the prepared WinGet manifest set as an upstream `microsoft/winget-pkgs` PR; the connected GitHub integration is not authorized to create a PR in that upstream repository.

**Goal achieved:** selected Japanese text can flow through the existing Windows capture/popup path to a real local OPUS `ja -> en -> id` translation worker, with a fresh-install model acquisition path and no cloud translation dependency.

## Feature Compass

**Shape:** select Japanese text -> `Ctrl+Shift+T` -> Windows selection capture -> local SentencePiece/CTranslate2 `ja -> en` -> local SentencePiece/CTranslate2 `en -> id` -> Indonesian popup result. If the model pack is absent, the popup exposes a verified one-time install action.

**Position:** the coherent capability was merged through PR #23 and released as `v0.1.0-alpha.1`. Integration CI, production model-pack build, native runtime probe, real non-deterministic OPUS translation smoke, NSIS packaging, checksum generation, and GitHub prerelease publication all passed.

**Delta:** no product-capability delta remains for this milestone. The downstream WinGet manifest set is complete in `howlil/winget-pkgs` branch `howlil-cliplingo-0.1.0-alpha.1`; upstream PR creation is the only unresolved distribution action.

**Next Move:** open the prepared WinGet branch against `microsoft/winget-pkgs:master` from a GitHub identity/integration authorized to create upstream pull requests. Do not start another product milestone until the user defines the next core capability.

## Delivered scope

- pinned native CTranslate2 4.8.2 + SentencePiece 0.2.2 runtime;
- one coherent static MSVC CRT policy across the native dependency graph;
- OPUS-MT `ja -> en -> id` CPU INT8 production route;
- real production inference inside isolated `cliplingo-worker.exe`;
- existing protocol v1, request correlation, latest-request-wins, popup, and Windows capture path preserved;
- installed/missing model-pack state;
- explicit model download/install with SHA256 verification and staged activation;
- model removal and stable local model-pack location;
- missing/corrupt model behavior that does not crash the shell;
- real non-deterministic production worker smoke with cold/warm timing instrumentation;
- Windows x64 NSIS alpha installer containing the worker;
- README + release documentation;
- GitHub prerelease/tag `v0.1.0-alpha.1`;
- complete WinGet 1.12.0 multi-file manifest set using the canonical release installer and SHA256.

## Explicit non-scope retained

- additional language routes;
- OCR;
- cloud fallback;
- GPU inference;
- accounts/sync;
- unrelated UI redesign;
- advanced model marketplace/automatic route selection;
- stable-grade signing or compatibility certification.

## Milestone slices

- [x] **Model-pack foundation** — catalog, pinned revisions, manifest/license/integrity intent.
- [x] **Native translation runtime** — configure, compile, link, and bounded runtime execution proven on Windows.
- [x] **Worker OPUS runtime** — real two-stage SentencePiece/CTranslate2 execution in production worker.
- [x] **Model-pack lifecycle** — installed/missing detection, verified download, staged activation, removal, worker path injection, and popup install action.
- [x] **End-to-end offline translation implementation** — existing selected-text/popup path wired to the real worker; production-model worker smoke passed.
- [x] **Alpha release gate** — production model pack, native probe, real translation smoke, NSIS installer, checksums, and GitHub prerelease/tag all published successfully.
- [~] **WinGet distribution** — manifests are complete on the user fork; upstream PR creation is externally blocked by GitHub integration authorization. Catalog publication remains upstream-owned after PR validation/merge.

## Release evidence

### Integration

- PR #23: `feat: ship real offline translation alpha`.
- PR #23 final CI run: `33688039468` — success.
- Merge commit on `master`: `30db32e2bdd69d96739709be6eff5642d6f30aae`.

The final PR run passed:

- frontend check/tests/build;
- model-pack dry-run;
- Rust formatting, Clippy, and tests;
- native CMake configure;
- C++ worker/runtime-probe build;
- native runtime probe;
- Rust -> C++ worker boundary regression;
- worker-backed translator regression.

### Release

- Release workflow run: `33718354847` — success.
- Tag/prerelease: `v0.1.0-alpha.1` / `ClipLingo 0.1.0 Alpha 1`.
- Release target: `30db32e2bdd69d96739709be6eff5642d6f30aae`.
- Windows installer: `ClipLingo_0.1.0-alpha.1_x64-setup.exe`.
- Installer SHA256: `96a26f3985c95553177cb20f120b0bd1abb31040d8ded6af32132438c667e39b`.
- Production model pack: `cliplingo-ja-id-opus-v1.zip`.
- Model-pack SHA256: `e3e6873d688d4ba3860ce20b6e2539481fc6dc1f8ae2396fc037e7329c754e30`.

The release workflow passed, in order:

1. build pinned production OPUS model pack;
2. validate required pack files;
3. hash/archive pack;
4. configure and build native worker/runtime probe;
5. execute native runtime probe;
6. extract production model pack;
7. execute real offline Japanese translation smoke with no deterministic test mode;
8. stage bundled worker;
9. build NSIS installer;
10. normalize/hash installer;
11. publish gated alpha release and tag;
12. publish release evidence artifact.

## WinGet evidence

Prepared fork branch:

- repository: `howlil/winget-pkgs`;
- branch: `howlil-cliplingo-0.1.0-alpha.1`;
- package identifier: `Howlil.ClipLingo`;
- package version: `0.1.0-alpha.1`;
- schema: WinGet multi-file manifest `1.12.0`;
- installer type: `nullsoft`;
- installer URL: canonical GitHub Release NSIS asset;
- installer SHA256 matches the published GitHub asset digest.

Attempting to create the cross-repository PR through the connected GitHub integration returned `403 Resource not accessible by integration`. This is an authorization boundary, not a manifest/product failure. Do not claim WinGet catalog availability until an upstream PR is actually opened, validated, and merged.

## Final milestone assessment

The advertised Real Offline Translation Alpha capability and GitHub alpha distribution gate are complete. No known source/runtime/release blocker remains for the published alpha. The remaining WinGet action is external submission authorization only.
