# Release Strategy

ClipLingo releases by product maturity. Do not require stable-grade packaging/signing evidence to merge ordinary alpha development slices.

## Channels

- **Development:** every verified `master` commit is integratable product state.
- **Alpha (`v0.x.y-alpha.N`):** core path works; targeted manual/native smoke is acceptable; compatibility can be narrow and documented.
- **Beta (`v0.x.y-beta.N`):** intended feature set exists; broaden compatibility, performance, reliability, update and migration evidence.
- **RC (`v0.x.y-rc.N`):** production-shaped candidate; only release blockers change.
- **Stable (`v0.x.y`, later `v1.x.y`):** supported-platform, signing, install/update/uninstall and regression guarantees are enforced.

## Release rules

1. Release only from verified `master`.
2. Prefer small frequent prereleases over accumulating many finished features on branches.
3. Tag immutable versions; never replace published binaries under the same tag.
4. A failed release becomes a normal fix PR and a new patch/prerelease version.
5. Release automation/signing/package-manager work is implemented only when the current maturity stage needs it.
6. WinGet/Chocolatey are downstream distribution; they never block the canonical GitHub Release.
7. ARM64 or additional platforms require a real test target before publication.

## Canonical binary source

GitHub Releases is canonical. Planned Windows artifacts, when applicable:
- primary x64 installer;
- optional MSI only when there is a concrete consumer;
- updater artifact/signature when updater exists;
- SHA-256 checksums;
- concise release notes.

Do not create duplicate independently-built distribution channels.

## Signing

Windows code signing and Tauri updater signing are separate trust mechanisms. Private keys never enter the repository. Stable release requires the relevant signatures; alpha development does not wait for signing infrastructure unless signing itself is under test.

## Release acceptance by maturity

### Alpha
- mandatory CI green;
- application launches;
- changed core path smoke-tested on a representative supported Windows setup;
- no known privacy/correctness blocker for the advertised alpha behavior.

### Beta
- broader representative application/DPI/monitor compatibility;
- latency/resource measurements where user experience depends on them;
- installer/update path if beta is distributed as installed software;
- regression suite for accumulated product behaviors.

### Stable
- clean supported Windows VM fresh install;
- launch and primary workflow;
- upgrade from previous supported stable version;
- uninstall/data-retention behavior;
- signature/checksum validation;
- updater/package metadata when enabled;
- release notes and rollback/fix-forward path.

## Rollback

Desktop releases are immutable. If a published version is defective, stop promoting it and publish a corrected version. Source changes are reverted or fixed through normal small PRs; do not mutate history or published artifacts.
