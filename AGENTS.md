# ClipLingo Agent Instructions

This repository uses `.agents/` as the canonical project knowledge and active engineering state layer. `AGENTS.md` is a thin entrypoint and authority map; do not duplicate global engineering doctrine or create parallel planning systems here.

Before a meaningful change, read only the canonical sources relevant to the task. Always inspect `.agents/CURRENT_ITERATION.md` when continuing active work.

## Canonical sources

- `.agents/PROJECT.md` — product intent, behavior, scope, contracts, constraints, non-goals, and material open questions.
- `.agents/ARCHITECTURE.md` — runtime topology, responsibility ownership, boundaries, data flow, and architecture invariants.
- `.agents/CURRENT_ITERATION.md` — current milestone, Feature Compass, active slice, evidence, blockers, and single next action.
- `.agents/CODE_PATTERNS.md` — repository-specific implementation patterns and conventions.
- `.agents/QUALITY.md` — proportional verification strategy, risk-routed CI contract, executable checks, and evidence discipline.
- `.agents/DECISIONS.md` — durable material decisions and rationale.
- `.agents/RELEASE.md` — release/distribution rules; read for release, packaging, signing, updater, or package-manager work.
- `DESIGN.md` — canonical ClipLingo UI/interaction quality contract. For UI work, product behavior comes first, then information hierarchy, interaction model, visual hierarchy, components, and only then decoration.
- `.agents/skills/*` — specialized recurring ClipLingo workflows only; load only when relevant.

## Authority

Use this order when repository information conflicts:

```text
explicit current user instruction
  -> PROJECT.md + approved material decisions
  -> ARCHITECTURE.md / DECISIONS.md
  -> DESIGN.md for UI/interaction decisions within approved product behavior
  -> CURRENT_ITERATION.md
  -> CODE_PATTERNS.md / QUALITY.md
  -> current code and tests
  -> PR descriptions and stale docs
```

`DESIGN.md` may constrain how approved behavior is presented, but it must not invent or override product scope, contracts, permissions, or architecture.

If code and canonical documentation disagree, determine which is stale and repair the bounded inconsistency. Do not invent a product or architecture decision.

## Principal-implementer operating rule

Use this delivery model:

```text
USER INTENT
  -> UNDERSTAND / BOUND
  -> PLAN THE MILESTONE
  -> EXECUTE COHERENT HIGH-VALUE VERTICAL SLICES CONTINUOUSLY
  -> VERIFY PROPORTIONALLY TO CHANGED RISK
  -> INTEGRATE AT LOGICAL CHANGE BOUNDARIES
  -> UPDATE CURRENT_ITERATION
  -> CONTINUE UNTIL THE MILESTONE GATE
  -> RELEASE READY
  -> STOP
```

For non-trivial work, keep the Feature Compass in `CURRENT_ITERATION.md` current: **Shape -> Position -> Delta -> Next Move**.

Optimize for **high user value × high product capability density × correctness × maintainability**, while minimizing outcome lead time, rework, waiting, verification waste, planning overhead, and agent/tool loops.

Do not turn work into tiny ceremony-driven slices, branch-per-tweak churn, repeated planning ceremonies, mandatory full-test ladders, speculative refactors, nice-to-have roadmap expansion, or duplicated task documentation. A technical/foundation slice is valid only when it removes a real prerequisite/blocker for an already-defined product capability; never report it as the completed user feature.

## Verification rule

`.agents/QUALITY.md` owns test depth. Use the cheapest highest-signal evidence that proves the changed behavior or boundary. Do not run every available test merely because it exists.

CI is risk-routed:

- frontend checks only for frontend/tooling risk;
- Rust checks only for Tauri/application-core risk;
- model-contract checks only for model-pack/build-contract risk;
- native C++/protocol checks only for worker/native-boundary risk;
- release qualification owns real production model inference and packaging gates.

A CI/workflow change intentionally exercises every lane once because the verification mechanism itself changed. Documentation-only follow-ups should not recompile the product.

When CI fails, fix the concrete evidence-backed cause. Do not weaken a valid assertion, add unrelated refactors, or broaden scope merely to make the pipeline green.

## UI/design rule

For UI changes, follow `DESIGN.md` and reject generic AI/SaaS treatment. Every visible element must support the ClipLingo translation workflow or a necessary control. Prefer compact, restrained, Windows-utility behavior and system-adaptive presentation over decorative chrome.

The decision order is:

```text
Product Intent
  -> Information Hierarchy
  -> Interaction Model
  -> Visual Hierarchy
  -> Components
  -> Decoration
```

Do not start from gradients, glass, cards, bento, animation, icons, or a generic design trend and force the product into it.

## Material stop conditions

Do not change product behavior, public contracts, architecture boundaries, data ownership, security/privacy boundaries, or other material decisions without explicit user approval.

Do not create persistent task plans, secondary iteration-state directories, or additional `.agents/*.md` files unless the information has a durable project-level owner or the project genuinely requires an optional canonical document.
