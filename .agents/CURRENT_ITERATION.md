# Current Milestone — EN→ID Native Tray Translation Experience

**Status:** implementation is active on branch `feat/en-id-native-tray-experience`; all four product slices are implemented in source and awaiting Windows CI/release qualification. Do not report the milestone complete until the coherent branch passes proportional automated gates and the native interaction acceptance is checked where automation cannot prove it.

**Goal:** make ClipLingo behave as a focused Windows background utility: run from the system tray, translate selected English directly to Indonesian offline, show nothing when no valid selection exists, and present the result in a compact draggable utility popup.

## Feature Compass

**Shape:** system tray -> select English text -> `Ctrl+Alt+T` -> validate/capture selection -> direct local OPUS `en -> id` -> compact draggable Indonesian popup. No selection means no visible UI. Tray Settings exposes route, shortcut, model lifecycle, running status, and explicit Quit.

**Position:** the capability branch has replaced the two-stage JA -> EN -> ID active route with a one-stage EN -> ID model pack/runtime, changed coordinator ordering so capture precedes popup visibility, added a silent no-selection cancellation path and regression test, added Tauri system tray + Settings lifecycle, and redesigned the popup as a restrained draggable Windows utility surface.

**Delta:** prove frontend/Rust/native compilation and regressions on Windows, fix only evidence-backed failures, merge the coherent capability, then run the gated `v0.1.0-alpha.2` release route to build the real EN -> ID model pack, execute production inference smoke, and package the tray-enabled installer.

**Next Move:** complete CI on the latest branch head. Do not add more product scope while qualification is running.

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
- broad settings dashboard.

## Slices

- [~] **Direct EN→ID Runtime** — source implementation complete: catalog/model lifecycle/worker/smoke/CI dry-run now target `en-id-opus-v1`; Windows qualification pending.
- [~] **Selection-Gated Interaction** — source implementation complete: capture occurs before `popup.show`; capture failures cancel invisibly; explicit regression asserts no popup and zero translator calls for `NoSelection`; CI pending.
- [~] **Windows Tray + Settings Shell** — source implementation complete: native Tauri tray, left-click Settings, tray Settings/Quit menu, close-to-tray Settings behavior, and minimal model controls; Windows compile/interaction evidence pending.
- [~] **Movable Translation Surface + UI Quality** — source implementation complete: `EN → ID` drag region, restrained utility styling, source secondary/result primary, no decorative blur/gradient/bento treatment; frontend/Windows evidence pending.
- [ ] **Alpha 2 qualification/release** — after merge from verified `master`, build real EN -> ID pack, execute real production inference smoke, build NSIS installer, emit hashes, and publish immutable `v0.1.0-alpha.2` only if all gates pass.

## Implementation decisions

- Rust remains application/workflow authority; Svelte remains presentation/control-surface code.
- System tray is implemented with Tauri native tray support; no separate tray framework is introduced.
- Settings is a hidden Tauri window loaded with the same frontend bundle and selected by window label.
- Closing Settings is intercepted and hides the window instead of terminating the tray process.
- Popup remains frameless/always-on-top and uses a Tauri drag region instead of restoring decorative native chrome.
- No-selection is not represented as an error UI. Capture failures are logged without raw source content and cancel the current request invisibly.
- The active production worker loads only `stages/en-id`; the former JA -> EN stage is removed from the active catalog and pack completeness contract.
- Alpha 2 is the next prerelease because published Alpha 1 artifacts are immutable and must not be replaced.

## Required evidence

### Automated integration

- frontend Svelte check;
- frontend component tests;
- frontend build;
- EN -> ID model catalog dry-run;
- Rust formatting/Clippy/tests;
- explicit no-selection popup/translator regression;
- native worker configure/build/runtime probe;
- deterministic Rust <-> C++ protocol regressions.

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

Do not substitute this manual interaction evidence with claims from source inspection. Conversely, do not block implementation/CI work that is independently automatable while waiting for that manual interaction check.
