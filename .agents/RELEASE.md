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

## Current alpha release route

The first requested distributable release is `v0.1.0-alpha.1`.

The release is intentionally gated rather than created by a manual source-only tag:

1. merge the verified Real Offline Translation Alpha capability to `master`;
2. create `release/v0.1.0-alpha.1` from that exact verified `master` commit;
3. `.github/workflows/release-alpha.yml` builds the pinned `ja-id-opus-v1` production model pack;
4. validate required model-pack files and compute its SHA256;
5. build the source-pinned Windows C++ worker/runtime probe;
6. execute the bounded native runtime probe;
7. execute a real non-deterministic Japanese translation through the production OPUS pack;
8. stage `cliplingo-worker.exe` into the Tauri resource directory;
9. build the x64 NSIS installer with the immutable GitHub Release model URL and model SHA256 compiled into the model lifecycle;
10. compute the installer SHA256;
11. only then create the immutable GitHub prerelease/tag and upload installer, model pack, checksums, and release notes.

If any prerequisite fails, no release tag is created. Fix the concrete failure and rerun qualification rather than publishing an unqualified artifact.

### Alpha 1 canonical assets

- tag: `v0.1.0-alpha.1`;
- installer: `ClipLingo_0.1.0-alpha.1_x64-setup.exe`;
- model pack: `cliplingo-ja-id-opus-v1.zip`;
- checksum sidecars for both assets;
- release notes: `docs/releases/v0.1.0-alpha.1.md`.

## WinGet downstream rule

WinGet submission occurs only after the canonical GitHub Release exists.

For ClipLingo Alpha 1:

- WinGet must reference the exact GitHub Release NSIS installer URL;
- `InstallerSha256` must be the SHA256 emitted by the release workflow for that exact binary;
- the package manager must not rebuild, mirror, or substitute the installer;
- package identity is `Howlil.ClipLingo` unless a future product authority decision changes publisher identity;
- because the ClipLingo application repository currently declares no public application license, WinGet metadata must not invent one; use the package-manager-appropriate proprietary/undeclared license representation until an application license is explicitly adopted;
- model licenses remain governed by the model-pack manifest and are not rewritten as the application license;
- submission completion means an upstream `microsoft/winget-pkgs` PR exists. Catalog availability is not claimed until upstream validation and merge succeed.

## Signing

Windows code signing and Tauri updater signing are separate trust mechanisms. Private keys never enter the repository. Stable release requires the relevant signatures; alpha development does not wait for signing infrastructure unless signing itself is under test.

The first alpha may therefore be unsigned and can trigger Windows SmartScreen. This must be documented, not bypassed by weakening Windows security behavior.

## Release acceptance by maturity

### Alpha

- required repository integration evidence is green for the advertised alpha behavior;
- the advertised alpha path is proven by the smallest credible automated/native evidence available;
- no known privacy/correctness blocker exists for the advertised behavior;
- a distributed alpha installer must contain the runtime components required by the advertised path;
- a model-dependent alpha must prove model acquisition/integrity and a real production inference smoke before publication;
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
