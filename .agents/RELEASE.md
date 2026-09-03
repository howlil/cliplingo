# Release Strategy

ClipLingo releases by product maturity. Release requirements are project-specific; ordinary alpha development must not inherit stable-grade packaging/signing gates.

Manual acceptance testing, black-box browser/native testing, and human visual-review gates are not required release criteria. Release confidence comes from deterministic repository tests, automated native/runtime probes, packaging checks, integrity validation, and CI appropriate to the changed release boundary.

## Channels

- **Development:** every verified `master` commit is integratable product state.
- **Alpha (`v0.x.y-alpha.N`):** core path is proven by automated evidence required for the advertised alpha behavior; compatibility may remain narrow and documented.
- **Beta (`v0.x.y-beta.N`):** intended feature set exists; broaden compatibility, performance, reliability, update, and migration evidence.
- **RC (`v0.x.y-rc.N`):** production-shaped candidate; only release blockers change.
- **Stable (`v0.x.y`, later `v1.x.y`):** supported-platform, signing, install/update/uninstall, and regression guarantees are enforced through reproducible automated checks where repository-owned automation can observe them.

## Release rules

1. Release only from verified `master`.
2. Integrate at coherent feature/slice or logical-change boundaries. Do not accumulate completed value on long-lived branches or split one outcome into branch-per-tweak ceremony.
3. Published tags/binaries are immutable; never replace an existing release with different behavior.
4. A failed release becomes a bounded fix and a new patch/prerelease version.
5. GitHub Releases is the canonical binary/model source; package managers and convenience installers reference those exact assets and do not rebuild them.
6. Release automation/signing/package-manager work is implemented only when current maturity requires it.
7. A new platform requires a real executable CI/runtime target before publication; do not substitute manual acceptance as the release gate.

## Canonical binary source

GitHub Releases is canonical. Alpha Windows artifacts include:

- x64 NSIS installer;
- versioned offline model pack required by the advertised route;
- SHA-256 checksum sidecars;
- concise release notes.

Do not create duplicate independently-built distribution channels.

## PowerShell bootstrap installer

`scripts/install.ps1` is the canonical convenience installer for Windows PowerShell 5.1 and PowerShell 7. It does not build, mirror, or redefine release artifacts.

Its contract is:

1. query `howlil/cliplingo` GitHub Releases;
2. choose the newest published, non-draft release by publication time, including prereleases;
3. require exactly one `ClipLingo_*_x64-setup.exe` asset;
4. obtain expected SHA256 from GitHub release asset `digest`, falling back only to the matching `.sha256` release asset;
5. download the installer from its canonical GitHub Release URL;
6. verify the downloaded SHA256 before execution;
7. run the installer normally, or with NSIS `/S` only when the caller explicitly supplies `-Silent`;
8. never bypass SmartScreen, code-signing, checksum, or architecture checks.

The stable bootstrap command points at `master/scripts/install.ps1`; the script resolves the latest published release dynamically so users do not need a version-specific command.

CI validates PowerShell syntax and `-ResolveOnly` release/checksum resolution when this bootstrap changes. `-ResolveOnly` must not download or execute the installer.

## Current release candidate — Alpha 2

The EN→ID Native Tray Translation Experience is integrated and verified on `master` at commit `a0d30205dfacb4ed9f15f493a95a7acbd2945135`. PR #25 is merged. The remaining work is release qualification/publication, not feature integration.

`v0.1.0-alpha.2` is **not published yet**. `v0.1.0-alpha.1` remains the newest published release until the Alpha 2 workflow completes successfully.

The automated gated route is now:

1. start from a verified `master` commit containing the EN -> ID/tray capability;
2. create `release/v0.1.0-alpha.2` from that exact verified commit;
3. `.github/workflows/release-alpha.yml` builds pinned `en-id-opus-v1`;
4. validate required EN -> ID model files and compute model SHA256;
5. build and execute the source-pinned native worker runtime probe;
6. execute a real non-deterministic English -> Indonesian worker translation using the production pack;
7. stage `cliplingo-worker.exe` into the Tauri resource directory;
8. build the tray-enabled x64 NSIS installer with immutable model URL/SHA compiled into model lifecycle;
9. compute installer SHA256;
10. only then publish `v0.1.0-alpha.2` with installer, model pack, checksums, and release notes.

If any prerequisite fails, no Alpha 2 tag is created. Fix the concrete failure and rerun automated qualification rather than publishing an unqualified artifact.

### Alpha 2 planned canonical assets

- tag: `v0.1.0-alpha.2`;
- installer: `ClipLingo_0.1.0-alpha.2_x64-setup.exe`;
- model pack: `cliplingo-en-id-opus-v1.zip`;
- release notes: `docs/releases/v0.1.0-alpha.2.md`.

## Published Alpha 1

`v0.1.0-alpha.1` remains historical and immutable. It used the earlier JA -> EN -> ID pack. Documentation may correct factual mistakes, but published binaries/hashes are never replaced.

## WinGet downstream rule

WinGet submission occurs only after the corresponding canonical GitHub Release exists. Every WinGet version references the exact release installer URL and SHA256; it must not rebuild or mirror ClipLingo.

Package identity remains `Howlil.ClipLingo` unless product authority changes publisher identity. Because the application repository does not currently declare a public application license, package metadata must not invent one. Catalog availability is not claimed until the upstream `microsoft/winget-pkgs` PR is validated and merged.

The PowerShell bootstrap is independent from WinGet catalog publication and may install the latest verified GitHub Release before a WinGet manifest is merged.

## Signing

Windows code signing and Tauri updater signing are separate trust mechanisms. Private keys never enter the repository. Stable release requires the relevant signatures; alpha development does not wait for signing infrastructure unless signing itself is under test.

An unsigned alpha may trigger Windows SmartScreen. Document that fact rather than weakening Windows security behavior.

## Release evidence by maturity

### Alpha

- required repository integration evidence is green for the advertised behavior;
- the advertised path is proven by the smallest credible deterministic repository/native evidence available;
- a model-dependent alpha proves model acquisition/integrity and real production inference through automated release/runtime probes;
- a distributed installer contains runtime components required by the advertised path;
- no known privacy/correctness blocker exists for the advertised behavior.

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

Environment-specific limitations that cannot be automated are documented as residual risk; they do not create a mandatory manual acceptance gate.

## Rollback

Desktop releases are immutable. If a published version is defective, stop promoting it and publish a corrected version. Source changes are reverted or fixed through normal bounded logical changes; do not mutate history or published artifacts.
