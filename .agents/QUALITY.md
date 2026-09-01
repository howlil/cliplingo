# Quality

`QUALITY.md` is the canonical repository-specific verification contract. `.github/workflows/ci.yml` is the executable CI truth; this file explains which risks the checks protect and which evidence matters.

## Product invariants

Changes must preserve these unless the user explicitly approves a material boundary change:

- Normal translation is offline.
- Selected and translated content is not emitted to normal logs/telemetry.
- Worker failure does not terminate the shell.
- Interactive popup results obey latest-request-wins semantics.
- Worker responses are request-ID correlated and protocol-bounded.
- Clipboard fallback preserves/restores user clipboard state within the supported safety policy.
- Model/update artifacts are verified before activation and carry license/compatibility metadata.

## Fast targeted checks

Run only the checks relevant to the touched surface during implementation, then satisfy required pre-merge CI.

### Frontend

From `app/`:

```text
npm run check
npm test
npm run build
```

### Rust

From `app/`:

```text
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml --all
```

### Model-pack contract

```text
python ../scripts/models/build_opus_pack.py --catalog ../models/catalog/ja-id-opus-v1.json --output ../models/build/ja-id-opus-v1 --dry-run
```

This validates catalog/build intent without making ordinary CI depend on production model-weight downloads.

### C++ worker/native runtime

When the worker/native dependency boundary changes:

```text
cmake -S ../worker -B ../worker/build -A x64
cmake --build ../worker/build --config Release
```

Run the corresponding native runtime probe/integration tests defined by the current workflow/change.

## Required integration evidence

The repository has Windows integration coverage for:

- Rust -> C++ worker protocol/Named Pipe round-trip (`worker_cpp_integration`).
- `WorkerTranslator` spawning/using the worker (`worker_translator_integration`).

When a change crosses those boundaries, keep these regressions green and add only the smallest new executable evidence needed for the changed behavior.

## Current CI contract

On `master`, Windows CI currently performs:

1. frontend dependency install/check/test/build;
2. model-pack dry-run validation;
3. Rust format/Clippy/tests;
4. C++ worker configure/build;
5. Rust-to-C++ worker integration;
6. worker-backed translator integration.

Normal PR CI does not build or upload a manual acceptance package. A release or bounded native slice may add focused evidence when a behavior genuinely cannot be verified by the normal automated gates.

## Native/manual evidence

Manual Windows interaction is valid evidence when a real native behavior cannot be automated. It is **currently deferred and non-blocking** for active alpha development unless the user or the active slice explicitly makes it a gate.

Do not substitute inspection claims for a native/manual gate when such a gate is explicitly required.

## Performance evidence

Performance conclusions require release-mode measurement of the user path and enough environment context to reproduce the result. Relevant ClipLingo metrics include:

- hotkey -> popup latency;
- selection-capture latency;
- warm/cold inference latency;
- worker startup/model-load time;
- idle CPU;
- working-set/peak memory.

Do not turn unmeasured targets in `PROJECT.md` into claims or merge blockers for unrelated alpha slices.

## Security/supply-chain evidence

For model/update/release acquisition paths:

- validate configured/trusted origin where applicable;
- verify checksum/signature/manifest compatibility before activation;
- preserve license/redistribution metadata;
- fail closed on verification failure;
- keep signing credentials outside repository/frontend/logs.

## Evidence discipline

Never claim a test, CI run, benchmark, package, release, deployment, or manual acceptance that was not actually observed. A valid failing regression is a defect signal; do not weaken the assertion solely to make CI green.

Release-specific gates are owned by `RELEASE.md`.
