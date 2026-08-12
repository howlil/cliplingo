# Release and Distribution Plan

This is the target release process. Implement only the parts required by the current iteration.

## Versioning

Use Semantic Versioning and Git tags prefixed with `v`.

During early development:

- `0.x.y` for public development releases.
- `-alpha.N` for unstable internal/public experiments.
- `-beta.N` when the intended feature set exists but compatibility/quality is still being validated.
- `-rc.N` for production-shaped release candidates.
- Stable: `v0.x.y`, later `v1.0.0` when compatibility expectations are deliberately accepted.

When Tauri is scaffolded, keep the application version in `tauri.conf.json` as the canonical app version and validate any package metadata against it in CI.

## Canonical release source

**GitHub Releases is the canonical binary source.** Website/package-manager entries should reference immutable stable release assets rather than independently built binaries.

Planned Windows artifacts:

1. `ClipLingo_<version>_x64-setup.exe` — primary NSIS installer.
2. `ClipLingo_<version>_x64.msi` — optional MSI for users/enterprise tooling that need it.
3. Tauri updater artifacts/signatures.
4. `SHA256SUMS.txt`.
5. Release notes.

Add ARM64 only after Windows x64 is stable and ARM64 has a real test target. Do not publish architectures we cannot test.

## Signing

There are two separate trust concerns:

- **Windows code signing** for executable/installer publisher trust and SmartScreen reputation.
- **Tauri updater signing** for cryptographic update authenticity.

Private signing material must live only in a secure CI secret/key service, never the repository. Losing updater private keys can break the ability to update installed clients, so key backup/rotation planning is a release prerequisite.

## Release pipeline

Stable release flow:

```text
verified master
 -> choose semver
 -> update version + changelog/release notes
 -> PR + CI
 -> squash merge
 -> create signed tag vX.Y.Z
 -> Windows release build
 -> tests/smoke install
 -> code-sign artifacts
 -> generate updater artifacts/signatures
 -> generate SHA-256 checksums
 -> publish GitHub Release
 -> verify clean-machine install/update/uninstall
 -> update WinGet manifest
 -> update Chocolatey package when supported
```

A failed release is fixed on a normal task branch; do not mutate an already-published stable binary under the same version. Publish a new patch version.

## Direct download

The project website, if/when added, should link to the current stable GitHub Release artifact or a controlled redirect to it. Do not create a second untracked distribution source.

## Command-line install

After stable signing and release automation exist, provide a small versioned PowerShell installer script that downloads the canonical GitHub Release asset and verifies SHA-256 before execution.

Prefer a debuggable flow such as:

```powershell
curl.exe -fsSL <official-install-script-url> -o install-cliplingo.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-cliplingo.ps1
```

Do not make blind `curl | iex` the only documented path. The script must fail closed on checksum/download errors.

## WinGet

Stable public releases should be submitted to the Windows Package Manager Community Repository after the stable installer URL is immutable and tested. Use `wingetcreate` or an equivalent manifest workflow, validate the manifest, and point it to the publisher-owned canonical release asset.

Do not make WinGet submission block the GitHub Release; it is a downstream distribution step.

## Chocolatey

Chocolatey is a secondary channel. Add it after direct installer/release automation is stable. The package should download the publisher-owned stable installer, verify checksums, support silent installation/uninstallation, and be tested in a clean Windows VM before publication.

Do not maintain a separately built binary inside Chocolatey when the canonical installer is sufficient.

## Tauri updater

Use the Tauri updater only for stable/prerelease channels that have an explicit update feed. Update artifacts must be signed and the private updater key must be protected in CI.

Keep update channels separated so stable users do not accidentally receive prereleases.

## Release acceptance

At minimum verify on a clean supported Windows VM:

- fresh install
- launch and tray startup behavior
- normal hotkey path
- model download/install if applicable
- upgrade from previous supported release
- uninstall
- no unexpected user data/model deletion unless documented
- signature/checksum validity

Package-manager publication happens only after the canonical installer passes this acceptance.
