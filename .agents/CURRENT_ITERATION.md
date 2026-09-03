# Current Milestone — EN→ID Native Tray Translation Experience

**Status:** the product slices and proportional CI architecture are implemented on branch `feat/en-id-native-tray-experience`. A PowerShell latest-release bootstrap has now been added as the requested distribution path. Previous full CI self-validation passed; the new PowerShell distribution lane still needs exact-head CI evidence before merge. Do not call the native UX completely accepted until explicit Windows interaction acceptance is observed where automation cannot credibly prove shell behavior.

**Goal:** make ClipLingo behave as a focused Windows background utility: run from the system tray, translate selected English directly to Indonesian offline, show nothing when no valid selection exists, present the result in a compact draggable utility popup, and provide a safe one-command Windows PowerShell installation route to the newest published release.

## Feature Compass

**Shape:** system tray -> select English text -> `Ctrl+Alt+T` -> validate/capture selection -> direct local OPUS `en -> id` -> compact draggable Indonesian popup. Installation can start from PowerShell and resolves the newest published GitHub Release automatically. No selection means no visible translation UI.

**Position:** the branch contains the one-stage EN -> ID model/runtime, selection-before-popup ordering, silent no-selection regression, native Tauri tray + Settings lifecycle, restrained draggable popup UI, risk-routed CI, and `scripts/install.ps1`. The bootstrap resolves the newest published non-draft release including prereleases, requires the x64 installer asset, verifies SHA256 from GitHub release metadata or its checksum sidecar, and only then executes the installer. CI run `33793098041` previously proved the frontend/Rust/model/native risk lanes and aggregate `required`; the new distribution lane is awaiting exact-head evidence.

**Delta:** validate the PowerShell bootstrap parser + `-ResolveOnly` release/checksum resolution together with the workflow change, merge PR #25, then run gated `v0.1.0-alpha.2` release qualification. Native Windows interaction acceptance remains explicit evidence for calling the shell UX fully accepted.

**Next Move:** complete exact-head CI for the PowerShell distribution change. Fix only evidence-backed failures, then merge the coherent milestone.

## Milestone scope

### In

- direct OPUS-MT English -> Indonesian CPU INT8 route;
- `en-id-opus-v1` single-stage model pack and lifecycle;
- `Ctrl+Alt+T` as the canonical default shortcut across code/docs/UI;
- capture/selection validation before popup visibility;
- no-selection silent stop with no translator call;
- Windows system tray/notification-area application home;
- left-click tray -> Settings;
- tray menu -> Settings / Quit ClipLingo;
- close Settings -> remain running in tray;
- minimal Settings surface for route, shortcut, model state/install/remove, status, and quit;
- draggable translation popup;
- popup auto-position near selection/cursor for each new translation;
- anti-slop UI pass: compact Windows utility hierarchy, no decorative glass/gradient/glow/bento treatment;
- proportional risk-routed CI/testing contract;
- PowerShell bootstrap that installs the newest published GitHub Release with SHA256 verification;
- real EN -> ID production inference smoke and alpha packaging route.

### Out

- Japanese/additional production routes;
- automatic language detection;
- OCR;
- cloud fallback;
- GPU inference;
- translation history;
- accounts/sync;
- configurable shortcut UI;
- updater;
- broad settings dashboard;
- package mirroring or a separate distribution backend;
- test-framework expansion or E2E ceremony unrelated to changed risk.

## Slices

- [x] **Direct EN→ID Runtime** — source implementation complete and applicable PR CI green; release qualification still owns real production weights.
- [x] **Selection-Gated Interaction** — capture occurs before `popup.show`; capture failures cancel invisibly; regression proves no popup and zero translator calls for `NoSelection`.
- [~] **Windows Tray + Settings Shell** — source implementation and Windows compile evidence complete; native shell interaction acceptance remains.
- [~] **Movable Translation Surface + UI Quality** — source implementation and frontend verification complete; manual drag behavior remains part of native interaction acceptance.
- [x] **Fast Accurate CI Contract** — classifier, parallel frontend/Rust/model/native lanes, stable aggregate `required`, bounded timeouts, cancellation of superseded runs, release-only real-model cost, and no aggressive opaque CMake cache. Baseline full self-validation passed in run `33793098041`.
- [~] **PowerShell Latest-Release Installer** — `scripts/install.ps1`, README usage, release contract, SHA256 verification, optional `-Silent`, `-ResolveOnly`, and a dedicated fast CI distribution lane are implemented; exact-head CI evidence pending.
- [ ] **Alpha 2 qualification/release** — after merge from verified `master`, build real EN -> ID pack, execute real production inference smoke, build NSIS installer, emit hashes, and publish immutable `v0.1.0-alpha.2` only if release gates pass.

## Existing CI evidence — run 33793098041

The prior CI workflow change intentionally forced every existing lane to run once because the verification mechanism itself changed.

Passed:

1. `classify risk` — changed-file risk routing;
2. `frontend` — install, Svelte static check, component tests, production build;
3. `rust core` — format, production-target Clippy, unit/application tests;
4. `model contract` — EN -> ID model-pack dry-run without production weights;
5. `native boundary` — C++ configure/build, runtime probe, Rust -> C++ protocol regression, WorkerTranslator regression, real-production smoke harness compile;
6. `required` — aggregate accepted all applicable lane results.

The PowerShell bootstrap change adds one small distribution lane and therefore requires a new exact-head run before merge.

## Implementation decisions

- Rust remains application/workflow authority; Svelte remains presentation/control-surface code.
- System tray uses Tauri native tray support; no separate tray framework is introduced.
- Settings is a hidden Tauri window loaded with the same frontend bundle and selected by window label.
- Closing Settings is intercepted and hides the window instead of terminating the tray process.
- Popup remains frameless/always-on-top and uses a Tauri drag region instead of restoring decorative native chrome.
- No-selection is not represented as an error UI. Capture failures are logged without raw source content and cancel the current request invisibly.
- The active production worker loads only `stages/en-id`; the former JA -> EN stage is removed from the active catalog and pack completeness contract.
- Alpha 2 is the next prerelease because published Alpha 1 artifacts are immutable and must not be replaced.
- PR CI proves changed contracts/boundaries with the cheapest credible evidence; real production model inference and packaging remain release qualification concerns.
- The PowerShell bootstrap installs only published non-draft GitHub Release assets, including prereleases, and never installs unmerged branch code.
- Installer integrity is checked before execution using GitHub asset SHA256 metadata with the release `.sha256` asset as a fallback.
- The bootstrap does not bypass SmartScreen, code-signing, architecture, or checksum protections.

## Required evidence

### PR / integration

Use the risk-routed CI contract from `.agents/QUALITY.md`:

- **frontend lane** when frontend/tooling changes: Svelte check, component tests, production build;
- **Rust lane** when `app/src-tauri/**` changes: fmt, production-target Clippy, library/application tests including no-selection regression;
- **model-contract lane** when model catalog/build/pack contract changes: EN -> ID dry-run only, no production weights;
- **native boundary lane** only when worker/protocol/native boundary changes: C++ configure/build/runtime probe, deterministic Rust <-> C++ regressions, real-smoke harness compile;
- **distribution lane** when `scripts/install.ps1` changes: PowerShell parser + `-ResolveOnly` newest-release/checksum resolution, with no installer execution;
- **required aggregate**: all applicable lanes must succeed; irrelevant lanes may be skipped.

### Release qualification

- build pinned production `en-id-opus-v1` pack;
- validate required single-stage files and SHA256;
- execute real non-deterministic English -> Indonesian worker smoke;
- build tray-enabled Windows x64 NSIS installer;
- publish checksums and immutable prerelease only after prior steps pass.

### Native interaction acceptance

Automation does not fully prove Windows shell interaction. Before calling the native UX completely accepted, verify on Windows:

1. launch ClipLingo and observe its notification-area/system-tray icon;
2. left-click tray -> Settings opens;
3. close Settings -> ClipLingo remains in tray;
4. invoke `Ctrl+Alt+T` without selection -> nothing visible;
5. select `The deployment failed yesterday.` -> invoke shortcut -> Indonesian result popup;
6. drag popup by `EN → ID` header;
7. dismiss result without terminating ClipLingo;
8. tray menu Quit terminates the app.

Do not substitute this manual interaction evidence with source-inspection claims. Conversely, do not block independently automatable implementation/CI work while waiting for the manual interaction check.
