# Decisions

Record only durable, material ClipLingo decisions whose rationale would be costly or risky for a future agent to reconstruct. Current task state belongs in `CURRENT_ITERATION.md`; cheap implementation choices stay in code.

## D001 — Windows-first local desktop product

**Decision:** Windows 10/11 is the V1 product platform and normal translation is local/offline after model installation.

**Rationale:** the core product depends on selected-text capture, global shortcut behavior, popup positioning, and privacy-sensitive low-latency interaction with arbitrary desktop applications.

**Consequence:** do not introduce cloud translation, a web backend, or speculative cross-platform abstraction as a default implementation path.

## D002 — Rust application authority with Svelte presentation

**Decision:** Rust/Tauri owns application workflow/state and native integration; Svelte owns presentation only.

**Rationale:** native Windows behavior and lifecycle must remain independent from WebView/component details while still allowing a fast-changing UI surface.

**Consequence:** Tauri commands are thin adapters and Svelte consumes view models/user-intent contracts rather than owning durable application state.

## D003 — Isolated C++ inference worker

**Decision:** inference runs in `cliplingo-worker.exe`, isolated from the Rust shell and connected through a local Windows Named Pipe.

**Rationale:** inference has a distinct runtime/language/failure/resource domain. Isolation lets worker failure or model-runtime changes remain behind a stable application boundary.

**Consequence:** the worker must not own UI, clipboard, hotkeys, Tauri state, or user workflow, and its failure must not terminate the shell.

## D004 — Worker protocol v1 is bounded binary framing

**Decision:** Rust/C++ IPC uses the versioned `docs/protocol/worker-v1.md` binary frame rather than REST/gRPC/JSON transport.

**Rationale:** the messages are tiny, local, cross-language, and need explicit versioning, correlation, and allocation bounds without introducing a server stack.

**Consequence:** protocol v1 uses `CLNG` magic, explicit type/version, little-endian request ID/length, UTF-8 payloads, and a 1 MiB maximum. Incompatible semantic expansion requires a new version/message definition rather than reinterpretation.

## D005 — First distributable offline alpha route is OPUS-MT `ja -> en -> id`

**Decision:** the first committed real-translation pack uses pinned Helsinki-NLP OPUS-MT Japanese-to-English and English-to-Indonesian stages, converted for CTranslate2 CPU INT8 with SentencePiece assets.

**Rationale:** this gives a concrete Japanese-to-Indonesian route using model entries whose repository catalog records Apache-2.0 licensing and explicit pinned revisions, while preserving the worker/model-pack boundary.

**Consequence:** direct CJK->Indonesian or other model families remain future evidence-based choices; they are not silently added to the active milestone.

## D006 — Model weights are external versioned artifacts

**Decision:** production model binaries are generated/acquired outside Git; the repository stores catalogs, pinned build inputs, manifests, and verification tooling.

**Rationale:** model weights are large independent dependencies with their own versions, licensing, integrity, and update lifecycle.

**Consequence:** ordinary CI validates the model-pack plan without downloading production weights and uses bounded fixtures where runtime evidence is sufficient.

## D007 — GitHub Releases is the canonical public binary source

**Decision:** release binaries are built through the release pipeline and GitHub Releases is the canonical asset source; WinGet/Chocolatey are downstream distribution metadata/channels.

**Rationale:** one canonical binary build avoids divergent artifacts across package channels and keeps rollback/fix-forward understandable.

**Consequence:** published versions/assets are immutable; release signing credentials stay outside the repository. See `RELEASE.md` for maturity-specific release gates.
