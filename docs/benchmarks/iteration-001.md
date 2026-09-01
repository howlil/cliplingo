# Iteration 001 — Windows Alpha Evidence

## Status

The Windows interaction slice is implemented on `feat/iteration-001-windows-interaction`.

Automated verification exists for the product path. The only remaining merge blocker is a **targeted interactive Windows alpha smoke**. Do not expand this document back into beta/stable compatibility or performance certification.

## Implemented slice

```text
select text in another Windows application
 -> Ctrl+Alt+T
 -> pre-created popup shows Capturing state
 -> UI Automation selection capture
 -> safe clipboard fallback only when needed
 -> selection bounds or cursor fallback positioning
 -> deterministic [FAKE] translation
 -> latest request wins
 -> dismiss cleanly
```

Real translation inference is not part of Iteration 001.

## Automated verification

The mandatory Windows CI gate checks the committed frontend and Rust dependency graphs with:

```text
npm ci
npm run check
npm test
npm run build
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
PowerShell parse validation for the acceptance collector
acceptance executable/package build
```

Run #41 passed on the previous product head `69e5fae87148060ef23f4566d6c74e0bda1c890f`. The final PR head must also be green before merge.

Automated coverage includes:

- stale results cannot overwrite a newer request;
- UI Automation falls back to clipboard only for the supported fallback condition;
- unsupported clipboard preservation cases are refused before mutation;
- Unicode clipboard snapshot/restore behavior is covered by tests;
- popup positioning policy is unit-tested;
- runtime logging is designed around IDs, timing, status, source, and error metadata rather than selected/translated content.

## Required alpha smoke

Run `scripts/windows/iteration-001-acceptance.ps1` from the Windows acceptance package produced by the final CI run.

The collector intentionally asks for only the evidence needed to merge this alpha slice:

1. **Notepad happy path** — selected text produces the expected `[FAKE]` result.
2. **One additional representative selectable Windows application** — the same core interaction works.
3. **Popup usability** — the popup remains fully visible and usable on the tested display/DPI configuration.
4. **Clipboard safety when exercised** — if either tested path naturally uses clipboard fallback, the collector verifies restoration. If both paths use UI Automation, record that fallback was not required by the tested paths.
5. **Privacy log inspection** — a known selected-text sentinel must not appear in stdout/stderr.

The generated `iteration-001-alpha-smoke.md` is the manual evidence artifact.

## Explicitly deferred from the alpha merge gate

These remain useful follow-up evidence, but they must not block this slice:

- exhaustive Notepad/browser/VS Code/PDF compatibility matrix;
- broad DPI and multi-monitor matrix;
- edge-by-edge placement certification;
- >=20 warm p50/p95 interaction benchmark;
- idle working-set/private-memory/CPU certification;
- production translation runtime/model performance.

Treat them as beta/stable hardening or as evidence required by the specific future change that needs them.

## Exit decision

Iteration 001 can close when all of the following are true:

- final-head CI is green;
- the targeted alpha smoke artifact reports PASS;
- no defect discovered by that smoke remains unresolved.

Then mark PR #3 ready and squash-merge immediately. Do not add unrelated feature, architecture, refactor, dependency, model, release, or documentation work before merge.
