# Release Strategy

ClipLingo releases by product maturity. Release requirements are project-specific; ordinary alpha development must not inherit stable-grade packaging/signing/manual gates.

## Channels

- **Development:** every verified `master` commit is integratable product state.
- **Alpha (`v0.x.y-alpha.N`):** core path is proven by the automated evidence required for the advertised alpha behavior; compatibility may remain narrow and documented.
- **Beta (`v0.x.y-beta.N`):** intended feature set exists; broaden compatibility, performance, reliability, update, and migration evidence.
- **RC (`v0.x.y-rc.N`):** production-shaped candidate; only release blockers change.
- **Stable (`v0.x.y`, later `v1.x.y`):** supported-platform, signing, install/update/uninstall, and regression guarantees are enforced.

## Release rules

1. Release only from verified `master`.
2. Integrate at coherent feature/slice or logical-change boundaries. Do not accumulate completed value on long-lived branches, and do not split one logical outcome into branch-per-tweak delivery ceremony.
3. Tag immutable versions; never replace published binaries under the same tag.
4. A failed release becomes a bounded fix and a new patch/prerelease version.
5. Release automation/signing/package-manager work is implemented only when the current maturity stage requires it.
6. WinGet/Chocolatey are downstream distribution; they do not replace or rebuild the canonical GitHub Release asset.
7. ARM64 or another platform requires a real test target before publication.

## Canonical binary source

GitHub Releases is canonical. Planned Windows artifacts, when applicable:

- primary x64 installer;
- optional MSI only for a concrete consumer;
- updater artifact/signature when updater exists;
- SHA-256 checksums;
- concise release notes.

Do not create duplicate independently-built distribution channels.

## Signing

Windows code signing and Tauri updater signing are separate trust mechanisms. Private keys never enter the repository. Stable release requires the relevant signatures; alpha development does not wait for signing infrastructure unless signing itself is under test.

## Release acceptance by maturity

### Alpha

- required repository integration evidence is green for the advertised alpha behavior;
- the advertised alpha path is proven by the smallest credible automated/native evidence available;
- no known privacy/correctness blocker exists for the advertised behavior;
- manual Windows interaction is not a default blocker and is required only when the release/slice explicitly declares it necessary because the behavior cannot be credibly verified otherwise.

### Beta

- broader representative application/DPI/monitor compatibility;
- latency/resource measurements where user experience depends on them;
- installer/update path if beta is distributed as installed software;
- regression coverage for accumulated product behaviors;
- targeted manual/native evidence only for behavior that remains impractical to automate.

### Stable

- clean supported Windows VM fresh install;
- launch and primary workflow;
- upgrade from previous supported stable version;
- uninstall/data-retention behavior;
- signature/checksum validation;
- updater/package metadata when enabled;
- release notes and rollback/fix-forward path;
- any required manual release acceptance must be explicit in the release work, not inherited as a generic development gate.

## Rollback

Desktop releases are immutable. If a published version is defective, stop promoting it and publish a corrected version. Source changes are reverted or fixed through normal bounded logical changes; do not mutate history or published artifacts.