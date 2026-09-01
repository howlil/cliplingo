# Milestone — Isolated Inference Boundary Alpha

## Why

ClipLingo now has the Windows selected-text -> popup workflow integrated. The next architectural risk is isolating inference so model/runtime failures cannot take down the desktop shell.

## Desired end state

The Rust shell can send a deterministic translation request to an isolated worker boundary and receive a versioned response while preserving request identity, bounded resource usage, privacy, and shell survivability.

This milestone proves the **process boundary**, not translation quality.

## Scope

1. **Worker protocol contract**
   - language-agnostic versioned frame format;
   - request/response/error message kinds;
   - request ID correlation;
   - bounded UTF-8 payload;
   - Rust codec + deterministic tests.
2. **Windows process transport**
   - `WorkerSupervisor` lifecycle: `Stopped`, `Starting`, `Ready`, `Busy`, `Failed`;
   - Windows Named Pipe transport;
   - bounded startup/restart behavior;
   - worker failure must not crash the shell.
3. **Cross-language deterministic worker**
   - minimal C++ worker executable implementing protocol v1;
   - deterministic fake translation through the real process boundary;
   - CI builds and exercises shell <-> worker round-trip.
4. **Shell integration**
   - replace the in-process fake translator with the isolated worker adapter;
   - preserve latest-request-wins behavior and popup error mapping;
   - keep selected/translated text out of normal logs.

## Non-goals

- CTranslate2 or SentencePiece integration;
- production model selection or model packs;
- language detection/routing;
- model download/update flow;
- OCR/settings/history;
- installer/signing/updater work;
- broad manual Windows compatibility testing;
- stable-grade latency/resource certification.

## Fixed architecture decisions

- Rust remains the application/workflow owner.
- Inference remains out-of-process.
- Production transport is Windows Named Pipe as defined by `.agents/DESIGN.md`.
- Worker protocol is language-agnostic and versioned so the C++ runtime can replace test implementations without changing shell workflow contracts.
- Worker startup is demand-driven; no unbounded restart loop.

## Slices

### Slice A — Protocol contract

Acceptance:
- protocol v1 has explicit magic/version/message type/request ID/payload length;
- translate request, translate response, and error response round-trip deterministically;
- Unicode text is preserved byte-for-byte through UTF-8 encode/decode;
- malformed magic/version/type/truncated/oversized frames fail explicitly;
- payload size is bounded before allocation/encode;
- no runtime dependency is added merely for framing.

### Slice B — Supervisor + Named Pipe

Acceptance:
- supervisor state transitions are deterministic and tested;
- pipe connect/read/write failures map to bounded worker errors;
- startup/restart attempts are bounded;
- shell remains alive when worker is missing or exits unexpectedly.

### Slice C — C++ deterministic worker

Acceptance:
- CI builds `cliplingo-worker.exe` with the repository Windows toolchain;
- C++ implementation interoperates with protocol v1;
- deterministic fake translation traverses the real pipe/process boundary;
- protocol conformance tests cover request ID and Unicode text.

### Slice D — Shell integration

Acceptance:
- popup workflow uses the worker-backed translator;
- stale worker responses cannot overwrite newer popup state;
- worker failures surface as translation failure without crashing the shell;
- current frontend/Rust/Windows CI remains green.

## Milestone gate

The milestone is complete when an automated Windows run proves:

`Rust shell -> Named Pipe -> isolated worker -> deterministic response -> Rust shell`

with request correlation, bounded failure handling, and no selected/translated text in normal logs.

Manual interactive Windows testing is not a milestone blocker unless explicitly reintroduced by the user.

## Next milestone

Real offline translation model integration starts only after this boundary is proven. Model/language-direction selection remains a user/product decision and is intentionally not pulled into this milestone.
