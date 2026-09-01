# ClipLingo Agent Instructions

This repository uses `.agents/` as the canonical project knowledge and active engineering state layer.

Before making a meaningful change, read only the canonical sources relevant to the task. Always inspect `CURRENT_ITERATION.md` when continuing active work.

## Canonical sources

- `.agents/PROJECT.md` — product intent, behavior, scope, contracts, constraints, non-goals, and material open questions.
- `.agents/ARCHITECTURE.md` — runtime topology, responsibility ownership, boundaries, data flow, and architecture invariants.
- `.agents/CURRENT_ITERATION.md` — current milestone, active slice, completed work, evidence, blockers, and single next action.
- `.agents/CODE_PATTERNS.md` — repository-specific implementation patterns and conventions.
- `.agents/QUALITY.md` — verification strategy, executable checks, and project quality gates.
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
  -> historical plans, PR descriptions, and stale docs
```

If code and canonical documentation disagree, determine which is stale and repair the bounded inconsistency. Do not invent a product or architecture decision.

## Operating rule

Follow the canonical engineering lifecycle and user-authority model. Global engineering behavior is not duplicated in this repository.

Do not change product behavior, public contracts, architecture boundaries, data ownership, security boundaries, or other material decisions without explicit user approval.

Prefer the smallest coherent change. Read the minimum context required to act safely. Do not create persistent task plans or additional `.agents/*.md` files unless the information has a durable project-level owner or the project genuinely requires an optional canonical document.

`.devland/*` may remain as auxiliary tooling metadata, but it does not override the canonical `.agents` sources above.
