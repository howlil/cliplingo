# Lean SDLC and Git Strategy

`AGENT_FLOW.md` defines the canonical milestone delivery model and authority rules. This file defines the repository mechanics used to execute that model.

## Delivery flow

```text
USER INTENT
  -> UNDERSTAND
  -> BOUND
  -> MILESTONE PLAN
  -> EXECUTE SLICES CONTINUOUSLY
  -> MILESTONE GATE
  -> RELEASE READY
  -> STOP
```

Inside each slice, use the smallest useful engineering loop:

```text
problem + acceptance
  -> smallest vertical slice
  -> specify/design only as needed
  -> implement + focused test
  -> focused CI
  -> review/fix on same PR
  -> integrate
```

Plan at milestone boundaries. Execute continuously at slice boundaries. Integrate at logical-change boundaries.

The unit of planning is a **milestone**. The unit of execution is a **slice**. The unit of integration is a **logical change or tightly coupled slice**. Commits are history units, not planning units.

Do not turn lifecycle stages, milestones, or slices into mandatory documents or meetings.

## Milestone and slice mechanics

A milestone groups a bounded product/engineering outcome and may contain several slices. It does not imply one milestone branch or one milestone PR.

A slice is independently useful or independently evidence-producing progress toward the milestone. Prefer slices small enough to verify, review, revert, and merge without waiting for sibling slices.

Historical iteration/sprint identifiers may remain in state or plans for traceability. They do not control branch lifetime, merge cadence, or require repeated sprint planning.

## WIP and scope

- Keep implementation WIP low: normally one active implementation slice per agent.
- Prefer same-day mergeable logical changes. A normal PR older than ~2 working days triggers scope review/splitting.
- If a slice contains independently useful behaviors, split before adding more code.
- Do not leave a release-ready change unintegrated while starting unrelated milestone work.
- A blocker on one slice does not automatically block independent slices, unless the blocked slice is an explicit dependency or milestone gate.
- Do not mix unrelated policy/docs/refactor work into an active feature PR.

## Git

- `master` is the only integration branch.
- Branches are short-lived: `feat/`, `fix/`, `perf/`, `docs/`, `chore/`.
- One coherent logical change or tightly coupled slice = one branch = one PR.
- Do **not** create one branch per milestone, sprint, iteration label, planning stage, or trivial edit.
- CI fixes, review fixes, and same-change cleanup stay on that PR.
- Squash merge normal work.
- Delete merged/abandoned branches.
- Never keep a long-lived `develop`, release-development, milestone, or iteration branch.

## Requirement before coding

For an ordinary slice, write only:
- problem;
- user-visible behavior;
- acceptance example/criteria;
- non-goals;
- risk level;
- cheapest useful verification;
- rollback/revert path.

Use `.agents/REQUIREMENTS.md`. A deeper slice plan is justified only by ambiguity, uncertainty, or high blast radius. Broader planning belongs at the milestone boundary rather than being recreated for every small change.

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

## Integration readiness

A logical change or slice is ready to integrate when:
- its acceptance example/criteria work;
- required focused tests/checks are green;
- CI appropriate to the changed risk is green;
- required native/manual evidence exists for changed native behavior;
- no known blocker prevents this change from being useful at its intended maturity;
- docs/state changed only where truth changed;
- rollback/revert is understood.

Merge ready work to `master` without waiting for the rest of the milestone.

Beta/stable hardening that is not necessary for the current alpha slice becomes explicit follow-up work, not an invisible merge blocker.

## Milestone gate and release ready

After planned slices are integrated, run only the cross-slice or outcome-level checks needed to confirm the milestone's desired end state.

Do not re-run every slice-level ceremony at the milestone gate. A milestone is release ready when its agreed outcome and maturity-level gates are satisfied and no known release blocker remains.

After release ready, stop. Do not automatically continue into cleanup, the next milestone, or future hardening.

## Failure/recovery

Prefer `git revert`/a corrective patch over prolonged branch repair after a bad merge. Keep changes small enough that rollback is cheap.

## Retrospective

Do not run a retrospective after every trivial change. When required by `AGENT_FLOW.md`, run it after a meaningful milestone/release or evidence of repeated delivery friction. Use repository evidence to identify the dominant bottleneck and choose one small improvement to verify in subsequent work.

## Metrics

Use `.agents/METRICS.md`. Optimize product cycle time, PR age/size, CI feedback, rework, change failure, escaped defects, WIP age, and useful release frequency. Never use commit/PR/LOC counts as productivity targets.
