# Quality

`QUALITY.md` is the canonical repository-specific verification contract. `.github/workflows/ci.yml` is the executable PR/integration CI truth; `.agents/RELEASE.md` owns release qualification.

The goal is not maximum test count. The goal is **fast, accurate evidence proportional to changed risk**.

## Product invariants

Preserve these unless the user explicitly approves a material boundary change:

- Normal translation is offline.
- Selected and translated content is not emitted to normal logs/telemetry.
- Worker failure does not terminate the shell.
- Interactive popup results obey latest-request-wins semantics.
- Worker responses are request-ID correlated and protocol-bounded.
- Clipboard fallback preserves/restores user clipboard state within the supported safety policy.
- Model/update artifacts are verified before activation and carry license/compatibility metadata.
- No valid selection means no translation popup and no translator invocation.
- PowerShell installation executes only a published ClipLingo GitHub Release installer after SHA256 verification.

## Verification principle

Use the cheapest, fastest, highest-signal **automated** verification that can actually prove the changed behavior or boundary. Escalate only when a cheaper level leaves material risk invisible and another deterministic repository-owned check can observe it.

Do **not** automatically run a fixed `static -> unit -> integration -> E2E -> manual` ladder. Browser black-box testing, native manual acceptance, staging ceremony, and human visual-review gates are not required merge/release gates. Test depth follows changed behavior and blast radius, not ceremony.

Before adding another verification layer, answer:

1. What observable behavior changed?
2. Which repository-owned boundary can fail because of that change?
3. What is the cheapest automated check that observes that boundary?
4. What material failure remains invisible after that check?
5. Is there another deterministic repository-owned check that can observe that risk?

If #5 is no, document the residual risk and stop adding gates.

## Verification depth

### Startup / micro

Use for syntax, configuration, documentation, and tiny local low-risk changes. Run only the directly relevant parser/static/test check. Documentation-only changes do not justify product recompilation.

### Focused — default

Use for normal features, fixes, and bounded refactors. Combine only the relevant subset of:

- static/lint checks for the touched stack;
- tests for changed behavior;
- affected build target;
- boundary integration where the change crosses that boundary.

### Full

Use broad repository/release validation only when blast radius warrants it, such as:

- CI/build-system/toolchain/dependency changes;
- packaging/release changes;
- shared contract or architecture-boundary changes;
- security/privacy-critical changes spanning multiple boundaries;
- milestone/release qualification;
- explicit user request.

A workflow change is special: because the verification mechanism itself changed, the changed CI workflow intentionally exercises every CI lane once.

## Risk-routed CI architecture

PR/integration CI uses independent lanes that run in parallel. A lightweight classifier maps changed files to risk surfaces. A stable final `required` job aggregates lane results so branch protection can depend on one check even when irrelevant lanes are correctly skipped.

### Frontend lane

Runs only when frontend/tooling risk changes, such as `app/src/**`, package metadata, Vite/TypeScript configuration, or the CI workflow itself.

Evidence:

```text
npm run check
npm test
npm run build
```

The frontend lane runs on Linux because Svelte/TypeScript production verification is platform-independent. Windows-native behavior is not duplicated here.

### Rust core lane

Runs when `app/src-tauri/**` changes.

Evidence:

```text
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --lib --bins --all-features -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
```

This proves production Rust/Tauri compilation, linting, and application/core unit behavior without compiling every native integration harness on every Rust change.

### Model-contract lane

Runs only when model catalogs/build scripts/model-pack contract code changes.

Evidence:

```text
python ../scripts/models/build_opus_pack.py --catalog ../models/catalog/en-id-opus-v1.json --output ../models/build/en-id-opus-v1 --dry-run
```

Ordinary PR verification must not download production model weights. Real weights are a release-gate concern.

### Native boundary lane

Runs only when the C++ worker, worker/protocol Rust boundary, worker integration tests, Cargo dependency boundary, or CI workflow changes.

Evidence:

- configure C++ worker/runtime;
- build `cliplingo_worker` + runtime probe;
- execute native runtime probe;
- deterministic Rust -> C++ protocol regression;
- deterministic `WorkerTranslator` regression;
- compile the real-production smoke harness without downloading production weights.

Do not run this lane for unrelated popup/settings/application-core changes merely because the product contains a native worker.

### PowerShell distribution lane

Runs when `scripts/install.ps1` or the CI workflow changes. It is deliberately small and Windows-native.

Evidence:

- parse the PowerShell file with `System.Management.Automation.Language.Parser`;
- execute `scripts/install.ps1 -ResolveOnly` using the CI read-only GitHub token;
- prove the resolver can identify the newest published non-draft release, the x64 installer asset, and trusted SHA256 metadata;
- do **not** download or execute the installer in PR CI.

This tests the distribution bootstrap contract without turning installer-script changes into a product build or release installation.

### Required aggregate

The final `required` job succeeds when the classifier succeeds and every applicable lane is either `success` or intentionally `skipped`. Failure/cancellation of an applicable lane fails the aggregate.

## CI performance rules

- Keep `cancel-in-progress: true` so superseded branch runs stop consuming time.
- Parallelize independent risk lanes instead of building frontend -> Rust -> native serially.
- Preserve npm/Rust caches where deterministic and low-risk.
- Do not cache opaque CMake build outputs by default; stale native artifacts can create false confidence.
- Add a dependency/tool only when it materially improves signal or lead time.
- Use bounded timeouts so hung tooling does not become a delivery gate.
- Do not rerun a full pipeline after a failure when a focused rerun can prove the fix; the next integration run remains authoritative.
- Once an exact-head qualification is running, do not add documentation/cleanup commits that merely cancel and restart expensive gates.

## User-facing feature evidence

A user-facing feature is complete only when the required user path is integrated across the repository-owned layers it actually depends on. Backend/native implementation evidence or frontend rendering alone is insufficient when the product behavior requires both.

Use the smallest credible deterministic path:

```text
user intent/state
  -> UI / Tauri application behavior
  -> Rust application behavior
  -> worker/native boundary when applicable
  -> deterministic result or error-state assertion
```

Technical foundation slices may use narrower evidence while they remain prerequisites. Do not report those as completed product features.

## Current ClipLingo targeted evidence

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
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --lib --bins --all-features -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
```

Use broader Cargo targets only when integration-test/test-target changes require them.

### Model-pack contract

```text
python ../scripts/models/build_opus_pack.py --catalog ../models/catalog/en-id-opus-v1.json --output ../models/build/en-id-opus-v1 --dry-run
```

### Native worker boundary

```text
cmake -S ../worker -B ../worker/build -A x64
cmake --build ../worker/build --config Release --target cliplingo_worker cliplingo_worker_runtime_probe
```

Then run only the relevant runtime probe/protocol integration regressions.

### PowerShell installer bootstrap

```text
./scripts/install.ps1 -ResolveOnly
```

Use parser + `-ResolveOnly` evidence for bootstrap changes. Actual installer publication and artifact integrity are release concerns; manual installer execution is optional debugging/observational work, not a mandatory release gate.

## Release qualification boundary

Real production model download/conversion, non-deterministic EN -> ID inference smoke, NSIS packaging, checksums, and release publication belong to `.github/workflows/release-alpha.yml` / `.agents/RELEASE.md`.

PR CI proves contracts and executable boundaries cheaply; release CI proves the immutable distributed artifact with real production weights. Do not duplicate release-cost evidence on every PR.

## Native/environment evidence

Use automated native probes/integration tests for native behavior when that boundary matters. Manual Windows interaction may be useful for debugging, but it is not verification evidence required for merge, milestone completion, or release readiness.

When an environment-specific behavior cannot be credibly automated, document the limitation and residual risk rather than creating a mandatory manual acceptance gate.

## Performance evidence

Performance claims require release-mode measurement and enough environment context to reproduce them. Relevant ClipLingo metrics include hotkey -> popup latency, selection-capture latency, warm/cold inference latency, worker startup/model-load time, idle CPU, and working-set/peak memory.

Do not turn unmeasured targets into claims or blockers for unrelated slices.

## Security / supply-chain evidence

For model/update/release acquisition paths:

- validate configured/trusted origin where applicable;
- verify checksum/signature/manifest compatibility before activation;
- preserve license/redistribution metadata;
- fail closed on verification failure;
- keep signing credentials outside repository/frontend/logs.

For `scripts/install.ps1`, specifically verify release selection, x64 asset selection, trusted SHA256 metadata resolution, downloaded-file hashing before execution, and no bypass of Windows trust/security behavior.

## Evidence discipline

Never claim a test, CI run, benchmark, package, release, or deployment that was not actually observed. A valid failing regression is a defect signal; fix the defect rather than weakening the assertion.

When CI fails, use the failure as the next bounded engineering input. Do not add unrelated cleanup while waiting for evidence.
