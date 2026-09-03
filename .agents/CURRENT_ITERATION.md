# Current Milestone — EN→ID Native Tray Translation Experience

**Status:** implemented, verified, and merged to `master` in commit `a0d30205dfacb4ed9f15f493a95a7acbd2945135`. PR #25 is closed and merged. The next remaining milestone action is Alpha 2 automated release qualification/publication; no merge blocker remains for this feature milestone.

**Goal:** make ClipLingo behave as a focused Windows background utility: run from the system tray, translate selected English directly to Indonesian offline, show nothing when no valid selection exists, present the result in a compact draggable utility popup, and provide a safe one-command Windows PowerShell installation route to the newest published release.

## Feature Compass

**Shape:** system tray -> select English text -> `Ctrl+Alt+T` -> validate/capture selection -> direct local OPUS `en -> id` -> compact draggable Indonesian popup. PowerShell installation resolves the newest published GitHub Release automatically. No selection means no visible translation UI.

**Position:** the complete EN -> ID/tray milestone is integrated on `master`: one-stage EN -> ID model/runtime, selection-before-popup ordering, silent no-selection regression, Tauri tray + Settings lifecycle, restrained draggable popup UI, risk-routed CI, and `scripts/install.ps1`.

**Delta:** source integration is complete. Only the separate `v0.1.0-alpha.2` release qualification/publication path remains for this milestone.

**Next Move:** run the gated Alpha 2 release workflow from verified `master`. Fix only concrete release failures; if qualification passes, publish immutable `v0.1.0-alpha.2` assets and checksums.

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

## Slice status

- [x] **Direct EN→ID Runtime** — merged; active production worker/model-pack contract is direct EN -> ID. Real production weights remain a release-qualification concern.
- [x] **Selection-Gated Interaction** — merged; capture occurs before popup visibility and regression proves no popup/translator call for no-selection.
- [x] **Windows Tray + Settings Shell** — merged and covered by Windows compilation/application evidence. Manual shell observation may be useful for debugging but is not a mandatory milestone gate.
- [x] **Movable Translation Surface + UI Quality** — merged; draggable Tauri surface and restrained UI contract are implemented and frontend verification is green.
- [x] **Fast Accurate CI Contract** — merged; classifier, parallel risk lanes, stable `required` aggregate, bounded timeouts, cancellation, and release-only real-model cost are active.
- [x] **PowerShell Latest-Release Installer** — merged; parser + `-ResolveOnly` distribution CI passed, SHA256 verification is enforced before installer execution, and the bootstrap resolves published releases only.
- [ ] **Alpha 2 qualification/release** — build real EN -> ID pack, execute real production inference smoke, build NSIS installer, emit hashes, and publish immutable `v0.1.0-alpha.2` only if automated release gates pass.

## Integration evidence

Exact-head PR CI run `33797782476` passed before merge:

1. `classify risk` — success;
2. `frontend` — Svelte static check, component tests, production build — success;
3. `rust core` — format, production-target Clippy, unit/application tests — success;
4. `model contract` — EN -> ID model-pack dry-run without production weights — success;
5. `native boundary` — C++ configure/build, native runtime probe, Rust -> C++ protocol regression, WorkerTranslator regression, real-production smoke harness compile — success;
6. `powershell installer` — PowerShell parser + newest-release/checksum `-ResolveOnly` contract — success;
7. `required` — aggregate success.

The milestone was then merged to `master` as `a0d30205dfacb4ed9f15f493a95a7acbd2945135`.

## Implementation decisions

- Rust remains application/workflow authority; Svelte remains presentation/control-surface code.
- System tray uses Tauri native tray support; no separate tray framework is introduced.
- Settings is a hidden Tauri window loaded from the same frontend bundle and selected by window label.
- Closing Settings hides the window instead of terminating the tray process.
- Popup remains frameless/always-on-top and uses a Tauri drag region instead of decorative native chrome.
- No-selection is not represented as error UI. Capture failures cancel invisibly and diagnostics must not contain raw source content.
- The active production worker loads only `stages/en-id`; the former JA -> EN stage is historical and not part of the active pack.
- Alpha 2 is the next prerelease because published Alpha 1 artifacts are immutable.
- PR CI proves changed contracts/boundaries with the cheapest credible evidence; real production model inference and packaging belong to release qualification.
- The PowerShell bootstrap installs only published non-draft GitHub Release assets, including prereleases, and never installs development code directly from `master`.
- Installer integrity is checked before execution using GitHub asset SHA256 metadata with the release `.sha256` asset as fallback.
- The bootstrap does not bypass SmartScreen, code-signing, architecture, or checksum protections.
- Manual Windows acceptance is not a mandatory merge, milestone, or release gate. Environment-specific behavior that automation cannot credibly prove is documented as residual risk.

## Verification contract

Use `.agents/QUALITY.md` as the canonical implementation/CI verification contract. Relevant risk lanes are:

- **frontend** for Svelte/tooling changes;
- **rust core** for `app/src-tauri/**` changes;
- **model contract** for model catalog/build/pack changes;
- **native boundary** for worker/protocol/native changes;
- **powershell installer** for `scripts/install.ps1` changes;
- **required** as the stable aggregate.

Documentation-only updates do not justify recompiling unrelated product lanes.

## Release qualification

Alpha 2 remains unpublished until automated release qualification succeeds:

- build pinned production `en-id-opus-v1` pack;
- validate required single-stage files and SHA256;
- build and execute the native worker runtime probe;
- execute real non-deterministic English -> Indonesian worker inference;
- build tray-enabled Windows x64 NSIS installer;
- compute installer/model checksums;
- publish immutable prerelease assets only after all prior gates pass.
