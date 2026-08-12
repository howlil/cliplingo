# Engineering Rules

These are defaults with teeth. Deviations require evidence and must be documented in the task/PR when they materially affect architecture, security, performance, or release behavior.

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

- Behavior changes follow RED → GREEN → REFACTOR unless a test is technically impossible; document the reason.
- Test public behavior and boundaries, not private implementation trivia.
- Unit tests use fakes for Windows/inference boundaries; do not require real 100 MB+ models for ordinary core tests.
- Native compatibility must eventually be exercised against representative Windows applications.
- A bug fix requires a regression test at the lowest useful level.

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
