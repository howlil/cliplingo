# ClipLingo Agent Instructions

This repository uses `.agents/` as the canonical project knowledge and active engineering state layer. `AGENTS.md` is only the entrypoint and authority map; do not duplicate global engineering doctrine or create parallel planning systems here.

Before making a meaningful change, read only the canonical sources relevant to the task. Always inspect `.agents/CURRENT_ITERATION.md` when continuing active work.

## Canonical sources

- `.agents/PROJECT.md` — product intent, behavior, scope, contracts, constraints, non-goals, and material open questions.
- `.agents/ARCHITECTURE.md` — runtime topology, responsibility ownership, boundaries, data flow, and architecture invariants.
- `.agents/CURRENT_ITERATION.md` — current milestone, Feature Compass, active slice, evidence, blockers, and single next action.
- `.agents/CODE_PATTERNS.md` — repository-specific implementation patterns and conventions.
- `.agents/QUALITY.md` — proportional verification strategy, executable checks, and project quality evidence.
- `.agents/DECISIONS.md` — durable material decisions and rationale.
- `.agents/RELEASE.md` — ClipLingo release/distribution rules; read for release, packaging, signing, or updater work.
- `.agents/skills/*` — specialized recurring ClipLingo workflows only; load only when relevant.

## Authority

Use this order when repository information conflicts:

```text
explicit current user instruction
  -> PROJECT.md + approved material decisions
  -> ARCHITECTURE.md / DECISIONS.md
  -> CURRENT_ITERATION.md
  -> CODE_PATTERNS.md / QUALITY.md
  -> current code and tests
  -> PR descriptions and stale docs
```

If code and canonical documentation disagree, determine which is stale and repair the bounded inconsistency. Do not invent a product or architecture decision.

## Operating rule

Use the Principal-Implementer delivery model:

```text
USER INTENT
  -> UNDERSTAND / BOUND
  -> PLAN THE MILESTONE
  -> EXECUTE THE HIGHEST-VALUE COHERENT VERTICAL SLICE
  -> VERIFY PROPORTIONALLY TO CHANGED RISK
  -> INTEGRATE AT A LOGICAL CHANGE BOUNDARY
  -> UPDATE CURRENT_ITERATION
  -> CONTINUE UNTIL THE MILESTONE GATE
  -> RELEASE READY
  -> STOP
```

For non-trivial work, keep the Feature Compass in `CURRENT_ITERATION.md` current: **Shape -> Position -> Delta -> Next Move**.

Optimize for user value, capability density, correctness, and maintainability. Do not turn work into tiny ceremony-driven slices, branch-per-tweak churn, mandatory full-test ladders, speculative refactors, nice-to-have roadmap expansion, or duplicated task documentation.

A technical/foundation slice is valid only when it is a necessary prerequisite or blocker removal for an already-defined product capability. Do not report it as the completed user feature.

Do not change product behavior, public contracts, architecture boundaries, data ownership, security boundaries, or other material decisions without explicit user approval.

Do not create persistent task plans, secondary iteration-state directories, or additional `.agents/*.md` files unless the information has a durable project-level owner or the project genuinely requires an optional canonical document.