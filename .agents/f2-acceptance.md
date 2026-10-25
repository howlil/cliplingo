# F2 — Native Windows model E2E acceptance (not yet executed)

| Proof | Faithful boundary | Status |
| --- | --- | --- |
| Missing/tampered model rejected | Rust `verify_pack` unit tests and real filesystem | CI dependent |
| `offline-ct2` C++/Rust ABI builds on Windows | Windows Cargo feature build | CI dependent |
| Model conversion, tokenizer parity, real EN→ID fixture | Actual downloaded OPUS-MT with source/target SentencePiece | Not run |
| UIA selection → CT2 → target native popup | Interactive Notepad/Chrome/PDF Windows desktop | Not run |
| Stale A/B never shows A, source app focus unchanged | Real concurrent Windows GUI | Not run |
| Missing, corrupt model, no network | Run with network off and no fake output | Not run |
| Cold/warm p50/p95/p99, RAM with Settings open/closed | Representative Windows hardware and whole process-tree measurement | Not run |
| Model shutdown/idle eviction, crash recovery | Windows host soak and fault-injection | Not run |
| Native unicode/long paragraph accessibility | DirectWrite/accessible text renderer (pending) | Not run |

Model is local-only. A green default Rust unit check does **not** prove optional CTranslate2, model weights, or end-to-end rendering; do not mark F2 production-ready until measured.
