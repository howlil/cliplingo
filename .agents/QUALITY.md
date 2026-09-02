# Quality

`QUALITY.md` is the canonical repository-specific verification contract. `.github/workflows/ci.yml` remains the executable CI truth. This file explains which risks the checks protect, how much verification is appropriate during implementation, and which evidence matters.

## Product invariants

Changes must preserve these unless the user explicitly approves a material boundary change:

- Normal translation is offline.
- Selected and translated content is not emitted to normal logs/telemetry.
- Worker failure does not terminate the shell.
- Interactive popup results obey latest-request-wins semantics.
- Worker responses are request-ID correlated and protocol-bounded.
- Clipboard fallback preserves/restores user clipboard state within the supported safety policy.
- Model/update artifacts are verified before activation and carry license/compatibility metadata.

## Verification principle

Use the cheapest, fastest, highest-signal verification that can actually prove the changed behavior. Escalate only when a cheaper level leaves material risk invisible.

Do **not** automatically run a fixed `unit -> integration -> E2E -> manual/staging` ladder after every change. Test depth follows the behavior and boundary that changed, not ceremony.

Before adding another verification layer, ask:

1. What observable behavior changed?
2. At which boundary can it fail?
3. What is the cheapest check that observes that boundary?
4. What material failure would remain invisible after that check?
5. Is that remaining risk large enough to justify a deeper check?

Only escalate when the answer to #5 is yes.

## Verification depth

### Startup

Use for trivial structural/configuration edits where parse/load/basic startup is the meaningful failure mode.

Examples: syntax validation, configuration parse, basic command startup.

### Micro

Use for tiny local low-risk behavior with a narrow failure surface.

Run the smallest directly relevant check or test. Do not broaden verification merely because more suites exist.

### Focused — default

Use for normal feature, bug-fix, and bounded refactor work.

Typically combine only the relevant subset of:

- lint/type/static checks for the touched stack;
- tests covering the changed behavior;
- the build target affected by the change;
- integration/smoke evidence at a boundary the change actually crosses.

### Full

Use full repository/release validation when the blast radius genuinely warrants it, such as:

- build system, dependency, toolchain, CI, packaging, or release changes;
- project-wide refactors or shared contracts with broad consumers;
- security/privacy-critical changes whose risk spans multiple boundaries;
- milestone/release qualification;
- an explicit user request.

Full CI may remain broader than local implementation checks. Green CI is integration evidence; it is not a requirement to reproduce every CI step locally after every logical change.

## User-facing feature evidence

A user-facing feature is complete only when the required user path is integrated across the layers it depends on. Backend/native implementation evidence or frontend rendering evidence alone is not sufficient when the product behavior requires both.

Verification should prove the smallest credible end-to-end path, for example:

```text
user action
  -> UI / Tauri intent
  -> Rust application behavior
  -> worker / native / persistence boundary when applicable
  -> user-visible result or error state
```

Technical foundation slices may use narrower evidence while they remain prerequisites. Do not report those slices as completed product features, and do not weaken end-to-end acceptance merely because individual layer tests are green.

## Available targeted checks

These commands are available evidence. Select them proportionally; the list is not a mandatory sequence.

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

Use this when model catalog/build intent changes. Ordinary verification must not require production model-weight downloads unless the behavior under test genuinely depends on the real weights.

### C++ worker/native runtime

When the worker/native dependency boundary changes:

```text
cmake -S ../worker -B ../worker/build -A x64
cmake --build ../worker/build --config Release
```

Run the corresponding native runtime probe or integration test only when the changed boundary requires that evidence.

## Integration evidence

The repository has Windows integration coverage for:

- Rust -> C++ worker protocol/Named Pipe round-trip (`worker_cpp_integration`).
- `WorkerTranslator` spawning/using the worker (`worker_translator_integration`).

When a change crosses one of those boundaries, keep the relevant regression green and add only the smallest new executable evidence needed for the changed behavior.

## Current CI contract

On `master`, Windows CI currently performs broad repository integration checks for frontend, model-pack intent, Rust, C++ worker build, and worker-backed integration. `.github/workflows/ci.yml` is authoritative if this summary becomes stale.

Do not copy the entire CI workflow into every slice plan or treat each CI step as a separate delivery gate. During implementation, use targeted evidence; use the repository CI result at the integration boundary.

## Native/manual evidence

Manual Windows interaction is valid evidence when a real native behavior cannot be credibly automated. It is **not** a default merge blocker for active alpha development.

Require manual evidence only when the active behavior genuinely cannot be proven otherwise or the user explicitly makes it a gate. Do not substitute inspection claims for an explicitly required native/manual gate.

## Performance evidence

Performance conclusions require release-mode measurement of the relevant user path and enough environment context to reproduce the result. Relevant ClipLingo metrics include:

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