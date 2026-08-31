# Lean SDLC and Git Strategy

`AGENT_FLOW.md` defines the canonical engineering lifecycle and authority model. This file defines the repository mechanics used to execute that lifecycle.

## Flow

```text
USER INTENT
  -> UNDERSTAND
  -> BOUND
  -> SPECIFY
  -> DESIGN
  -> IMPLEMENT
  -> VERIFY
  -> QUALITY GATES
  -> RELEASE READY
  -> STOP
```

For a small, reversible, unambiguous task, stages may be fused into one tight loop:

```text
problem + acceptance
  -> smallest vertical slice
  -> implement + focused test
  -> focused CI
  -> review/fix on same PR
  -> release ready
  -> squash merge to master
```

The unit of delivery is a **small behavior**, not an iteration. Do not turn lifecycle stages into mandatory documents or meetings.

## WIP and scope

- WIP limit: one active implementation PR per agent.
- Prefer same-day mergeable slices. A normal PR older than ~2 working days triggers scope review/splitting.
- If a task contains independently useful behaviors, split before adding more code.
- Do not start Iteration N+1 work while a merge-ready slice from Iteration N sits unintegrated.
- Iterations are labels/backlog groupings only.
- Do not mix unrelated policy/docs/refactor work into an active feature PR.

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
- acceptance example/criteria;
- non-goals;
- risk level;
- cheapest useful verification;
- rollback/revert path.

Use `.agents/REQUIREMENTS.md`. A deeper plan is justified only by ambiguity, uncertainty, or high blast radius.

Acceptance criteria and observable product behavior belong to the user/product authority. The agent may clarify or propose them, but must not silently redefine them to make implementation easier.

## Design before coding

Prefer the smallest design that stays inside approved boundaries. Reuse repository patterns before introducing a new component or abstraction.

If implementation requires a public contract change, security boundary change, material architecture change, destructive migration, or a new ownership model for state/data, stop and surface the decision before coding.

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

Manual/native evidence is valid engineering evidence when automation cannot exercise the real boundary. Do not replace it with inspection-only claims.

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

Review the actual diff against acceptance and approved boundaries. Do not use review to introduce unrelated cleanup or scope expansion.

## Release Ready / Definition of Done

A slice is release ready when:
- its acceptance example/criteria work;
- required focused tests/checks are green;
- CI appropriate to the changed risk is green;
- required native/manual evidence exists for changed native behavior;
- no known blocker prevents this slice from being useful at its intended maturity;
- docs/state changed only where truth changed;
- rollback/revert is understood.

For repository integration, normal work is done when the release-ready slice is merged to `master`.

Beta/stable hardening that is not necessary for the current alpha slice becomes explicit follow-up work, not an invisible merge blocker.

After release ready, stop. Do not automatically continue into cleanup, the next slice, or future hardening.

## Failure/recovery

Prefer `git revert`/a corrective patch over prolonged branch repair after a bad merge. Keep changes small enough that rollback is cheap.

## Retrospective

Do not run a retrospective after every trivial change. When required by `AGENT_FLOW.md`, use repository evidence to identify the dominant delivery bottleneck and choose one small improvement to verify in subsequent work.

## Metrics

Use `.agents/METRICS.md`. Optimize product cycle time, PR age/size, CI feedback, rework, change failure, escaped defects, WIP age, and useful release frequency. Never use commit/PR/LOC counts as productivity targets.
