# Engineering Rules

These are defaults with teeth. Deviations require evidence and must be documented in the task/PR when they materially affect architecture, security, performance, or release behavior.

The project-specific delivery constraints in `.devland/project.yaml` are deliberate repository overrides/constraints. This file mirrors and explains them for implementation work; Devland core still supplies the universal baseline where no project-specific rule exists.

## Delivery operating model

Optimize for **fast verified delivery**, not raw activity.

```text
problem / evidence
  -> acceptance behavior
  -> RED
  -> GREEN
  -> REFACTOR
  -> focused verification
  -> PR / CI
  -> review and fixes on the same branch
  -> merge
  -> observe / measure
```

- Deliver the smallest coherent vertical slice that proves useful behavior or removes a measured risk.
- Keep WIP low. One agent should normally finish one bounded task before starting unrelated implementation work.
- Use the shortest safe feedback loop first; widen verification only as risk increases.
- Keep test failures, review fixes, CI corrections, and small same-task follow-ups on the same branch and PR.
- Do not over-plan low-risk local work. Use deeper design for native boundaries, IPC, worker lifecycle, privacy/security, updater/release, model/runtime integration, and performance-sensitive architecture.
- Apply YAGNI aggressively. Speed never justifies bypassing privacy, correctness, compatibility, performance evidence, signing, or release safety.

## Delivery metrics

Use metrics to improve the engineering system, not to score contributors or agents.

Prefer:

- cycle time
- PR lead time
- CI feedback time
- change failure rate
- escaped defect rate
- rework rate
- flaky-test rate
- WIP age
- release frequency when the release signal is meaningful

Commit count, branch count, PR count, lines changed, and generated-code volume are **not productivity KPIs**. If delivery slows, inspect oversized scope, slow CI, flaky tests, review latency, unclear acceptance criteria, native-boundary friction, or architecture coupling before increasing WIP.

## Pragmatism and scope

- Implement what the current iteration needs; do not pre-build future subsystems.
- Choose the simplest design that preserves a known future boundary.
- Do not introduce a server, database, message broker, dependency injection framework, GraphQL/gRPC/REST layer, GPU stack, OCR engine, or multiple inference engines without a demonstrated requirement.
- Refactor only code touched by the task or code that directly blocks a clean implementation.
- Do not perform repository-wide cleanup as camouflage for a feature.

## Dependencies

- Prefer the standard library, existing dependencies, and official Tauri/Windows facilities before adding a new dependency.
- Add a dependency only when it meaningfully reduces complexity or risk.
- Before adding one, check maintenance status, license, security history, binary/runtime cost, Windows support, and whether the same behavior is trivial to implement locally.
- Keep lockfiles committed once the project is scaffolded.
- Put vendor-specific APIs behind a boundary only when replacement is plausible and meaningful. Do not make wrappers around wrappers for aesthetics.

## Architecture

- Rust owns application state and workflows.
- Svelte is presentation only.
- Tauri commands are thin adapters, not services containing business logic.
- Windows-specific APIs live in a Windows adapter/module, not scattered through core logic.
- CTranslate2/C++ is isolated behind the worker protocol.
- No HTTP localhost server for internal IPC.
- Use Windows Named Pipe for the inference worker unless measured constraints force another transport.

## Correctness and resilience

- Interactive translation uses latest-request-wins semantics.
- Long-running work must not block the UI/event loop.
- Worker/process/network operations require timeouts where a hang can affect UX.
- Errors must be structured enough for the UI to provide an actionable result.
- Do not silently swallow native/COM/IPC failures.

## Privacy and security

- Never log selected source text or translated text by default.
- Never send selected text to telemetry or remote APIs in the normal translation path.
- Minimize Tauri capabilities and expose narrow commands; never expose arbitrary file/process/shell execution to the WebView.
- Model/update artifacts must be verified before use.
- Secrets/signing keys never enter the repository.

## Performance

- No polling when an OS event/callback exists.
- CPU-first inference. Do not add GPU requirements before benchmarks prove value.
- Optimize p95 user latency and idle footprint, not synthetic throughput.
- Benchmark before and after performance-motivated changes.
- Avoid loading translation models at boot without evidence.

## Testing

- Behavior changes follow RED → GREEN → REFACTOR unless a test is technically impossible; document the reason and use the nearest deterministic executable reproduction.
- Confirm RED fails for the intended missing/incorrect behavior, not because of tooling or fixture mistakes.
- GREEN implements only enough to satisfy the current acceptance behavior.
- Refactor only while focused tests remain green.
- Test public behavior and boundaries, not private implementation trivia.
- Unit tests use fakes for Windows/inference boundaries; do not require real 100 MB+ models for ordinary core tests.
- Native compatibility must eventually be exercised against representative Windows applications.
- A bug fix requires a regression test at the lowest useful level.
- Do not weaken, delete, skip, or rewrite a valid regression merely to make CI green.
- Treat flaky tests and slow CI as delivery-system defects rather than normal friction.

### Verification by risk

All executable behavior still uses RED → GREEN → REFACTOR. These tiers determine how broadly to verify before merge.

- **Low risk:** docs, repository metadata, formatting, or behavior-preserving local refactor. Use focused/static verification only; do not invent expensive tests with no signal.
- **Medium risk:** Svelte interaction, Rust application workflow, command contract, deterministic worker protocol logic, or local state behavior. Run focused unit/component tests plus the relevant integration boundary.
- **High risk:** Windows selection/clipboard/UI Automation behavior, native COM/Win32 integration, IPC/process lifecycle, privacy/security boundaries, updater/signing/release behavior, model/runtime integration, concurrency/latest-request-wins semantics, or performance-sensitive architecture. Require negative/error-path coverage, relevant native/integration/E2E evidence, privacy/log inspection when relevant, benchmark evidence for performance claims, and full mandatory CI.

## Git and branch hygiene

- One task/bugfix uses at most one working branch.
- Do not create a new branch because one test failed or a small follow-up appeared.
- Intermediate RED/GREEN commits are allowed on the task branch.
- Before merge, normal tasks use squash merge so `master` receives one coherent task commit.
- Do not create standalone commits for formatting, tiny typos, CI retries, or "fix previous commit" while the task is still open.
- Do not leave abandoned experiment/iteration branches; delete them.
- Branch names: `feat/`, `fix/`, `perf/`, `docs/`, `chore/` plus a short task name.
- Release tags come only from a verified `master` commit.

## Documentation

Update docs in the same task when product behavior, architectural boundaries, commands, packaging, or release procedures change. Do not maintain speculative documentation for code that does not exist.
