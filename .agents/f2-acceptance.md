# F2 — Verified native offline EN→ID backend; Windows GUI E2E pending

Status (2026-10-26): **Rust/C++ backend build and real model-loaded inference PASS on GitHub-hosted Windows x64**. This is a **headless native smoke**, not proof of UI Automation/browser compatibility, non-activating tooltip behavior, latency SLO or production installer.

| Proof | Boundary | Actual evidence |
| --- | --- | --- |
| Missing/tampered pack rejected | Rust SHA-256 manifest/filesystem tests | **PASS** — [Windows cargo tests](https://github.com/howlil/cliplingo/actions/runs/40243341796) |
| `offline-ct2` C++ and Rust build | Windows MSVC, `RUSTFLAGS=-C target-feature=+crt-static` | **PASS** — [F2 Windows CI](https://github.com/howlil/cliplingo/actions/runs/40243341975) |
| Reproducible OPUS-MT INT8 conversion | Pinned Helsinki-NLP revision, source/target SentencePiece, SHA-256 pack | **PASS** on Windows CI; Xet-enabled checkpoint download |
| Real native inference | Headless `offline_model_smoke` compiled, linked and ran model; nonempty changed EN→ID fixture output | **PASS** — [F2 Windows CI](https://github.com/howlil/cliplingo/actions/runs/40243341975) |
| Correct translation quality / tokenizer parity with PyTorch reference | Curated EN→ID fixtures, human evaluation and reference output comparison | **NOT RUN**; smoke proves existence, not quality |
| `Ctrl+Alt+T` → UIA selection → CT2 → native tooltip | Interactive Notepad/Chrome/PDF Windows desktop | **NOT RUN** |
| Stale A/B cannot display A; foreground focus unchanged | Real concurrent Windows GUI | **NOT RUN** |
| Offline model corruption and no network | Interactive disconnected environment, rollback/error rendering | **NOT RUN** (unit tamper tests pass; no offline network proof) |
| Idle model eviction and crash/COM hang containment | Native soak/fault injection | **NOT RUN** |
| RAM, power, p50/p95/p99 cold/warm | Representative Windows hardware + process tree | **NOT RUN** |
| Long paragraphs, mixed DPI, screen reader text | Windows desktop with final accessible renderer | **NOT RUN** (current GDI preview is not final DirectWrite) |

**MSVC ABI contract:** Upstream `ct2rs` compiles its CTranslate2 dependency with `/MT`; Rust/CXX bridges must also use static CRT. The F2 CI and documented local feature build set `RUSTFLAGS=-C target-feature=+crt-static`. A plain `cargo check --features offline-ct2` can succeed while the binary fails to link if CRT flags are mixed. Never weaken or bypass linker mismatch diagnostics.

**Distribution:** default Cargo build omits the optional CT2 engine. The local model pack is a separate, developer-prepared dependency; SHA-256 detects corruption but does not authenticate a maliciously rewritten manifest. No signed installer, app updater or public F2 beta is claimed.
