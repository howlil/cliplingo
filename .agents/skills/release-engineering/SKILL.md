---
name: release-engineering
description: Use for GitHub Actions, versioning, tags, signing, installers, updater, GitHub Releases, WinGet, Chocolatey, and release verification.
---

# Release Engineering Skill

## Source of truth

Stable binaries are built once in the release pipeline and published to GitHub Releases. Downstream channels reference those canonical assets.

## Version/release rules

- SemVer with `v` Git tags.
- Tags only from verified `master`.
- Stable published binaries are immutable; fix with a new patch version.
- Tauri app version is canonical in `tauri.conf.json` once scaffolded.
- Separate prerelease and stable update channels.

## Windows artifacts

Primary: signed NSIS setup `.exe`.
Optional: `.msi` when needed.
Also: updater signatures/artifacts, SHA-256 checksums, release notes.

## Distribution order

1. GitHub Release.
2. Verify clean install/upgrade/uninstall.
3. WinGet manifest submission.
4. Chocolatey package after its automation is worth maintaining.
5. Microsoft Store or additional channels only after demand.

## Package rules

WinGet/Chocolatey must point to publisher-owned stable assets and verify expected installer/hash behavior. Do not rebuild a different binary for each channel.

## CI safety

Signing secrets remain in secure CI storage. Release jobs should use protected environments/permissions and minimal tokens. A normal PR cannot obtain production signing secrets.

## References

- https://v2.tauri.app/distribute/
- https://v2.tauri.app/distribute/windows-installer/
- https://v2.tauri.app/plugin/updater/
- https://learn.microsoft.com/windows/package-manager/package/
- https://docs.chocolatey.org/en-us/create/
