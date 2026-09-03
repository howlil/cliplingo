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
2. Integrate at coherent feature/slice or logical-change boundaries.
3. Published tags/binaries are immutable; never replace Alpha 1 artifacts with Alpha 2 behavior.
4. A failed release becomes a bounded fix and a new prerelease version.
5. GitHub Releases is the canonical binary/model source; package managers and convenience installers reference those exact assets.
6. ARM64 or another platform requires a real test target before publication.

## Canonical binary source

GitHub Releases is canonical. Alpha Windows artifacts include:

- x64 NSIS installer;
- versioned offline model pack required by the advertised route;
- SHA-256 checksum sidecars;
- concise release notes.

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

The stable bootstrap command points at `master/scripts/install.ps1`; the script itself resolves the latest published release dynamically, so users do not need a version-specific command.

CI must validate PowerShell syntax and `-ResolveOnly` release/checksum resolution when this bootstrap changes. `-ResolveOnly` must not download or execute the installer.

## Current release candidate — Alpha 2

The EN→ID Native Tray Translation Experience qualifies as `v0.1.0-alpha.2` because `v0.1.0-alpha.1` is already published and immutable.

The gated route is:

1. merge the verified EN→ID/tray capability to `master`;
2. create `release/v0.1.0-alpha.2` from that exact verified commit;
3. `.github/workflows/release-alpha.yml` builds pinned `en-id-opus-v1`;
4. validate required EN -> ID model files and compute model SHA256;
5. build/execute the source-pinned native worker runtime probe;
6. execute a real non-deterministic English -> Indonesian worker translation using the production pack;
7. stage `cliplingo-worker.exe` into the Tauri resource directory;
8. build the tray-enabled x64 NSIS installer with the immutable model URL/SHA compiled into the model lifecycle;
9. compute installer SHA256;
10. only then publish `v0.1.0-alpha.2` with installer, model pack, checksums, and release notes.

If any prerequisite fails, no Alpha 2 tag is created. Fix the concrete failure and rerun qualification.

### Alpha 2 planned canonical assets

- tag: `v0.1.0-alpha.2`;
- installer: `ClipLingo_0.1.0-alpha.2_x64-setup.exe`;
- model pack: `cliplingo-en-id-opus-v1.zip`;
- release notes: `docs/releases/v0.1.0-alpha.2.md`.

## Published Alpha 1

`v0.1.0-alpha.1` remains historical and immutable. It used the earlier JA -> EN -> ID pack. Documentation may be corrected for factual mistakes, but published binaries/hashes are never replaced.

## WinGet downstream rule

WinGet submission occurs only after the corresponding canonical GitHub Release exists. Every WinGet version must reference the exact release installer URL and SHA256 for that version; it must not rebuild or mirror ClipLingo.

Package identity remains `Howlil.ClipLingo` unless product authority changes publisher identity. Because the application repository does not currently declare a public application license, package metadata must not invent one. Catalog availability is not claimed until the upstream `microsoft/winget-pkgs` PR is validated and merged.

The PowerShell bootstrap is independent from WinGet catalog publication and may install the latest verified GitHub Release before a WinGet manifest is merged.

## Signing

Windows code signing and Tauri updater signing are separate trust mechanisms. Private keys never enter the repository. Stable release requires the relevant signatures; alpha development does not wait for signing infrastructure unless signing itself is under test.

An unsigned alpha may trigger Windows SmartScreen. Document that fact rather than weakening Windows security behavior.

## Release acceptance by maturity

### Alpha

- repository integration evidence is green for the advertised behavior;
- model-dependent alpha proves model acquisition/integrity and real production inference before publication;
- distributed installer contains required runtime components;
- no known privacy/correctness blocker exists;
- native Windows interaction is checked when automation cannot credibly prove tray/window behavior.

### Beta

- broader representative application/DPI/monitor compatibility;
- latency/resource measurements where UX depends on them;
- installer/update path if distributed;
- accumulated regression coverage.

### Stable

- clean supported Windows VM fresh install;
- launch and primary workflow;
- upgrade/uninstall/data-retention behavior;
- signature/checksum validation;
- package/update metadata;
- release notes and rollback/fix-forward path.

## Rollback

Desktop releases are immutable. If a published version is defective, stop promoting it and publish a corrected version. Do not mutate published artifacts or history.
