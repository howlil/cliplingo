# Lean SDLC and Git Strategy

## Flow

```text
problem
  -> acceptance example
  -> smallest vertical slice
  -> implement + focused test
  -> focused CI
  -> review/fix on same PR
  -> squash merge to master
  -> observe
```

The unit of delivery is a **small behavior**, not an iteration.

## WIP and scope

- WIP limit: one active implementation PR per agent.
- Prefer same-day mergeable slices. A normal PR older than ~2 working days triggers scope review/splitting.
- If a task contains independently useful behaviors, split before adding more code.
- Do not start Iteration N+1 work while a merge-ready slice from Iteration N sits unintegrated.
- Iterations are labels/backlog groupings only.

## Git

- `master` is the only integration branch.
- Branches are short-lived: `feat/`, `fix/`, `perf/`, `docs/`, `chore/`.
- One logical task = one branch = one PR.
- CI fixes, review fixes, and same-task cleanup stay on that PR.
- Squash merge normal work.
- Delete merged/abandoned branches.
- Never keep a long-lived `develop`, release-development, or iteration branch.

## Requirement before coding

For ordinary work, write only:
- problem;
- user-visible behavior;
- acceptance example;
- non-goals;
- risk level;
- cheapest useful verification.

Use `.agents/REQUIREMENTS.md`. A deeper plan is justified only by uncertainty or high blast radius.

## Testing and verification

Verification expands by **risk and maturity**, not by ceremony.

### Low risk
Docs, metadata, local behavior-preserving refactor:
- focused/static check only.

### Medium risk
UI behavior, Rust workflow, deterministic state/command logic:
- focused unit/component test;
- relevant integration boundary if crossed.

### High risk
Native Win32/UIA/clipboard, concurrency, privacy/security, process/IPC, real model integration:
- focused regression/error-path coverage;
- targeted integration/manual executable evidence for the changed boundary;
- full mandatory CI before merge.

Do not turn one high-risk slice into an exhaustive product certification. Alpha proves the changed core path; beta broadens compatibility/reliability/performance; stable enforces supported-platform and release guarantees.

## CI design

CI is a feedback system, not a ritual.

Rules:
- fail fast on format/type/test errors;
- run independent jobs in parallel when useful;
- keep expensive native/release/model checks conditional to relevant changes;
- avoid repeated dependency/model downloads where caching or fixtures provide equal signal;
- remove duplicate jobs and flaky gates;
- first actionable failure should arrive quickly enough to avoid context switching.

Current stack gates when product code is present:
- frontend install/check/test/build;
- Rust fmt/clippy/test;
- Windows build for native changes;
- targeted acceptance packaging only for native/manual verification work.

## Review

A PR description should answer:
- what user-visible behavior changed?
- what is explicitly not included?
- what important risk/trade-off exists?
- what verification actually ran?
- how can it be reverted?

Avoid architecture essays and exhaustive checklists unless the change genuinely needs them.

## Definition of Done

A slice is done when:
- its acceptance example works;
- required focused tests/checks are green;
- CI appropriate to the changed risk is green;
- no known blocker prevents this slice from being useful at its intended maturity;
- docs/state changed only where truth changed;
- it is merged to `master`.

Beta/stable hardening that is not necessary for the current alpha slice becomes explicit follow-up work, not an invisible merge blocker.

## Failure/recovery

Prefer `git revert`/a corrective patch over prolonged branch repair after a bad merge. Keep changes small enough that rollback is cheap.

## Metrics

Use `.agents/METRICS.md`. Optimize product cycle time, PR age/size, CI feedback, rework, change failure, escaped defects, WIP age, and useful release frequency. Never use commit/PR/LOC counts as productivity targets.
