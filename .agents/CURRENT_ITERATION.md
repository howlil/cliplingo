# Current Milestone — EN→ID Native Tray Translation Experience

**Status:** implementation is active on branch `feat/en-id-native-tray-experience`. All four product slices are implemented in source. The CI/testing contract is being tightened to the canonical proportional, risk-routed workflow before merge. Do not report the milestone complete until the coherent branch passes its applicable automated gates and the explicitly required native interaction acceptance is checked where automation cannot credibly prove it.

**Goal:** make ClipLingo behave as a focused Windows background utility: run from the system tray, translate selected English directly to Indonesian offline, show nothing when no valid selection exists, and present the result in a compact draggable utility popup.

## Feature Compass

**Shape:** system tray -> select English text -> `Ctrl+Alt+T` -> validate/capture selection -> direct local OPUS `en -> id` -> compact draggable Indonesian popup. No selection means no visible UI. Tray Settings exposes route, shortcut, model lifecycle, running status, and explicit Quit.

**Position:** the capability branch has replaced the two-stage JA -> EN -> ID active route with a one-stage EN -> ID model pack/runtime, changed coordinator ordering so capture precedes popup visibility, added a silent no-selection cancellation regression, added Tauri system tray + Settings lifecycle, and redesigned the popup as a restrained draggable Windows utility surface. The repository CI is now being changed from one serial full Windows ladder into parallel risk lanes so frontend, Rust, model-contract, and native evidence run only when their boundary changes. `AGENTS.md` and `.agents/QUALITY.md` are aligned to that operating rule and to `DESIGN.md` as the canonical UI quality contract.

**Delta:** validate the new CI workflow itself with all lanes enabled once, fix only evidence-backed failures, merge the coherent capability, then run gated `v0.1.0-alpha.2` release qualification for the real EN -> ID model pack, production inference smoke, and installer.

**Next Move:** complete CI self-validation on the latest branch head. Do not add product scope while qualification is running.

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
- test-framework expansion or E2E ceremony unrelated to changed risk.

## Slices

- [~] **Direct EN→ID Runtime** — source implementation complete: catalog/model lifecycle/worker/smoke now target `en-id-opus-v1`; final applicable CI/release qualification pending.
- [~] **Selection-Gated Interaction** — source implementation complete: capture occurs before `popup.show`; capture failures cancel invisibly; explicit regression asserts no popup and zero translator calls for `NoSelection`; final branch qualification pending.
- [~] **Windows Tray + Settings Shell** — source implementation complete: native Tauri tray, left-click Settings, tray Settings/Quit menu, close-to-tray Settings behavior, and minimal model controls; native interaction acceptance remains.
- [~] **Movable Translation Surface + UI Quality** — source implementation complete: `EN → ID` drag region, restrained utility styling, source secondary/result primary, no decorative blur/gradient/bento treatment; final branch qualification pending.
- [~] **Fast Accurate CI Contract** — implementation added: changed-risk classifier, parallel frontend/Rust/model/native lanes, stable aggregate `required` check, bounded timeouts, cancellation of superseded runs, docs-only skip behavior, and release-only real-model cost. Workflow self-validation pending.
- [ ] **Alpha 2 qualification/release** — after merge from verified `master`, build real EN -> ID pack, execute real production inference smoke, build NSIS installer, emit hashes, and publish immutable `v0.1.0-alpha.2` only if release gates pass.

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
- CI workflow changes intentionally exercise all CI lanes once. Ordinary docs-only changes do not recompile the product.

## Required evidence

### PR / integration

Use the risk-routed CI contract from `.agents/QUALITY.md`:

- **frontend lane** when frontend/tooling changes: Svelte check, component tests, production build;
- **Rust lane** when `app/src-tauri/**` changes: fmt, production-target Clippy, library/application tests including no-selection regression;
- **model-contract lane** when model catalog/build/pack contract changes: EN -> ID dry-run only, no production weights;
- **native boundary lane** only when worker/protocol/native boundary changes: C++ configure/build/runtime probe, deterministic Rust <-> C++ regressions, real-smoke harness compile;
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
