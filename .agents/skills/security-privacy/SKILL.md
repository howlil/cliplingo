---
name: security-privacy
description: Use for Tauri capabilities, selected-text privacy, logs, model/update supply chain, signing, file/process access, and secrets.
---

# Security and Privacy Skill

## Privacy invariant

Selected source text and translation content stay on-device in the normal translation path and are not written to normal logs/telemetry.

Diagnostics may record metadata such as character count, language, capture strategy, route, duration and status.

## Tauri/WebView

- Minimum required capabilities only.
- Expose narrow typed commands.
- Never expose arbitrary shell execution, arbitrary file read/write, or generic process spawn to Svelte.
- Treat all WebView input as untrusted transport input and validate it at the native boundary.

## Model/update supply chain

- Download only from configured trusted origins.
- Verify hashes and manifest compatibility before activation.
- Track model licenses/redistribution rights.
- Updates require Tauri updater signatures when that updater is enabled.
- Stable Windows binaries should be code-signed before broad public distribution.

## Secrets

Signing private keys, tokens and credentials belong in secure CI secret/key systems. Never commit them, print them in logs, or embed them in frontend bundles.

## Failure behavior

Verification failure must fail closed: do not run an unverified worker/model/update artifact.
