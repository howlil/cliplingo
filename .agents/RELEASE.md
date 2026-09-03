# Release Strategy

ClipLingo releases by product maturity. Release requirements are project-specific; ordinary alpha development must not inherit stable-grade packaging/signing gates.

Manual acceptance testing, black-box browser/native testing, and human visual-review gates are not required release criteria. Release confidence comes from deterministic repository tests, automated native/runtime probes, packaging checks, integrity validation, and CI appropriate to the changed release boundary.

## Channels

- **Development:** every verified `master` commit is integratable product state.
- **Alpha (`v0.x.y-alpha.N`):** core path is proven by the automated evidence required for the advertised alpha behavior; compatibility may remain narrow and documented.
- **Beta (`v0.x.y-beta.N`):** intended feature set exists; broaden compatibility, performance, reliability, update, and migration evidence.
- **RC (`v0.x.y-rc.N`):** production-shaped candidate; only release blockers change.
- **Stable (`v0.x.y`, later `v1.x.y`):** supported-platform, signing, install/update/uninstall, and regression guarantees are enforced through reproducible automated checks where repository-owned automation can observe them.

## Release rules

1. Release only from verified `master`.
2. Integrate at coherent feature/slice or logical-change boundaries. Do not accumulate completed value on long-lived branches, and do not split one logical outcome into branch-per-tweak delivery ceremony.
3. Tag immutable versions; never replace published binaries under the same tag.
4. A failed release becomes a bounded fix and a new patch/prerelease version.
5. Release automation/signing/package-manager work is implemented only when the current maturity stage requires it.
6. WinGet/Chocolatey are downstream distribution; they do not replace or rebuild the canonical GitHub Release asset.
7. A new platform requires an executable CI/runtime target before publication; do not substitute manual acceptance as the release gate.

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

The release is intentionally automated rather than created by a manual source-only tag:

1. merge the verified Real Offline Translation Alpha capability to `master`;
2. create `release/v0.1.0-alpha.1` from that exact verified `master` commit;
3. `.github/workflows/release-alpha.yml` builds the pinned `ja-id-opus-v1` production model pack;
4. validate required model-pack files and compute its SHA256;
5. build the source-pinned Windows C++ worker/runtime probe;
6. execute the bounded automated native runtime probe;
7. execute a real non-deterministic Japanese translation through the production OPUS pack;
8. stage `cliplingo-worker.exe` into the Tauri resource directory;
9. build the x64 NSIS installer with the immutable GitHub Release model URL and model SHA256 compiled into the model lifecycle;
10. compute the installer SHA256;
11. only then create the immutable GitHub prerelease/tag and upload installer, model pack, checksums, and release notes.

If any prerequisite fails, no release tag is created. Fix the concrete failure and rerun automated qualification rather than publishing an unqualified artifact.

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

## Release evidence by maturity

### Alpha

- required repository integration evidence is green for the advertised alpha behavior;
- the advertised alpha path is proven by the smallest credible automated/native evidence available;
- no known privacy/correctness blocker exists for the advertised behavior;
- a distributed alpha installer contains the runtime components required by the advertised path;
- a model-dependent alpha proves model acquisition/integrity and production inference through automated release/runtime probes.

### Beta

- broader representative compatibility through reproducible automated targets where available;
- latency/resource measurements where user experience depends on them;
- installer/update automation if beta is distributed as installed software;
- regression coverage for accumulated product behaviors.

### Stable

- reproducible installer/package build;
- automated launch/startup and primary contract probes available to the repository;
- upgrade/migration tests for supported prior versions when implemented;
- uninstall/data-retention contract verification where it can be automated;
- signature/checksum validation;
- updater/package metadata when enabled;
- release notes and rollback/fix-forward path.

Environment-specific limitations that cannot be automated are documented as residual risk; they do not create a manual acceptance gate.

## Rollback

Desktop releases are immutable. If a published version is defective, stop promoting it and publish a corrected version. Source changes are reverted or fixed through normal bounded logical changes; do not mutate history or published artifacts.
